use crate::error::{AppError, Result};
use crate::library::scanner::{self, FileEntry};
use crate::library::watcher;
use crate::state::AppState;
use crate::store::{KeyBinding, Settings};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.store.lock().unwrap().settings.clone()
}

#[tauri::command]
pub fn set_library_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<FileEntry>> {
    let root = PathBuf::from(&path);
    if !root.is_dir() {
        return Err(AppError::Message(format!("not a folder: {path}")));
    }

    {
        let mut store = state.store.lock().unwrap();
        store.settings.library_path = Some(root.clone());
        store.save_settings()?;
    }

    // Replace the watcher; dropping the old one stops it.
    let new_watcher = watcher::start(app, root.clone())?;
    *state.watcher.lock().unwrap() = Some(new_watcher);

    let mut store = state.store.lock().unwrap();
    let mut cache = state.identity_cache.lock().unwrap();
    let entries = scanner::scan_with_pins(Some(&root), &mut store.files, &mut cache);
    drop(cache);
    let _ = store.save_files();
    Ok(entries)
}

#[tauri::command]
pub fn get_library(state: State<'_, AppState>) -> Vec<FileEntry> {
    let mut store = state.store.lock().unwrap();
    let root = store.settings.library_path.clone();
    let mut cache = state.identity_cache.lock().unwrap();
    let entries = scanner::scan_with_pins(root.as_deref(), &mut store.files, &mut cache);
    drop(cache);
    let _ = store.save_files();
    entries
}

/// Records (or clears, with `None`) the document to reopen on next launch.
#[tauri::command]
pub fn set_last_file(state: State<'_, AppState>, path: Option<String>) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    store.settings.last_file = path.map(PathBuf::from);
    store.save_settings()
}

#[tauri::command]
pub fn set_pinned(state: State<'_, AppState>, id: String, path: String, pinned: bool) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    let entry = store.files.entry(id).or_default();
    entry.pinned = pinned;
    entry.last_known_path = Some(path);
    store.save_files()
}

#[tauri::command]
pub fn set_shortcuts(
    state: State<'_, AppState>,
    shortcuts: HashMap<String, KeyBinding>,
) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    store.settings.shortcuts = shortcuts;
    store.save_settings()
}
