use crate::ruuvi::Ruuvi;
use bluer::monitor::{data_type, Monitor, MonitorEvent, MonitorHandle, Pattern};
use bluer::{Adapter, DeviceEvent, DeviceProperty};
use futures::StreamExt;
use macaddr::MacAddr6;
use std::collections::HashSet;
use std::error::Error;

fn manufacturer_pattern(manufacturer_id: u16) -> Monitor {
    Monitor {
        patterns: Some(vec![Pattern {
            data_type: data_type::MANUFACTURER_SPECIFIC_DATA,
            start_position: 0x00,
            content: manufacturer_id.to_le_bytes().to_vec(),
        }]),
        ..Default::default()
    }
}

#[tokio::main(flavor = "current_thread")]
pub async fn scan(opt_macs: Option<Vec<MacAddr6>>) -> Result<(), Box<dyn Error>> {
    let id = 0x0499;
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    let mm = adapter.monitor().await?;
    adapter.set_powered(true).await?;
    let mut mh = mm.register(manufacturer_pattern(id)).await?;

    match opt_macs {
        Some(macs) => scan_cached(&mut mh, id, &adapter, macs.into_iter().collect()).await,
        None => scan_everything(&mut mh, id, &adapter).await,
    }
}

async fn scan_cached(
    mh: &mut MonitorHandle,
    manufacturer_id: u16,
    adapter: &Adapter,
    mut macs: HashSet<MacAddr6>,
) -> Result<(), Box<dyn Error>> {
    while let Some(mevt) = mh.next().await {
        let opt_data = match mevt {
            MonitorEvent::DeviceFound(d) => adapter
                .device(d.device)?
                .manufacturer_data()
                .await?
                .and_then(|mut md| md.remove(&manufacturer_id)),
            _ => None,
        };
        if let Some(data) = opt_data {
            let ruuvi = Ruuvi::from_rawv5(data.as_slice())?;

            if macs.remove(&ruuvi.mac()) {
                println!("{}", serde_json::to_string(&ruuvi)?);
            }

            if macs.is_empty() {
                return Ok(());
            }
        };
    }
    Err("unexpected end of events".into())
}

async fn scan_everything(
    mh: &mut MonitorHandle,
    id: u16,
    adapter: &Adapter,
) -> Result<(), Box<dyn Error>> {
    while let Some(mevt) = mh.next().await {
        let dev = match mevt {
            MonitorEvent::DeviceFound(devid) => adapter.device(devid.device)?,
            _ => continue,
        };
        tokio::spawn(async move {
            let mut events = dev.events().await.unwrap();
            while let Some(data) = events.next().await.and_then(|e| get_md(e, id)) {
                let ruuvi = Ruuvi::from_rawv5(data.as_slice()).unwrap();
                println!("{}", serde_json::to_string(&ruuvi).unwrap());
            }
        });
    }
    Ok(())
}

fn get_md(event: DeviceEvent, manufacturer_id: u16) -> Option<Vec<u8>> {
    match event {
        DeviceEvent::PropertyChanged(DeviceProperty::ManufacturerData(mut md)) => {
            md.remove(&manufacturer_id)
        }
        _ => None,
    }
}
