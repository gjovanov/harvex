use std::path::Path;
use tracing::{debug, info, warn};

/// Rendered pages from a PDF.
pub struct RenderedPages {
    /// JPEG bytes for each rendered page.
    pub pages: Vec<Vec<u8>>,
}

/// Render PDF pages to JPEG images using `pdftoppm` (poppler-utils).
///
/// # Arguments
/// * `pdf_path` - Path to the PDF file
/// * `dpi` - Resolution for rendering (e.g. 200)
/// * `max_pages` - Maximum number of pages to render
pub fn render_pdf_pages(
    pdf_path: &Path,
    dpi: u32,
    max_pages: u32,
) -> Result<RenderedPages, anyhow::Error> {
    let tmp_dir = tempfile::TempDir::new()?;
    let output_prefix = tmp_dir.path().join("page");

    debug!(
        "Rendering PDF pages: path={}, dpi={}, max_pages={}",
        pdf_path.display(),
        dpi,
        max_pages
    );

    let output = std::process::Command::new("pdftoppm")
        .args([
            "-jpeg",
            "-jpegopt",
            "quality=85",
            "-r",
            &dpi.to_string(),
            "-l",
            &max_pages.to_string(),
        ])
        .arg(pdf_path)
        .arg(&output_prefix)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "pdftoppm not found. Install poppler-utils: apt-get install poppler-utils"
                )
            } else {
                anyhow::anyhow!("Failed to run pdftoppm: {e}")
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("pdftoppm failed: {stderr}"));
    }

    // pdftoppm outputs files like: page-1.jpg, page-2.jpg, ...
    // or page-01.jpg, page-02.jpg, ... depending on page count
    let mut page_files: Vec<_> = std::fs::read_dir(tmp_dir.path())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|ext| ext == "jpg")
        })
        .collect();

    // Sort by filename to ensure correct page order
    page_files.sort_by_key(|e| e.file_name());

    let mut pages = Vec::new();
    for entry in &page_files {
        let bytes = std::fs::read(entry.path())?;
        debug!(
            "Rendered page: {} ({} bytes)",
            entry.file_name().to_string_lossy(),
            bytes.len()
        );
        pages.push(bytes);
    }

    if pages.is_empty() {
        warn!("pdftoppm produced no output files for {}", pdf_path.display());
        return Err(anyhow::anyhow!(
            "pdftoppm produced no output for {}",
            pdf_path.display()
        ));
    }

    info!(
        "Rendered {} pages from {} (dpi={})",
        pages.len(),
        pdf_path.display(),
        dpi
    );

    Ok(RenderedPages { pages })
}
