//! Custom `pdfp://` URI scheme serving rendered page bitmaps, so the
//! frontend can load pages as plain `<img>` elements with browser caching
//! instead of shuttling megabytes of pixels through the IPC bridge.
//!
//! URL shape: `pdfp://localhost/{docId}/{pageIndex}?w={pixelWidth}`

use crate::state::AppState;
use tauri::http;
use tauri::{Manager, UriSchemeContext, UriSchemeResponder};

pub const SCHEME: &str = "pdfp";

pub fn handle<R: tauri::Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let uri = request.uri();
    let Some((doc_id, page_index, width)) = parse(uri.path(), uri.query()) else {
        responder.respond(error_response(400, "malformed pdfp:// URL"));
        return;
    };

    let state = ctx.app_handle().state::<AppState>();
    state
        .pdf
        .render_with(doc_id, page_index, width, move |result: crate::error::Result<Vec<u8>>| match result {
            Ok(png) => {
                let response = http::Response::builder()
                    .status(200)
                    .header("Content-Type", "image/png")
                    .header("Cache-Control", "public, max-age=3600")
                    .body(png)
                    .unwrap();
                responder.respond(response);
            }
            Err(e) => responder.respond(error_response(500, &e.to_string())),
        });
}

fn parse(path: &str, query: Option<&str>) -> Option<(u64, u16, u32)> {
    let mut segments = path.trim_start_matches('/').split('/');
    let doc_id: u64 = segments.next()?.parse().ok()?;
    let page_index: u16 = segments.next()?.parse().ok()?;
    let width: u32 = query?
        .split('&')
        .find_map(|kv| kv.strip_prefix("w="))?
        .parse()
        .ok()?;
    Some((doc_id, page_index, width))
}

fn error_response(status: u16, message: &str) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(status)
        .header("Content-Type", "text/plain")
        .body(message.as_bytes().to_vec())
        .unwrap()
}
