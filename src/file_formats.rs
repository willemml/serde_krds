use serde::{Deserialize, Serialize};

fn note_magic() -> String {
    const NOTE_MAGIC: &[u8; 5] = b"\x30\xef\xbf\xbc\x30";
    std::str::from_utf8(NOTE_MAGIC).unwrap().to_string()
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

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct AnnotationCacheObject {
    #[serde(rename = "0", skip_serializing_if = "Option::is_none")]
    pub bookmarks: Option<Vec<Bookmark>>,
    #[serde(rename = "1", skip_serializing_if = "Option::is_none")]
    pub highlights: Option<Vec<Highlight>>,
    #[serde(rename = "2", skip_serializing_if = "Option::is_none")]
    pub typed_notes: Option<Vec<TypedNote>>,
    #[serde(rename = "10", skip_serializing_if = "Option::is_none")]
    pub handwritten_notes: Option<Vec<HandwrittenNote>>,
    #[serde(rename = "11", skip_serializing_if = "Option::is_none")]
    pub sticky_notes: Option<Vec<StickyNote>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Note(
    pub String, // Start pos
    pub String, // End pos
    pub i64,    // created time
    pub i64,    // last modified
    pub String, // template
    pub String, // note nbk ref
);

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename = "annotation.personal.bookmark")]
pub struct Bookmark(pub Note);

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename = "annotation.personal.highlight")]
pub struct Highlight(pub Note);

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename = "annotation.personal.note")]
pub struct TypedNote(pub Note);

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename = "annotation.personal.handwritten_note")]
pub struct HandwrittenNote(pub Note);

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename = "annotation.personal.sticky_note")]
pub struct StickyNote(pub Note);

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
    pub fn yjr_file_1() -> KRDSFileTypes {
        let handwritten = vec![
            HandwrittenNote(Note(
                "AdgGAAAAAAAA:2586".to_string(),
                "AdgGAAAAAAAA:2586".to_string(),
                1693039707755,
                1693039707755,
                note_magic(),
                "cRgtuIx_zS-m4geT-n6qiDQX".to_string(),
            )),
            HandwrittenNote(Note(
                "AUYGAAAAAAAA:2".to_string(),
                "AUYGAAAAAAAA:2".to_string(),
                1693039682836,
                1693039682836,
                note_magic(),
                "cRgtuIx_zS-m4geT-n6qiDQ0".to_string(),
            )),
            HandwrittenNote(Note(
                "AeAGAAAAAAAA:10314".to_string(),
                "AeAGAAAAAAAA:10314".to_string(),
                1693039698886,
                1693039698886,
                note_magic(),
                "cRgtuIx_zS-m4geT-n6qiDQN".to_string(),
            )),
            HandwrittenNote(Note(
                "Ad0GAAAAAAAA:3196".to_string(),
                "Ad0GAAAAAAAA:3196".to_string(),
                1693106752941,
                1693106752941,
                note_magic(),
                "cQqrFiHphTNa4dSTQKbnzvQ7".to_string(),
            )),
            HandwrittenNote(Note(
                "AUIEAAAAAAAA:32195".to_string(),
                "AUIEAAAAAAAA:32195".to_string(),
                1693167153299,
                1693167153299,
                note_magic(),
                "c0mArJzWjReSnNaskkkQWkw0".to_string(),
            )),
        ];
        let ls = LanguageStore("en-US".to_string(), 4);
        let rm = ReaderMetrics {
            booklaunchedbefore: "true".to_string(),
        };

        KRDSFileTypes::YJRFile {
            nis_info_data: "".to_string(),
            annotation_cache: AnnotationCacheObject {
                handwritten_notes: Some(handwritten),
                ..Default::default()
            },
            language_store: ls,
            reader_metrics: rm,
        }
    }
}
