use config::Config;
use ruuvi::advertisements;
use std::{env, process};

mod config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    match config {
        Config::Latest(v) => ruuvi::advertisements::print_advertisements(Some(v)),
        Config::Log(mac, n) => ruuvi::log::print_log(mac, n),
        Config::Scan => advertisements::print_advertisements(None),
    }
    .unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });
}
