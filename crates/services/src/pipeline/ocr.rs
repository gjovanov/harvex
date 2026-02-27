use image::DynamicImage;
use std::path::Path;
use tracing::{debug, warn};

/// Preprocess an image for better OCR/LLM readability.
///
/// Converts to grayscale, adjusts contrast, and optionally resizes
/// to ensure the image is at a reasonable resolution.
pub fn preprocess_image(file_path: &Path) -> Result<DynamicImage, anyhow::Error> {
    debug!("Preprocessing image: {}", file_path.display());

    let img = image::open(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to open image: {e}"))?;

    let (w, h) = (img.width(), img.height());
    debug!("Image dimensions: {}x{}", w, h);

    // Convert to grayscale for text extraction
    let gray = img.grayscale();

    // If image is very small, upscale for better OCR
    let processed = if w < 300 || h < 300 {
        let scale = (300.0 / w.min(h) as f64).ceil() as u32;
        gray.resize(
            w * scale,
            h * scale,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        gray
    };

    // Adjust contrast for better text visibility
    let processed = image::imageops::contrast(&processed.to_luma8(), 30.0);

    Ok(DynamicImage::ImageLuma8(processed))
}

/// Extract text from an image file.
///
/// Currently returns the image metadata since Tesseract is not installed.
/// When Tesseract (leptess) or LLM vision is available, this will perform
/// actual OCR. For now, the image can be passed to the LLM in Phase 6.
pub fn extract_text(file_path: &Path) -> Result<ExtractedImage, anyhow::Error> {
    debug!("Extracting text from image: {}", file_path.display());

    let img = image::open(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to open image: {e}"))?;

    let (w, h) = (img.width(), img.height());

    warn!(
        "OCR not available (Tesseract not installed). \
         Image {}x{} will need LLM vision processing.",
        w, h
    );

    Ok(ExtractedImage {
        text: String::new(),
        width: w,
        height: h,
        needs_llm_vision: true,
    })
}

pub struct ExtractedImage {
    pub text: String,
    pub width: u32,
    pub height: u32,
    pub needs_llm_vision: bool,
}
