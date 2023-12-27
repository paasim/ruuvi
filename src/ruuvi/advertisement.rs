use crate::err::Res;
use bluer::monitor::MonitorEvent;
use bluer::{Adapter, Device, DeviceEvent, DeviceProperty};
use macaddr::MacAddr6;
use serde::{Serialize, Serializer};
use std::slice::Iter;

#[derive(Debug, PartialEq, Serialize)]
pub struct Advertisement {
    pub temperature: f64,
    pub humidity: f64,
    pub air_pressure: u32,
    pub acceleration: [f64; 3],
    pub voltage: f64,
    pub tx_power: i8,
    pub movement: u8,
    pub measurement: u16,
    #[serde(serialize_with = "ser_mac")]
    pub mac: MacAddr6,
}

fn ser_mac<S: Serializer>(mac: &MacAddr6, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&mac.to_string())
}

impl Advertisement {
    pub fn from_rawv5(data_vec: impl AsRef<[u8]>) -> Res<Advertisement> {
        let mut data = data_vec.as_ref().iter();
        let _version = version(&mut data)?;
        let temperature = temp(&mut data)?;
        let humidity = humidity(&mut data)?;

        let air_pressure = air_pressure(&mut data)?;
        let acceleration = acceleration(&mut data)?;
        let (voltage, tx_power) = ele(&mut data)?;
        let movement = movement(&mut data)?;
        let measurement = measurement(&mut data)?;
        let mac = mac(&mut data)?;
        Ok(Advertisement {
            temperature,
            humidity,
            air_pressure,
            acceleration,
            voltage,
            tx_power,
            movement,
            measurement,
            mac,
        })
    }

    pub fn mac(&self) -> MacAddr6 {
        self.mac
    }

    pub async fn from_monitor_event(
        e: MonitorEvent,
        adapter: &Adapter,
        id: u16,
    ) -> Res<Option<(Device, Advertisement)>> {
        let dev = match e {
            MonitorEvent::DeviceFound(d) => adapter.device(d.device)?,
            _ => return Ok(None),
        };
        let man_data = dev.manufacturer_data().await?;
        let data = man_data.and_then(|mut md| md.remove(&id));
        data.map(|d| Advertisement::from_rawv5(d).map(|r| (dev, r)))
            .transpose()
    }

    pub fn from_device_event(e: DeviceEvent, id: u16) -> Res<Option<Self>> {
        let mut man_data = match e {
            DeviceEvent::PropertyChanged(DeviceProperty::ManufacturerData(md)) => md,
            _ => return Ok(None),
        };
        man_data
            .remove(&id)
            .map(Advertisement::from_rawv5)
            .transpose()
    }
}

impl std::fmt::Display for Advertisement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // to_string really should not fail...
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

fn next_u8(data: &mut Iter<u8>, name: &str) -> Res<u8> {
    data.next()
        .copied()
        .ok_or(format!("No {} data.", name).into())
}

fn next_n<const N: usize>(data: &mut Iter<u8>, name: &str) -> Res<[u8; N]> {
    let mut buf = [0; N];
    for v in buf.iter_mut() {
        *v = next_u8(data, name)?;
    }
    Ok(buf)
}

fn validate<T: PartialEq>(t: T, inv: T, name: &str) -> Res<T> {
    if t != inv {
        Ok(t)
    } else {
        Err(format!("Invalid {}.", name))?
    }
}

fn version(data: &mut Iter<u8>) -> Res<u8> {
    let version = next_u8(data, "version")?;
    if version != 5 {
        Err(format!("Invalid version {}.", version))?;
    }
    Ok(version)
}

fn temp(data: &mut Iter<u8>) -> Res<f64> {
    let name = "temperature";
    let v = next_n(data, name).and_then(|v| validate(v, [0x80, 0x00], name))?;
    let value = i16::from_be_bytes(v);
    Ok((value as f64) * 0.005)
}

fn humidity(data: &mut Iter<u8>) -> Res<f64> {
    let name = "humidity";
    let v = next_n(data, name).and_then(|v| validate(v, [0xff, 0xff], name))?;
    let value = u16::from_be_bytes(v);
    Ok((value as f64) * 0.0025)
}

fn air_pressure(data: &mut Iter<u8>) -> Res<u32> {
    let name = "air_pressure";
    let v = next_n(data, name).and_then(|v| validate(v, [0xff, 0xff], name))?;
    let value = u16::from_be_bytes(v);
    Ok(50_000 + (value as u32))
}

fn acceleration_d(data: &mut Iter<u8>) -> Res<f64> {
    let name = "acceleration";
    let v = next_n(data, name).and_then(|v| validate(v, [0x80, 0x00], name))?;
    let value = i16::from_be_bytes(v);
    Ok((value as f64) * 0.001)
}

