use crate::library::watcher::LibraryWatcher;
use crate::pdf::engine::PdfWorker;
use crate::store::Store;
use std::sync::Mutex;

pub struct AppState {
    pub store: Mutex<Store>,
    pub pdf: PdfWorker,
    pub watcher: Mutex<Option<LibraryWatcher>>,
}
