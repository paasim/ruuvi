use chrono::{DateTime, Utc};
use serde::{Serialize, Serializer};
use std::error::Error;

#[derive(Debug, PartialEq, Serialize)]
pub struct LogRecord {
    #[serde(serialize_with = "ser_dt")]
    datetime: DateTime<Utc>,
    temperature: f64,
    humidity: f64,
    air_pressure: u32,
}

fn ser_dt<S: Serializer>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&dt.to_rfc3339())
}

#[derive(Debug, PartialEq)]
pub struct MeasurementState {
    temperature: Option<(DateTime<Utc>, f64)>,
    humidity: Option<(DateTime<Utc>, f64)>,
    air_pressure: Option<(DateTime<Utc>, u32)>,
}

impl MeasurementState {
    pub fn new() -> Self {
        Self {
            temperature: None,
            humidity: None,
            air_pressure: None,
        }
    }

    fn extract_record(&mut self) -> Result<Option<LogRecord>, Box<dyn Error>> {
        let ((ts0, temp), (ts1, hum), (ts2, air_pres)) =
            match (self.temperature, self.humidity, self.air_pressure) {
                (Some(t), Some(h), Some(ap)) => (t, h, ap),
                _ => return Ok(None),
            };
        if ts0 != ts1 || ts0 != ts2 {
            return Err("conflicting timestamps".into());
        }
        self.temperature = None;
        self.humidity = None;
        self.air_pressure = None;

        Ok(Some(LogRecord {
            datetime: ts0,
            temperature: temp,
            humidity: hum,
            air_pressure: air_pres,
        }))
    }

    pub fn update(&mut self, meas: Measurement) -> Result<(), Box<dyn Error>> {
        match meas {
            Measurement::Temperature(ts, temp) if self.temperature.is_none() => {
                self.temperature = Some((ts, temp))
            }
            Measurement::Humidity(ts, hum) if self.humidity.is_none() => {
                self.humidity = Some((ts, hum))
            }
            Measurement::AirPressure(ts, air_pres) if self.air_pressure.is_none() => {
                self.air_pressure = Some((ts, air_pres))
            }
            _ => return Err(format!("trying to add {:?} while it already exists", meas).into()),
        }
        if let Some(rec) = self.extract_record()? {
            println!("{}", serde_json::to_string(&rec)?);
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum Measurement {
    Temperature(DateTime<Utc>, f64),
    Humidity(DateTime<Utc>, f64),
    AirPressure(DateTime<Utc>, u32),
}

impl Measurement {
    pub fn from_bytes(obs: &[u8]) -> Result<Option<Self>, Box<dyn Error>> {
        if obs.len() != 11 {
            return Err(format!("payload length should be 11 (was {})", obs.len()).into());
        }
        if obs[0] != 0x3A {
            return Err(format!("header should start with 0x3a (was 0x{:x})", obs[0]).into());
        }
        if obs[2] != 0x10 {
            return Err(format!("action should be read (0x10, was 0x{:x})", obs[0]).into());
        }
        if obs[3..11] == [0xFF; 8] {
            return Ok(None);
        }
        let ts = datetime_from_bytes([obs[3], obs[4], obs[5], obs[6]])?;
        let val = [obs[7], obs[8], obs[9], obs[10]];
        match obs[1] {
            0x30 => Ok(Some(Self::Temperature(
                ts,
                i32::from_be_bytes(val) as f64 * 0.01,
            ))),
            0x31 => Ok(Some(Self::Humidity(
                ts,
                u32::from_be_bytes(val) as f64 * 0.01,
            ))),
            0x32 => Ok(Some(Self::AirPressure(ts, u32::from_be_bytes(val)))),
            _ => Err(format!("invalid observation type {:x}", obs[1]).into()),
        }
    }
}

pub fn datetime_from_bytes(ts: [u8; 4]) -> Result<DateTime<Utc>, Box<dyn Error>> {
    let unix_time = u32::from_be_bytes(ts);
    DateTime::<Utc>::from_timestamp(unix_time.into(), 0)
        .ok_or(format!("invalid timestamp {}", unix_time).into())
}

pub fn datetime_to_bytes(ts: DateTime<Utc>) -> Result<[u8; 4], Box<dyn Error>> {
    Ok(i32::try_from(ts.timestamp())?.to_be_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_measurement() {
        let ts = datetime_to_bytes(Utc::now())
            .and_then(datetime_from_bytes)
            .unwrap(); // to floor to seconds
        let meas_data = [
            &[0x3A, 0x30, 0x10],
            datetime_to_bytes(ts).unwrap().as_slice(),
            &[0x00, 0x00, 0x00, 0x85],
        ]
        .concat();
        let meas_exp = Measurement::Temperature(ts, 1.33);
        assert_eq!(
            Measurement::from_bytes(&meas_data).unwrap().unwrap(),
            meas_exp
        )
    }
}
