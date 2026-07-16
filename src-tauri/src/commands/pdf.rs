use crate::error::{AppError, Result};
use crate::pdf::engine::OpenInfo;
use crate::pdf::links::PageLink;
use crate::pdf::text::TextRun;
use crate::state::{AppState, PendingOpenFiles};
use crate::store::FileState;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
    // Only one document is shown at a time: a newer open supersedes any
    // previous one still waiting in the worker queue.
    let cancel = Arc::new(AtomicBool::new(false));
    if let Some(prev) = state.open_cancel.lock().unwrap().replace(cancel.clone()) {
        prev.store(true, Ordering::Relaxed);
    }
    let info = state.pdf.open_cancellable(PathBuf::from(&path), Some(cancel)).await?;

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
pub async fn get_page_text(
    state: State<'_, AppState>,
    doc_id: u64,
    page_index: u16,
) -> Result<Vec<TextRun>> {
    state.pdf.page_text(doc_id, page_index).await
}

#[tauri::command]
pub async fn get_page_links(
    state: State<'_, AppState>,
    doc_id: u64,
    page_index: u16,
) -> Result<Vec<PageLink>> {
    state.pdf.page_links(doc_id, page_index).await
}

/// Opens an external link from a PDF in the system browser. Only web and
/// mail schemes are allowed; anything else a document might carry (file:,
/// custom app schemes) is refused.
#[tauri::command]
pub fn open_url(url: String) -> Result<()> {
    let allowed = ["http://", "https://", "mailto:"];
    if !allowed
        .iter()
        .any(|p| url.get(..p.len()).is_some_and(|s| s.eq_ignore_ascii_case(p)))
    {
        return Err(AppError::Message(format!("refusing to open link: {url}")));
    }
    open::that_detached(&url).map_err(|e| AppError::Message(format!("failed to open {url}: {e}")))
}

#[tauri::command]
pub fn close_document(state: State<'_, AppState>, doc_id: u64) {
    state.pdf.close(doc_id);
}

#[tauri::command]
pub fn take_pending_open_files(pending: State<'_, PendingOpenFiles>) -> Vec<String> {
    std::mem::take(&mut *pending.0.lock().unwrap())
}
