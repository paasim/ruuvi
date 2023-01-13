use chrono::Utc;
use chrono_tz::Europe::Helsinki;
use std::error::Error;
use std::fmt::Display;
use std::slice::Iter;

#[derive(Debug, PartialEq)]
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
    pub fn new(data_vec: &[u8]) -> Result<Ruuvi, Box<dyn Error>> {
        let mut data = data_vec.iter();
        if *data.next().ok_or("No data format")? != 5 {
            return Err("Data format is not v5.".into());
        }
        let temperature = temp(&mut data)?;
        let humidity = humidity(&mut data)?;

        let air_pressure = air_pressure(&mut data)?;
        let acceleration = acceleration(&mut data)?;
        let (voltage, tx_power) = ele(&mut data)?;
        let movement = movement(&mut data)?;
        let measurement = measurement(&mut data)?;
        let mac = mac(&mut data)?;
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
        self.mac
            .iter()
            .map(|b| format!("{:02X?}", b))
            .reduce(|mut acc, byte| {
                acc.push(':');
                acc.push_str(&byte);
                acc
            })
            .unwrap()
    }

    pub fn to_json(&self) -> String {
        let now = Utc::now().with_timezone(&Helsinki).to_rfc3339();
        let inner = [
            format_field("time", now, true),
            format_field("device_id", self.mac(), true),
            format_field("temperature", self.temperature, false),
            format_field("humidity", self.humidity, false),
            format_field("air_pressure", self.air_pressure, false),
        ]
        .join(", ");
        format!("{{ {} }}", inner)
    }
}

fn format_field<T: Display>(name: &str, val: T, escape: bool) -> String {
    if escape {
        format!("\"{}\": {}", name, val)
    } else {
        format!("\"{}\": \"{}\"", name, val)
    }
}

fn next_u8(data: &mut Iter<u8>, name: &str) -> Result<u8, Box<dyn Error>> {
    data.next()
        .map(|u| *u)
        .ok_or(format!("No {} data.", name).into())
}

fn next_two(data: &mut Iter<u8>, name: &str) -> Result<[u8; 2], Box<dyn Error>> {
    Ok([next_u8(data, name)?, next_u8(data, name)?])
}

fn validate<T: PartialEq>(t: T, inv: T, name: &str) -> Result<T, Box<dyn Error>> {
    if t != inv {
        Ok(t)
    } else {
        Err(format!("Invalid {}.", name).into())
    }
}

fn temp(data: &mut Iter<u8>) -> Result<f64, Box<dyn Error>> {
    let name = "temperature";
    let v = next_two(data, name).and_then(|v| validate(v, [0x80, 0x00], name))?;
    let value = i16::from_be_bytes(v);
    Ok((value as f64) * 0.005)
}

fn humidity(data: &mut Iter<u8>) -> Result<f64, Box<dyn Error>> {
    let name = "humidity";
    let v = next_two(data, name).and_then(|v| validate(v, [0xff, 0xff], name))?;
    let value = u16::from_be_bytes(v);
    Ok((value as f64) * 0.0025)
}

fn air_pressure(data: &mut Iter<u8>) -> Result<u32, Box<dyn Error>> {
    let name = "air_pressure";
    let v = next_two(data, name).and_then(|v| validate(v, [0xff, 0xff], name))?;
    let value = u16::from_be_bytes(v);
    Ok(50_000 + (value as u32))
}

fn acceleration_d(data: &mut Iter<u8>) -> Result<f64, Box<dyn Error>> {
    let name = "acceleration";
    let v = next_two(data, name).and_then(|v| validate(v, [0x80, 0x00], name))?;
    let value = i16::from_be_bytes(v);
    Ok((value as f64) * 0.001)
}

fn acceleration(data: &mut Iter<u8>) -> Result<[f64; 3], Box<dyn Error>> {
    let x = acceleration_d(data)?;
    let y = acceleration_d(data)?;
    let z = acceleration_d(data)?;
    Ok([x, y, z])
}

fn ele(data: &mut Iter<u8>) -> Result<(f64, i8), Box<dyn Error>> {
    let v1 = next_u8(data, "voltage")?;
    let v2 = next_u8(data, "transmission power")?;
    let voltage = u16::from_be_bytes([v1, v2]) >> 5;
    if voltage == 2047 {
        return Err("Invalid voltage".into());
    }
    let tx_power = u16::from_be_bytes([v1, v2]) & 0x1f;
    if tx_power == 31 {
        return Err("Invalid transmission power".into());
    }
    Ok(((voltage as f64 + 1600.0) * 0.001, (tx_power as i8) * 2 - 40))
}

fn movement(data: &mut Iter<u8>) -> Result<u8, Box<dyn Error>> {
    let name = "movement";
    next_u8(data, name).and_then(|v| validate(v, 0xff, name))
}

fn measurement(data: &mut Iter<u8>) -> Result<u16, Box<dyn Error>> {
    let name = "measurement";
    let v = next_two(data, name).and_then(|v| validate(v, [0xff, 0xff], name))?;
    Ok(u16::from_be_bytes(v))
}

fn mac(data: &mut Iter<u8>) -> Result<[u8; 6], Box<dyn Error>> {
    let [v1, v2] = next_two(data, "mac address")?;
    let [v3, v4] = next_two(data, "mac address")?;
    let [v5, v6] = next_two(data, "mac address")?;
    Ok([v1, v2, v3, v4, v5, v6])
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
        assert_eq!(Ruuvi::new(&valid_record).unwrap(), valid_val);
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
        assert_eq!(Ruuvi::new(&max_record).unwrap(), max_val);
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
        assert_eq!(Ruuvi::new(&min_record).unwrap(), min_val);
    }

    #[test]
    fn invalid_data() {
        let invalid_data: Vec<u8> = vec![0x05, 0x80, 0x01];
        assert_eq!(
            Ruuvi::new(&invalid_data).unwrap_err().to_string(),
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
            Ruuvi::new(&invalid_record).unwrap_err().to_string(),
            "Invalid temperature.".to_string()
        );
    }
}
