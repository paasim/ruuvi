use crate::err::Res;
use crate::ruuvi::{datetime_to_bytes, Measurement, Record};
use bluer::gatt::remote::{Characteristic, Service};
use bluer::{Adapter, AdapterEvent, Address, Device};
use chrono::{DateTime, Duration, Utc};
use futures::{future, Stream, StreamExt, TryStreamExt};
use macaddr::MacAddr6;
use uuid::Uuid;

const UART_SVC: Uuid = Uuid::from_u128(0x6e400001b5a3f393e0a9e50e24dcca9e);
const UART_RX: Uuid = Uuid::from_u128(0x6e400002b5a3f393e0a9e50e24dcca9e); // write
const UART_TX: Uuid = Uuid::from_u128(0x6e400003b5a3f393e0a9e50e24dcca9e); // read

/// Print log for the last `n_hours`. See [`get_log`].
#[tokio::main(flavor = "current_thread")]
pub async fn print_log(mac: MacAddr6, n_hours: u8) -> Res<()> {
    let begin_ts = Utc::now() - Duration::hours(n_hours.into());
    for r in get_log(mac, begin_ts).await? {
        println!("{}", r);
    }
    Ok(())
}

/// Get log starting from `log_start`.
///
/// If `log_start` is newer than current timestamp - 2 minutes or older than
/// current timestamp - 240 hours, it is set to those limits.
pub async fn get_log(mac: MacAddr6, log_start: DateTime<Utc>) -> Res<Vec<Record>> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let device = find_device(&adapter, mac).await?;
    try_to_connect(&device, 3).await?;

    get_records(&device, log_start).await
}

async fn get_service(device: &Device, uuid: Uuid) -> Res<Service> {
    for svc in device.services().await? {
        if svc.uuid().await? == uuid {
            return Ok(svc);
        }
    }
    Err(format!("unable to find service with uuid {}", uuid))?
}

async fn get_characteristic(svc: &Service, uuid: Uuid) -> Res<Characteristic> {
    for char in svc.characteristics().await? {
        if char.uuid().await? == uuid {
            return Ok(char);
        }
    }
    Err(format!("unable to find characteristic with uuid {}", uuid))?
}

async fn get_event_stream(
    device: &Device,
    log_start: DateTime<Utc>,
) -> Res<impl Stream<Item = Vec<u8>>> {
    let uart_svc = get_service(device, UART_SVC).await?;
    let recv_char = get_characteristic(&uart_svc, UART_TX).await?;
    let send_char = get_characteristic(&uart_svc, UART_RX).await?;
    let stream = recv_char.notify().await?;

    let current_ts = Utc::now();
    let start_ts = log_start
        .min(current_ts - Duration::minutes(1))
        .max(current_ts - Duration::hours(240));
    let data = [
        &[0x3A, 0x3A, 0x11],
        datetime_to_bytes(current_ts)?.as_slice(),
        datetime_to_bytes(start_ts)?.as_slice(),
    ]
    .concat();

    send_char.write(&data).await?;
    Ok(stream)
}

async fn get_records(device: &Device, log_start: DateTime<Utc>) -> Res<Vec<Record>> {
    let stream = get_event_stream(device, log_start).await?;
    let measurements: Vec<_> = stream
        .filter_map(Measurement::from_11_bytes)
        .try_take_while(|x| future::ready(Ok(x != &Measurement::EndOfMeasurements)))
        .try_collect()
        .await?;
    measurements.chunks(3).map(Record::from_chunk).collect()
}

async fn find_device(adapter: &Adapter, mac: MacAddr6) -> Res<Device> {
    let mac = Address::from(mac);
    let mut discover = adapter.discover_devices().await?;
    while let Some(evt) = discover.next().await {
        if let AdapterEvent::DeviceAdded(addr) = evt {
            if addr == mac {
                return Ok(adapter.device(addr)?);
            }
        }
    }
    Err("unable to find device")?
}

async fn try_to_connect(device: &Device, max_tries: u8) -> Res<()> {
    if device.is_connected().await? {
        return Ok(());
    }
    for _ in 0..max_tries {
        if device.connect().await.is_ok() {
            return Ok(());
        }
    }
    Err(format!("unable to connect device after {}", max_tries))?
}
