use harvex_db::models::Extraction;
use harvex_db::DbPool;
use rust_xlsxwriter::{Format, Workbook};
use serde::Serialize;

use crate::dao::{BatchDao, DocumentDao, ExtractionDao};

/// Export filter options.
pub struct ExportFilter {
    pub document_type: Option<String>,
    pub min_confidence: Option<f64>,
}

/// Enriched export record combining batch, document, and extraction data.
#[derive(Serialize)]
struct ExportRecord {
    // Extraction fields
    extraction_id: String,
    document_type: String,
    confidence: f64,
    model_used: Option<String>,
    processing_time_ms: i64,
    // Document fields
    document_id: String,
    original_name: String,
    content_type: String,
    file_size: i64,
    // Structured data (flattened at top level)
    #[serde(flatten)]
    structured_data: Option<serde_json::Value>,
    // Raw text (optional, can be large)
    raw_text: Option<String>,
}

/// Full batch export envelope.
#[derive(Serialize)]
struct BatchExport {
    batch_id: String,
    batch_name: String,
    status: String,
    total_files: i32,
    processed_files: i32,
    failed_files: i32,
    model_name: Option<String>,
    created_at: String,
    extractions: Vec<ExportRecord>,
}

pub struct ExportService;

impl ExportService {
    /// Export batch extractions as enriched JSON with metadata.
    pub fn to_json(
        pool: &DbPool,
        batch_id: &str,
        filter: &ExportFilter,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let batch = BatchDao::get_by_id(pool, batch_id)
            .map_err(|_| anyhow::anyhow!("Batch not found"))?;
        let records = build_export_records(pool, batch_id, filter)?;

        let export = BatchExport {
            batch_id: batch.id,
            batch_name: batch.name,
            status: batch.status,
            total_files: batch.total_files,
            processed_files: batch.processed_files,
            failed_files: batch.failed_files,
            model_name: batch.model_name,
            created_at: batch.created_at,
            extractions: records,
        };

        let json = serde_json::to_vec_pretty(&export)?;
        Ok(json)
    }

    /// Export batch extractions as CSV.
    pub fn to_csv(
        pool: &DbPool,
        batch_id: &str,
        filter: &ExportFilter,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let extractions = get_filtered_extractions(pool, batch_id, filter)?;
        let documents = DocumentDao::list_by_batch(pool, batch_id)?;
        let doc_map: std::collections::HashMap<String, String> = documents
            .into_iter()
            .map(|d| (d.id, d.original_name))
            .collect();

        // Collect all unique keys from structured_data across extractions
        let all_keys = collect_structured_keys(&extractions);

        let mut csv = String::new();

        // Header row
        let mut headers = vec![
            "extraction_id",
            "document_id",
            "filename",
            "document_type",
            "confidence",
            "model_used",
            "processing_time_ms",
        ];
        let key_strings: Vec<String> = all_keys.iter().cloned().collect();
        let key_refs: Vec<&str> = key_strings.iter().map(|s| s.as_str()).collect();
        headers.extend_from_slice(&key_refs);
        csv.push_str(&headers.join(","));
        csv.push('\n');

        // Data rows
        for ext in &extractions {
            let filename = doc_map.get(&ext.document_id).cloned().unwrap_or_default();
            let model = ext.model_used.as_deref().unwrap_or("");

            let mut row = vec![
                csv_escape(&ext.id),
                csv_escape(&ext.document_id),
                csv_escape(&filename),
                csv_escape(&ext.document_type),
                format!("{:.2}", ext.confidence),
                csv_escape(model),
                ext.processing_time_ms.to_string(),
            ];

            // Structured data columns
            for key in &all_keys {
                let value = ext
                    .structured_data
                    .as_ref()
                    .and_then(|sd| sd.get(key))
                    .map(|v| flatten_json_value(v))
                    .unwrap_or_default();
                row.push(csv_escape(&value));
            }

            csv.push_str(&row.join(","));
            csv.push('\n');
        }

        Ok(csv.into_bytes())
    }

