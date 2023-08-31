use crate::de::from_bytes;
use crate::ser::to_bytes;
use crate::DataType;
use kindle_formats::krds::*;

use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};

pub const PDFANNOT_YJR: &[u8] = include_bytes!("../test_files/pdfannot.yjr");
pub const PDFANNOT_YJF: &[u8] = include_bytes!("../test_files/pdfannot.yjf");
pub const BOOK_HL_NOTE_AZW3R: &[u8] = include_bytes!("../test_files/bookhl+note.azw3r");
pub const BOOK_HL_NOTE_AZW3F: &[u8] = include_bytes!("../test_files/bookhl+note.azw3f");

fn note_magic() -> String {
    const NOTE_MAGIC: &[u8; 5] = b"\x30\xef\xbf\xbc\x30";
    std::str::from_utf8(NOTE_MAGIC).unwrap().to_string()
}

pub fn de_no_magic<'a, T>(input: &'a [u8]) -> T
where
    T: Deserialize<'a>,
{
    let mut deserializer = crate::de::Deserializer::from_bytes(input);
    T::deserialize(&mut deserializer).unwrap()
}

pub fn ser_no_magic<T>(input: T) -> Vec<u8>
where
    T: Serialize,
{
    let mut serializer = crate::ser::Serializer { output: Vec::new() };
    input.serialize(&mut serializer).unwrap();
    serializer.output
}

pub fn handwritten_note() -> Note {
    Note::Handwritten(AnnotationData(
        "AdgGAAAAAAAA:2586".to_string(),
        "AdgGAAAAAAAA:2586".to_string(),
        1693039707755,
        1693039707755,
        note_magic(),
        Some("cRgtuIx_zS-m4geT-n6qiDQX".to_string()),
    ))
}

pub fn handwritten_note_vec() -> Vec<Note> {
    vec![
        handwritten_note(),
        Note::Handwritten(AnnotationData(
            "AUYGAAAAAAAA:2".to_string(),
            "AUYGAAAAAAAA:2".to_string(),
            1693039682836,
            1693039682836,
            note_magic(),
            Some("cRgtuIx_zS-m4geT-n6qiDQ0".to_string()),
        )),
        Note::Handwritten(AnnotationData(
            "AeAGAAAAAAAA:10314".to_string(),
            "AeAGAAAAAAAA:10314".to_string(),
            1693039698886,
            1693039698886,
            note_magic(),
            Some("cRgtuIx_zS-m4geT-n6qiDQN".to_string()),
        )),
        Note::Handwritten(AnnotationData(
            "Ad0GAAAAAAAA:3196".to_string(),
            "Ad0GAAAAAAAA:3196".to_string(),
            1693106752941,
            1693106752941,
            note_magic(),
            Some("cQqrFiHphTNa4dSTQKbnzvQ7".to_string()),
        )),
        Note::Handwritten(AnnotationData(
            "AUIEAAAAAAAA:32195".to_string(),
            "AUIEAAAAAAAA:32195".to_string(),
            1693167153299,
            1693167153299,
            note_magic(),
            Some("c0mArJzWjReSnNaskkkQWkw0".to_string()),
        )),
    ]
}

pub fn test_num<T>(num: T, dtype: DataType) -> Vec<u8>
where
    T: num_traits::ToBytes,
{
    [&[dtype as u8] as &[_], num.to_be_bytes().as_ref()].concat()
}

pub fn test_string() -> (Vec<u8>, String) {
    let string = "testing stuff".to_string();
    (
        [
            &[DataType::String as u8] as &[_],
            &[0],
            &(string.len() as u16).to_be_bytes(),
            string.as_bytes(),
        ]
        .concat(),
        string,
    )
}

pub fn empty_string() -> (Vec<u8>, String) {
    (vec![0x03, 0x01], "".to_string())
}

pub fn test_vec_int() -> (Vec<u8>, Vec<i32>) {
    let vec = vec![0, 1, 2, 3, 45, 44, 60];
    let mut bytes = vec![];
    bytes.append(&mut test_num(vec.len() as i32, DataType::Int));
    for n in &vec {
        bytes.append(&mut test_num(*n, DataType::Int));
    }
    (bytes, vec)
}

pub fn test_vec_strings() -> (Vec<u8>, Vec<String>) {
    let (b, s) = test_string();
    let (eb, es) = empty_string();
    let vec = vec![s.clone(), es.clone(), s.clone(), es.clone(), s.clone()];
    (
        [
            &test_num(vec.len() as i32, DataType::Int) as &[_],
            &b,
            &eb,
            &b,
            &eb,
            &b,
        ]
        .concat(),
        vec,
    )
}

