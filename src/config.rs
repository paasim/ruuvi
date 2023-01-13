use std::{env, error::Error};

#[derive(Debug)]
pub struct Config {
    pub endpoint: Option<String>,
    pub macs: Vec<String>,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, Box<dyn Error>> {
        let mut macs = Vec::new();
        args.next();
        let url = args.next().ok_or("No url specified")?;
        let endpoint = if url == "print-only" { None } else { Some(url) };

        while let Some(val) = args.next() {
            macs.push(val.to_uppercase());
        }

        if macs.is_empty() {
            return Err("Must have 1 or more mac addresses".into());
        }

        Ok(Config { endpoint, macs })
    }

    pub fn destr(self) -> (Option<String>, Vec<String>) {
        (self.endpoint, self.macs)
    }
}
