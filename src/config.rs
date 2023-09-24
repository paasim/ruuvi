use bluez_async::MacAddress;
use std::{env, error::Error};

#[derive(Debug)]
pub struct Config {
    pub macs: Option<Vec<MacAddress>>,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, Box<dyn Error>> {
        args.next();
        match args.next().as_ref().map(|s| s.as_str()) {
            Some("--macs") => Ok(Self {
                macs: Some(get_macs(args)?),
            }),
            Some(s) => Err(format!("unexpected argument {}", s).into()),
            _ => Ok(Self { macs: None }),
        }
    }
}

fn get_macs(args: env::Args) -> Result<Vec<MacAddress>, Box<dyn Error>> {
    Ok(args
        .into_iter()
        .map(|a| a.parse())
        .collect::<Result<Vec<_>, _>>()?)
}
