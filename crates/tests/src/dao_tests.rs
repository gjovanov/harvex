#[cfg(test)]
mod batch_dao {
    use harvex_db::DbPool;
    use harvex_services::BatchDao;

    fn pool() -> DbPool {
        DbPool::new_in_memory().unwrap()
    }

    #[test]
    fn create_and_get() {
        let pool = pool();
        let batch = BatchDao::create(&pool, "Test Batch", Some("test-model")).unwrap();

        assert_eq!(batch.name, "Test Batch");
        assert_eq!(batch.status, "pending");
        assert_eq!(batch.total_files, 0);
        assert_eq!(batch.processed_files, 0);
        assert_eq!(batch.failed_files, 0);
        assert_eq!(batch.model_name.as_deref(), Some("test-model"));

        let fetched = BatchDao::get_by_id(&pool, &batch.id).unwrap();
        assert_eq!(fetched.id, batch.id);
        assert_eq!(fetched.name, "Test Batch");
    }

    #[test]
    fn create_without_model() {
        let pool = pool();
        let batch = BatchDao::create(&pool, "No Model", None).unwrap();
        assert!(batch.model_name.is_none());
    }

    #[test]
    fn list_batches() {
        let pool = pool();
        BatchDao::create(&pool, "Batch A", None).unwrap();
        BatchDao::create(&pool, "Batch B", None).unwrap();
        BatchDao::create(&pool, "Batch C", None).unwrap();

        let batches = BatchDao::list(&pool).unwrap();
        assert_eq!(batches.len(), 3);
    }

    #[test]
    fn update_status() {
        let pool = pool();
        let batch = BatchDao::create(&pool, "Status Test", None).unwrap();
        assert_eq!(batch.status, "pending");

        BatchDao::update_status(&pool, &batch.id, "processing").unwrap();
        let updated = BatchDao::get_by_id(&pool, &batch.id).unwrap();
        assert_eq!(updated.status, "processing");
    }

    #[test]
    fn update_progress() {
        let pool = pool();
        let batch = BatchDao::create(&pool, "Progress Test", None).unwrap();

        BatchDao::update_progress(&pool, &batch.id, 3, 1).unwrap();
        let updated = BatchDao::get_by_id(&pool, &batch.id).unwrap();
        assert_eq!(updated.processed_files, 3);
        assert_eq!(updated.failed_files, 1);
    }

    #[test]
    fn set_total_files() {
        let pool = pool();
        let batch = BatchDao::create(&pool, "Total Test", None).unwrap();

        BatchDao::set_total_files(&pool, &batch.id, 10).unwrap();
        let updated = BatchDao::get_by_id(&pool, &batch.id).unwrap();
        assert_eq!(updated.total_files, 10);
    }

    #[test]
    fn delete_batch() {
        let pool = pool();
        let batch = BatchDao::create(&pool, "Delete Me", None).unwrap();

        let deleted = BatchDao::delete(&pool, &batch.id).unwrap();
        assert!(deleted);

        let result = BatchDao::get_by_id(&pool, &batch.id);
        assert!(result.is_err());
    }

