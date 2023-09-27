use std::{ops::Sub, error::Error};

use bluez_async::{
    BluetoothEvent, BluetoothSession, CharacteristicEvent, DeviceId, DeviceInfo, MacAddress,
};
use chrono::{Days, Utc};
use futures::{stream::StreamExt, Stream};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::log_record::{datetime_to_bytes, Measurement, MeasurementState};

#[tokio::main(flavor = "current_thread")]
pub async fn read(mac: MacAddress, n_days: u8) -> Result<(), Box<dyn Error>> {
    let (_, session) = BluetoothSession::new().await?;
    let device_info = get_device_info(&session, mac, 5, 3).await?;

    try_to_connect(&session, &device_info.id, 3).await?;

    let events_res = get_event_stream(&session, &device_info.id, n_days).await;
    let res = match events_res {
        Ok(events) => handle_events(events).await,
        Err(e) => Err(e),
    };
    try_to_disconnect(&session, &device_info.id, 3).await?;
    res
}

const UART_SVC: Uuid = Uuid::from_u128(0x6e400001b5a3f393e0a9e50e24dcca9e);
const UART_RX: Uuid = Uuid::from_u128(0x6e400002b5a3f393e0a9e50e24dcca9e); // write
const UART_TX: Uuid = Uuid::from_u128(0x6e400003b5a3f393e0a9e50e24dcca9e); // read

async fn get_event_stream(
    session: &BluetoothSession,
    device_id: &DeviceId,
    max_days: u8,
) -> Result<impl Stream<Item = BluetoothEvent>, Box<dyn Error>> {
    let uart_service = session.get_service_by_uuid(&device_id, UART_SVC).await?;
    let recieve_characteristic = session
        .get_characteristic_by_uuid(&uart_service.id, UART_TX)
        .await?;
    session.start_notify(&recieve_characteristic.id).await?;
    let stream = session
        .characteristic_event_stream(&recieve_characteristic.id)
        .await?;

    let send_characteristic = session
        .get_characteristic_by_uuid(&uart_service.id, UART_RX)
        .await?;

    let end_ts = Utc::now();
    let begin_ts = end_ts.sub(Days::new(max_days.into()));
    let data = [
        &[0x3A, 0x3A, 0x11],
        datetime_to_bytes(end_ts)?.as_slice(),
        datetime_to_bytes(begin_ts)?.as_slice(),
    ]
    .concat();
    session
        .write_characteristic_value(&send_characteristic.id, data)
        .await?;

    Ok(stream)
}

async fn try_to_connect(
    session: &BluetoothSession,
    device_id: &DeviceId,
    max_tries: u8,
) -> Result<(), Box<dyn Error>> {
    for _ in 0..max_tries {
        if let Ok(_) = session.connect(&device_id).await {
            return Ok(());
        }
    }
    Err(format!("connection failed after {} retries", max_tries).into())
}

async fn try_to_disconnect(
    session: &BluetoothSession,
    device_id: &DeviceId,
    max_tries: u8,
) -> Result<(), Box<dyn Error>> {
    for _ in 0..max_tries {
        if let Ok(_) = session.disconnect(&device_id).await {
            return Ok(());
        }
    }
    Err(format!("disconnect failed after {} retries", max_tries).into())
}

async fn get_device_info(
    session: &BluetoothSession,
    mac: MacAddress,
    scan_secs: u64,
    n_scans: u8,
) -> Result<DeviceInfo, Box<dyn Error>> {
    for _ in 0..n_scans {
        session.start_discovery().await?;
        sleep(Duration::from_secs(scan_secs)).await;
        session.stop_discovery().await?;

        let devices = session.get_devices().await?;
        if let Some(device) = devices.into_iter().find(|dev| dev.mac_address == mac) {
            return Ok(device);
        }
    }
    Err(format!("Device not found after {} scans.", n_scans).into())
}

async fn handle_events<E: Stream<Item = BluetoothEvent> + std::marker::Unpin>(
    mut events: E,
) -> Result<(), Box<dyn Error>> {
    let mut state = MeasurementState::new();
    while let Some(event) = events.next().await {
        let value = match event {
            BluetoothEvent::Characteristic {
                event: CharacteristicEvent::Value { value },
                ..
            } if value.len() == 11 => value, // only values with correct length
            _ => continue,
        };
        if let Some(meas) = Measurement::from_bytes(value.as_slice())? {
            state.update(meas)?;
        } else {
            return Ok(());
        }
    }
    Err("unexpected end of event stream".into())
}
