use crate::error::{AppError, Result};
use crate::library::identity;
use crate::pdf::engine::OpenInfo;
use crate::pdf::links::PageLink;
use crate::pdf::text::TextRun;
use crate::state::{AppState, PendingOpenFiles};
use crate::store::FileState;
use serde::Serialize;
use std::path::{Path, PathBuf};
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
    pub id: String,
}

#[tauri::command]
pub async fn open_document(state: State<'_, AppState>, path: String) -> Result<OpenResult> {
    // Only one document is shown at a time: a newer open supersedes any
    // previous one still waiting in the worker queue.
    let cancel = Arc::new(AtomicBool::new(false));
    if let Some(prev) = state.open_cancel.lock().unwrap().replace(cancel.clone()) {
        prev.store(true, Ordering::Relaxed);
    }
    let info = state
        .pdf
        .open_cancellable(PathBuf::from(&path), Some(cancel))
        .await?;

    // Hash failure (permission race, deleted mid-open) shouldn't block
    // showing a document pdfium already loaded successfully: fall back to
    // the raw path as the id. It's naturally shaped like a legacy key, so a
    // later scan will opportunistically migrate it once the file is stable.
    let id = std::fs::metadata(&path)
        .ok()
        .and_then(|m| {
            state
                .identity_cache
                .lock()
                .unwrap()
                .resolve(Path::new(&path), m.len(), identity::mtime_secs(&m))
                .ok()
        })
        .unwrap_or_else(|| path.clone());

    let mut store = state.store.lock().unwrap();
    let entry = store.files.entry(id.clone()).or_default();
    entry.last_opened = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs());
    entry.last_known_path = Some(path.clone());
    let file_state = entry.clone();
    let _ = store.save_files();

    Ok(OpenResult {
        info,
        state: file_state,
        id,
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
    if !allowed.iter().any(|p| {
        url.get(..p.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(p))
    }) {
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
