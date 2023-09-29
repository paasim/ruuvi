use config::Config;
use std::{env, process};

mod advertisement;
mod config;
mod log;
mod log_record;
mod ruuvi;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    match config {
        Config::Latest(v) => advertisement::scan(Some(v)),
        Config::Log(mac, n) => log::read(mac, n),
        Config::Scan => advertisement::scan(None),
    }
    .unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });
}