pub fn simple_newtype() -> (Vec<u8>, PHRWrapper) {
    let (sb, s) = test_string();
    let n = 07734i64;
    let nb = test_num(n, DataType::Long);
    let sn = PHRWrapper(PageHistoryRecord(s, n));
    let newtype_name = b"page.history.record";
    (
        [
            &[DataType::FieldBegin as u8, 0] as &[_],
            &(newtype_name.len() as i16).to_be_bytes(),
            newtype_name,
            &sb,
            &nb,
            &[DataType::FieldEnd as u8],
        ]
        .concat(),
        sn,
    )
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SimpleStruct {
    field_1: i32,
    field_2: String,
}

pub fn simple_struct() -> (Vec<u8>, SimpleStruct) {
    let test_orig = SimpleStruct {
        field_1: 1234,
        field_2: test_string().1,
    };

    let test_bytes = [
        &test_num(2i32, DataType::Int) as &[_],
        &[DataType::FieldBegin as u8, 0, 0, 7],
        b"field_1",
        &test_num(test_orig.field_1, DataType::Int),
        &[
            DataType::FieldEnd as u8,
            DataType::FieldBegin as u8,
            0,
            0,
            7,
        ],
        b"field_2",
        &test_string().0,
        &[DataType::FieldEnd as u8],
    ]
    .concat();
    (test_bytes, test_orig)
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct VecMapStruct {
    field_1: i32,
    field_2: String,
    field_3: Vec<i32>,
    field_4: LinkedHashMap<NoteType, String>,
}

pub fn vec_map_struct() -> (Vec<u8>, VecMapStruct) {
    let (map_bytes, map) = test_map();
    let (vec_bytes, vec) = test_vec_int();
    let (string_bytes, string) = test_string();
    let int = 70053;
    let int_bytes = test_num(int, DataType::Int);
    let field_starter = [DataType::FieldBegin as u8, 0, 0, 7];
    let field_end = [DataType::FieldEnd as u8];
    (
        [
            &test_num(4i32, DataType::Int) as &[_],
            &field_starter,
            b"field_1",
            &int_bytes,
            &field_end,
            &field_starter,
            b"field_2",
            &string_bytes,
            &field_end,
            &field_starter,
            b"field_3",
            &vec_bytes,
            &field_end,
            &field_starter,
            b"field_4",
            &map_bytes,
            &field_end,
        ]
        .concat(),
        VecMapStruct {
            field_1: int,
            field_2: string,
            field_3: vec,
            field_4: map,
        },
    )
}

fn str_to_bytes(string: &str) -> Vec<u8> {
    [
        &[DataType::String as u8, 0] as &[_],
        &(string.len() as u16).to_be_bytes(),
        string.as_bytes(),
    ]
    .concat()
}

pub fn test_map() -> (Vec<u8>, LinkedHashMap<NoteType, String>) {
    let mut map = LinkedHashMap::new();
    let k1 = NoteType::Bookmark;
    let k2 = NoteType::Highlight;
    let k3 = NoteType::Handwritten;
    let string_1 = "testing string";
    let string_2 = "this is neat";
    let string_3 = "TEST YOUR CODE!";
    map.insert(k1, string_1.to_string());
    map.insert(k2, string_2.to_string());
    map.insert(k3, string_3.to_string());
    (
        [
            &test_num(3i32, DataType::Int) as &[_],
            &test_num(k1 as i32, DataType::Int),
            &str_to_bytes(string_1),
            &test_num(k2 as i32, DataType::Int),
            &str_to_bytes(string_2),
            &test_num(k3 as i32, DataType::Int),
            &str_to_bytes(string_3),
        ]
        .concat(),
        map,
    )
}

pub fn pdfannot_yjr() -> ReaderDataFile {
    let mut annotations = LinkedHashMap::new();
    let handwritten = handwritten_note_vec();
    annotations.insert(NoteType::Handwritten, IntervalTree(handwritten));
    let ls = LanguageStore("en-US".to_string(), 4);
    let mut rm = LinkedHashMap::new();

    rm.insert("booklaunchedbefore".to_string(), "true".to_string());

    ReaderDataFile {
        nis_info_data: Some("".to_string()),
        annotation_cache: Some(annotations),
        language_store: Some(ls),
        reader_metrics: Some(rm),
        ..Default::default()
    }
}

pub fn pdfannot_yjf() -> TimerDataFile {
    TimerDataFile {
        timer_model: Some(TimerModel(
            0,
            0,
            0,
            0.0,
            TACWrapper(TimerAverageCalculator(0, 0, vec![], vec![])),
        )),
        fpr: Some(FPR(
            "Ad0GAAAAAAAA:3196".to_string(),
            -1,
            -1,
            "".to_string(),
            "".to_string(),
        )),
        book_info_store: Some(BookInfoStore(0, 0.0)),
        page_history_store: Some(vec![]),
        whisperstore_migration_status: Some(WhisperstoreMigrationStatus(false, false)),
        lpr: Some(LPR(2, "Ad0GAAAAAAAA:3196".to_string(), 1693167158664)),
    }
}

#[test]
fn pdfannot_yjr_de_ser() {
    assert_eq!(
        &to_bytes(&from_bytes::<ReaderDataFile>(PDFANNOT_YJR).unwrap()).unwrap(),
        PDFANNOT_YJR
    )
}

#[test]
fn pdfannot_yjr_ser_de() {
    assert_eq!(
        from_bytes::<ReaderDataFile>(&to_bytes(&pdfannot_yjr()).unwrap()).unwrap(),
        pdfannot_yjr()
    )
}

#[test]
fn pdfannot_yjf_de_ser() {
    assert_eq!(
        &to_bytes(&from_bytes::<TimerDataFile>(PDFANNOT_YJF).unwrap()).unwrap(),
        PDFANNOT_YJF
    )
}

#[test]
fn bookhlnote_azw3r_de_ser() {
    assert_eq!(
        &to_bytes(&from_bytes::<ReaderDataFile>(BOOK_HL_NOTE_AZW3R).unwrap()).unwrap(),
        BOOK_HL_NOTE_AZW3R
    )
}

#[test]
fn bookhlnote_azw3f_de_ser() {
    assert_eq!(
        &to_bytes(&from_bytes::<TimerDataFile>(BOOK_HL_NOTE_AZW3F).unwrap()).unwrap(),
        BOOK_HL_NOTE_AZW3F
    )
}
