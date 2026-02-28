use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use tokio::sync::broadcast;
use tracing::{info, warn};

use harvex_config::LlmSettings;
use harvex_db::models::Document;
use harvex_db::DbPool;

use crate::dao::{BatchDao, DocumentDao, ExtractionDao};
use crate::llm::LlmEngine;

use super::detector::FileType;
use super::{excel, ocr, pdf, pdf_render, word};

/// Progress event sent via SSE to clients.
#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub batch_id: String,
    pub document_id: String,
    pub document_name: String,
    pub status: String,
    pub message: String,
    pub processed: i32,
    pub failed: i32,
    pub total: i32,
}

/// Result of text extraction — either usable text or a path needing vision processing.
enum ExtractedContent {
    /// Text was extracted successfully; proceed with text LLM.
    Text(String),
    /// Scanned PDF — needs vision LLM. Contains the file path.
    NeedsVisionPdf(PathBuf),
    /// Image file — needs vision LLM. Contains the raw image bytes.
    NeedsVisionImage(Vec<u8>),
}

/// The processing pipeline. Holds a broadcast sender for progress events
/// and an LLM engine for structured data extraction.
pub struct Pipeline {
    db: DbPool,
    max_concurrent: usize,
    llm: Arc<LlmEngine>,
    progress_tx: broadcast::Sender<ProgressEvent>,
}

impl Pipeline {
    pub fn new(db: DbPool, max_concurrent: usize, llm_settings: LlmSettings) -> Self {
        let (progress_tx, _) = broadcast::channel(256);
        let llm = Arc::new(LlmEngine::new(llm_settings));
        Self {
            db,
            max_concurrent,
            llm,
            progress_tx,
        }
    }

    /// Get a clone of the LLM engine (for sharing with API routes).
    pub fn llm_engine(&self) -> Arc<LlmEngine> {
        self.llm.clone()
    }

    /// Get a clone of the sender (for sharing with AppState).
    pub fn progress_sender(&self) -> broadcast::Sender<ProgressEvent> {
        self.progress_tx.clone()
    }

