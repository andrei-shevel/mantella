use crate::error::Result;
use crate::pdf::engine::OpenInfo;
use crate::state::AppState;
use crate::store::FileState;
use serde::Serialize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenResult {
    #[serde(flatten)]
    pub info: OpenInfo,
    pub state: FileState,
}

#[tauri::command]
pub async fn open_document(state: State<'_, AppState>, path: String) -> Result<OpenResult> {
    let info = state.pdf.open(PathBuf::from(&path)).await?;

    let mut store = state.store.lock().unwrap();
    let entry = store.files.entry(path).or_default();
    entry.last_opened = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs());
    let file_state = entry.clone();
    let _ = store.save_files();

    Ok(OpenResult {
        info,
        state: file_state,
    })
}

#[tauri::command]
pub fn close_document(state: State<'_, AppState>, doc_id: u64) {
    state.pdf.close(doc_id);
}
