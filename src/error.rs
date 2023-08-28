use std::fmt::{self, Display};

use serde::{de, ser};

use crate::DataType;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Eof,
    UnknownType(i8),
    ReadError(std::io::Error),
    BadMagic,
    BadValue,
    Unexpected(DataType),
    Expected { want: DataType, got: DataType, pos: usize },
    ExpectedIntervalTree,
    TrailingBytes,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::UnknownType(i) => formatter.write_fmt(format_args!("unknown data type {}", i)),
            Error::ReadError(e) => formatter.write_str(&e.to_string()),
            _ => formatter.write_fmt(format_args!("{:?}", self)),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::ReadError(value)
    }
}
