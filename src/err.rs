use std::{error, fmt, num};

#[derive(Debug)]
pub enum Error {
    Bluer(bluer::Error),
    Other(String),
    Parse(String),
}

pub type Res<T> = Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Bluer(e) => write!(f, "{}", e),
            Error::Other(e) => write!(f, "{}", e),
            Error::Parse(e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for Error {}

impl From<bluer::Error> for Error {
    fn from(value: bluer::Error) -> Self {
        Self::Bluer(value)
    }
}

impl From<macaddr::ParseError> for Error {
    fn from(value: macaddr::ParseError) -> Self {
        Self::Parse(value.to_string())
    }
}

impl From<num::ParseIntError> for Error {
    fn from(value: num::ParseIntError) -> Self {
        Self::Parse(value.to_string())
    }
}

impl From<num::TryFromIntError> for Error {
    fn from(value: num::TryFromIntError) -> Self {
        Self::Parse(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Other(value.to_owned())
    }
}
