use crate::error::Result;
use crate::state::AppState;
use crate::store::Bookmark;
use tauri::State;

#[tauri::command]
pub fn save_bookmarks(
    state: State<'_, AppState>,
    path: String,
    bookmarks: Vec<Bookmark>,
) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    store.files.entry(path).or_default().bookmarks = bookmarks;
    store.save_files()
}
