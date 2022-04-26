use chrono::Utc;
use chrono_tz::Europe::Helsinki;
use std::collections::HashSet;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub endpoint: String,
    pub macs: Vec<String>,
    pub print_only: bool,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        let mut print_only = false;
        let mut macs = Vec::new();
        args.next();
        let mut nxt = args.next();
            //"E7:4A:20:38:55:7F".to_string(),
            //"C2:40:1D:B7:C4:C8".to_string(),
        if nxt == Some("--echo".to_string()) {
            print_only = true;
            nxt = args.next();
        }
        let endpoint = nxt.ok_or("No url specified")?;

        while let Some(val) = args.next() {
            macs.push(val.to_uppercase());
        }
        if macs.is_empty() {
            return Err("Must have 1 or more mac addresses");
        }

        Ok(Config {
            endpoint,
            macs,
            print_only,
        })
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
