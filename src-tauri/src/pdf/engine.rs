//! All pdfium access happens on a single dedicated worker thread, because
//! pdfium-render types wrap raw FPDF_* pointers and are `!Send`. The rest of
//! the app talks to the worker through `PdfWorker`, a cheap cloneable handle.

use crate::error::{AppError, Result};
use pdfium_render::prelude::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use tokio::sync::oneshot;

/// Set to true to make the worker skip a queued `Open` when it is dequeued.
/// The queue is FIFO and a pdfium call can't be interrupted, so this is how
/// a stale open (user already switched to another file) is cancelled.
pub type CancelFlag = Arc<AtomicBool>;

/// Page dimensions in PDF points (1/72 inch).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenInfo {
    pub doc_id: u64,
    pub page_count: u16,
    pub pages: Vec<PageSize>,
}

type RenderCallback = Box<dyn FnOnce(Result<Vec<u8>>) + Send>;

enum Request {
    Open {
        path: PathBuf,
        cancel: Option<CancelFlag>,
        reply: oneshot::Sender<Result<OpenInfo>>,
    },
    Render {
        doc_id: u64,
        page_index: u16,
        width: u32,
        reply: RenderCallback,
    },
    PageText {
        doc_id: u64,
        page_index: u16,
        reply: oneshot::Sender<Result<Vec<super::text::TextRun>>>,
    },
    PageLinks {
        doc_id: u64,
        page_index: u16,
        reply: oneshot::Sender<Result<Vec<super::links::PageLink>>>,
    },
    Close {
        doc_id: u64,
    },
}

#[derive(Clone)]
pub struct PdfWorker {
    tx: mpsc::Sender<Request>,
    /// Docs whose `Close` is still queued behind other work; lets the worker
    /// skip their queued renders instead of rendering pages nobody will see.
    closing: Arc<Mutex<HashSet<u64>>>,
}

impl PdfWorker {
    pub fn spawn(library_dirs: Vec<PathBuf>) -> Self {
        let (tx, rx) = mpsc::channel();
        let closing = Arc::new(Mutex::new(HashSet::new()));
        let worker_closing = closing.clone();
        thread::Builder::new()
            .name("pdf-worker".into())
            .spawn(move || worker_loop(rx, library_dirs, worker_closing))
            .expect("failed to spawn pdf worker thread");
        Self { tx, closing }
    }

    pub async fn open_cancellable(
        &self,
        path: PathBuf,
        cancel: Option<CancelFlag>,
    ) -> Result<OpenInfo> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(Request::Open {
                path,
                cancel,
                reply,
            })
            .map_err(|_| worker_gone())?;
        rx.await.map_err(|_| worker_gone())?
    }

    /// Render a page to PNG; the callback fires on the worker thread.
    pub fn render_with(
        &self,
        doc_id: u64,
        page_index: u16,
        width: u32,
        reply: impl FnOnce(Result<Vec<u8>>) + Send + 'static,
    ) {
        if let Err(mpsc::SendError(Request::Render { reply, .. })) = self.tx.send(Request::Render {
            doc_id,
            page_index,
            width,
            reply: Box::new(reply),
        }) {
            reply(Err(worker_gone()));
        }
    }

    pub async fn page_text(
        &self,
        doc_id: u64,
        page_index: u16,
    ) -> Result<Vec<super::text::TextRun>> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(Request::PageText {
                doc_id,
                page_index,
                reply,
            })
            .map_err(|_| worker_gone())?;
        rx.await.map_err(|_| worker_gone())?
    }

    pub async fn page_links(
        &self,
        doc_id: u64,
        page_index: u16,
    ) -> Result<Vec<super::links::PageLink>> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(Request::PageLinks {
                doc_id,
                page_index,
                reply,
            })
            .map_err(|_| worker_gone())?;
        rx.await.map_err(|_| worker_gone())?
    }

    pub fn close(&self, doc_id: u64) {
        // Flag first so renders already queued ahead of the Close are skipped.
        self.closing.lock().unwrap().insert(doc_id);
        let _ = self.tx.send(Request::Close { doc_id });
    }
}

fn worker_gone() -> AppError {
    AppError::Message("PDF worker is not running".into())
}

