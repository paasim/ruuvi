use crate::err::Res;
use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq)]
pub enum Measurement {
    Temp(DateTime<Utc>, f64),
    Hum(DateTime<Utc>, f64),
    AirPres(DateTime<Utc>, u32),
    EndOfMeasurements,
}

impl Measurement {
    // async to make it easier to use with filter_map
    pub async fn from_11_bytes(v: Vec<u8>) -> Option<Res<Self>> {
        if v.len() != 11 {
            None
        } else {
            Some(Measurement::from_bytes(v))
        }
    }

    pub fn from_bytes(obs: impl AsRef<[u8]>) -> Res<Self> {
        let obs = obs.as_ref();
        if obs.len() != 11 {
            Err(format!("payload length should be 11 (was {})", obs.len()))?
        }
        if obs[0] != 0x3A {
            Err(format!(
                "header should start with 0x3A (was 0x{:x})",
                obs[0]
            ))?
        }
        if obs[2] != 0x10 {
            Err(format!("action should be read (0x10, was 0x{:x})", obs[0]))?
        }
        if obs[1] == 0x3A && obs[3..11] == [0xFF; 8] {
            return Ok(Measurement::EndOfMeasurements);
        }
        let ts = datetime_from_bytes([obs[3], obs[4], obs[5], obs[6]])?;
        let val = [obs[7], obs[8], obs[9], obs[10]];
        match obs[1] {
            0x30 => Ok(Self::Temp(ts, i32::from_be_bytes(val) as f64 * 0.01)),
            0x31 => Ok(Self::Hum(ts, u32::from_be_bytes(val) as f64 * 0.01)),
            0x32 => Ok(Self::AirPres(ts, u32::from_be_bytes(val))),
            _ => Err(format!("invalid observation type {:x}", obs[1]))?,
        }
    }
}

pub fn datetime_from_bytes(ts: [u8; 4]) -> Res<DateTime<Utc>> {
    let unix_time = u32::from_be_bytes(ts);
    DateTime::<Utc>::from_timestamp(unix_time.into(), 0)
        .ok_or(format!("invalid timestamp {}", unix_time).into())
}

pub fn datetime_to_bytes(ts: DateTime<Utc>) -> Res<[u8; 4]> {
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
        let meas_exp = Measurement::Temp(ts, 1.33);
        assert_eq!(Measurement::from_bytes(meas_data).unwrap(), meas_exp)
    }
}
