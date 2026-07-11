use crate::error::{AppError, Result};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};
use pdfium_render::prelude::*;
use std::io::Cursor;

const MIN_WIDTH: u32 = 16;
const MAX_WIDTH: u32 = 8192;

/// Render one page at the requested pixel width and encode it as PNG.
pub fn render_page_png(document: &PdfDocument, page_index: u16, width: u32) -> Result<Vec<u8>> {
    let page = document
        .pages()
        .get(page_index)
        .map_err(|e| AppError::Pdf(format!("page {page_index}: {e}")))?;

    let width = width.clamp(MIN_WIDTH, MAX_WIDTH);
    let config = PdfRenderConfig::new()
        .set_target_width(width as i32)
        .render_form_data(true);

    let bitmap = page
        .render_with_config(&config)
        .map_err(|e| AppError::Pdf(format!("render page {page_index}: {e}")))?;

    let (w, h) = (bitmap.width() as u32, bitmap.height() as u32);
    let rgba = bitmap.as_rgba_bytes();

    let mut out = Cursor::new(Vec::with_capacity((w * h) as usize));
    PngEncoder::new_with_quality(&mut out, CompressionType::Fast, FilterType::Adaptive)
        .write_image(&rgba, w, h, ExtendedColorType::Rgba8)
        .map_err(|e| AppError::Message(format!("PNG encoding failed: {e}")))?;

    Ok(out.into_inner())
}
