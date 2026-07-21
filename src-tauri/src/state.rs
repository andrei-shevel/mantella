use crate::library::identity::IdentityCache;
use crate::library::watcher::LibraryWatcher;
use crate::pdf::engine::{CancelFlag, PdfWorker};
use crate::store::Store;
use std::sync::Mutex;

/// Lock order: always lock `store` before `identity_cache` when a call site
/// needs both, and don't hold `identity_cache` across a `store` operation
/// that might block — resolve the id, drop that lock, then take `store`.
pub struct AppState {
    pub store: Mutex<Store>,
    pub pdf: PdfWorker,
    pub watcher: Mutex<Option<LibraryWatcher>>,
    /// Cancel flag of the most recent open. The app shows one document at a
    /// time, so starting a new open cancels the previous one if it is still
    /// waiting in the worker queue.
    pub open_cancel: Mutex<Option<CancelFlag>>,
    pub identity_cache: Mutex<IdentityCache>,
}

/// Files the OS asked us to open (Finder "Open With", double-click) that the
/// frontend hasn't picked up yet. Buffered because on a cold start the open
/// event arrives before the webview is ready to handle it.
#[derive(Default)]
pub struct PendingOpenFiles(pub Mutex<Vec<String>>);
