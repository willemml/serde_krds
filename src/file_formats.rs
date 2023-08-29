use std::collections::HashMap;

use serde::{Deserialize, Serialize};

fn note_magic() -> String {
    const NOTE_MAGIC: &[u8; 5] = b"\x30\xef\xbf\xbc\x30";
    std::str::from_utf8(NOTE_MAGIC).unwrap().to_string()
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct TimerDataFile {
    #[serde(rename = "timer.model", skip_serializing_if = "Option::is_none")]
    timer_model: Option<TimerModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fpr: Option<FPR>,
    #[serde(rename = "book.info.store", skip_serializing_if = "Option::is_none")]
    book_info_store: Option<BookInfoStore>,
    #[serde(rename = "page.history.store", skip_serializing_if = "Option::is_none")]
    page_history_store: Option<Vec<PHRWrapper>>,
    #[serde(
        rename = "whisperstore.migration.status",
        skip_serializing_if = "Option::is_none"
    )]
    whisperstore_migration_status: Option<WhisperstoreMigrationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lpr: Option<LPR>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct FPR(pub String, pub i64, pub i64, pub String, pub String);

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct LPR(pub i8, pub String, pub i64);

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct WhisperstoreMigrationStatus(bool, bool);

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct TimerModel(
    pub i64,
    pub i64,
    pub i64,
    pub f64,
    pub TACWrapper,
);

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct BookInfoStore(pub i64, pub f64);


#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct PHRWrapper(pub PageHistoryRecord);
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct PageHistoryRecord(pub String, pub i64);

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename = "timer.average.calculator")]
pub struct TACWrapper(pub TimerAverageCalculator);

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct TimerAverageCalculator(
    pub i32,
    pub i32,
    pub Vec<TimerAverageDistributionNormal>, // normal
    pub Vec<TimerAverageOutliers>,           // outliers
);

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct TimerAverages(pub i64, pub f64, pub f64);

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct TimerAverageDistributionNormal(TimerAverages);

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct TimerAverageOutliers(TimerAverages);

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct ReaderDataFile {
    #[serde(rename = "font.prefs", skip_serializing_if = "Option::is_none")]
    font_preferences: Option<FontPreferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sync_lpr: Option<bool>,
    #[serde(
        rename = "next.in.series.info.data",
        skip_serializing_if = "Option::is_none"
    )]
    nis_info_data: Option<String>,
    #[serde(
        rename = "annotation.cache.object",
        skip_serializing_if = "Option::is_none"
    )]
    annotation_cache: Option<HashMap<NoteType, IntervalTree<Note>>>,
    #[serde(rename = "apnx.key", skip_serializing_if = "Option::is_none")]
    apnx_key: Option<APNXKey>,
    #[serde(rename = "language.store", skip_serializing_if = "Option::is_none")]
    language_store: Option<LanguageStore>,
    #[serde(rename = "ReaderMetrics", skip_serializing_if = "Option::is_none")]
    reader_metrics: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FontPreferences(
    pub String, // font
    pub i32,
    pub i32, // font size
    pub i32,
    pub i32,
    pub i32,
    pub i32,
    pub i32,
    pub i32,
    pub i32, // bold level
    pub String,
    pub i32,
    pub String,
    pub bool,
    pub String,
    pub i32,
);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct APNXKey(
    pub String,
    pub String, // type
    pub bool,
    pub Vec<i32>,
    pub i32,
    pub i32,
    pub i32,
    pub String,
);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AnnotationData(
    pub String, // Start pos
    pub String, // End pos
    pub i64,    // created time
    pub i64,    // last modified
    pub String, // template
    pub String, // note nbk ref for handwritten, or note text for typed
);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct HighlightData(
    pub String, // Start pos
    pub String, // End pos
    pub i64,    // created time
    pub i64,    // last modified
    pub String, // template
);

#[repr(i32)]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
pub enum NoteType {
    Bookmark = 0,
    Highlight = 1,
    Typed = 2,
    Handwritten = 10,
    Sticky = 11,
}

impl TryFrom<i32> for NoteType {
    type Error = crate::error::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Bookmark,
            1 => Self::Highlight,
            2 => Self::Typed,
            10 => Self::Handwritten,
            11 => Self::Sticky,
            _ => return Err(Self::Error::BadValue),
        })
    }
}

impl Serialize for NoteType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

use serde::de::{self, Visitor};

struct NoteTypeVisitor;

impl<'de> Visitor<'de> for NoteTypeVisitor {
    type Value = NoteType;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value
            .try_into()
            .map_err(|_| E::custom(format!("i32 out of range: -2..9")))
    }
}

impl<'de> Deserialize<'de> for NoteType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_i32(NoteTypeVisitor)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename = "saved.avl.interval.tree")]
pub struct IntervalTree<T>(Vec<T>);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Note {
    #[serde(rename = "annotation.personal.bookmark")]
    Bookmark(AnnotationData),
    #[serde(rename = "annotation.personal.highlight")]
    Highlight(HighlightData),
    #[serde(rename = "annotation.personal.note")]
    Typed(AnnotationData),
    #[serde(rename = "annotation.personal.handwritten_note")]
    Handwritten(AnnotationData),
    #[serde(rename = "annotation.personal.sticky_note")]
    Sticky(AnnotationData),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LanguageStore(pub String, pub i32);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ReaderMetrics {
    pub booklaunchedbefore: String,
}

/// Rust representations of actual files taken from a Kindle Scribe.
pub mod example_files {
    use super::*;

    /// Contains location info for scribbles on a write-on PDF.
    pub fn reader_data_file_1() -> ReaderDataFile {
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
}
