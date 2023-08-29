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

    let serialized = include_bytes!("../../testfiles/krds/pdfannot.yjr"); //pdfannot.yjr");

    let deserialized: file_formats::TimerDataFile = de::from_slice(serialized).unwrap();

    stdout.write_all(&ser::to_bytevec(&deserialized).unwrap())?;

    //stdout.write_fmt(format_args!("{:#?}", &deserialized))?;

    stdout.flush()?;
    Ok(())
}

mod test {
    use std::collections::HashMap;

    use crate::de::from_slice;
    use crate::file_formats::*;
    use crate::ser::to_bytevec;

    const PDFANNOT_YJR: &[u8] = include_bytes!("../test_files/pdfannot.yjr");
    const PDFANNOT_YJF: &[u8] = include_bytes!("../test_files/pdfannot.yjf");
    const BOOK_HL_NOTE_AZW3R: &[u8] = include_bytes!("../test_files/bookhl+note.azw3r");
    const BOOK_HL_NOTE_AZW3F: &[u8] = include_bytes!("../test_files/bookhl+note.azw3f");

    fn note_magic() -> String {
        const NOTE_MAGIC: &[u8; 5] = b"\x30\xef\xbf\xbc\x30";
        std::str::from_utf8(NOTE_MAGIC).unwrap().to_string()
    }

    fn pdfannot_yjr_struct_repr() -> ReaderDataFile {
        let mut annotations = HashMap::new();
        let handwritten = vec![
            Note::Handwritten(AnnotationData(
                "AdgGAAAAAAAA:2586".to_string(),
                "AdgGAAAAAAAA:2586".to_string(),
                1693039707755,
                1693039707755,
                note_magic(),
                "cRgtuIx_zS-m4geT-n6qiDQX".to_string(),
            )),
            Note::Handwritten(AnnotationData(
                "AUYGAAAAAAAA:2".to_string(),
                "AUYGAAAAAAAA:2".to_string(),
                1693039682836,
                1693039682836,
                note_magic(),
                "cRgtuIx_zS-m4geT-n6qiDQ0".to_string(),
            )),
            Note::Handwritten(AnnotationData(
                "AeAGAAAAAAAA:10314".to_string(),
                "AeAGAAAAAAAA:10314".to_string(),
                1693039698886,
                1693039698886,
                note_magic(),
                "cRgtuIx_zS-m4geT-n6qiDQN".to_string(),
            )),
            Note::Handwritten(AnnotationData(
                "Ad0GAAAAAAAA:3196".to_string(),
                "Ad0GAAAAAAAA:3196".to_string(),
                1693106752941,
                1693106752941,
                note_magic(),
                "cQqrFiHphTNa4dSTQKbnzvQ7".to_string(),
            )),
            Note::Handwritten(AnnotationData(
                "AUIEAAAAAAAA:32195".to_string(),
                "AUIEAAAAAAAA:32195".to_string(),
                1693167153299,
                1693167153299,
                note_magic(),
                "c0mArJzWjReSnNaskkkQWkw0".to_string(),
            )),
        ];
        annotations.insert(NoteType::Handwritten, IntervalTree(handwritten));
        let ls = LanguageStore("en-US".to_string(), 4);
        let mut rm = HashMap::new();

        rm.insert("booklaunchedbefore".to_string(), "true".to_string());

        ReaderDataFile {
            nis_info_data: Some("".to_string()),
            annotation_cache: Some(annotations),
            language_store: Some(ls),
            reader_metrics: Some(rm),
            ..Default::default()
        }
    }

    #[test]
    fn pdfannot_yjr_de() {
        assert_eq!(
            from_slice::<ReaderDataFile>(PDFANNOT_YJR).unwrap(),
            pdfannot_yjr_struct_repr()
        );
    }

    #[test]
    fn pdfannot_yjr_de_ser() {
        assert_eq!(
            &to_bytevec(&from_slice::<ReaderDataFile>(PDFANNOT_YJR).unwrap()).unwrap(),
            PDFANNOT_YJR
        )
    }

    #[test]
    fn pdfannot_yjr_ser_de() {
        assert_eq!(
            from_slice::<ReaderDataFile>(&to_bytevec(&pdfannot_yjr_struct_repr()).unwrap())
                .unwrap(),
            pdfannot_yjr_struct_repr()
        )
    }
}
