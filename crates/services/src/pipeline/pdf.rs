use std::path::Path;
use tracing::{debug, warn};

/// Extract text from a PDF file.
///
/// Uses `pdf-extract` for text-based PDFs. If the extracted text is empty
/// or very short (likely a scanned/image PDF), returns an indication
/// that OCR is needed.
pub fn extract_text(file_path: &Path) -> Result<ExtractedPdf, anyhow::Error> {
    debug!("Extracting text from PDF: {}", file_path.display());

    let bytes = std::fs::read(file_path)?;
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| anyhow::anyhow!("PDF text extraction failed: {e}"))?;

    let trimmed = text.trim().to_string();

    if trimmed.is_empty() || trimmed.len() < 20 {
        warn!(
            "PDF has little/no extractable text ({} chars), likely scanned",
            trimmed.len()
        );
        Ok(ExtractedPdf {
            text: trimmed,
            is_scanned: true,
            page_count: count_pages(&bytes),
        })
    } else {
        Ok(ExtractedPdf {
            text: trimmed,
            is_scanned: false,
            page_count: count_pages(&bytes),
        })
    }
}

pub struct ExtractedPdf {
    pub text: String,
    pub is_scanned: bool,
    pub page_count: Option<usize>,
}

/// Rough page count from PDF bytes by counting "/Type /Page" occurrences.
fn count_pages(bytes: &[u8]) -> Option<usize> {
    let content = String::from_utf8_lossy(bytes);
    let count = content.matches("/Type /Page").count();
    // Subtract 1 for the /Type /Pages catalog entry (if present)
    if count > 1 {
        Some(count - 1)
    } else if count == 1 {
        Some(1)
    } else {
        None
    }
}
