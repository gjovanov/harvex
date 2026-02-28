use duckdb::params;
use harvex_db::models::Extraction;
use harvex_db::DbPool;

pub struct ExtractionDao;

impl ExtractionDao {
    pub fn create(
        pool: &DbPool,
        document_id: &str,
        batch_id: &str,
        document_type: &str,
        raw_text: Option<&str>,
        structured_data: Option<&serde_json::Value>,
        confidence: f64,
        model_used: Option<&str>,
        processing_time_ms: i64,
    ) -> Result<Extraction, duckdb::Error> {
        let id = nanoid::nanoid!();
        let conn = pool.conn();

        let structured_json = structured_data.map(|v| v.to_string());

        conn.execute(
            "INSERT INTO extractions (id, document_id, batch_id, document_type, raw_text,
             structured_data, confidence, model_used, processing_time_ms)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id,
                document_id,
                batch_id,
                document_type,
                raw_text,
                structured_json,
                confidence,
                model_used,
                processing_time_ms
            ],
        )?;

        Self::get_by_id(pool, &id)
    }

    pub fn get_by_id(pool: &DbPool, id: &str) -> Result<Extraction, duckdb::Error> {
        let conn = pool.conn();
        conn.query_row(
            "SELECT id, document_id, batch_id, document_type, raw_text,
                    structured_data, confidence, model_used, processing_time_ms, created_at
             FROM extractions WHERE id = ?",
            params![id],
            Self::map_row,
        )
    }

    pub fn list_by_batch(pool: &DbPool, batch_id: &str) -> Result<Vec<Extraction>, duckdb::Error> {
        let conn = pool.conn();
        let mut stmt = conn.prepare(
            "SELECT id, document_id, batch_id, document_type, raw_text,
                    structured_data, confidence, model_used, processing_time_ms, created_at
             FROM extractions WHERE batch_id = ? ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map(params![batch_id], Self::map_row)?;
        rows.collect()
    }

    /// List extractions with optional filtering by document_type and min confidence.
    pub fn list_by_batch_filtered(
        pool: &DbPool,
        batch_id: &str,
        document_type: Option<&str>,
        min_confidence: Option<f64>,
    ) -> Result<Vec<Extraction>, duckdb::Error> {
        let conn = pool.conn();

        let mut sql = String::from(
            "SELECT id, document_id, batch_id, document_type, raw_text,
                    structured_data, confidence, model_used, processing_time_ms, created_at
             FROM extractions WHERE batch_id = ?",
        );

        let mut param_values: Vec<Box<dyn duckdb::ToSql>> = vec![Box::new(batch_id.to_string())];

        if let Some(dt) = document_type {
            sql.push_str(" AND document_type = ?");
            param_values.push(Box::new(dt.to_string()));
        }

        if let Some(mc) = min_confidence {
            sql.push_str(" AND confidence >= ?");
            param_values.push(Box::new(mc));
        }

        sql.push_str(" ORDER BY created_at ASC");

        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&dyn duckdb::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params.as_slice(), Self::map_row)?;
        rows.collect()
    }

    pub fn update_structured(
        pool: &DbPool,
        id: &str,
        document_type: &str,
        structured_data: Option<&serde_json::Value>,
        confidence: f64,
        model_used: Option<&str>,
        processing_time_ms: i64,
    ) -> Result<(), duckdb::Error> {
        let conn = pool.conn();
        let structured_json = structured_data.map(|v| v.to_string());

        conn.execute(
            "UPDATE extractions SET document_type = ?, structured_data = ?, confidence = ?,
             model_used = ?, processing_time_ms = ? WHERE id = ?",
            params![document_type, structured_json, confidence, model_used, processing_time_ms, id],
        )?;
        Ok(())
    }

    /// Delete all extractions for a batch.
    pub fn delete_by_batch(pool: &DbPool, batch_id: &str) -> Result<usize, duckdb::Error> {
        let conn = pool.conn();
        conn.execute(
            "DELETE FROM extractions WHERE batch_id = ?",
            params![batch_id],
        )
    }

    fn map_row(row: &duckdb::Row<'_>) -> Result<Extraction, duckdb::Error> {
        let structured_str: Option<String> = row.get(5)?;
        let structured_data = structured_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());

        Ok(Extraction {
            id: row.get(0)?,
            document_id: row.get(1)?,
            batch_id: row.get(2)?,
            document_type: row.get(3)?,
            raw_text: row.get(4)?,
            structured_data,
            confidence: row.get(6)?,
            model_used: row.get(7)?,
            processing_time_ms: row.get(8)?,
            created_at: row.get(9)?,
        })
    }
}
