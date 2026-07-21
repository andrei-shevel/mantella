use super::identity::{self, IdentityCache};
use crate::store::FileState;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    /// Content-based id (partial hash + size); see `library::identity`.
    pub id: String,
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
    file_states: &mut HashMap<String, FileState>,
    cache: &mut IdentityCache,
) -> Vec<FileEntry> {
    let mut entries = root
        .map(|r| scan(r, file_states, cache))
        .unwrap_or_default();
    let scanned_ids: HashSet<&str> = entries.iter().map(|e| e.id.as_str()).collect();

    // Pinned files outside the library root are never visited by scan()'s
    // WalkDir pass, so a legacy path-keyed entry for one of them gets its
    // own one-time migration chance here.
    let legacy_pinned: Vec<String> = file_states
        .iter()
        .filter(|(key, state)| state.pinned && is_legacy_key(key))
        .map(|(key, _)| key.clone())
        .collect();
    for path in legacy_pinned {
        let Ok(metadata) = std::fs::metadata(&path) else {
            continue;
        };
        let Ok(id) = cache.resolve(
            Path::new(&path),
            metadata.len(),
            identity::mtime_secs(&metadata),
        ) else {
            continue;
        };
        migrate_legacy(file_states, &path, &id);
    }

    let external: Vec<FileEntry> = file_states
        .iter()
        .filter(|(id, state)| state.pinned && !scanned_ids.contains(id.as_str()))
        .filter_map(|(id, state)| external_entry(id, state))
        .collect();
    entries.extend(external);

    entries.sort_by_key(|e| e.name.to_lowercase());
    entries
}

/// A pinned file outside the library, looked up by its last known path;
/// `None` if it no longer exists on disk.
fn external_entry(id: &str, state: &FileState) -> Option<FileEntry> {
    let path = state.last_known_path.as_deref()?;
    let p = Path::new(path);
    let metadata = std::fs::metadata(p).ok()?;
    if !metadata.is_file() {
        return None;
    }
    Some(FileEntry {
        id: id.to_owned(),
        name: p.file_name()?.to_string_lossy().into_owned(),
        rel_path: path.to_owned(),
        size: metadata.len(),
        modified: identity::mtime_secs(&metadata),
        pinned: true,
        in_library: false,
        path: path.to_owned(),
    })
}

/// Recursively scan `root` for PDF files, skipping hidden directories.
fn scan(
    root: &Path,
    file_states: &mut HashMap<String, FileState>,
    cache: &mut IdentityCache,
) -> Vec<FileEntry> {
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
            let size = metadata.len();
            let modified = identity::mtime_secs(&metadata);
            // Unreadable or vanished mid-walk: skip, same as e.metadata().ok()? above.
            let id = cache.resolve(path, size, modified).ok()?;
            let abs = path.to_string_lossy().into_owned();

            migrate_legacy(file_states, &abs, &id);
            if let Some(state) = file_states.get_mut(&id) {
                state.last_known_path = Some(abs.clone());
            }

            Some(FileEntry {
                pinned: file_states.get(&id).map(|s| s.pinned).unwrap_or(false),
                id,
                name: path.file_name()?.to_string_lossy().into_owned(),
                // normalized to forward slashes so the frontend tree view can
                // split on "/" on every platform
                rel_path: path
                    .strip_prefix(root)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/"),
                size,
                modified,
                in_library: true,
                path: abs,
            })
        })
        .collect()
}

/// One-time opportunistic move of a pre-upgrade, path-keyed `FileState` to
/// its content-based id, the first time that path is seen again. Naturally
/// becomes a no-op once every legacy key has drained — no migration-version
/// flag needed. Known gap: a file renamed *before* its first post-upgrade
/// scan won't match its old path key and starts fresh under the new id; the
/// orphaned legacy key lingers harmlessly and is an accepted edge case, not
/// engineered around.
fn migrate_legacy(file_states: &mut HashMap<String, FileState>, path: &str, id: &str) {
    if file_states.contains_key(id) {
        file_states.remove(path); // an id-keyed entry already won; drop the dupe
        return;
    }
    if let Some(mut legacy) = file_states.remove(path) {
        legacy.last_known_path = Some(path.to_owned());
        file_states.insert(id.to_owned(), legacy);
    }
}

/// Our generated ids never contain a path separator, so any store key that
/// does is a pre-migration absolute-path key.
fn is_legacy_key(key: &str) -> bool {
    key.contains('/') || key.contains('\\')
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.depth() > 0 && entry.file_name().to_string_lossy().starts_with('.')
}

fn has_pdf_extension(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}
