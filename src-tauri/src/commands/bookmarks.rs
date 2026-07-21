use crate::error::Result;
use crate::state::AppState;
use crate::store::Bookmark;
use tauri::State;

#[tauri::command]
pub fn save_bookmarks(
    state: State<'_, AppState>,
    id: String,
    bookmarks: Vec<Bookmark>,
) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    store.files.entry(id).or_default().bookmarks = bookmarks;
    store.save_files()
}