    #[test]
    fn delete_nonexistent() {
        let pool = pool();
        let deleted = BatchDao::delete(&pool, "nonexistent").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn get_nonexistent() {
        let pool = pool();
        let result = BatchDao::get_by_id(&pool, "nonexistent");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod document_dao {
    use harvex_db::DbPool;
    use harvex_services::{BatchDao, DocumentDao};

    fn pool_with_batch() -> (DbPool, String) {
        let pool = DbPool::new_in_memory().unwrap();
        let batch = BatchDao::create(&pool, "Test Batch", None).unwrap();
        (pool, batch.id)
    }

    #[test]
    fn create_and_get() {
        let (pool, batch_id) = pool_with_batch();
        let doc = DocumentDao::create(
            &pool,
            &batch_id,
            "abc123_test.pdf",
            "test.pdf",
            "application/pdf",
            12345,
            "/tmp/test.pdf",
        )
        .unwrap();

        assert_eq!(doc.batch_id, batch_id);
        assert_eq!(doc.filename, "abc123_test.pdf");
        assert_eq!(doc.original_name, "test.pdf");
        assert_eq!(doc.content_type, "application/pdf");
        assert_eq!(doc.file_size, 12345);
        assert_eq!(doc.status, "pending");

        let fetched = DocumentDao::get_by_id(&pool, &doc.id).unwrap();
        assert_eq!(fetched.id, doc.id);
    }

    #[test]
    fn list_by_batch() {
        let (pool, batch_id) = pool_with_batch();
        DocumentDao::create(&pool, &batch_id, "a.pdf", "a.pdf", "application/pdf", 100, "/a").unwrap();
        DocumentDao::create(&pool, &batch_id, "b.jpg", "b.jpg", "image/jpeg", 200, "/b").unwrap();

        let docs = DocumentDao::list_by_batch(&pool, &batch_id).unwrap();
        assert_eq!(docs.len(), 2);
    }

    #[test]
    fn list_empty_batch() {
        let (pool, batch_id) = pool_with_batch();
        let docs = DocumentDao::list_by_batch(&pool, &batch_id).unwrap();
        assert!(docs.is_empty());
    }

    #[test]
    fn delete_document() {
        let (pool, batch_id) = pool_with_batch();
        let doc = DocumentDao::create(&pool, &batch_id, "d.pdf", "d.pdf", "application/pdf", 100, "/d").unwrap();

        let deleted = DocumentDao::delete(&pool, &doc.id).unwrap();
        assert!(deleted);

        let result = DocumentDao::get_by_id(&pool, &doc.id);
        assert!(result.is_err());
    }

    #[test]
    fn delete_by_batch() {
        let (pool, batch_id) = pool_with_batch();
        DocumentDao::create(&pool, &batch_id, "x.pdf", "x.pdf", "application/pdf", 100, "/x").unwrap();
        DocumentDao::create(&pool, &batch_id, "y.pdf", "y.pdf", "application/pdf", 200, "/y").unwrap();

        let paths = DocumentDao::delete_by_batch(&pool, &batch_id).unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"/x".to_string()));
        assert!(paths.contains(&"/y".to_string()));

        let docs = DocumentDao::list_by_batch(&pool, &batch_id).unwrap();
        assert!(docs.is_empty());
    }

    #[test]
    fn update_status() {
        let (pool, batch_id) = pool_with_batch();
        let doc = DocumentDao::create(&pool, &batch_id, "s.pdf", "s.pdf", "application/pdf", 100, "/s").unwrap();

        DocumentDao::update_status(&pool, &doc.id, "completed", None).unwrap();
        let updated = DocumentDao::get_by_id(&pool, &doc.id).unwrap();
        assert_eq!(updated.status, "completed");
        assert!(updated.error_message.is_none());
    }

    #[test]
    fn update_status_with_error() {
        let (pool, batch_id) = pool_with_batch();
        let doc = DocumentDao::create(&pool, &batch_id, "e.pdf", "e.pdf", "application/pdf", 100, "/e").unwrap();

        DocumentDao::update_status(&pool, &doc.id, "failed", Some("Parse error")).unwrap();
        let updated = DocumentDao::get_by_id(&pool, &doc.id).unwrap();
        assert_eq!(updated.status, "failed");
        assert_eq!(updated.error_message.as_deref(), Some("Parse error"));
    }
}

#[cfg(test)]
mod extraction_dao {
    use harvex_db::DbPool;
    use harvex_services::{BatchDao, DocumentDao, ExtractionDao};

    fn pool_with_doc() -> (DbPool, String, String) {
        let pool = DbPool::new_in_memory().unwrap();
        let batch = BatchDao::create(&pool, "Test", None).unwrap();
        let doc = DocumentDao::create(
            &pool,
            &batch.id,
            "test.pdf",
            "test.pdf",
            "application/pdf",
            100,
            "/test",
        )
        .unwrap();
        (pool, batch.id, doc.id)
    }

    #[test]
    fn create_and_get() {
        let (pool, batch_id, doc_id) = pool_with_doc();
        let data = serde_json::json!({"vendor": "Acme", "total": 100.0});

        let ext = ExtractionDao::create(
            &pool,
            &doc_id,
            &batch_id,
            "invoice",
            Some("Invoice from Acme"),
            Some(&data),
            0.85,
            Some("test-model"),
            1500,
        )
        .unwrap();

        assert_eq!(ext.document_id, doc_id);
        assert_eq!(ext.batch_id, batch_id);
        assert_eq!(ext.document_type, "invoice");
        assert_eq!(ext.raw_text.as_deref(), Some("Invoice from Acme"));
        assert_eq!(ext.structured_data.as_ref().unwrap()["vendor"], "Acme");
        assert!((ext.confidence - 0.85).abs() < 0.01);
        assert_eq!(ext.model_used.as_deref(), Some("test-model"));
        assert_eq!(ext.processing_time_ms, 1500);

        let fetched = ExtractionDao::get_by_id(&pool, &ext.id).unwrap();
        assert_eq!(fetched.id, ext.id);
    }

