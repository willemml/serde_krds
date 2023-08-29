use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct TimerDataFile {
    #[serde(rename = "timer.model", skip_serializing_if = "Option::is_none")]
    pub timer_model: Option<TimerModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fpr: Option<FPR>,
    #[serde(rename = "book.info.store", skip_serializing_if = "Option::is_none")]
    pub book_info_store: Option<BookInfoStore>,
    #[serde(rename = "page.history.store", skip_serializing_if = "Option::is_none")]
    pub page_history_store: Option<Vec<PHRWrapper>>,
    #[serde(
        rename = "whisperstore.migration.status",
        skip_serializing_if = "Option::is_none"
    )]
    pub whisperstore_migration_status: Option<WhisperstoreMigrationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lpr: Option<LPR>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct FPR(pub String, pub i64, pub i64, pub String, pub String);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct LPR(pub i8, pub String, pub i64);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct WhisperstoreMigrationStatus(bool, bool);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct TimerModel(pub i64, pub i64, pub i64, pub f64, pub TACWrapper);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct BookInfoStore(pub i64, pub f64);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename = "page.history.record")]
pub struct PHRWrapper(pub PageHistoryRecord);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct PageHistoryRecord(pub String, pub i64);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename = "timer.average.calculator")]
pub struct TACWrapper(pub TimerAverageCalculator);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct TimerAverageCalculator(
    pub i32,
    pub i32,
    pub Vec<TADNWrapper>, // normal
    pub Vec<TAOWrapper>,  // outliers
);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename = "timer.average.calculator.distribution.normal")]
pub struct TADNWrapper(pub TimerAverageDistributionNormal);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct TimerAverageDistributionNormal(pub i64, pub f64, pub f64);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename = "timer.average.calculator.outliers")]
pub struct TAOWrapper(pub TimerAverageOutliers);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct TimerAverageOutliers(pub i32, pub f64, pub f64);

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct ReaderDataFile {
    #[serde(rename = "font.prefs", skip_serializing_if = "Option::is_none")]
    pub font_preferences: Option<FontPreferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_lpr: Option<bool>,
    #[serde(
        rename = "next.in.series.info.data",
        skip_serializing_if = "Option::is_none"
    )]
    pub nis_info_data: Option<String>,
    #[serde(
        rename = "annotation.cache.object",
        skip_serializing_if = "Option::is_none"
    )]
    pub annotation_cache: Option<HashMap<NoteType, IntervalTree<Note>>>,
    #[serde(rename = "apnx.key", skip_serializing_if = "Option::is_none")]
    pub apnx_key: Option<APNXKey>,
    #[serde(rename = "language.store", skip_serializing_if = "Option::is_none")]
    pub language_store: Option<LanguageStore>,
    #[serde(rename = "ReaderMetrics", skip_serializing_if = "Option::is_none")]
    pub reader_metrics: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct AnnotationData(
    pub String, // Start pos
    pub String, // End pos
    pub i64,    // created time
    pub i64,    // last modified
    pub String, // template
    pub String, // note nbk ref for handwritten, or note text for typed
);

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename = "saved.avl.interval.tree")]
pub struct IntervalTree<T>(pub Vec<T>);

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct LanguageStore(pub String, pub i32);

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ReaderMetrics {
    pub booklaunchedbefore: String,
}