fn acceleration(data: &mut Iter<u8>) -> Res<[f64; 3]> {
    let x = acceleration_d(data)?;
    let y = acceleration_d(data)?;
    let z = acceleration_d(data)?;
    Ok([x, y, z])
}

fn ele(data: &mut Iter<u8>) -> Res<(f64, i8)> {
    let v = next_n(data, "voltage and transmission power")?;
    let voltage = u16::from_be_bytes(v) >> 5;
    if voltage == 2047 {
        Err(format!("Invalid voltage {}", voltage))?;
    }
    let tx_power = u16::from_be_bytes(v) & 0x1f;
    if tx_power == 31 {
        Err(format!("Invalid transmission power {}", tx_power))?;
    }
    Ok(((voltage as f64 + 1600.0) * 0.001, (tx_power as i8) * 2 - 40))
}

fn movement(data: &mut Iter<u8>) -> Res<u8> {
    let name = "movement";
    next_u8(data, name).and_then(|v| validate(v, 0xff, name))
}

fn measurement(data: &mut Iter<u8>) -> Res<u16> {
    let name = "measurement";
    let v = next_n(data, name).and_then(|v| validate(v, [0xff, 0xff], name))?;
    Ok(u16::from_be_bytes(v))
}

fn mac(data: &mut Iter<u8>) -> Res<MacAddr6> {
    next_n(data, "mac address").map(MacAddr6::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        let valid_record: Vec<u8> = vec![
            0x05, 0x12, 0xFC, 0x53, 0x94, 0xC3, 0x7C, 0x00, 0x04, 0xFF, 0xFC, 0x04, 0x0C, 0xAC,
            0x36, 0x42, 0x00, 0xCD, 0xCB, 0xB8, 0x33, 0x4C, 0x88, 0x4F,
        ];

        let valid_val: Advertisement = Advertisement {
            temperature: 24.3,
            air_pressure: 100044,
            humidity: 53.49,
            acceleration: [0.004, -0.004, 1.036],
            tx_power: 4,
            voltage: 2.977,
            movement: 66,
            measurement: 205,
            mac: MacAddr6::from([0xcb, 0xb8, 0x33, 0x4c, 0x88, 0x4f]),
        };
        assert_eq!(Advertisement::from_rawv5(valid_record).unwrap(), valid_val);
    }

    #[test]
    fn max() {
        let max_record: Vec<u8> = vec![
            0x05, 0x7F, 0xFF, 0xFF, 0xFE, 0xFF, 0xFE, 0x7F, 0xFF, 0x7F, 0xFF, 0x7F, 0xFF, 0xFF,
            0xDE, 0xFE, 0xFF, 0xFE, 0xCB, 0xB8, 0x33, 0x4C, 0x88, 0x4F,
        ];
        let max_val: Advertisement = Advertisement {
            temperature: 163.835,
            air_pressure: 115534,
            humidity: 163.8350,
            acceleration: [32.767, 32.767, 32.767],
            tx_power: 20,
            voltage: 3.646,
            movement: 254,
            measurement: 65534,
            mac: MacAddr6::from([0xcb, 0xb8, 0x33, 0x4c, 0x88, 0x4f]),
        };
        assert_eq!(Advertisement::from_rawv5(max_record).unwrap(), max_val);
    }

    #[test]
    fn min() {
        let min_record: Vec<u8> = vec![
            0x05, 0x80, 0x01, 0x00, 0x00, 0x00, 0x00, 0x80, 0x01, 0x80, 0x01, 0x80, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xCB, 0xB8, 0x33, 0x4C, 0x88, 0x4F,
        ];
        let min_val: Advertisement = Advertisement {
            temperature: -163.835,
            air_pressure: 50000,
            humidity: 0.0,
            acceleration: [-32.767, -32.767, -32.767],
            tx_power: -40,
            voltage: 1.6,
            movement: 0,
            measurement: 0,
            mac: MacAddr6::from([0xcb, 0xb8, 0x33, 0x4c, 0x88, 0x4f]),
        };
        assert_eq!(Advertisement::from_rawv5(min_record).unwrap(), min_val);
    }

    #[test]
    fn invalid_version() {
        let invalid_data: Vec<u8> = vec![0x02, 0x80, 0x01];
        assert_eq!(
            Advertisement::from_rawv5(invalid_data)
                .unwrap_err()
                .to_string(),
            "Invalid version 2.".to_string()
        );
    }

    #[test]
    fn invalid_data() {
        let invalid_data: Vec<u8> = vec![0x05, 0x80, 0x01];
        assert_eq!(
            Advertisement::from_rawv5(invalid_data)
                .unwrap_err()
                .to_string(),
            "No humidity data.".to_string()
        );
    }

    #[test]
    fn invalid_record() {
        let invalid_record: Vec<u8> = vec![
            0x05, 0x80, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        assert_eq!(
            Advertisement::from_rawv5(invalid_record)
                .unwrap_err()
                .to_string(),
            "Invalid temperature.".to_string()
        );
    }
}
