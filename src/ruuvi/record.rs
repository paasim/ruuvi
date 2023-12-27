use super::Measurement;
use crate::err::{Error, Res};
use chrono::{DateTime, Utc};
use serde::{Serialize, Serializer};

#[derive(Debug, PartialEq, Serialize)]
pub struct Record {
    #[serde(serialize_with = "ser_dt")]
    pub datetime: DateTime<Utc>,
    pub temperature: f64,
    pub humidity: f64,
    pub air_pressure: u32,
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // to_string really should not fail...
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

fn ser_dt<S: Serializer>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&dt.to_rfc3339())
}

impl Record {
    pub fn from_chunk<V: AsRef<[Measurement]>>(chunk: V) -> Res<Self> {
        match chunk.as_ref() {
            [m1, m2, m3] => Self::try_from((m1, m2, m3)),
            v => Err(format!("Invalid chunk of size {}", v.len()))?,
        }
    }
}

impl TryFrom<(&Measurement, &Measurement, &Measurement)> for Record {
    type Error = Error;

    fn try_from(value: (&Measurement, &Measurement, &Measurement)) -> Res<Self> {
        let (tst, tsh, tsa, t, h, a) = match value {
            (Measurement::Temp(tst, t), Measurement::Hum(tsh, h), Measurement::AirPres(tsa, a)) => {
                (tst, tsh, tsa, t, h, a)
            }
            (Measurement::Temp(tst, t), Measurement::AirPres(tsa, a), Measurement::Hum(tsh, h)) => {
                (tst, tsh, tsa, t, h, a)
            }
            (Measurement::Hum(tsh, h), Measurement::Temp(tst, t), Measurement::AirPres(tsa, a)) => {
                (tst, tsh, tsa, t, h, a)
            }
            (Measurement::Hum(tsh, h), Measurement::AirPres(tsa, a), Measurement::Temp(tst, t)) => {
                (tst, tsh, tsa, t, h, a)
            }
            (Measurement::AirPres(tsa, a), Measurement::Temp(tst, t), Measurement::Hum(tsh, h)) => {
                (tst, tsh, tsa, t, h, a)
            }
            (Measurement::AirPres(tsa, a), Measurement::Hum(tsh, h), Measurement::Temp(tst, t)) => {
                (tst, tsh, tsa, t, h, a)
            }
            _ => Err("not a valid measurement triplet")?,
        };
        if tst != tsh || tst != tsa {
            Err("conflicting timestamps")?;
        }
        Ok(Record {
            datetime: *tst,
            temperature: *t,
            humidity: *h,
            air_pressure: *a,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_log() {
        let datetime = Utc::now();
        let temperature = 15.0;
        let humidity = 40.0;
        let air_pressure = 10000;
        let temp = Measurement::Temp(datetime, temperature);
        let hum = Measurement::Hum(datetime, humidity);
        let air_pres = Measurement::AirPres(datetime, air_pressure);
        let log_exp = Record {
            datetime,
            temperature,
            humidity,
            air_pressure,
        };
        let log = Record::try_from((&temp, &hum, &air_pres)).unwrap();
        assert_eq!(log, log_exp)
    }
}
