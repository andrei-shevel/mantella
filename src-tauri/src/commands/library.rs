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

    let store = state.store.lock().unwrap();
    Ok(scanner::scan_with_pins(Some(&root), &store.files))
}

#[tauri::command]
pub fn get_library(state: State<'_, AppState>) -> Vec<FileEntry> {
    let store = state.store.lock().unwrap();
    scanner::scan_with_pins(store.settings.library_path.as_deref(), &store.files)
}

/// Records (or clears, with `None`) the document to reopen on next launch.
#[tauri::command]
pub fn set_last_file(state: State<'_, AppState>, path: Option<String>) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    store.settings.last_file = path.map(PathBuf::from);
    store.save_settings()
}

#[tauri::command]
pub fn set_pinned(state: State<'_, AppState>, path: String, pinned: bool) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    store.files.entry(path).or_default().pinned = pinned;
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
