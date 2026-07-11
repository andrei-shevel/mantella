//! All pdfium access happens on a single dedicated worker thread, because
//! pdfium-render types wrap raw FPDF_* pointers and are `!Send`. The rest of
//! the app talks to the worker through `PdfWorker`, a cheap cloneable handle.

use crate::error::{AppError, Result};
use pdfium_render::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use tokio::sync::oneshot;

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
        reply: oneshot::Sender<Result<OpenInfo>>,
    },
    Render {
        doc_id: u64,
        page_index: u16,
        width: u32,
        reply: RenderCallback,
    },
    Close {
        doc_id: u64,
    },
}

#[derive(Clone)]
pub struct PdfWorker {
    tx: mpsc::Sender<Request>,
}

impl PdfWorker {
    pub fn spawn(library_dirs: Vec<PathBuf>) -> Self {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("pdf-worker".into())
            .spawn(move || worker_loop(rx, library_dirs))
            .expect("failed to spawn pdf worker thread");
        Self { tx }
    }

    pub async fn open(&self, path: PathBuf) -> Result<OpenInfo> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(Request::Open { path, reply })
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

    pub fn close(&self, doc_id: u64) {
        let _ = self.tx.send(Request::Close { doc_id });
    }
}

fn worker_gone() -> AppError {
    AppError::Message("PDF worker is not running".into())
}

fn worker_loop(rx: mpsc::Receiver<Request>, library_dirs: Vec<PathBuf>) {
    // Leaking the Pdfium instance gives it a 'static lifetime so documents
    // borrowing it can be stored in the cache. It lives for the whole app anyway.
    let pdfium: std::result::Result<&'static Pdfium, String> = match init_pdfium(&library_dirs) {
        Ok(p) => Ok(Box::leak(Box::new(p))),
        Err(e) => Err(e.to_string()),
    };

    let mut docs: HashMap<u64, PdfDocument<'static>> = HashMap::new();
    let mut next_id: u64 = 1;

    while let Ok(request) = rx.recv() {
        match request {
            Request::Open { path, reply } => {
                let result = match pdfium {
                    Ok(pdfium) => open_document(pdfium, &path, &mut docs, &mut next_id),
                    Err(ref e) => Err(AppError::Pdf(e.clone())),
                };
                let _ = reply.send(result);
            }
            Request::Render {
                doc_id,
                page_index,
                width,
                reply,
            } => {
                let result = docs
                    .get(&doc_id)
                    .ok_or_else(|| AppError::Message(format!("unknown document {doc_id}")))
                    .and_then(|doc| super::renderer::render_page_png(doc, page_index, width));
                reply(result);
            }
            Request::Close { doc_id } => {
                docs.remove(&doc_id);
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
    let document = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|e| AppError::Pdf(format!("failed to open {}: {e}", path.display())))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Full path through the worker: bind pdfium, open a document, render a
    /// page to PNG. Requires `npm run setup` to have downloaded the library.
    #[test]
    fn opens_and_renders_a_page() {
        let pdfium_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/pdfium");
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
        let worker = PdfWorker::spawn(vec![pdfium_dir]);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let info = runtime.block_on(worker.open(fixture)).expect("open fixture");
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

        worker.close(info.doc_id);
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
