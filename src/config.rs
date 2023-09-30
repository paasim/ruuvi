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
            .map(|s| Ok(s.parse()?))
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

fn get_usage(program_name: &str) -> String {
    format!(
        "usage: {} [--log mac n_hours | --latest mac1 mac2 ...]",
        program_name
    )
}
