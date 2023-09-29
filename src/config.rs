use macaddr::MacAddr6;
use std::env::Args;
use std::error::Error;

#[derive(Debug)]
pub enum Config {
    Latest(Vec<MacAddr6>),
    Log(MacAddr6, u8),
    Scan,
}

impl Config {
    pub fn new(mut args: Args) -> Result<Config, Box<dyn Error>> {
        let progname = args.next().ok_or(String::from("arguments missing"))?;
        match args.next().as_ref().map(|s| s.as_str()) {
            Some("--latest") => Self::latest_config(args),
            Some("--log") => Self::log_config(args, &progname),
            Some(_) => Err(get_usage(&progname).into()),
            None => Ok(Self::Scan),
        }
    }

    fn latest_config(args: Args) -> Result<Self, Box<dyn Error>> {
        args.into_iter()
            .map(|s| Ok(s.parse()?)) //MacAddr6::from_str(&s))//parse_mac(&s))
            .collect::<Result<Vec<_>, _>>()
            .map(Self::Latest)
    }

    fn log_config(mut args: Args, progname: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Config::Log(
            args.next().ok_or(get_usage(&progname))?.parse()?,
            args.next().ok_or(get_usage(&progname))?.parse()?,
        ))
    }
}

/*
fn parse_mac(s: &str) -> Result<[u8; 6], Box<dyn Error>> {
    let v = s
        .split(':')
        .map(|s| u8::from_str_radix(s, 16))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(v.as_slice().try_into()?)
}
    */

fn get_usage(progname: &str) -> String {
    format!(
        "usage: {} [--log mac n_days | --latest mac1 mac2 ...]",
        progname
    )
}
