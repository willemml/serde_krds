#![feature(buf_read_has_data_left)]
#![feature(iter_next_chunk)]

use std::io::Write;

mod cli;
pub mod de;
pub mod error;
pub mod file_formats;
pub mod ser;

pub(crate) const MAGIC: &[u8; 17] =
    b"\x00\x00\x00\x00\x00\x1A\xB1\x26\x02\x00\x00\x00\x00\x00\x00\x00\x01";

#[repr(i8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DataType {
    Boolean = 0,
    Int = 1,
    Long = 2,
    String = 3,
    Double = 4,
    Short = 5,
    Float = 6,
    Byte = 7,
    Char = 9,
    FieldBegin = -2,
    FieldEnd = -1,
}

impl TryFrom<i8> for DataType {
    type Error = error::Error;
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Boolean,
            1 => Self::Int,
            2 => Self::Long,
            3 => Self::String,
            4 => Self::Double,
            5 => Self::Short,
            6 => Self::Float,
            7 => Self::Byte,
            9 => Self::Char,
            -2 => Self::FieldBegin,
            -1 => Self::FieldEnd,
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
    let mut stdout = std::io::stdout();

    let serialized = include_bytes!("../../testfiles/krds/pdfannot.yjr");//pdfannot.yjr");


    let deserialized: file_formats::TimerDataFile = de::from_slice(serialized).unwrap();

    stdout.write_all(&ser::to_bytevec(&deserialized).unwrap())?;

    //stdout.write_fmt(format_args!("{:#?}", &deserialized))?;

    stdout.flush()?;
    Ok(())
}
