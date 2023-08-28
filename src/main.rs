#![feature(buf_read_has_data_left)]
#![feature(iter_next_chunk)]

use std::io::Write;

mod cli;
pub mod de;
pub mod error;
pub mod file_formats;
pub mod ser;

pub(crate) const MAGIC: &[u8; 8] = b"\x00\x00\x00\x00\x00\x1A\xB1\x26";

#[repr(i8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum DataType {
    Boolean = 0,
    Int = 1,
    Long = 2,
    UTF = 3,
    Double = 4,
    Short = 5,
    Float = 6,
    Byte = 7,
    Char = 9,
    ObjectBegin = -2,
    ObjectEnd = -1,
}

impl TryFrom<i8> for DataType {
    type Error = error::Error;
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Boolean,
            1 => Self::Int,
            2 => Self::Long,
            3 => Self::UTF,
            4 => Self::Double,
            5 => Self::Short,
            6 => Self::Float,
            7 => Self::Byte,
            9 => Self::Char,
            -2 => Self::ObjectBegin,
            -1 => Self::ObjectEnd,
            _ => {
                return Err(Self::Error::UnknownType(value));
            }
        })
    }
}

impl TryFrom<u8> for DataType {
    type Error = error::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from(value as i8)
    }
}

fn main() -> Result<(), std::io::Error> {
    let yjr = file_formats::example_files::yjr_file_1();

    let serialized = ser::to_bytevec(&yjr).unwrap();

    let deserialized: file_formats::KRDSFileTypes = de::from_slice(&serialized).unwrap();

    let mut stdout = std::io::stdout();
    stdout.write_all(&serialized)?;
    stdout.flush()?;
    Ok(())
}
