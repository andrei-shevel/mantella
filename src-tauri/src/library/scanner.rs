use crate::store::FileState;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    /// Relative to the library root; the absolute path for files outside it.
    pub rel_path: String,
    pub size: u64,
    pub modified: Option<u64>,
    pub pinned: bool,
    /// False for pinned files living outside the library folder.
    pub in_library: bool,
}

/// The full file list for the frontend: the library scan plus pinned files
/// that live outside the library folder (or lost their folder entirely).
pub fn scan_with_pins(
    root: Option<&Path>,
    file_states: &HashMap<String, FileState>,
) -> Vec<FileEntry> {
    let mut entries = root.map(|r| scan(r, file_states)).unwrap_or_default();

    let external: Vec<FileEntry> = {
        let scanned: HashSet<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        file_states
            .iter()
            .filter(|(path, state)| state.pinned && !scanned.contains(path.as_str()))
            .filter_map(|(path, _)| external_entry(path))
            .collect()
    };
    entries.extend(external);

    entries.sort_by_key(|e| e.name.to_lowercase());
    entries
}

/// A pinned file outside the library; `None` if it no longer exists on disk.
fn external_entry(path: &str) -> Option<FileEntry> {
    let p = Path::new(path);
    let metadata = std::fs::metadata(p).ok()?;
    if !metadata.is_file() {
        return None;
    }
    Some(FileEntry {
        name: p.file_name()?.to_string_lossy().into_owned(),
        rel_path: path.to_owned(),
        size: metadata.len(),
        modified: metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs()),
        pinned: true,
        in_library: false,
        path: path.to_owned(),
    })
}

/// Recursively scan `root` for PDF files, skipping hidden directories.
fn scan(root: &Path, file_states: &HashMap<String, FileState>) -> Vec<FileEntry> {
    // sorted by scan_with_pins together with the external pins
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && has_pdf_extension(e.path()))
        .filter_map(|e| {
            let path = e.path();
            let metadata = e.metadata().ok()?;
            let abs = path.to_string_lossy().into_owned();
            Some(FileEntry {
                name: path.file_name()?.to_string_lossy().into_owned(),
                // normalized to forward slashes so the frontend tree view can
                // split on "/" on every platform
                rel_path: path
                    .strip_prefix(root)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/"),
                size: metadata.len(),
                modified: metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs()),
                pinned: file_states.get(&abs).map(|s| s.pinned).unwrap_or(false),
                in_library: true,
                path: abs,
            })
        })
        .collect()
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.depth() > 0 && entry.file_name().to_string_lossy().starts_with('.')
}

fn has_pdf_extension(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}
