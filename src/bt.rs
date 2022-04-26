use bluez_async::{BluetoothEvent, BluetoothSession, DeviceEvent, DiscoveryFilter};
use futures::stream::StreamExt;
use std::error::Error;
use crate::request;
use crate::ruuvi::Ruuvi;
use crate::util::{Config, Cacher};

#[tokio::main]
pub async fn scan(config: Config) -> Result<(), Box<dyn Error>> {
    let (_, session) = BluetoothSession::new().await?;
    let mut events = session.event_stream().await?;
    session
        .start_discovery_with_filter(&DiscoveryFilter {
            duplicate_data: Some(true),
            ..DiscoveryFilter::default()
        })
        .await?;

    println!("Scanning for devices {:?}:", config.macs);
    let mut cacher: Cacher = Cacher::new(&config.macs)?;

    while let Some(event) = events.next().await {
        if let Some(data) = get_manufacturer_data_for(event, &0x0499) {
            let ruuvi = Ruuvi::new(&data)?;
            let mac = ruuvi.mac();
            if cacher.see(&mac) {
                if let Some(ref url) = config.endpoint {
                    println!("{} observed", mac);
                    request::post(ruuvi.to_json(), &url).await?;
                } else {
                    println!("{:?}", ruuvi);
                }
                if cacher.all_seen() { break; }
            }
        }
    }
    session.stop_discovery().await?;
    println!("Scan stopped succesfully!");
    Ok(())
}

fn get_manufacturer_data_for(event: BluetoothEvent, manufacturer: &u16) -> Option<Vec<u8>> {
    match event {
        BluetoothEvent::Device {
            id: _id,
            event: DeviceEvent::ManufacturerData { mut manufacturer_data },
        } => {
            manufacturer_data.remove(manufacturer)
        }
        _ => { None }
    }
}
