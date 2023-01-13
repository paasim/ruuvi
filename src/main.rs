use config::Config;
use std::{env, process};

mod bt;
mod cacher;
mod config;
mod ruuvi;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|e| {
        eprintln!("Application error: {}", e);
        process::exit(1);
    });
    bt::scan(config).unwrap_or_else(|e| {
        eprintln!("Application error: {}", e);
        process::exit(1);
    });
}