    #[test]
    fn create_minimal() {
        let (pool, batch_id, doc_id) = pool_with_doc();
        let ext = ExtractionDao::create(
            &pool, &doc_id, &batch_id, "other", None, None, 0.0, None, 0,
        )
        .unwrap();

        assert_eq!(ext.document_type, "other");
        assert!(ext.raw_text.is_none());
        assert!(ext.structured_data.is_none());
    }

    #[test]
    fn list_by_batch() {
        let (pool, batch_id, doc_id) = pool_with_doc();
        ExtractionDao::create(&pool, &doc_id, &batch_id, "invoice", None, None, 0.9, None, 100).unwrap();
        ExtractionDao::create(&pool, &doc_id, &batch_id, "receipt", None, None, 0.7, None, 200).unwrap();

        let exts = ExtractionDao::list_by_batch(&pool, &batch_id).unwrap();
        assert_eq!(exts.len(), 2);
    }

    #[test]
    fn list_filtered_by_type() {
        let (pool, batch_id, doc_id) = pool_with_doc();
        ExtractionDao::create(&pool, &doc_id, &batch_id, "invoice", None, None, 0.9, None, 100).unwrap();
        ExtractionDao::create(&pool, &doc_id, &batch_id, "receipt", None, None, 0.7, None, 200).unwrap();

        let invoices = ExtractionDao::list_by_batch_filtered(&pool, &batch_id, Some("invoice"), None).unwrap();
        assert_eq!(invoices.len(), 1);
        assert_eq!(invoices[0].document_type, "invoice");
    }

    #[test]
    fn list_filtered_by_confidence() {
        let (pool, batch_id, doc_id) = pool_with_doc();
        ExtractionDao::create(&pool, &doc_id, &batch_id, "invoice", None, None, 0.9, None, 100).unwrap();
        ExtractionDao::create(&pool, &doc_id, &batch_id, "receipt", None, None, 0.3, None, 200).unwrap();

        let high_conf = ExtractionDao::list_by_batch_filtered(&pool, &batch_id, None, Some(0.5)).unwrap();
        assert_eq!(high_conf.len(), 1);
        assert!((high_conf[0].confidence - 0.9).abs() < 0.01);
    }

    #[test]
    fn list_filtered_combined() {
        let (pool, batch_id, doc_id) = pool_with_doc();
        ExtractionDao::create(&pool, &doc_id, &batch_id, "invoice", None, None, 0.9, None, 100).unwrap();
        ExtractionDao::create(&pool, &doc_id, &batch_id, "invoice", None, None, 0.3, None, 200).unwrap();
        ExtractionDao::create(&pool, &doc_id, &batch_id, "receipt", None, None, 0.8, None, 150).unwrap();

        let result = ExtractionDao::list_by_batch_filtered(&pool, &batch_id, Some("invoice"), Some(0.5)).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].document_type, "invoice");
        assert!(result[0].confidence >= 0.5);
    }

    #[test]
    fn update_structured() {
        let (pool, batch_id, doc_id) = pool_with_doc();
        let ext = ExtractionDao::create(&pool, &doc_id, &batch_id, "other", None, None, 0.0, None, 0).unwrap();

        let data = serde_json::json!({"total": 250.0});
        ExtractionDao::update_structured(
            &pool, &ext.id, "invoice", Some(&data), 0.92, Some("qwen2.5:7b"), 3000,
        )
        .unwrap();

        let updated = ExtractionDao::get_by_id(&pool, &ext.id).unwrap();
        assert_eq!(updated.document_type, "invoice");
        assert_eq!(updated.structured_data.as_ref().unwrap()["total"], 250.0);
        assert!((updated.confidence - 0.92).abs() < 0.01);
        assert_eq!(updated.model_used.as_deref(), Some("qwen2.5:7b"));
        assert_eq!(updated.processing_time_ms, 3000);
    }

    #[test]
    fn delete_by_batch() {
        let (pool, batch_id, doc_id) = pool_with_doc();
        ExtractionDao::create(&pool, &doc_id, &batch_id, "invoice", None, None, 0.9, None, 100).unwrap();
        ExtractionDao::create(&pool, &doc_id, &batch_id, "receipt", None, None, 0.7, None, 200).unwrap();

        let deleted = ExtractionDao::delete_by_batch(&pool, &batch_id).unwrap();
        assert_eq!(deleted, 2);

        let exts = ExtractionDao::list_by_batch(&pool, &batch_id).unwrap();
        assert!(exts.is_empty());
    }
}
