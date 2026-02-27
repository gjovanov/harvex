use calamine::{open_workbook_auto, Data, Reader};
use std::path::Path;
use tracing::debug;

/// Extract text from an Excel/CSV/ODS file by reading all sheets.
///
/// Converts tabular data into a readable text format with pipe-delimited columns
/// so an LLM can parse the structure.
pub fn extract_text(file_path: &Path) -> Result<ExtractedExcel, anyhow::Error> {
    debug!("Extracting text from Excel: {}", file_path.display());

    let mut workbook = open_workbook_auto(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to open spreadsheet: {e}"))?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    let mut all_text = String::new();
    let mut total_rows = 0usize;

    for sheet_name in &sheet_names {
        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| anyhow::anyhow!("Failed to read sheet '{sheet_name}': {e}"))?;

        all_text.push_str(&format!("=== Sheet: {} ===\n", sheet_name));

        for row in range.rows() {
            let cells: Vec<String> = row
                .iter()
                .map(|cell| match cell {
                    Data::Empty => String::new(),
                    Data::String(s) => s.clone(),
                    Data::Float(f) => format_number(*f),
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::Error(e) => format!("#ERR:{e:?}"),
                    Data::DateTime(dt) => format!("{dt}"),
                    Data::DateTimeIso(s) => s.clone(),
                    Data::DurationIso(s) => s.clone(),
                })
                .collect();

            // Skip fully empty rows
            if cells.iter().all(|c| c.is_empty()) {
                continue;
            }

            all_text.push_str(&cells.join(" | "));
            all_text.push('\n');
            total_rows += 1;
        }

        all_text.push('\n');
    }

    Ok(ExtractedExcel {
        text: all_text.trim().to_string(),
        sheet_count: sheet_names.len(),
        total_rows,
    })
}

pub struct ExtractedExcel {
    pub text: String,
    pub sheet_count: usize,
    pub total_rows: usize,
}

/// Format a float, removing trailing zeros for cleaner output.
fn format_number(f: f64) -> String {
    if f == f.floor() && f.abs() < 1e15 {
        format!("{}", f as i64)
    } else {
        format!("{f}")
    }
}
