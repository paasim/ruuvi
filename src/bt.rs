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

    let (endpoint, macs) = config.destr();
    println!("Scanning for devices {:?}:", macs);
    event_loop(&mut events, macs, &endpoint).await?;

    session.stop_discovery().await?;
    println!("Scan stopped succesfully!");
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

async fn event_loop<E>(
    events: &mut E,
    macs: Vec<MacAddress>,
    endpoint: &Option<String>,
) -> Result<(), Box<dyn Error>>
where
    E: Stream<Item = BluetoothEvent> + std::marker::Unpin,
{
    let cacher = &mut Cacher::from(macs);
    while let Some(event) = events.next().await.as_mut() {
        if let Some(data) = get_data_for_manufacturer(event, &0x0499) {
            let ruuvi = Ruuvi::new(&data)?;
            let mac = ruuvi.mac();
            if cacher.see(&mac) {
                if let Some(url) = endpoint.as_ref() {
                    println!("{} observed", mac);
                    post(ruuvi.to_json(), &url).await?;
                } else {
                    println!("{:?}", ruuvi);
                }
                if cacher.all_seen() {
                    break;
                }
            }
        }
    }
    Ok(())
}

async fn post(body: String, url: &str) -> Result<reqwest::Response, Box<dyn Error>> {
    Ok(reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?)
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
