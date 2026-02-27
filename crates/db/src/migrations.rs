use crate::DbPool;
use tracing::info;

pub fn run(pool: &DbPool) -> Result<(), duckdb::Error> {
    let conn = pool.conn();

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS batches (
            id              VARCHAR PRIMARY KEY,
            name            VARCHAR NOT NULL,
            status          VARCHAR NOT NULL DEFAULT 'pending',
            total_files     INTEGER NOT NULL DEFAULT 0,
            processed_files INTEGER NOT NULL DEFAULT 0,
            failed_files    INTEGER NOT NULL DEFAULT 0,
            model_name      VARCHAR,
            created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            completed_at    TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS documents (
            id              VARCHAR PRIMARY KEY,
            batch_id        VARCHAR NOT NULL REFERENCES batches(id),
            filename        VARCHAR NOT NULL,
            original_name   VARCHAR NOT NULL,
            content_type    VARCHAR NOT NULL,
            file_size       BIGINT NOT NULL,
            file_path       VARCHAR NOT NULL,
            status          VARCHAR NOT NULL DEFAULT 'pending',
            error_message   VARCHAR,
            created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS extractions (
            id                  VARCHAR PRIMARY KEY,
            document_id         VARCHAR NOT NULL REFERENCES documents(id),
            batch_id            VARCHAR NOT NULL REFERENCES batches(id),
            document_type       VARCHAR NOT NULL DEFAULT 'other',
            raw_text            TEXT,
            structured_data     JSON,
            confidence          DOUBLE DEFAULT 0.0,
            model_used          VARCHAR,
            processing_time_ms  BIGINT DEFAULT 0,
            created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        ",
    )?;

    info!("Database migrations completed");
    Ok(())
}
