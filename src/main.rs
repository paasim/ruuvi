use crate::util::Config;
use std::env;
use std::process;
mod bt;
mod request;
mod ruuvi;
mod util;

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