    /// Process all documents in a batch.
    ///
    /// For each document: extract text, call LLM for structured data, store results.
    /// Documents are processed with limited concurrency using a semaphore.
    pub async fn process_batch(&self, batch_id: &str) -> Result<(), anyhow::Error> {
        let batch = BatchDao::get_by_id(&self.db, batch_id)
            .map_err(|_| anyhow::anyhow!("Batch {batch_id} not found"))?;

        if batch.status == "processing" {
            return Err(anyhow::anyhow!("Batch is already being processed"));
        }

        BatchDao::update_status(&self.db, batch_id, "processing")?;
        info!(
            "Starting batch processing: {} ({} files)",
            batch.name, batch.total_files
        );

        let documents = DocumentDao::list_by_batch(&self.db, batch_id)?;
        let total = documents.len() as i32;

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent));
        let processed = Arc::new(std::sync::atomic::AtomicI32::new(0));
        let failed = Arc::new(std::sync::atomic::AtomicI32::new(0));

        let mut handles = Vec::new();

        for doc in documents {
            let db = self.db.clone();
            let tx = self.progress_tx.clone();
            let sem = semaphore.clone();
            let proc_count = processed.clone();
            let fail_count = failed.clone();
            let batch_id = batch_id.to_string();
            let llm = self.llm.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.expect("semaphore closed");
                let result = process_document(&db, &doc, &llm).await;

                match result {
                    Ok(msg) => {
                        let p =
                            proc_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                        let f = fail_count.load(std::sync::atomic::Ordering::Relaxed);
                        let _ = BatchDao::update_progress(&db, &batch_id, p, f);

                        let _ = tx.send(ProgressEvent {
                            batch_id: batch_id.clone(),
                            document_id: doc.id.clone(),
                            document_name: doc.original_name.clone(),
                            status: "completed".into(),
                            message: msg,
                            processed: p,
                            failed: f,
                            total,
                        });
                    }
                    Err(e) => {
                        let f =
                            fail_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                        let p = proc_count.load(std::sync::atomic::Ordering::Relaxed);
                        let _ = BatchDao::update_progress(&db, &batch_id, p, f);

                        let _ = tx.send(ProgressEvent {
                            batch_id: batch_id.clone(),
                            document_id: doc.id.clone(),
                            document_name: doc.original_name.clone(),
                            status: "failed".into(),
                            message: format!("Failed: {e}"),
                            processed: p,
                            failed: f,
                            total,
                        });
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all documents to finish
        for handle in handles {
            let _ = handle.await;
        }

        // Determine final batch status
        let p = processed.load(std::sync::atomic::Ordering::Relaxed);
        let f = failed.load(std::sync::atomic::Ordering::Relaxed);

        let final_status = if f == 0 {
            "completed"
        } else if p == 0 {
            "failed"
        } else {
            "partially_completed"
        };

        BatchDao::update_status(&self.db, batch_id, final_status)?;
        info!(
            "Batch {} finished: {} processed, {} failed",
            batch_id, p, f
        );

        // Send final event
        let _ = self.progress_tx.send(ProgressEvent {
            batch_id: batch_id.to_string(),
            document_id: String::new(),
            document_name: String::new(),
            status: final_status.into(),
            message: format!("Batch complete: {p} processed, {f} failed"),
            processed: p,
            failed: f,
            total,
        });

        Ok(())
    }
}

/// Process a single document: detect type → extract text → LLM inference → store.
async fn process_document(
    db: &DbPool,
    doc: &Document,
    llm: &LlmEngine,
) -> Result<String, anyhow::Error> {
    let file_path = Path::new(&doc.file_path);

    if !file_path.exists() {
        DocumentDao::update_status(db, &doc.id, "failed", Some("File not found on disk"))?;
        return Err(anyhow::anyhow!("File not found: {}", doc.file_path));
    }

    DocumentDao::update_status(db, &doc.id, "processing", None)?;

    let file_type = FileType::detect(&doc.original_name, &doc.content_type);
    info!("Processing {} as {}", doc.original_name, file_type.label());

    let start = Instant::now();

    // Step 1: Extract text based on file type (blocking I/O)
    let path = file_path.to_path_buf();
    let ft = file_type.clone();

    let extracted =
        tokio::task::spawn_blocking(move || -> Result<ExtractedContent, anyhow::Error> {
            match ft {
                FileType::Pdf => {
                    let result = pdf::extract_text(&path)?;
                    if result.is_scanned && result.text.is_empty() {
                        warn!("Scanned PDF detected, no text extracted. Needs LLM vision.");
                        Ok(ExtractedContent::NeedsVisionPdf(path))
                    } else {
                        Ok(ExtractedContent::Text(result.text))
                    }
                }
                FileType::Excel => {
                    let result = excel::extract_text(&path)?;
                    Ok(ExtractedContent::Text(result.text))
                }
                FileType::Word => {
                    let result = word::extract_text(&path)?;
                    Ok(ExtractedContent::Text(result.text))
                }
                FileType::Image => {
                    let result = ocr::extract_text(&path)?;
                    if result.needs_llm_vision {
                        let bytes = std::fs::read(&path)?;
                        Ok(ExtractedContent::NeedsVisionImage(bytes))
                    } else {
                        Ok(ExtractedContent::Text(result.text))
                    }
                }
                FileType::Unknown(ext) => {
                    Err(anyhow::anyhow!("Unsupported file type: .{ext}"))
                }
            }
        })
        .await??;

    let extract_elapsed_ms = start.elapsed().as_millis() as i64;

    match extracted {
        ExtractedContent::Text(raw_text) => {
            process_text_path(db, doc, llm, &raw_text, extract_elapsed_ms).await
        }
        ExtractedContent::NeedsVisionPdf(pdf_path) => {
            process_vision_pdf_path(db, doc, llm, &pdf_path, extract_elapsed_ms).await
        }
        ExtractedContent::NeedsVisionImage(image_bytes) => {
            process_vision_image_path(db, doc, llm, &image_bytes, extract_elapsed_ms).await
        }
    }
}

/// Text path: classify → text LLM → store (existing behavior).
async fn process_text_path(
    db: &DbPool,
    doc: &Document,
    llm: &LlmEngine,
    raw_text: &str,
    extract_elapsed_ms: i64,
) -> Result<String, anyhow::Error> {
    let doc_type = classify_document_type(raw_text);

    let extraction = ExtractionDao::create(
        db,
        &doc.id,
        &doc.batch_id,
        doc_type,
        Some(raw_text),
        None,
        0.0,
        None,
        extract_elapsed_ms,
    )?;

    let llm_result = llm.extract_structured(raw_text, doc_type).await;

    match llm_result {
        Ok(response) => {
            ExtractionDao::update_structured(
                db,
                &extraction.id,
                &response.document_type,
                Some(&response.structured_data),
                response.confidence,
                Some(&response.model_used),
                extract_elapsed_ms + response.processing_time_ms,
            )?;

            DocumentDao::update_status(db, &doc.id, "completed", None)?;

            Ok(format!(
                "Extracted {} chars, LLM structured as {} (confidence: {:.0}%)",
                raw_text.len(),
                response.document_type,
                response.confidence * 100.0
            ))
        }
        Err(e) => {
            warn!(
                "LLM inference failed for {}: {e}. Keeping raw text extraction.",
                doc.original_name
            );
            DocumentDao::update_status(db, &doc.id, "completed", None)?;

            Ok(format!(
                "Extracted {} chars (LLM unavailable: {e})",
                raw_text.len()
            ))
        }
    }
}

/// Vision PDF path: render pages → vision LLM → store.
async fn process_vision_pdf_path(
    db: &DbPool,
    doc: &Document,
    llm: &LlmEngine,
    pdf_path: &Path,
    extract_elapsed_ms: i64,
) -> Result<String, anyhow::Error> {
    if !llm.has_vision() {
        warn!(
            "Scanned PDF {} needs vision LLM but no vision model configured. Skipping.",
            doc.original_name
        );
        let raw_text = "[Scanned PDF — no text extracted. Vision model not configured.]";

        let extraction = ExtractionDao::create(
            db,
            &doc.id,
            &doc.batch_id,
            "other",
            Some(raw_text),
            None,
            0.0,
            None,
            extract_elapsed_ms,
        )?;

        DocumentDao::update_status(db, &doc.id, "completed", None)?;

        return Ok(format!(
            "Scanned PDF — vision model not configured (extraction {})",
            extraction.id
        ));
    }

    let settings = llm.settings();
    let dpi = settings.vision_dpi;
    let max_pages = settings.vision_max_pages;

    // Render PDF pages to JPEG (blocking I/O)
    let path = pdf_path.to_path_buf();
    let rendered = tokio::task::spawn_blocking(move || {
        pdf_render::render_pdf_pages(&path, dpi, max_pages)
    })
    .await??;

    let page_count = rendered.pages.len();
    let raw_text = format!("[Vision: {} pages processed]", page_count);

    let extraction = ExtractionDao::create(
        db,
        &doc.id,
        &doc.batch_id,
        "other",
        Some(&raw_text),
        None,
        0.0,
        None,
        extract_elapsed_ms,
    )?;

    // Vision LLM extraction
    let llm_result = llm
        .extract_structured_with_vision(&rendered.pages, "other")
        .await;

    match llm_result {
        Ok(response) => {
            ExtractionDao::update_structured(
                db,
                &extraction.id,
                &response.document_type,
                Some(&response.structured_data),
                response.confidence,
                Some(&response.model_used),
                extract_elapsed_ms + response.processing_time_ms,
            )?;

            DocumentDao::update_status(db, &doc.id, "completed", None)?;

            Ok(format!(
                "Vision: {} pages, structured as {} (confidence: {:.0}%, model: {})",
                page_count,
                response.document_type,
                response.confidence * 100.0,
                response.model_used
            ))
        }
        Err(e) => {
            warn!(
                "Vision LLM failed for {}: {e}",
                doc.original_name
            );
            DocumentDao::update_status(db, &doc.id, "completed", None)?;

            Ok(format!(
                "Vision: {} pages rendered (LLM failed: {e})",
                page_count
            ))
        }
    }
}

/// Vision image path: send image bytes → vision LLM → store.
async fn process_vision_image_path(
    db: &DbPool,
    doc: &Document,
    llm: &LlmEngine,
    image_bytes: &[u8],
    extract_elapsed_ms: i64,
) -> Result<String, anyhow::Error> {
    if !llm.has_vision() {
        warn!(
            "Image {} needs vision LLM but no vision model configured. Skipping.",
            doc.original_name
        );
        let raw_text = "[Image — requires vision LLM. Vision model not configured.]";

        ExtractionDao::create(
            db,
            &doc.id,
            &doc.batch_id,
            "other",
            Some(raw_text),
            None,
            0.0,
            None,
            extract_elapsed_ms,
        )?;

        DocumentDao::update_status(db, &doc.id, "completed", None)?;

        return Ok("Image — vision model not configured".to_string());
    }

    let raw_text = format!("[Vision: 1 image processed ({} bytes)]", image_bytes.len());

    let extraction = ExtractionDao::create(
        db,
        &doc.id,
        &doc.batch_id,
        "other",
        Some(&raw_text),
        None,
        0.0,
        None,
        extract_elapsed_ms,
    )?;

    let llm_result = llm
        .extract_structured_with_vision(&[image_bytes.to_vec()], "other")
        .await;

    match llm_result {
        Ok(response) => {
            ExtractionDao::update_structured(
                db,
                &extraction.id,
                &response.document_type,
                Some(&response.structured_data),
                response.confidence,
                Some(&response.model_used),
                extract_elapsed_ms + response.processing_time_ms,
            )?;

            DocumentDao::update_status(db, &doc.id, "completed", None)?;

            Ok(format!(
                "Vision image: structured as {} (confidence: {:.0}%, model: {})",
                response.document_type,
                response.confidence * 100.0,
                response.model_used
            ))
        }
        Err(e) => {
            warn!(
                "Vision LLM failed for image {}: {e}",
                doc.original_name
            );
            DocumentDao::update_status(db, &doc.id, "completed", None)?;

            Ok(format!("Image processed (LLM failed: {e})"))
        }
    }
}

/// Simple heuristic to classify document type based on extracted text.
fn classify_document_type(text: &str) -> &'static str {
    let lower = text.to_lowercase();

    if lower.contains("invoice")
        || lower.contains("faktura")
        || lower.contains("bill to")
        || lower.contains("invoice number")
        || lower.contains("inv no")
    {
        "invoice"
    } else if lower.contains("bank statement")
        || lower.contains("account statement")
        || lower.contains("transaction history")
        || (lower.contains("balance") && (lower.contains("debit") || lower.contains("credit")))
    {
        "bank_statement"
    } else if lower.contains("payment")
        || lower.contains("paid")
        || lower.contains("amount due")
    {
        "payment"
    } else if lower.contains("receipt")
        || lower.contains("cash register")
        || (lower.contains("total") && lower.contains("tax"))
    {
        "receipt"
    } else {
        "other"
    }
}