    /// Export batch extractions as Excel with smart column layout.
    pub fn to_excel(
        pool: &DbPool,
        batch_id: &str,
        filter: &ExportFilter,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let batch = BatchDao::get_by_id(pool, batch_id)
            .map_err(|_| anyhow::anyhow!("Batch not found"))?;
        let extractions = get_filtered_extractions(pool, batch_id, filter)?;
        let documents = DocumentDao::list_by_batch(pool, batch_id)?;
        let doc_map: std::collections::HashMap<String, String> = documents
            .into_iter()
            .map(|d| (d.id, d.original_name))
            .collect();

        let mut workbook = Workbook::new();

        // Header format
        let header_fmt = Format::new().set_bold();

        // --- Summary sheet ---
        let summary = workbook.add_worksheet();
        summary.set_name("Summary")?;
        let summary_fields = [
            ("Batch ID", &batch.id),
            ("Batch Name", &batch.name),
            ("Status", &batch.status),
            ("Created", &batch.created_at),
        ];
        for (row, (label, value)) in summary_fields.iter().enumerate() {
            summary.write_string_with_format(row as u32, 0, *label, &header_fmt)?;
            summary.write_string(row as u32, 1, *value)?;
        }
        let r = summary_fields.len() as u32;
        summary.write_string_with_format(r, 0, "Total Files", &header_fmt)?;
        summary.write_number(r, 1, batch.total_files as f64)?;
        summary.write_string_with_format(r + 1, 0, "Processed", &header_fmt)?;
        summary.write_number(r + 1, 1, batch.processed_files as f64)?;
        summary.write_string_with_format(r + 2, 0, "Failed", &header_fmt)?;
        summary.write_number(r + 2, 1, batch.failed_files as f64)?;
        if let Some(ref model) = batch.model_name {
            summary.write_string_with_format(r + 3, 0, "Model", &header_fmt)?;
            summary.write_string(r + 3, 1, model)?;
        }

        // --- All Extractions sheet ---
        let all_keys = collect_structured_keys(&extractions);
        let all_sheet = workbook.add_worksheet();
        all_sheet.set_name("Extractions")?;

        // Write headers
        let base_headers = [
            "Extraction ID",
            "Filename",
            "Document Type",
            "Confidence",
            "Model",
            "Time (ms)",
        ];
        for (col, h) in base_headers.iter().enumerate() {
            all_sheet.write_string_with_format(0, col as u16, *h, &header_fmt)?;
        }
        let base_col = base_headers.len() as u16;
        for (i, key) in all_keys.iter().enumerate() {
            all_sheet.write_string_with_format(0, base_col + i as u16, key, &header_fmt)?;
        }

        // Write data
        for (row_idx, ext) in extractions.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            let filename = doc_map.get(&ext.document_id).cloned().unwrap_or_default();

            all_sheet.write_string(row, 0, &ext.id)?;
            all_sheet.write_string(row, 1, &filename)?;
            all_sheet.write_string(row, 2, &ext.document_type)?;
            all_sheet.write_number(row, 3, ext.confidence)?;
            all_sheet.write_string(row, 4, ext.model_used.as_deref().unwrap_or(""))?;
            all_sheet.write_number(row, 5, ext.processing_time_ms as f64)?;

            // Structured data columns
            for (i, key) in all_keys.iter().enumerate() {
                let col = base_col + i as u16;
                if let Some(sd) = &ext.structured_data {
                    if let Some(val) = sd.get(key) {
                        write_json_cell(all_sheet, row, col, val)?;
                    }
                }
            }
        }

        // --- Per-type sheets for common document types ---
        let doc_types: Vec<String> = {
            let mut types: Vec<String> = extractions
                .iter()
                .map(|e| e.document_type.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            types.sort();
            types
        };

        for doc_type in &doc_types {
            let typed: Vec<&Extraction> = extractions
                .iter()
                .filter(|e| &e.document_type == doc_type)
                .collect();

            if typed.is_empty() {
                continue;
            }

            let type_keys = collect_structured_keys_from(&typed);

            let sheet_name = capitalize(doc_type);
            // Excel sheet names max 31 chars
            let sheet_name = if sheet_name.len() > 31 {
                sheet_name[..31].to_string()
            } else {
                sheet_name
            };

            let sheet = workbook.add_worksheet();
            sheet.set_name(&sheet_name)?;

            // Headers
            let type_base = ["Filename", "Confidence", "Model"];
            for (col, h) in type_base.iter().enumerate() {
                sheet.write_string_with_format(0, col as u16, *h, &header_fmt)?;
            }
            let tbase = type_base.len() as u16;
            for (i, key) in type_keys.iter().enumerate() {
                sheet.write_string_with_format(0, tbase + i as u16, key, &header_fmt)?;
            }

            // Data
            for (row_idx, ext) in typed.iter().enumerate() {
                let row = (row_idx + 1) as u32;
                let filename = doc_map.get(&ext.document_id).cloned().unwrap_or_default();

                sheet.write_string(row, 0, &filename)?;
                sheet.write_number(row, 1, ext.confidence)?;
                sheet.write_string(row, 2, ext.model_used.as_deref().unwrap_or(""))?;

                for (i, key) in type_keys.iter().enumerate() {
                    let col = tbase + i as u16;
                    if let Some(sd) = &ext.structured_data {
                        if let Some(val) = sd.get(key) {
                            write_json_cell(sheet, row, col, val)?;
                        }
                    }
                }
            }
        }

        let buf = workbook.save_to_buffer()?;
        Ok(buf)
    }
}

