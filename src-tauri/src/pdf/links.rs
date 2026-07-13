use crate::error::{AppError, Result};
use pdfium_render::prelude::*;
use serde::Serialize;

/// A clickable link region on a page. Coordinates are PDF points with a
/// top-left origin, matching the frontend's page layout. Exactly one of
/// `uri` (external target) or `page` (zero-based internal target) is set.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageLink {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub uri: Option<String>,
    pub page: Option<u16>,
}

pub fn extract_links(document: &PdfDocument, page_index: u16) -> Result<Vec<PageLink>> {
    let page = document
        .pages()
        .get(page_index)
        .map_err(|e| AppError::Pdf(format!("page {page_index}: {e}")))?;

    let page_height = page.height().value;

    let links = page
        .links()
        .iter()
        .filter_map(|link| {
            let rect = link.rect().ok()?;
            let (uri, target_page) = link_target(&link)?;
            Some(PageLink {
                x: rect.left().value,
                y: page_height - rect.top().value,
                width: rect.width().value,
                height: rect.height().value,
                uri,
                page: target_page,
            })
        })
        .collect();

    Ok(links)
}

fn link_target(link: &PdfLink) -> Option<(Option<String>, Option<u16>)> {
    match link.action() {
        Some(PdfAction::Uri(action)) => {
            let uri = action.uri().ok()?;
            (!uri.is_empty()).then_some((Some(uri), None))
        }
        Some(PdfAction::LocalDestination(action)) => {
            let page = action.destination().ok()?.page_index().ok()?;
            Some((None, Some(page as u16)))
        }
        // Launch, remote/embedded goto: not supported.
        Some(_) => None,
        // Some GoTo links carry a bare destination instead of an action.
        None => {
            let page = link.destination()?.page_index().ok()?;
            Some((None, Some(page as u16)))
        }
    }
}
