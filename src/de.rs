use std::io::Read;

use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;

use crate::error::{Error, Result};

use crate::DataType;

#[derive(Debug)]
pub struct Deserializer<'de> {
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    pub fn from_slice(input: &'de [u8]) -> Self {
        Deserializer { input }
    }
}

pub fn from_slice<'a, T>(b: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    if b.len() < crate::MAGIC.len() + 5 {
        return Err(Error::Eof);
    }

    let magic = &b[..crate::MAGIC.len()];

    if magic != crate::MAGIC {
        return Err(Error::BadMagic);
    }

    let mut deserializer = Deserializer::from_slice(&b[crate::MAGIC.len()..]);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingBytes)
    }
}

impl<'de> Deserializer<'de> {
    fn peek_byte(&mut self) -> Result<u8> {
        let byte = self.input.bytes().next().ok_or(Error::Eof)??;
        Ok(byte)
    }

    fn next_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.input = &self.input[1..];
        Ok(byte)
    }

    fn get_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.input.len() < N {
            return Err(Error::Eof);
        }
        let buf: [u8; N] = *&self.input[0..N].try_into().unwrap();
        self.input = &self.input[N..];
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
        let value = i32::from_be_bytes(self.get_array()?);
        Ok(value)
    }

    fn next_datatype(&mut self) -> Result<DataType> {
        self.next_byte()?.try_into()
    }

    fn parse_type(&mut self, datatype: DataType) -> Result<()> {
        if self.next_datatype()? != datatype {
            Err(Error::Expected(datatype))
        } else {
            Ok(())
        }
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let datatype: DataType = self.next_datatype()?;

        match datatype {
            DataType::Boolean => {
                visitor.visit_bool(if self.next_byte()? == 0 { false } else { true })
            }
            DataType::Int => visitor.visit_i32(self.parse_i32()?),
            DataType::Long => visitor.visit_i64(i64::from_be_bytes(self.get_array()?)),
            DataType::String => visitor.visit_string(self.parse_string()?.to_string()),
            DataType::Double => visitor.visit_f64(f64::from_be_bytes(self.get_array()?)),
            DataType::Short => visitor.visit_i16(i16::from_be_bytes(self.get_array()?)),
            DataType::Float => visitor.visit_f32(f32::from_be_bytes(self.get_array()?)),
            DataType::Byte => visitor.visit_i8(self.next_byte()? as i8),
            DataType::Char => visitor.visit_char(self.next_byte()? as char),
            DataType::FieldBegin => todo!(),
            DataType::FieldEnd => Err(Error::Unexpected(DataType::FieldEnd)),
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

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let next = self.peek_next_datatype()?;
        if next == DataType::FieldBegin {
            self.next_byte()?;
            if self.parse_string()? == "saved.avl.interval.tree" {
                self.parse_type(DataType::Int)?;
                let length = self.parse_i32()? as usize;
                let value = visitor.visit_seq(LengthBased::new(self, length))?;
                self.parse_type(DataType::FieldEnd)?;
                Ok(value)
            } else {
                Err(Error::ExpectedIntervalTree)
            }
        } else {
            visitor.visit_seq(Terminated::new(self))
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(LengthBased::new(self, len))
    }

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

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Int)?;
        let length = self.parse_i32()? as usize;
        visitor.visit_map(LengthBased::new(self, length))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Int)?;
        let length = self.parse_i32()? as usize;
        visitor.visit_map(LengthBasedStruct::new(self, length))
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
        if next == DataType::String {
            self.next_byte()?;
            visitor.visit_enum(self.parse_string()?.into_deserializer())
        } else if next == DataType::Int {
            todo!()
        } else if next == DataType::FieldBegin {
            let value = visitor.visit_enum(Enum::new(self))?;
            self.parse_type(DataType::FieldEnd)?;
            Ok(value)
        } else {
            Err(Error::Unexpected(next))
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

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

struct LengthBasedStruct<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    total: usize,
    done: usize,
}

impl<'a, 'de> LengthBasedStruct<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, total: usize) -> Self {
        Self { de, total, done: 0 }
    }
}

impl<'a, 'de> LengthBased<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, total: usize) -> Self {
        Self { de, total, done: 0 }
    }
}
impl<'de, 'a> MapAccess<'de> for LengthBasedStruct<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.done == self.total {
            Ok(None)
        } else {
            self.de.parse_type(DataType::FieldBegin)?;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.done += 1;
        let value = seed.deserialize(&mut *self.de)?;
        self.de.parse_type(DataType::FieldEnd)?;
        Ok(value)
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
        if self.de.peek_next_datatype()? == DataType::FieldEnd {
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
        if self.de.next_datatype()? == DataType::FieldBegin {
            Ok((seed.deserialize(&mut *self.de)?, self))
        } else {
            todo!()
        }
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::Expected(DataType::String))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = visitor.visit_seq(LengthBased::new(self.de, len))?;
        self.de.parse_type(DataType::FieldEnd)?;
        Ok(value)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }
}
