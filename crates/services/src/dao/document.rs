use duckdb::params;
use harvex_db::models::Document;
use harvex_db::DbPool;

pub struct DocumentDao;

impl DocumentDao {
    pub fn create(
        pool: &DbPool,
        batch_id: &str,
        filename: &str,
        original_name: &str,
        content_type: &str,
        file_size: i64,
        file_path: &str,
    ) -> Result<Document, duckdb::Error> {
        let id = nanoid::nanoid!();
        let conn = pool.conn();

        conn.execute(
            "INSERT INTO documents (id, batch_id, filename, original_name, content_type, file_size, file_path)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![id, batch_id, filename, original_name, content_type, file_size, file_path],
        )?;

        Self::get_by_id(pool, &id)
    }

    pub fn get_by_id(pool: &DbPool, id: &str) -> Result<Document, duckdb::Error> {
        let conn = pool.conn();
        conn.query_row(
            "SELECT id, batch_id, filename, original_name, content_type, file_size,
                    file_path, status, error_message, created_at, updated_at
             FROM documents WHERE id = ?",
            params![id],
            |row| {
                Ok(Document {
                    id: row.get(0)?,
                    batch_id: row.get(1)?,
                    filename: row.get(2)?,
                    original_name: row.get(3)?,
                    content_type: row.get(4)?,
                    file_size: row.get(5)?,
                    file_path: row.get(6)?,
                    status: row.get(7)?,
                    error_message: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            },
        )
    }

    pub fn list_by_batch(pool: &DbPool, batch_id: &str) -> Result<Vec<Document>, duckdb::Error> {
        let conn = pool.conn();
        let mut stmt = conn.prepare(
            "SELECT id, batch_id, filename, original_name, content_type, file_size,
                    file_path, status, error_message, created_at, updated_at
             FROM documents WHERE batch_id = ? ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map(params![batch_id], |row| {
            Ok(Document {
                id: row.get(0)?,
                batch_id: row.get(1)?,
                filename: row.get(2)?,
                original_name: row.get(3)?,
                content_type: row.get(4)?,
                file_size: row.get(5)?,
                file_path: row.get(6)?,
                status: row.get(7)?,
                error_message: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        rows.collect()
    }

    pub fn delete(pool: &DbPool, id: &str) -> Result<bool, duckdb::Error> {
        let conn = pool.conn();
        let affected = conn.execute("DELETE FROM documents WHERE id = ?", params![id])?;
        Ok(affected > 0)
    }

    pub fn update_status(
        pool: &DbPool,
        id: &str,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), duckdb::Error> {
        let conn = pool.conn();
        conn.execute(
            "UPDATE documents SET status = ?, error_message = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![status, error_message, id],
        )?;
        Ok(())
    }
}
