//! Watches the library folder and pushes the fresh file list to the frontend
//! via the `library-changed` event whenever files are added, removed or renamed.

use crate::error::{AppError, Result};
use crate::library::scanner;
use crate::state::AppState;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, RecommendedCache};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub type LibraryWatcher = Debouncer<RecommendedWatcher, RecommendedCache>;

pub fn start(app: AppHandle, root: PathBuf) -> Result<LibraryWatcher> {
    let watch_root = root.clone();
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        None,
        move |result: DebounceEventResult| {
            if result.is_ok() {
                sync_library(&app, &root);
            }
        },
    )
    .map_err(|e| AppError::Message(format!("failed to create watcher: {e}")))?;

    debouncer
        .watch(&watch_root, RecursiveMode::Recursive)
        .map_err(|e| AppError::Message(format!("failed to watch {}: {e}", watch_root.display())))?;

    Ok(debouncer)
}

/// Re-scan the library, prune persisted state of files that vanished from it,
/// and broadcast the new list.
fn sync_library(app: &AppHandle, root: &Path) {
    let state = app.state::<AppState>();
    let mut store = state.store.lock().unwrap();
    let mut cache = state.identity_cache.lock().unwrap();

    let entries = scanner::scan_with_pins(Some(root), &mut store.files, &mut cache);
    drop(cache);

    // An id carries no location info, so "was this under the watched root"
    // is decided from last_known_path instead of the map key.
    let root_prefix = root.to_string_lossy().into_owned();
    let existing_ids: HashSet<&str> = entries.iter().map(|e| e.id.as_str()).collect();
    store.files.retain(|id, state| {
        existing_ids.contains(id.as_str())
            || state
                .last_known_path
                .as_deref()
                .is_none_or(|p| !p.starts_with(&root_prefix))
    });
    // Always save: migration/last_known_path refresh can change the store
    // without changing its length.
    let _ = store.save_files();
    drop(store);

    let _ = app.emit("library-changed", &entries);
}