fn worker_loop(
    rx: mpsc::Receiver<Request>,
    library_dirs: Vec<PathBuf>,
    closing: Arc<Mutex<HashSet<u64>>>,
) {
    // Leaking the Pdfium instance gives it a 'static lifetime so documents
    // borrowing it can be stored in the cache. It lives for the whole app anyway.
    let pdfium: std::result::Result<&'static Pdfium, String> = match init_pdfium(&library_dirs) {
        Ok(p) => Ok(Box::leak(Box::new(p))),
        Err(e) => Err(e.to_string()),
    };

    let mut docs: HashMap<u64, PdfDocument<'static>> = HashMap::new();
    let mut next_id: u64 = 1;

    // Fails fast for docs that are closing, so work queued ahead of a Close
    // isn't performed for nothing.
    fn get_doc<'a>(
        docs: &'a HashMap<u64, PdfDocument<'static>>,
        closing: &Mutex<HashSet<u64>>,
        doc_id: u64,
    ) -> Result<&'a PdfDocument<'static>> {
        if closing.lock().unwrap().contains(&doc_id) {
            return Err(AppError::Message(format!("document {doc_id} is closing")));
        }
        docs.get(&doc_id)
            .ok_or_else(|| AppError::Message(format!("unknown document {doc_id}")))
    }

    while let Ok(request) = rx.recv() {
        match request {
            Request::Open {
                path,
                cancel,
                reply,
            } => {
                let result = if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
                    Err(AppError::Message("open cancelled".into()))
                } else {
                    match pdfium {
                        Ok(pdfium) => open_document(pdfium, &path, &mut docs, &mut next_id),
                        Err(ref e) => Err(AppError::Pdf(e.clone())),
                    }
                };
                let _ = reply.send(result);
            }
            Request::Render {
                doc_id,
                page_index,
                width,
                reply,
            } => {
                let result = get_doc(&docs, &closing, doc_id)
                    .and_then(|doc| super::renderer::render_page_png(doc, page_index, width));
                reply(result);
            }
            Request::PageText {
                doc_id,
                page_index,
                reply,
            } => {
                let result = get_doc(&docs, &closing, doc_id)
                    .and_then(|doc| super::text::extract_text_runs(doc, page_index));
                let _ = reply.send(result);
            }
            Request::PageLinks {
                doc_id,
                page_index,
                reply,
            } => {
                let result = get_doc(&docs, &closing, doc_id)
                    .and_then(|doc| super::links::extract_links(doc, page_index));
                let _ = reply.send(result);
            }
            Request::Close { doc_id } => {
                docs.remove(&doc_id);
                closing.lock().unwrap().remove(&doc_id);
            }
        }
    }
}

fn open_document(
    pdfium: &'static Pdfium,
    path: &PathBuf,
    docs: &mut HashMap<u64, PdfDocument<'static>>,
    next_id: &mut u64,
) -> Result<OpenInfo> {
    let document = pdfium.load_pdf_from_file(path, None).map_err(|e| {
        eprintln!("failed to open {}: {e:?}", path.display());
        AppError::Message(open_error_message(&e).to_string())
    })?;

    let pages: Vec<PageSize> = document
        .pages()
        .iter()
        .map(|page| PageSize {
            width: page.width().value,
            height: page.height().value,
        })
        .collect();

    let doc_id = *next_id;
    *next_id += 1;
    docs.insert(doc_id, document);

    Ok(OpenInfo {
        doc_id,
        page_count: pages.len() as u16,
        pages,
    })
}

