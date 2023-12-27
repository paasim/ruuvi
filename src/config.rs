use macaddr::MacAddr6;
use ruuvi::err::Res;
use std::env::Args;

#[derive(Debug)]
pub enum Config {
    Latest(Vec<MacAddr6>),
    Log(MacAddr6, u8),
    Scan,
}

impl Config {
    pub fn new(mut args: Args) -> Res<Config> {
        let progname = args.next().ok_or("arguments missing")?;
        match args.next().as_deref() {
            Some("--latest") => Self::latest_config(args),
            Some("--log") => Self::log_config(args, &progname),
            Some(_) => Err(get_usage(&progname))?,
            None => Ok(Self::Scan),
        }
    }

    fn latest_config(args: Args) -> Res<Self> {
        args.into_iter()
            .map(|s| Ok(s.parse()?))
            .collect::<Res<Vec<_>>>()
            .map(Self::Latest)
    }

    fn log_config(mut args: Args, progname: &str) -> Res<Self> {
        Ok(Config::Log(
            args.next().ok_or(get_usage(progname))?.parse()?,
            args.next().ok_or(get_usage(progname))?.parse()?,
        ))
    }
}

fn get_usage(program_name: &str) -> String {
    format!(
        "usage: {} [--log mac n_hours | --latest mac1 mac2 ...]",
        program_name
    )
}
