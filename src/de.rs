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
    counter: usize,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { input, counter: 0 }
    }
}

pub fn from_bytes<'a, T>(b: &'a [u8]) -> Result<T>
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

    let mut deserializer = Deserializer::from_bytes(&b[crate::MAGIC.len()..]);

    deserializer.counter = crate::MAGIC.len();

    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingBytes)
    }
}

impl<'de> Deserializer<'de> {
    /// Does not check for EOF, make sure to check before calling.
    fn consume_unchecked(&mut self, count: usize) {
        self.input = &self.input[count..];
        self.counter += count;
    }

    fn peek_byte(&mut self) -> Result<u8> {
        let byte = self.input.bytes().next().ok_or(Error::Eof)??;
        Ok(byte)
    }

    fn next_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.consume_unchecked(1);
        Ok(byte)
    }

    fn get_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.input.len() < N {
            return Err(Error::Eof);
        }
        let buf: [u8; N] = *&self.input[0..N].try_into().unwrap();
        self.consume_unchecked(N);
        Ok(buf)
    }

    fn get_slice(&mut self, count: usize) -> Result<&[u8]> {
        if self.input.len() < count {
            return Err(Error::Eof);
        }
        let slice = &self.input[..count];
        self.consume_unchecked(count);
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
        let next = self.next_datatype()?;
        if next != datatype {
            Err(Error::Expected {
                want: datatype,
                got: next,
                pos: self.counter,
            })
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_next_datatype()? {
            DataType::Boolean => self.deserialize_bool(visitor),
            DataType::Int => self.deserialize_i32(visitor),
            DataType::Long => self.deserialize_i64(visitor),
            DataType::String => self.deserialize_string(visitor),
            DataType::Double => self.deserialize_f64(visitor),
            DataType::Short => self.deserialize_i16(visitor),
            DataType::Float => self.deserialize_f32(visitor),
            DataType::Byte => self.deserialize_i8(visitor),
            DataType::Char => self.deserialize_char(visitor),
            _ => Err(Error::WontImplement),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Boolean)?;
        visitor.visit_bool(if self.next_byte()? == 0 { false } else { true })
    }

    fn deserialize_i8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Byte)?;
        visitor.visit_i8(self.next_byte()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Short)?;
        visitor.visit_i16(i16::from_be_bytes(self.get_array()?))
    }

    fn deserialize_i32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Int)?;
        visitor.visit_i32(self.parse_i32()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Long)?;
        visitor.visit_i64(i64::from_be_bytes(self.get_array()?))
    }

    fn deserialize_f32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Float)?;
        visitor.visit_f32(f32::from_be_bytes(self.get_array()?))
    }

    fn deserialize_f64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Double)?;
        visitor.visit_f64(f64::from_be_bytes(self.get_array()?))
    }

    fn deserialize_char<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Char)?;
        visitor.visit_char(self.next_byte()? as char)
    }

    fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::String)?;
        visitor.visit_string(self.parse_string()?.to_string())
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Byte)?;
        visitor.visit_u8(self.next_byte()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Short)?;
        visitor.visit_u16(u16::from_be_bytes(self.get_array()?))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Int)?;
        visitor.visit_u32(u32::from_be_bytes(self.get_array()?))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Long)?;
        visitor.visit_u64(u64::from_be_bytes(self.get_array()?))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str((self.parse_string())?)
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

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.peek_next_datatype()? == DataType::FieldEnd {
            dbg!("me");
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::FieldBegin)?;
        self.parse_string()?;
        let value = visitor.visit_newtype_struct(&mut *self)?;
        self.parse_type(DataType::FieldEnd)?;
        Ok(value)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_type(DataType::Int)?;
        let length = self.parse_i32()? as usize;
        let value = visitor.visit_seq(LengthBased::new(self, length))?;
        Ok(value)
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
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = visitor.visit_seq(Terminated::new(self, Some(len)))?;
        Ok(value)
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
            Err(Error::Unexpected {
                want: None,
                got: next,
                pos: self.counter,
            })
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

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }
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

impl<'de, 'a> MapAccess<'de> for LengthBasedStruct<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.done > 0 {
            self.de.parse_type(DataType::FieldEnd)?;
        }
        if self.done == self.total {
            Ok(None)
        } else {
            self.done += 1;
            self.de.parse_type(DataType::FieldBegin)?;
            let key = DeserializeSeed::deserialize(seed, &mut *self.de)?;
            Ok(Some(key))
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.total - self.done)
    }
}

struct LengthBased<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    total: usize,
    done: usize,
}

impl<'a, 'de> LengthBased<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, total: usize) -> Self {
        Self { de, total, done: 0 }
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

    fn size_hint(&self) -> Option<usize> {
        Some(self.total - self.done)
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
    done: usize,
    total: Option<usize>,
}

impl<'a, 'de> Terminated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, len: Option<usize>) -> Self {
        Terminated {
            de,
            done: 0,
            total: len,
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for Terminated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.peek_next_datatype()? == DataType::FieldEnd {
            if let Some(total) = self.total {
                if self.done == total {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }
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
        Err(Error::Message("Should already be parsed".to_string()))
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

#[cfg(test)]
mod test {
    use linked_hash_map::LinkedHashMap;

    use super::*;
    use crate::DataType;

    use kindle_formats::krds::*;

    use crate::test::*;

    macro_rules! de_test {
        {$($type:ty => $name:ident $getter:ident),+} => {
            $(#[test]
            fn $name() {
                let (bytes, data) = $getter();
                assert_eq!(de_no_magic::<$type>(&bytes), data)
            })+
        };
    }

    macro_rules! de_num_test {
        {$($num:expr => $name:ident $dtype:expr),+} => {
            $(#[test]
              fn $name() {
                  let bytes = test_num($num, $dtype);
                  assert!($num == de_no_magic(&bytes))
            })+
        };
    }

    #[test]
    fn pdfannot_yjr_de() {
        assert_eq!(
            from_bytes::<ReaderDataFile>(PDFANNOT_YJR).unwrap(),
            pdfannot_yjr()
        );
    }

    #[test]
    fn pdfannot_yjf_de() {
        assert_eq!(
            from_bytes::<TimerDataFile>(PDFANNOT_YJF).unwrap(),
            pdfannot_yjf()
        );
    }

    de_num_test! {
        117_i8 => de_i8 DataType::Byte,
        2004_i16 => de_i16 DataType::Short,
        65555_i32 => de_i32 DataType::Int,
        4294967300_i64 => de_i64 DataType::Long,
        3.14_f32 => de_f32 DataType::Float,
        1293842345.00000000213_f64 => de_f64 DataType::Double
    }

    de_test! {
        SimpleStruct => simple_struct_de simple_struct,
        PHRWrapper => simple_newtype_de simple_newtype,
        String => string_de test_string,
        String => empty_string_de empty_string,
        Vec<i32> => int_vec_de test_vec_int,
        Vec<String> => string_vec_de test_vec_strings,
        LinkedHashMap<NoteType, String> => map_de test_map,
        VecMapStruct => vec_map_struct_de vec_map_struct
    }
}
