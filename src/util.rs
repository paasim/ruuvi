use chrono::Utc;
use chrono_tz::Europe::Helsinki;
use std::collections::HashSet;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub endpoint: Option<String>,
    pub macs: Vec<String>,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        let mut macs = Vec::new();
        args.next();
        let url = args.next().ok_or("No url specified")?;
        let endpoint = if url == "print-only" { None } else { Some(url) };

        while let Some(val) = args.next() {
            macs.push(val.to_uppercase());
        }

        if macs.is_empty() {
            return Err("Must have 1 or more mac addresses");
        }

        Ok(Config { endpoint, macs })
    }
}

pub struct Cacher<'a> {
    unseen: HashSet<&'a String>,
}

impl Cacher<'_> {
    pub fn new(to_be_seen: &Vec<String>) -> Result<Cacher, &str> {
        if to_be_seen.is_empty() {
            return Err("Must have 1 or more mac addresses");
        }

        let mut unseen = HashSet::new();
        for v in to_be_seen.iter() {
            unseen.insert(v);
        }
        Ok(Cacher { unseen })
    }
    pub fn see(&mut self, s: &String) -> bool {
        self.unseen.remove(s)
    }
    pub fn all_seen(&self) -> bool {
        self.unseen.is_empty()
    }
}

pub fn timestamp() -> String {
    Utc::now().with_timezone(&Helsinki).to_rfc3339()
}
