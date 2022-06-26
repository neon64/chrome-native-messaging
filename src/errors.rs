use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Serde(serde_json::Error),
    MessageTooLarge { size: usize },
    NoMoreInput,
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serde(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => {
                f.write_str("io error: ")?;
                err.fmt(f)
            }
            Error::Serde(err) => {
                f.write_str("serde error: ")?;
                err.fmt(f)
            }
            Error::MessageTooLarge { size } => {
                f.write_fmt(format_args!("message too large: {:?} bytes", size))
            }
            Error::NoMoreInput => f.write_str("the input stream reached the end"),
        }
    }
}

impl std::error::Error for Error {}
