use std::path::Path;
use std::time::Instant;

use serde::Serialize;
use tokio::sync::broadcast;
use tracing::{info, warn};

use harvex_db::models::Document;
use harvex_db::DbPool;

use crate::dao::{BatchDao, DocumentDao, ExtractionDao};

use super::detector::FileType;
use super::{excel, ocr, pdf, word};

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

/// The processing pipeline. Holds a broadcast sender for progress events.
pub struct Pipeline {
    db: DbPool,
    max_concurrent: usize,
    progress_tx: broadcast::Sender<ProgressEvent>,
}

impl Pipeline {
    pub fn new(db: DbPool, max_concurrent: usize) -> Self {
        let (progress_tx, _) = broadcast::channel(256);
        Self {
            db,
            max_concurrent,
            progress_tx,
        }
    }

    /// Get a receiver for progress events.
    pub fn subscribe(&self) -> broadcast::Receiver<ProgressEvent> {
        self.progress_tx.subscribe()
    }

    /// Get a clone of the sender (for sharing with AppState).
    pub fn progress_sender(&self) -> broadcast::Sender<ProgressEvent> {
        self.progress_tx.clone()
    }

    /// Process all documents in a batch.
    ///
    /// Documents are processed with limited concurrency using a semaphore.
    /// Progress events are sent for each document as it completes.
    pub async fn process_batch(&self, batch_id: &str) -> Result<(), anyhow::Error> {
        let batch = BatchDao::get_by_id(&self.db, batch_id)
            .map_err(|_| anyhow::anyhow!("Batch {batch_id} not found"))?;

        if batch.status == "processing" {
            return Err(anyhow::anyhow!("Batch is already being processed"));
        }

        BatchDao::update_status(&self.db, batch_id, "processing")?;
        info!("Starting batch processing: {} ({} files)", batch.name, batch.total_files);

        let documents = DocumentDao::list_by_batch(&self.db, batch_id)?;
        let total = documents.len() as i32;

        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(self.max_concurrent));
        let processed = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
        let failed = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));

        let mut handles = Vec::new();

        for doc in documents {
            let db = self.db.clone();
            let tx = self.progress_tx.clone();
            let sem = semaphore.clone();
            let proc_count = processed.clone();
            let fail_count = failed.clone();
            let batch_id = batch_id.to_string();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.expect("semaphore closed");
                let result = process_document(&db, &doc).await;

                match result {
                    Ok(text) => {
                        let p = proc_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                        let f = fail_count.load(std::sync::atomic::Ordering::Relaxed);
                        let _ = BatchDao::update_progress(&db, &batch_id, p, f);

                        let _ = tx.send(ProgressEvent {
                            batch_id: batch_id.clone(),
                            document_id: doc.id.clone(),
                            document_name: doc.original_name.clone(),
                            status: "completed".into(),
                            message: format!(
                                "Extracted {} chars from {}",
                                text.len(),
                                doc.original_name
                            ),
                            processed: p,
                            failed: f,
                            total,
                        });
                    }
                    Err(e) => {
                        let f = fail_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
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
        info!("Batch {} finished: {} processed, {} failed", batch_id, p, f);

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

/// Process a single document: detect type, extract text, create extraction record.
async fn process_document(db: &DbPool, doc: &Document) -> Result<String, anyhow::Error> {
    let file_path = Path::new(&doc.file_path);

    if !file_path.exists() {
        DocumentDao::update_status(db, &doc.id, "failed", Some("File not found on disk"))?;
        return Err(anyhow::anyhow!("File not found: {}", doc.file_path));
    }

    DocumentDao::update_status(db, &doc.id, "processing", None)?;

    let file_type = FileType::detect(&doc.original_name, &doc.content_type);
    info!("Processing {} as {}", doc.original_name, file_type.label());

    let start = Instant::now();

    // Extract text based on file type (blocking I/O in spawn_blocking)
    let path = file_path.to_path_buf();
    let ft = file_type.clone();

    let extraction_result = tokio::task::spawn_blocking(move || -> Result<String, anyhow::Error> {
        match ft {
            FileType::Pdf => {
                let result = pdf::extract_text(&path)?;
                if result.is_scanned && result.text.is_empty() {
                    // Scanned PDF with no text — try OCR on rendered pages
                    // For now, note it needs LLM vision processing
                    warn!("Scanned PDF detected, no text extracted. Needs LLM vision.");
                    Ok("[Scanned PDF — no text extracted. Requires vision LLM processing.]"
                        .to_string())
                } else {
                    Ok(result.text)
                }
            }
            FileType::Excel => {
                let result = excel::extract_text(&path)?;
                Ok(result.text)
            }
            FileType::Word => {
                let result = word::extract_text(&path)?;
                Ok(result.text)
            }
            FileType::Image => {
                let result = ocr::extract_text(&path)?;
                if result.needs_llm_vision {
                    Ok(format!(
                        "[Image {}x{} — requires vision LLM processing.]",
                        result.width, result.height
                    ))
                } else {
                    Ok(result.text)
                }
            }
            FileType::Unknown(ext) => {
                Err(anyhow::anyhow!("Unsupported file type: .{ext}"))
            }
        }
    })
    .await??;

    let elapsed_ms = start.elapsed().as_millis() as i64;

    // Determine document type hint from extracted text (simple heuristics)
    let doc_type = classify_document_type(&extraction_result);

    // Store extraction result
    ExtractionDao::create(
        db,
        &doc.id,
        &doc.batch_id,
        doc_type,
        Some(&extraction_result),
        None, // structured_data populated by LLM in Phase 6
        0.0,  // confidence set by LLM in Phase 6
        None, // model_used set by LLM in Phase 6
        elapsed_ms,
    )?;

    DocumentDao::update_status(db, &doc.id, "completed", None)?;

    Ok(extraction_result)
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
        || lower.contains("balance")
            && (lower.contains("debit") || lower.contains("credit"))
    {
        "bank_statement"
    } else if lower.contains("payment")
        || lower.contains("receipt")
        || lower.contains("paid")
        || lower.contains("amount due")
    {
        "payment"
    } else if lower.contains("receipt")
        || lower.contains("cash register")
        || lower.contains("total")
            && lower.contains("tax")
    {
        "receipt"
    } else {
        "other"
    }
}