/// Build enriched export records with document metadata.
fn build_export_records(
    pool: &DbPool,
    batch_id: &str,
    filter: &ExportFilter,
) -> Result<Vec<ExportRecord>, anyhow::Error> {
    let extractions = get_filtered_extractions(pool, batch_id, filter)?;
    let documents = DocumentDao::list_by_batch(pool, batch_id)?;
    let doc_map: std::collections::HashMap<_, _> = documents
        .into_iter()
        .map(|d| (d.id.clone(), d))
        .collect();

    let mut records = Vec::new();
    for ext in extractions {
        let doc = doc_map.get(&ext.document_id);
        records.push(ExportRecord {
            extraction_id: ext.id,
            document_type: ext.document_type,
            confidence: ext.confidence,
            model_used: ext.model_used,
            processing_time_ms: ext.processing_time_ms,
            document_id: ext.document_id,
            original_name: doc.map(|d| d.original_name.clone()).unwrap_or_default(),
            content_type: doc.map(|d| d.content_type.clone()).unwrap_or_default(),
            file_size: doc.map(|d| d.file_size).unwrap_or(0),
            structured_data: ext.structured_data,
            raw_text: ext.raw_text,
        });
    }
    Ok(records)
}

/// Get extractions with optional filtering.
fn get_filtered_extractions(
    pool: &DbPool,
    batch_id: &str,
    filter: &ExportFilter,
) -> Result<Vec<Extraction>, anyhow::Error> {
    let extractions = ExtractionDao::list_by_batch_filtered(
        pool,
        batch_id,
        filter.document_type.as_deref(),
        filter.min_confidence,
    )?;
    Ok(extractions)
}

/// Collect all unique top-level keys from structured_data across extractions,
/// excluding internal fields like confidence and document_type.
fn collect_structured_keys(extractions: &[Extraction]) -> Vec<String> {
    collect_structured_keys_from(&extractions.iter().collect::<Vec<_>>())
}

fn collect_structured_keys_from(extractions: &[&Extraction]) -> Vec<String> {
    let skip = ["confidence", "document_type"];
    let mut keys = std::collections::BTreeSet::new();
    for ext in extractions {
        if let Some(sd) = &ext.structured_data {
            if let Some(obj) = sd.as_object() {
                for key in obj.keys() {
                    if !skip.contains(&key.as_str()) {
                        keys.insert(key.clone());
                    }
                }
            }
        }
    }
    keys.into_iter().collect()
}

/// Write a JSON value to an Excel cell, handling different types.
fn write_json_cell(
    sheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    value: &serde_json::Value,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    match value {
        serde_json::Value::String(s) => {
            sheet.write_string(row, col, s)?;
        }
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                sheet.write_number(row, col, f)?;
            } else {
                sheet.write_string(row, col, &n.to_string())?;
            }
        }
        serde_json::Value::Bool(b) => {
            sheet.write_boolean(row, col, *b)?;
        }
        serde_json::Value::Null => {}
        // Arrays and objects: serialize to compact JSON string
        _ => {
            let s = value.to_string();
            // Truncate very long values for Excel cell limits
            if s.len() > 32767 {
                sheet.write_string(row, col, &format!("{}...", &s[..32760]))?;
            } else {
                sheet.write_string(row, col, &s)?;
            }
        }
    }
    Ok(())
}

/// Flatten a JSON value to a string for CSV.
fn flatten_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        _ => value.to_string(),
    }
}

/// Escape a string for CSV output.
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Capitalize a string (e.g. "invoice" → "Invoice", "bank_statement" → "Bank Statement").
fn capitalize(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_escape() {
        assert_eq!(csv_escape("hello"), "hello");
        assert_eq!(csv_escape("hello,world"), "\"hello,world\"");
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("invoice"), "Invoice");
        assert_eq!(capitalize("bank_statement"), "Bank Statement");
        assert_eq!(capitalize("other"), "Other");
    }

    #[test]
    fn test_flatten_json() {
        assert_eq!(
            flatten_json_value(&serde_json::json!("hello")),
            "hello"
        );
        assert_eq!(flatten_json_value(&serde_json::json!(42)), "42");
        assert_eq!(flatten_json_value(&serde_json::json!(true)), "true");
        assert_eq!(flatten_json_value(&serde_json::json!(null)), "");
    }
}
