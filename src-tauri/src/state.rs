use crate::library::watcher::LibraryWatcher;
use crate::pdf::engine::PdfWorker;
use crate::store::Store;
use std::sync::Mutex;

pub struct AppState {
    pub store: Mutex<Store>,
    pub pdf: PdfWorker,
    pub watcher: Mutex<Option<LibraryWatcher>>,
}

/// Files the OS asked us to open (Finder "Open With", double-click) that the
/// frontend hasn't picked up yet. Buffered because on a cold start the open
/// event arrives before the webview is ready to handle it.
#[derive(Default)]
pub struct PendingOpenFiles(pub Mutex<Vec<String>>);
