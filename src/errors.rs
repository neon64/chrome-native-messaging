use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("serde error")]
    Serde(#[from] serde_json::Error),
    #[error("message too large: {size:?} bytes")]
    MessageTooLarge {
        size: usize
    },
    #[error("the input stream reached the end")]
    NoMoreInput
}