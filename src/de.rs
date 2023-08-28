use std::io::Read;

use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;

use crate::error::{Error, Result};

use crate::DataType;

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    // That way basic use cases are satisfied by something like
    // `serde_json::from_str(...)` while advanced use cases that require a
    // deserializer can make one with `serde_json::Deserializer::from_str(...)`.
    pub fn from_slice(input: &'de [u8]) -> Self {
        Deserializer { input }
    }
}

// By convention, the public API of a Serde deserializer is one or more
// `from_xyz` methods such as `from_str`, `from_bytes`, or `from_reader`
// depending on what Rust types the deserializer is able to consume as input.
//
// This basic deserializer supports only `from_str`.
pub fn from_slice<'a, T>(b: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    if &b[..crate::MAGIC.len()] != crate::MAGIC
        && &b[crate::MAGIC.len()..crate::MAGIC.len() + 9]
            == &[DataType::Long as u8, 00, 00, 00, 00, 00, 00, 00, 01]
    {
        return Err(Error::BadMagic);
    }

    let mut deserializer = Deserializer::from_slice(&b[crate::MAGIC.len() + 9..]);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingBytes)
    }
}

impl<'de> Deserializer<'de> {
    fn peek_byte(&mut self) -> Result<u8> {
        Ok(self.input.bytes().next().ok_or(Error::Eof)??)
    }

    fn next_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.input = &self.input[1..];
        Ok(byte)
    }

    fn next_i8(&mut self) -> Result<i8> {
        Ok(self.next_byte()? as i8)
    }

    fn get_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.input.len() < N {
            return Err(Error::Eof);
        }
        let buf: [u8; N] = *&self.input[0..N].try_into().unwrap();
        self.input = &self.input[8..];
        Ok(buf)
    }

    fn get_slice(&mut self, count: usize) -> Result<&[u8]> {
        if self.input.len() < count {
            return Err(Error::Eof);
        }
        let slice = &self.input[..count];
        self.input = &self.input[count..];
        Ok(slice)
    }

    fn parse_string(&mut self) -> Result<&str> {
        let mut value = "";
        if self.next_byte()? != 1 {
            let length = u16::from_be_bytes(self.get_array()?) as usize;
            value = std::str::from_utf8(self.get_slice(length)?).unwrap();
        }
        Ok(value)
    }

    fn parse_i32(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.get_array()?))
    }

    fn next_datatype(&mut self) -> Result<DataType> {
        self.next_byte()?.try_into()
    }

    fn peek_next_datatype(&mut self) -> Result<DataType> {
        self.peek_byte()?.try_into()
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 f32 f64 char string
        option unit unit_struct
    }

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let datatype: DataType = self.peek_byte()?.try_into()?;

        match datatype {
            DataType::Boolean => {
                visitor.visit_bool(if self.next_byte()? == 0 { false } else { true })
            }
            DataType::Int => visitor.visit_i32(self.parse_i32()?),
            DataType::Long => visitor.visit_i64(i64::from_be_bytes(self.get_array()?)),
            DataType::UTF => visitor.visit_string(self.parse_string()?.to_string()),
            DataType::Double => visitor.visit_f64(f64::from_be_bytes(self.get_array()?)),
            DataType::Short => visitor.visit_i16(i16::from_be_bytes(self.get_array()?)),
            DataType::Float => visitor.visit_f32(f32::from_be_bytes(self.get_array()?)),
            DataType::Byte => visitor.visit_i8(self.next_byte()? as i8),
            DataType::Char => visitor.visit_char(self.next_byte()? as char),
            DataType::ObjectBegin => self.deserialize_map(visitor),
            DataType::ObjectEnd => Err(Error::UnexpectedObjectEnd),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.next_byte()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(u16::from_be_bytes(self.get_array()?))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(u32::from_be_bytes(self.get_array()?))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(u64::from_be_bytes(self.get_array()?))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.parse_string()?)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Parse the opening bracket of the sequence.
        if self.next_i8()? == -2 && self.parse_string()? == "saved.avl.interval.tree" {
            let value = visitor.visit_seq(Terminated::new(self))?;
            if self.peek_next_datatype()? == DataType::ObjectEnd {
                Ok(value)
            } else {
                Err(Error::ExpectedObjectEnd)
            }
        } else {
            Err(Error::ExpectedObject)
        }
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.next_datatype()? == DataType::Int {
            let length = self.parse_i32()? as usize;
            visitor.visit_map(LengthBased::new(self, length))
        } else {
            Err(Error::ExpectedInt)
        }
    }

    // Structs look just like maps in JSON.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let next = self.peek_next_datatype()?;
        if next == DataType::UTF {
            self.next_byte()?;
            visitor.visit_enum(self.parse_string()?.into_deserializer())
        } else if next == DataType::Int {
            self.deserialize_map(visitor)
        } else if next == DataType::ObjectBegin {
            let value = visitor.visit_enum(Enum::new(self))?;
            if self.next_datatype()? == DataType::ObjectEnd {
                Ok(value)
            } else {
                Err(Error::ExpectedObjectEnd)
            }
        } else {
            Err(Error::ExpectedObject)
        }
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct LengthBased<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    total: usize,
    done: usize,
}

impl<'a, 'de> LengthBased<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, total: usize) -> Self {
        LengthBased { de, total, done: 0 }
    }
}

impl<'de, 'a> MapAccess<'de> for LengthBased<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.done == self.total {
            Ok(None)
        } else {
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.done += 1;
        seed.deserialize(&mut *self.de)
    }
}

impl<'de, 'a> SeqAccess<'de> for LengthBased<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.done == self.total {
            return Ok(None);
        }

        self.done += 1;

        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct Terminated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Terminated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Terminated { de }
    }
}

impl<'de, 'a> SeqAccess<'de> for Terminated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.peek_next_datatype()? == DataType::ObjectEnd {
            return Ok(None);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((seed.deserialize(&mut *self.de)?, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedString)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = visitor.visit_seq(Terminated::new(self.de))?;
        if self.de.next_datatype()? == DataType::ObjectEnd {
            Ok(value)
        } else {
            Err(Error::ExpectedObjectEnd)
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::ExpectedStruct)
    }
}
