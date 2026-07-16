use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub library_path: Option<PathBuf>,
    pub last_file: Option<PathBuf>,
}

/// Per-file persisted state, keyed by absolute file path.
/// `zoom: None` means "fit to width".
///
/// The reading position is anchored as page + offset within that page
/// (fraction of the page height, may be slightly negative when the anchor
/// sits in the gap above the page). Unlike a global scroll ratio, this
/// restores correctly regardless of window size or zoom changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FileState {
    pub pinned: bool,
    pub page: u16,
    pub page_offset: f64,
    pub zoom: Option<f64>,
    pub last_opened: Option<u64>,
    pub bookmarks: Vec<Bookmark>,
}

impl Default for FileState {
    fn default() -> Self {
        Self {
            pinned: false,
            page: 1,
            page_offset: 0.0,
            zoom: None,
            last_opened: None,
            bookmarks: Vec::new(),
        }
    }
}

/// A named position inside a document, anchored the same way as the reading
/// position: page + offset within that page (fraction of the page height).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub page: u16,
    pub page_offset: f64,
}

impl Default for Bookmark {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            page: 1,
            page_offset: 0.0,
        }
    }
}
