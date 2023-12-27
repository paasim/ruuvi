use crate::err::Res;
use crate::ruuvi::Advertisement;
use bluer::monitor::{data_type, Monitor, MonitorHandle, Pattern};
use bluer::{Adapter, Device};
use futures::StreamExt;
use macaddr::MacAddr6;
use std::collections::HashSet;

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

/// Listen to ble advertisements and print the everything with ruuvi
/// manufacturer id (`0x0499`).
///
/// If `opt_macs` is not `None`, print advertisement from each device only once
/// until all the listed devices have been observed.
#[tokio::main(flavor = "current_thread")]
pub async fn print_advertisements(opt_macs: Option<Vec<MacAddr6>>) -> Res<()> {
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
    id: u16,
    adapter: &Adapter,
    mut macs: HashSet<MacAddr6>,
) -> Res<()> {
    while let Some(mevt) = mh.next().await {
        if let Some((_, ruuvi)) = Advertisement::from_monitor_event(mevt, adapter, id).await? {
            if macs.remove(&ruuvi.mac()) {
                println!("{}", ruuvi);
            }
            if macs.is_empty() {
                return Ok(());
            }
        };
    }
    Err("unexpected end of events")?
}

async fn scan_everything(mh: &mut MonitorHandle, id: u16, adapter: &Adapter) -> Res<()> {
    while let Some(mevt) = mh.next().await {
        if let Some((dev, ruuvi)) = Advertisement::from_monitor_event(mevt, adapter, id).await? {
            println!("{}", ruuvi);
            tokio::spawn(scan_device_events(dev, id));
        };
    }
    Ok(())
}

async fn scan_device_events(dev: Device, id: u16) -> Res<()> {
    let mut events = dev.events().await.map_err(|e| e.to_string())?;
    while let Some(devt) = events.next().await {
        if let Some(ruuvi) = Advertisement::from_device_event(devt, id)? {
            println!("{}", ruuvi);
        }
    }
    Ok(())
}
