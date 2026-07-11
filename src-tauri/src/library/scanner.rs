use crate::store::FileState;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub rel_path: String,
    pub size: u64,
    pub modified: Option<u64>,
    pub pinned: bool,
}

/// Recursively scan `root` for PDF files, skipping hidden directories.
pub fn scan(root: &Path, file_states: &HashMap<String, FileState>) -> Vec<FileEntry> {
    let mut entries: Vec<FileEntry> = WalkDir::new(root)
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
                path: abs,
            })
        })
        .collect();

    entries.sort_by_key(|e| e.name.to_lowercase());
    entries
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.depth() > 0 && entry.file_name().to_string_lossy().starts_with('.')
}

fn has_pdf_extension(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}