/// Turns a pdfium load failure into a message fit for the error screen.
fn open_error_message(error: &PdfiumError) -> &'static str {
    match error {
        PdfiumError::PdfiumLibraryInternalError(internal) => match internal {
            PdfiumInternalError::FormatError => {
                "The file is damaged or isn't a valid PDF, so it can't be displayed."
            }
            PdfiumInternalError::PasswordError => {
                "This PDF is password-protected, which isn't supported yet."
            }
            PdfiumInternalError::SecurityError => {
                "This PDF's security settings prevent it from being opened."
            }
            PdfiumInternalError::FileError => {
                "The file couldn't be read. It may have been moved, deleted, or is inaccessible."
            }
            _ => "Something went wrong while opening this PDF.",
        },
        _ => "Something went wrong while opening this PDF.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // pdfium-render's thread-safe bindings hold a global lock from
    // FPDF_InitLibrary until FPDF_DestroyLibrary, and the worker leaks its
    // Pdfium instance, so a second worker in the same process would block
    // forever. All tests share one worker, exactly like the app does.
    fn test_worker() -> &'static PdfWorker {
        static WORKER: std::sync::OnceLock<PdfWorker> = std::sync::OnceLock::new();
        WORKER.get_or_init(|| {
            let pdfium_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/pdfium");
            PdfWorker::spawn(vec![pdfium_dir])
        })
    }

    /// Full path through the worker: bind pdfium, open a document, render a
    /// page to PNG. Requires `npm run setup` to have downloaded the library.
    #[test]
    fn opens_and_renders_a_page() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
        let worker = test_worker();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let info = runtime
            .block_on(worker.open_cancellable(fixture, None))
            .expect("open fixture");
        assert_eq!(info.page_count, 3);
        assert_eq!(info.pages.len(), 3);
        assert!((info.pages[0].width - 612.0).abs() < 0.5);
        assert!((info.pages[0].height - 792.0).abs() < 0.5);

        let (tx, rx) = mpsc::channel();
        worker.render_with(info.doc_id, 0, 400, move |result| {
            let _ = tx.send(result);
        });
        let png = rx.recv().unwrap().expect("render page 0");
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));

        let runs = runtime
            .block_on(worker.page_text(info.doc_id, 0))
            .expect("text of page 0");
        assert!(!runs.is_empty());
        for run in &runs {
            assert!(!run.text.trim().is_empty());
            assert!(run.width > 0.0 && run.height > 0.0);
            assert!(run.x >= 0.0 && run.x + run.width <= info.pages[0].width + 1.0);
            assert!(run.y >= 0.0 && run.y + run.height <= info.pages[0].height + 1.0);
        }

        worker.close(info.doc_id);
    }

    /// links.pdf has two link annotations on page 1: a URI link over
    /// [72, 700, 200, 720] and a GoTo link to page 2 over [72, 650, 200, 670]
    /// (bottom-up PDF coordinates on a 612x792 page).
    #[test]
    fn extracts_page_links() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/links.pdf");
        let worker = test_worker();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let info = runtime
            .block_on(worker.open_cancellable(fixture, None))
            .expect("open fixture");
        assert_eq!(info.page_count, 2);

        let links = runtime
            .block_on(worker.page_links(info.doc_id, 0))
            .expect("links of page 0");
        assert_eq!(links.len(), 2);

        let uri_link = links.iter().find(|l| l.uri.is_some()).expect("uri link");
        assert_eq!(uri_link.uri.as_deref(), Some("https://example.com/"));
        assert!((uri_link.x - 72.0).abs() < 0.5);
        assert!((uri_link.y - (792.0 - 720.0)).abs() < 0.5);
        assert!((uri_link.width - 128.0).abs() < 0.5);
        assert!((uri_link.height - 20.0).abs() < 0.5);

        let goto_link = links.iter().find(|l| l.page.is_some()).expect("goto link");
        assert_eq!(goto_link.page, Some(1));
        assert!((goto_link.y - (792.0 - 670.0)).abs() < 0.5);

        assert!(runtime
            .block_on(worker.page_links(info.doc_id, 1))
            .expect("links of page 1")
            .is_empty());

        worker.close(info.doc_id);
    }

    /// An Open whose cancel flag is set by the time the worker dequeues it
    /// is skipped without touching pdfium.
    #[test]
    fn cancelled_open_is_skipped() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
        let worker = test_worker();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let cancel: CancelFlag = Arc::new(AtomicBool::new(true));
        let err = runtime
            .block_on(worker.open_cancellable(fixture, Some(cancel)))
            .expect_err("cancelled open should not succeed");
        assert_eq!(err.to_string(), "open cancelled");
    }

    /// Opening a corrupted/non-PDF file should fail with a friendly message
    /// rather than surfacing pdfium's internal error string.
    #[test]
    fn open_reports_friendly_error_for_corrupt_file() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/corrupt.pdf");
        let worker = test_worker();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let err = runtime
            .block_on(worker.open_cancellable(fixture, None))
            .expect_err("corrupt file should fail to open");
        assert_eq!(
            err.to_string(),
            "The file is damaged or isn't a valid PDF, so it can't be displayed."
        );
    }
}

fn init_pdfium(dirs: &[PathBuf]) -> Result<Pdfium> {
    for dir in dirs {
        let lib_path = Pdfium::pdfium_platform_library_name_at_path(dir);
        if let Ok(bindings) = Pdfium::bind_to_library(&lib_path) {
            return Ok(Pdfium::new(bindings));
        }
    }
    Pdfium::bind_to_system_library()
        .map(Pdfium::new)
        .map_err(|e| {
            AppError::Pdf(format!(
                "could not load the pdfium library (searched {dirs:?} and system paths); \
                 run `npm run setup` to download it ({e})"
            ))
        })
}
