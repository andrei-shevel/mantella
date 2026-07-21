use crate::error::Result;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn save_reading_state(
    state: State<'_, AppState>,
    id: String,
    page: u16,
    page_offset: f64,
    zoom: Option<f64>,
) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    let entry = store.files.entry(id).or_default();
    entry.page = page;
    entry.page_offset = page_offset;
    entry.zoom = zoom;
    store.save_files()
}
