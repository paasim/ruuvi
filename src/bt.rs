use crate::cacher::Cacher;
use crate::config::Config;
use crate::ruuvi::Ruuvi;
use bluez_async::{BluetoothEvent, BluetoothSession, DeviceEvent, DiscoveryFilter, MacAddress};
use futures::stream::StreamExt;
use futures::Stream;
use std::error::Error;

#[tokio::main]
pub async fn scan(config: Config) -> Result<(), Box<dyn Error>> {
    let (_, mut session) = BluetoothSession::new().await?;
    let mut events = start_discovery(&mut session).await?;

    let mut cacher = Cacher::from(config.macs);
    event_loop(&mut events, &mut cacher).await?;

    session.stop_discovery().await?;
    Ok(())
}

async fn start_discovery(
    session: &mut BluetoothSession,
) -> Result<impl Stream<Item = BluetoothEvent>, Box<dyn Error>> {
    let events = session.event_stream().await?;
    session
        .start_discovery_with_filter(&DiscoveryFilter {
            duplicate_data: Some(true),
            ..DiscoveryFilter::default()
        })
        .await?;
    Ok(events)
}

async fn event_loop<E: Stream<Item = BluetoothEvent> + std::marker::Unpin>(
    events: &mut E,
    cacher: &mut Cacher<MacAddress>,
) -> Result<(), Box<dyn Error>> {
    while let Some(event) = events.next().await.as_mut() {
        if let Some(data) = get_data_for_manufacturer(event, &0x0499) {
            if data[0] != 5 {
                continue;
            }

            let ruuvi = Ruuvi::from_rawv5(&data)?;
            if cacher.has_cached(ruuvi.mac()) {
                continue;
            }

            println!("{}", serde_json::to_string(&ruuvi)?);
            if cacher.is_done() {
                return Ok(());
            }
        }
    }
    Err(String::from("unexpected end of events").into())
}

fn get_data_for_manufacturer(event: &mut BluetoothEvent, manufacturer: &u16) -> Option<Vec<u8>> {
    match event {
        BluetoothEvent::Device {
            event: DeviceEvent::ManufacturerData { manufacturer_data },
            ..
        } => manufacturer_data.remove(manufacturer),
        _ => None,
    }
}
