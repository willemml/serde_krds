use std::io::Write;

use byteorder::WriteBytesExt;
use serde::{
    ser::{self, Impossible, SerializeSeq},
    Serialize,
};

use crate::error::{Error, Result};

pub struct Serializer {
    // This string starts empty and JSON is appended as values are serialized.
    output: Vec<u8>,
}

// By convention, the public API of a Serde serializer is one or more `to_abc`
// functions such as `to_string`, `to_bytes`, or `to_writer` depending on what
// Rust types the serializer is able to produce as output.
//
// This basic serializer supports only `to_string`.
pub fn to_bytevec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut output = Vec::from(crate::MAGIC.clone().as_slice());
    output.write_dtype(DataType::Long)?;
    output.write_all(&1i64.to_be_bytes())?;

    let mut serializer = Serializer { output };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

trait WriteDataType {
    fn write_dtype(&mut self, dtype: DataType) -> Result<()>;
}

impl<W: Write> WriteDataType for W {
    fn write_dtype(&mut self, dtype: DataType) -> Result<()> {
        self.write_i8(dtype as i8)?;
        Ok(())
    }
}

impl Serializer {
    fn write_str(&mut self, string: &str) -> Result<()> {
        if string.is_empty() {
            self.output.write_all(&[0u8; 3])?;
        } else {
            self.output.write_u8(0)?;
            self.output
                .write_all(&(string.len() as u16).to_be_bytes())?;
            self.output.write_all(string.as_bytes())?;
        }
        Ok(())
    }

    fn write_dtype(&mut self, dtype: DataType) -> Result<()> {
        self.output.write_i8(dtype as i8)?;
        Ok(())
    }
}

#[repr(i8)]
#[derive(Copy, Clone, Debug)]
enum DataType {
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

impl<'a> ser::Serializer for &'a mut Serializer {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output
            .write_all(&[DataType::Boolean as u8, if v { 1 } else { 0 }])?;
        Ok(())
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.output.write_all(&[DataType::Byte as u8, v as u8])?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.write_dtype(DataType::Short)?;
        self.output.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.write_dtype(DataType::Int)?;
        self.output.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.write_dtype(DataType::Long)?;
        self.output.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_i8(v as i8)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_i16(v as i16)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_i32(v as i32)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.write_dtype(DataType::Float)?;
        self.output.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.write_dtype(DataType::Double)?;
        self.output.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<()> {
        self.write_dtype(DataType::Char)?;
        self.output.write_all(&[v as u8])?;
        Ok(())
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid JSON if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_dtype(DataType::UTF)?;
        if !v.is_empty() {
            //self.output.write_u8(0)?;
        }
        self.write_str(v)
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<()> {
        Ok(())
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<()> {
        self.output.write_i8(-1)?;
        Ok(())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.write_str(variant)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_dtype(DataType::ObjectBegin)?;
        self.write_str(name)?;
        value.serialize(&mut *self)?;
        self.write_dtype(DataType::ObjectEnd)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_dtype(DataType::ObjectBegin)?;
        self.write_str(variant)?;
        value.serialize(&mut *self)?;
        self.write_dtype(DataType::ObjectEnd)?;
        Ok(())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.write_dtype(DataType::ObjectBegin)?;
        self.write_str("saved.avl.interval.tree")?;
        self.serialize_i32(len.unwrap() as i32)?;
        Ok(self)
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.write_dtype(DataType::ObjectBegin)?;
        self.write_str(variant)?;
        Ok(self)
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.serialize_i32(len.unwrap() as i32)?;
        Ok(self)
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_i32(len as i32)?;
        Ok(self)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_i32(len as i32)?;
        Ok(self)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a> ser::SerializeSeq for &'a mut Serializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        self.write_dtype(DataType::ObjectEnd)
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.write_dtype(DataType::ObjectEnd)
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.write_dtype(DataType::ObjectEnd)
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_dtype(DataType::ObjectBegin)?;
        self.write_str(&key.serialize(MapKeySerializer)?)
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.write_dtype(DataType::ObjectEnd)
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if key
            .chars()
            .try_for_each(|c| if c.is_ascii_digit() { Ok(()) } else { Err(()) })
            .is_ok()
        {
            key.parse::<i32>().unwrap().serialize(&mut **self)?;
        } else {
            key.serialize(&mut **self)?;
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_dtype(DataType::ObjectBegin)?;
        self.write_str(key)?;
        value.serialize(&mut **self)?;
        self.write_dtype(DataType::ObjectEnd)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

struct MapKeySerializer;

fn bad_key_err() -> Error {
    Error::Message("bad key".to_string())
}

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Impossible<String, Error>;
    type SerializeStructVariant = Impossible<String, Error>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String> {
        Ok(variant.to_owned())
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, value: bool) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i8(self, value: i8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, value: f32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_f64(self, value: f64) -> Result<String> {
        Ok(value.to_string())
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<String> {
        Ok({
            let mut s = String::new();
            s.push(value);
            s
        })
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<String> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String> {
        Err(bad_key_err())
    }

    fn serialize_unit(self) -> Result<String> {
        Err(bad_key_err())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        Err(bad_key_err())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(bad_key_err())
    }

    fn serialize_none(self) -> Result<String> {
        Err(bad_key_err())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(bad_key_err())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(bad_key_err())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(bad_key_err())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(bad_key_err())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(bad_key_err())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(bad_key_err())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(bad_key_err())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(bad_key_err())
    }

    fn collect_str<T>(self, value: &T) -> Result<String>
    where
        T: ?Sized + std::fmt::Display,
    {
        Ok(value.to_string())
    }
}
