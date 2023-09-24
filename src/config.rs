use std::{env, error::Error};
use bluez_async::MacAddress;

#[derive(Debug)]
pub struct Config {
    pub endpoint: Option<String>,
    pub macs: Vec<MacAddress>,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, Box<dyn Error>> {
        args.next();
        let url = args.next().ok_or("No url specified")?;
        let endpoint = if url == "print-only" { None } else { Some(url) };

        let macs = args.into_iter().map(|a| a.parse()).collect::<Result<Vec<_>, _>>()?;

        if macs.is_empty() {
            return Err("Must have 1 or more mac addresses".into());
        }

        Ok(Config { endpoint, macs })
    }

    pub fn destr(self) -> (Option<String>, Vec<MacAddress>) {
        (self.endpoint, self.macs)
    }
}
