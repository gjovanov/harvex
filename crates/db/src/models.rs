use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub id: String,
    pub name: String,
    pub status: String,
    pub total_files: i32,
    pub processed_files: i32,
    pub failed_files: i32,
    pub model_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub batch_id: String,
    pub filename: String,
    pub original_name: String,
    pub content_type: String,
    pub file_size: i64,
    pub file_path: String,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extraction {
    pub id: String,
    pub document_id: String,
    pub batch_id: String,
    pub document_type: String,
    pub raw_text: Option<String>,
    pub structured_data: Option<serde_json::Value>,
    pub confidence: f64,
    pub model_used: Option<String>,
    pub processing_time_ms: i64,
    pub created_at: String,
}
