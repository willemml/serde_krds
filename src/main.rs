#![feature(buf_read_has_data_left)]
#![feature(iter_next_chunk)]

use std::{collections::HashMap, io::Write};

use serde::{Deserialize, Serialize};

mod cli;
mod de;
mod error;
mod ser;

pub const MAGIC: &[u8; 8] = b"\x00\x00\x00\x00\x00\x1A\xB1\x26";

#[derive(Deserialize, Serialize, Clone, Debug, Hash, Eq, PartialEq)]
#[repr(i32)]
pub enum AnnotationClasses {
    Bookmark = 0,
    Highlight = 1,
    Note = 2,
    HandwrittenNote = 10,
    StickyNote = 11,
}

pub trait NumberedField {}

fn main() -> Result<(), std::io::Error> {
    let hr1 = HandwrittenNote(Note(
        "AUYGAAAAAAAA:2".to_string(),
        "AUYGAAAAAAAA:2".to_string(),
        1693039682836,
        1693039682836,
        "0 0".to_string(),
        "cRgtuIx_zS-m4geT-n6qiDQ0".to_string(),
    ));
    let ls = LanguageStore("en-US".to_string(), 4);
    let rm = ReaderMetrics {
        booklaunchedbefore: "true".to_string(),
    };

    let yjr = KRDSFileTypes::YJRFile {
        nis_info_data: "".to_string(),
        annotation_cache: AnnotationCacheObject {
            handwritten_notes: vec![hr1],
        },
        language_store: ls,
        reader_metrics: rm,
    };

    let serialized = ser::to_bytevec(&yjr).unwrap();

    let mut stdout = std::io::stdout();
    stdout.write_all(&serialized)?;
    stdout.flush()?;

    //    cli::do_cli()
    Ok(())
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Simple {
    #[serde(rename = "next.in.series.info.data")]
    nis: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum KRDSFileTypes {
    YJRFile {
        #[serde(rename = "next.in.series.info.data")]
        nis_info_data: String,
        #[serde(rename = "annotation.cache.object")]
        annotation_cache: AnnotationCacheObject,
        #[serde(rename = "language.store")]
        language_store: LanguageStore,
        #[serde(rename = "ReaderMetrics")]
        reader_metrics: ReaderMetrics,
    },
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AnnotationCacheObject {
    #[serde(rename = "10")]
    handwritten_notes: Vec<HandwrittenNote>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Note(
    String, // Start pos
    String, // End pos
    i64,    // created time
    i64,    // last modified
    String, // template
    String, // note nbk ref
);

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename = "annotation.personal.handwritten_note")]
pub struct HandwrittenNote(Note);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LanguageStore(String, i32);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ReaderMetrics {
    booklaunchedbefore: String,
}
