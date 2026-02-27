use harvex_db::DbPool;
use rust_xlsxwriter::Workbook;

use crate::dao::ExtractionDao;

pub struct ExportService;

impl ExportService {
    pub fn to_json(pool: &DbPool, batch_id: &str) -> Result<Vec<u8>, anyhow::Error> {
        let extractions = ExtractionDao::list_by_batch(pool, batch_id)?;
        let json = serde_json::to_vec_pretty(&extractions)?;
        Ok(json)
    }

    pub fn to_excel(pool: &DbPool, batch_id: &str) -> Result<Vec<u8>, anyhow::Error> {
        let extractions = ExtractionDao::list_by_batch(pool, batch_id)?;
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

        // Headers
        let headers = [
            "ID",
            "Document ID",
            "Document Type",
            "Confidence",
            "Model Used",
            "Processing Time (ms)",
            "Raw Text",
            "Structured Data",
        ];
        for (col, header) in headers.iter().enumerate() {
            sheet.write_string(0, col as u16, *header)?;
        }

        // Data rows
        for (row_idx, ext) in extractions.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            sheet.write_string(row, 0, &ext.id)?;
            sheet.write_string(row, 1, &ext.document_id)?;
            sheet.write_string(row, 2, &ext.document_type)?;
            sheet.write_number(row, 3, ext.confidence)?;
            sheet.write_string(row, 4, ext.model_used.as_deref().unwrap_or(""))?;
            sheet.write_number(row, 5, ext.processing_time_ms as f64)?;
            sheet.write_string(row, 6, ext.raw_text.as_deref().unwrap_or(""))?;
            let structured = ext
                .structured_data
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default();
            sheet.write_string(row, 7, &structured)?;
        }

        let buf = workbook.save_to_buffer()?;
        Ok(buf)
    }
}
