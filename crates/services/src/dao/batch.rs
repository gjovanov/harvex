use duckdb::params;
use harvex_db::models::Batch;
use harvex_db::DbPool;

pub struct BatchDao;

impl BatchDao {
    pub fn create(pool: &DbPool, name: &str, model_name: Option<&str>) -> Result<Batch, duckdb::Error> {
        let id = nanoid::nanoid!();
        let conn = pool.conn();

        conn.execute(
            "INSERT INTO batches (id, name, model_name) VALUES (?, ?, ?)",
            params![id, name, model_name],
        )?;

        Self::get_by_id(pool, &id)
    }

    pub fn get_by_id(pool: &DbPool, id: &str) -> Result<Batch, duckdb::Error> {
        let conn = pool.conn();
        conn.query_row(
            "SELECT id, name, status, total_files, processed_files, failed_files,
                    model_name, created_at, updated_at, completed_at
             FROM batches WHERE id = ?",
            params![id],
            |row| {
                Ok(Batch {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: row.get(2)?,
                    total_files: row.get(3)?,
                    processed_files: row.get(4)?,
                    failed_files: row.get(5)?,
                    model_name: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    completed_at: row.get(9)?,
                })
            },
        )
    }

    pub fn list(pool: &DbPool) -> Result<Vec<Batch>, duckdb::Error> {
        let conn = pool.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, status, total_files, processed_files, failed_files,
                    model_name, created_at, updated_at, completed_at
             FROM batches ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Batch {
                id: row.get(0)?,
                name: row.get(1)?,
                status: row.get(2)?,
                total_files: row.get(3)?,
                processed_files: row.get(4)?,
                failed_files: row.get(5)?,
                model_name: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                completed_at: row.get(9)?,
            })
        })?;

        rows.collect()
    }

    pub fn update_status(pool: &DbPool, id: &str, status: &str) -> Result<(), duckdb::Error> {
        let conn = pool.conn();
        conn.execute(
            "UPDATE batches SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![status, id],
        )?;
        Ok(())
    }

    pub fn update_progress(
        pool: &DbPool,
        id: &str,
        processed: i32,
        failed: i32,
    ) -> Result<(), duckdb::Error> {
        let conn = pool.conn();
        conn.execute(
            "UPDATE batches SET processed_files = ?, failed_files = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![processed, failed, id],
        )?;
        Ok(())
    }

    pub fn delete(pool: &DbPool, id: &str) -> Result<bool, duckdb::Error> {
        let conn = pool.conn();
        let affected = conn.execute("DELETE FROM batches WHERE id = ?", params![id])?;
        Ok(affected > 0)
    }

    pub fn set_total_files(pool: &DbPool, id: &str, total: i32) -> Result<(), duckdb::Error> {
        let conn = pool.conn();
        conn.execute(
            "UPDATE batches SET total_files = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![total, id],
        )?;
        Ok(())
    }
}
