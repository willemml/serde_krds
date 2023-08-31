//! Serializer and deserializer implementation for Amazon's KRDS
//! format (used by Kindle e-readers to store user reading data.)
//!
//! Warning, some types are fragile, for example Tuple Structs cannot
//! contain optionals anywhere except at the end. They will fail to
//! deserialize if the optional is none if this rule is not followed.
//! More stable implementations may be created as needs arise and I
//! understand serde more.

pub mod de;
pub mod error;
pub mod ser;

pub use de::{from_bytes, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_bytes, Serializer};

#[cfg(test)]
mod test;

/// All KRDS files start with the following magic bytes (magic number
/// followed by a 1_u64.)
pub(crate) const MAGIC: &[u8; 17] =
    b"\x00\x00\x00\x00\x00\x1A\xB1\x26\x02\x00\x00\x00\x00\x00\x00\x00\x01";

/// Map of data type specifiers to the name of the types they
/// represent.
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
    type Error = Error;

    fn try_from(value: i8) -> std::result::Result<Self, Self::Error> {
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
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        Self::try_from(value as i8)
    }
}
