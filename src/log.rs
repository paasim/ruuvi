use bluer::gatt::remote::{Characteristic, Service};
use bluer::{Adapter, AdapterEvent, Address, Device};
use chrono::{Days, Utc};
use futures::pin_mut;
use futures::{stream::StreamExt, Stream};
use macaddr::MacAddr6;
use std::{error::Error, ops::Sub};
use uuid::Uuid;

use crate::log_record::{datetime_to_bytes, Measurement, MeasurementState};

#[tokio::main(flavor = "current_thread")]
pub async fn read(mac: MacAddr6, n_days: u8) -> Result<(), Box<dyn Error>> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let device = find_device(&adapter, mac).await?;
    try_to_connect(&device, 3).await?;

    let events_res = get_event_stream(&device, n_days).await;
    let res = match events_res {
        Ok(events) => {
            pin_mut!(events);
            handle_events(events).await
        }
        Err(e) => Err(e),
    };
    try_to_disconnect(&device, 3).await?;
    res
}

const UART_SVC: Uuid = Uuid::from_u128(0x6e400001b5a3f393e0a9e50e24dcca9e);
const UART_RX: Uuid = Uuid::from_u128(0x6e400002b5a3f393e0a9e50e24dcca9e); // write
const UART_TX: Uuid = Uuid::from_u128(0x6e400003b5a3f393e0a9e50e24dcca9e); // read

async fn get_service(device: &Device, uuid: Uuid) -> Result<Service, Box<dyn Error>> {
    for svc in device.services().await? {
        if svc.uuid().await? == uuid {
            return Ok(svc);
        }
    }
    Err(format!("unable to find service with uuid {}", uuid).into())
}

async fn get_characteristic(svc: &Service, uuid: Uuid) -> Result<Characteristic, Box<dyn Error>> {
    for char in svc.characteristics().await? {
        if char.uuid().await? == uuid {
            return Ok(char);
        }
    }
    Err(format!("unable to find characteristic with uuid {}", uuid).into())
}

async fn get_event_stream(
    device: &Device,
    max_days: u8,
) -> Result<impl Stream<Item = Vec<u8>>, Box<dyn Error>> {
    let uart_svc = get_service(device, UART_SVC).await?;
    let recv_char = get_characteristic(&uart_svc, UART_TX).await?;
    let send_char = get_characteristic(&uart_svc, UART_RX).await?;
    let stream = recv_char.notify().await?;

    let end_ts = Utc::now();
    let begin_ts = end_ts.sub(Days::new(max_days.into()));
    let data = [
        &[0x3A, 0x3A, 0x11],
        datetime_to_bytes(end_ts)?.as_slice(),
        datetime_to_bytes(begin_ts)?.as_slice(),
    ]
    .concat();

    send_char.write(&data).await?;
    Ok(stream)
}

async fn find_device(adapter: &Adapter, mac: MacAddr6) -> Result<Device, Box<dyn Error>> {
    let mac = Address::from(mac);
    let mut discover = adapter.discover_devices().await?;
    while let Some(evt) = discover.next().await {
        match evt {
            AdapterEvent::DeviceAdded(addr) if addr == mac => return Ok(adapter.device(addr)?),
            _ => {}
        }
    }
    Err("unable to find device".into())
}

async fn try_to_connect(device: &Device, max_tries: u8) -> Result<(), Box<dyn Error>> {
    if !device.is_connected().await? {
        for _ in 0..max_tries {
            if device.connect().await.is_ok() {
                return Ok(());
            }
        }
        return Err(format!("unable to connect device after {}", max_tries).into());
    }
    return Ok(());
}

async fn try_to_disconnect(device: &Device, max_tries: u8) -> Result<(), Box<dyn Error>> {
    for _ in 0..max_tries {
        if let Ok(_) = device.disconnect().await {
            return Ok(());
        }
    }
    Err(format!("disconnect failed after {} retries", max_tries).into())
}

async fn handle_events<E: Stream<Item = Vec<u8>> + std::marker::Unpin>(
    mut events: E,
) -> Result<(), Box<dyn Error>> {
    let mut state = MeasurementState::new();
    while let Some(v) = events.next().await {
        if v.len() != 11 {
            continue; // only values with correct length
        };
        if let Some(meas) = Measurement::from_bytes(v.as_slice())? {
            state.update(meas)?;
        } else {
            return Ok(());
        }
    }
    Err("unexpected end of event stream".into())
}
