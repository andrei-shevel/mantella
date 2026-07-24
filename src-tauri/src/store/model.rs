use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub library_path: Option<PathBuf>,
    pub last_file: Option<PathBuf>,
    /// Keyboard shortcut overrides, keyed by shortcut id. Only entries the
    /// user has customized are present; everything else uses its frontend default.
    pub shortcuts: HashMap<String, KeyBinding>,
    pub theme: Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KeyBinding {
    pub key: String,
    pub meta: bool,
}

/// Per-file persisted state, keyed by a content-based file id (partial hash
/// and size; see `library::identity`) rather than by path, so state
/// survives the file being renamed or moved.
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
    /// Most recently known absolute path for this id. Only needed to (a)
    /// locate pinned files living outside the library root, which can't be
    /// rediscovered by re-scanning it, and (b) let the watcher tell whether
    /// a vanished id used to live under the watched root.
    pub last_known_path: Option<String>,
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
            last_known_path: None,
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
