use std::slice::Iter;
use crate::util;

#[derive(PartialEq, Debug)]
pub struct Ruuvi {
    temperature: f64,
    humidity: f64,
    air_pressure: u32,
    acceleration: [f64; 3],
    voltage: f64,
    tx_power: i8,
    movement: u8,
    measurement: u16,
    mac: [u8; 6],
}

impl Ruuvi {
    pub fn new(data_vec: &[u8]) -> Result<Ruuvi, &str> {
        let mut data = data_vec.iter();
        if *data.next().ok_or("No data format")? != 5 {
            return Err("Data format is not v5.");
        }
        let (temperature, data) = temp(data)?;
        let (humidity, data) = humidity(data)?;

        let (air_pressure, data) = air_pressure(data)?;
        let (acceleration, data) = acceleration(data)?;
        let (voltage, tx_power, data) = ele(data)?;
        let (movement, data) = movement(data)?;
        let (measurement, data) = measurement(data)?;
        let (mac, _) = mac(data)?;
        Ok(Ruuvi {
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
    pub fn mac(&self) -> String {
        self.mac.iter()
            .map(|b| format!("{:02X?}", b))
            .reduce(|mut acc, byte| {
                acc.push(':');
                acc.push_str(&byte);
                acc
            })
            .unwrap()
    }
    pub fn to_json(&self) -> String {
        format!(
            "{{\"time\": \"{}\", \"device_id\":\"{}\", \"temperature\":{}, \"humidity\": {}, \"air_pressure\":{}}}",
            util::timestamp(),
            self.mac(),
            self.temperature,
            self.humidity,
            self.air_pressure
        )
    }
}

fn temp(mut data: Iter<u8>) -> Result<(f64, Iter<u8>), &str> {
    let v1 = *data.next().ok_or("No temperature data.")?;
    let v2 = *data.next().ok_or("No temperature data.")?;
    if (v1, v2) == (0x80, 0x00) {
        return Err("Invalid temperature.");
    }
    let value = i16::from_be_bytes([v1, v2]);
    Ok(((value as f64) * 0.005, data))
}
fn humidity(mut data: Iter<u8>) -> Result<(f64, Iter<u8>), &str> {
    let v1 = *data.next().ok_or("No humidity data.")?;
    let v2 = *data.next().ok_or("No humidity data.")?;
    if (v1, v2) == (0xff, 0xff) {
        return Err("Invalid humidity.");
    }
    let value = u16::from_be_bytes([v1, v2]);
    Ok(((value as f64) * 0.0025, data))
}

fn air_pressure(mut data: Iter<u8>) -> Result<(u32, Iter<u8>), &str> {
    let v1 = *data.next().ok_or("No air pressure data.")?;
    let v2 = *data.next().ok_or("No air pressure data.")?;
    if (v1, v2) == (0xff, 0xff) {
        return Err("Invalid air pressure.");
    }
    let value = u16::from_be_bytes([v1, v2]);
    Ok((50_000 + (value as u32), data))
}

fn acceleration_help(v1: u8, v2: u8) -> Result<f64, &'static str> {
    if (v1, v2) == (0x80, 0x00) {
        return Err("Invalid acceleration.");
    }
    let value = i16::from_be_bytes([v1, v2]);
    Ok((value as f64) * 0.001)
}
fn acceleration(mut data: Iter<u8>) -> Result<([f64; 3], Iter<u8>), &str> {
    let v1 = *data.next().ok_or("No acceleration data.")?;
    let v2 = *data.next().ok_or("No acceleration data.")?;
    let v3 = *data.next().ok_or("No acceleration data.")?;
    let v4 = *data.next().ok_or("No acceleration data.")?;
    let v5 = *data.next().ok_or("No acceleration data.")?;
    let v6 = *data.next().ok_or("No acceleration data.")?;
    let x = acceleration_help(v1, v2)?;
    let y = acceleration_help(v3, v4)?;
    let z = acceleration_help(v5, v6)?;
    Ok(([x, y, z], data))
}

fn ele(mut data: Iter<u8>) -> Result<(f64, i8, Iter<u8>), &str> {
    let v1 = *data.next().ok_or("No voltage data.")?;
    let v2 = *data.next().ok_or("No transmission power data.")?;
    let voltage = u16::from_be_bytes([v1, v2]) >> 5;
    if voltage == 2047 {
        return Err("Invalid voltage");
    }
    let tx_power = u16::from_be_bytes([v1, v2]) & 0x1f;
    if tx_power == 31 {
        return Err("Invalid transmission power");
    }
    Ok((
        (voltage as f64 + 1600.0) * 0.001,
        (tx_power as i8) * 2 - 40,
        data,
    ))
}

fn movement(mut data: Iter<u8>) -> Result<(u8, Iter<u8>), &str> {
    let v = *data.next().ok_or("No movement data.")?;
    if v == 0xff {
        return Err("Invalid movement.");
    }
    Ok((v, data))
}
fn measurement(mut data: Iter<u8>) -> Result<(u16, Iter<u8>), &str> {
    let v1 = *data.next().ok_or("No measurement data.")?;
    let v2 = *data.next().ok_or("No measurement data.")?;
    if (v1, v2) == (0xff, 0xff) {
        return Err("Invalid measurement counter.");
    }
    let value = u16::from_be_bytes([v1, v2]);
    Ok((value, data))
}

fn mac(mut data: Iter<u8>) -> Result<([u8; 6], Iter<u8>), &str> {
    let v1 = *data.next().ok_or("No mac address data.")?;
    let v2 = *data.next().ok_or("No mac address data.")?;
    let v3 = *data.next().ok_or("No mac address data.")?;
    let v4 = *data.next().ok_or("No mac address data.")?;
    let v5 = *data.next().ok_or("No mac address data.")?;
    let v6 = *data.next().ok_or("No mac address data.")?;
    Ok(([v1, v2, v3, v4, v5, v6], data))
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

        let valid_val: Ruuvi = Ruuvi {
            temperature: 24.3,
            air_pressure: 100044,
            humidity: 53.49,
            acceleration: [0.004, -0.004, 1.036],
            tx_power: 4,
            voltage: 2.977,
            movement: 66,
            measurement: 205,
            mac: [0xcb, 0xb8, 0x33, 0x4c, 0x88, 0x4f],
        };
        assert_eq!(Ruuvi::new(&valid_record), Ok(valid_val));
    }

    #[test]
    fn max() {
        let max_record: Vec<u8> = vec![
            0x05, 0x7F, 0xFF, 0xFF, 0xFE, 0xFF, 0xFE, 0x7F, 0xFF, 0x7F, 0xFF, 0x7F, 0xFF, 0xFF,
            0xDE, 0xFE, 0xFF, 0xFE, 0xCB, 0xB8, 0x33, 0x4C, 0x88, 0x4F,
        ];
        let max_val: Ruuvi = Ruuvi {
            temperature: 163.835,
            air_pressure: 115534,
            humidity: 163.8350,
            acceleration: [32.767, 32.767, 32.767],
            tx_power: 20,
            voltage: 3.646,
            movement: 254,
            measurement: 65534,
            mac: [0xcb, 0xb8, 0x33, 0x4c, 0x88, 0x4f],
        };
        assert_eq!(Ruuvi::new(&max_record), Ok(max_val));
    }

    #[test]
    fn min() {
        let min_record: Vec<u8> = vec![
            0x05, 0x80, 0x01, 0x00, 0x00, 0x00, 0x00, 0x80, 0x01, 0x80, 0x01, 0x80, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xCB, 0xB8, 0x33, 0x4C, 0x88, 0x4F,
        ];
        let min_val: Ruuvi = Ruuvi {
            temperature: -163.835,
            air_pressure: 50000,
            humidity: 0.0,
            acceleration: [-32.767, -32.767, -32.767],
            tx_power: -40,
            voltage: 1.6,
            movement: 0,
            measurement: 0,
            mac: [0xcb, 0xb8, 0x33, 0x4c, 0x88, 0x4f],
        };
        assert_eq!(Ruuvi::new(&min_record), Ok(min_val));
    }

    #[test]
    fn invalid_data() {
        let invalid_data: Vec<u8> = vec![0x05, 0x80, 0x01];
        assert_eq!(Ruuvi::new(&invalid_data), Err("No humidity data."));
    }

    #[test]
    fn invalid_record() {
        let invalid_record: Vec<u8> = vec![
            0x05, 0x80, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        assert_eq!(Ruuvi::new(&invalid_record), Err("Invalid temperature."));
    }
}
