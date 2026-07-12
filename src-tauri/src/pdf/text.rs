use crate::error::{AppError, Result};
use pdfium_render::prelude::*;
use serde::Serialize;

/// A rectangular run of text on a page (pdfium merges characters that share a
/// baseline and font into one segment). Coordinates are PDF points with a
/// top-left origin, matching the frontend's page layout.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRun {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub fn extract_text_runs(document: &PdfDocument, page_index: u16) -> Result<Vec<TextRun>> {
    let page = document
        .pages()
        .get(page_index)
        .map_err(|e| AppError::Pdf(format!("page {page_index}: {e}")))?;

    let page_height = page.height().value;
    let text = page
        .text()
        .map_err(|e| AppError::Pdf(format!("text of page {page_index}: {e}")))?;

    let runs = text
        .segments()
        .iter()
        .filter_map(|segment| {
            let raw = segment.text();
            let trimmed = raw.trim_end_matches(['\r', '\n']);
            if trimmed.trim().is_empty() {
                return None;
            }
            let bounds = segment.bounds();
            Some(TextRun {
                text: trimmed.to_string(),
                x: bounds.left().value,
                y: page_height - bounds.top().value,
                width: bounds.width().value,
                height: bounds.height().value,
            })
        })
        .collect();

    Ok(runs)
}
