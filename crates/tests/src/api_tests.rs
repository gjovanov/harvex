#[cfg(test)]
mod health {
    use crate::helpers::TestApp;

    #[tokio::test]
    async fn health_check() {
        let app = TestApp::new();
        let (status, json) = app.get("/health").await;

        assert_eq!(status, 200);
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "harvex");
        assert!(json["version"].is_string());
    }
}

#[cfg(test)]
mod batch_api {
    use crate::helpers::TestApp;

    #[tokio::test]
    async fn create_batch() {
        let app = TestApp::new();
        let (status, json) = app
            .post("/api/batch", &serde_json::json!({ "name": "Test Batch" }))
            .await;

        assert_eq!(status, 200);
        assert_eq!(json["name"], "Test Batch");
        assert_eq!(json["status"], "pending");
        assert_eq!(json["total_files"], 0);
        assert!(json["id"].is_string());
    }

    #[tokio::test]
    async fn create_batch_with_model() {
        let app = TestApp::new();
        let (status, json) = app
            .post(
                "/api/batch",
                &serde_json::json!({ "name": "Model Batch", "model_name": "qwen2.5:3b" }),
            )
            .await;

        assert_eq!(status, 200);
        assert_eq!(json["model_name"], "qwen2.5:3b");
    }

    #[tokio::test]
    async fn list_batches_empty() {
        let app = TestApp::new();
        let (status, json) = app.get("/api/batch").await;

        assert_eq!(status, 200);
        assert!(json.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_batches() {
        let app = TestApp::new();
        app.create_batch("Batch 1").await;
        app.create_batch("Batch 2").await;

        let (status, json) = app.get("/api/batch").await;
        assert_eq!(status, 200);
        assert_eq!(json.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn get_batch() {
        let app = TestApp::new();
        let id = app.create_batch("Get Me").await;

        let (status, json) = app.get(&format!("/api/batch/{id}")).await;
        assert_eq!(status, 200);
        assert_eq!(json["id"], id);
        assert_eq!(json["name"], "Get Me");
    }

    #[tokio::test]
    async fn get_nonexistent_batch() {
        let app = TestApp::new();
        let (status, _) = app.get("/api/batch/nonexistent").await;
        assert_eq!(status, 404);
    }

    #[tokio::test]
    async fn delete_batch() {
        let app = TestApp::new();
        let id = app.create_batch("Delete Me").await;

        let (status, json) = app.delete(&format!("/api/batch/{id}")).await;
        assert_eq!(status, 200);
        assert_eq!(json["batch_id"], id);

        // Verify it's gone
        let (status, _) = app.get(&format!("/api/batch/{id}")).await;
        assert_eq!(status, 404);
    }

    #[tokio::test]
    async fn delete_nonexistent_batch() {
        let app = TestApp::new();
        let (status, _) = app.delete("/api/batch/nonexistent").await;
        assert_eq!(status, 404);
    }

    #[tokio::test]
    async fn process_nonexistent_batch() {
        let app = TestApp::new();
        let (status, _) = app
            .post("/api/batch/nonexistent/process", &serde_json::json!({}))
            .await;
        assert_eq!(status, 404);
    }
}

#[cfg(test)]
mod document_api {
    use crate::helpers::TestApp;

    #[tokio::test]
    async fn upload_single_file() {
        let app = TestApp::new();
        let (batch_id, doc_id) = app
            .upload_test_file("invoice.pdf", b"fake pdf content", "Upload Test")
            .await;

        assert!(!batch_id.is_empty());
        assert!(!doc_id.is_empty());
    }

    #[tokio::test]
    async fn upload_creates_batch() {
        let app = TestApp::new();
        let (batch_id, _) = app
            .upload_test_file("test.pdf", b"content", "Auto Batch")
            .await;

        let (status, json) = app.get(&format!("/api/batch/{batch_id}")).await;
        assert_eq!(status, 200);
        assert_eq!(json["name"], "Auto Batch");
        assert_eq!(json["total_files"], 1);
    }

    #[tokio::test]
    async fn list_documents() {
        let app = TestApp::new();
        let (batch_id, _) = app
            .upload_test_file("a.pdf", b"aaa", "List Test")
            .await;

        let (status, json) = app
            .get(&format!("/api/document?batch_id={batch_id}"))
            .await;
        assert_eq!(status, 200);

        let docs = json.as_array().unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0]["original_name"], "a.pdf");
    }

    #[tokio::test]
    async fn get_document() {
        let app = TestApp::new();
        let (_, doc_id) = app
            .upload_test_file("get.pdf", b"get content", "Get Doc Test")
            .await;

        let (status, json) = app.get(&format!("/api/document/{doc_id}")).await;
        assert_eq!(status, 200);
        assert_eq!(json["id"], doc_id);
        assert_eq!(json["original_name"], "get.pdf");
    }

    #[tokio::test]
    async fn get_nonexistent_document() {
        let app = TestApp::new();
        let (status, _) = app.get("/api/document/nonexistent").await;
        assert_eq!(status, 404);
    }

    #[tokio::test]
    async fn delete_document() {
        let app = TestApp::new();
        let (batch_id, doc_id) = app
            .upload_test_file("del.pdf", b"del content", "Del Doc Test")
            .await;

        let (status, json) = app.delete(&format!("/api/document/{doc_id}")).await;
        assert_eq!(status, 200);
        assert_eq!(json["deleted"], true);

        // Verify it's gone
        let (status, _) = app.get(&format!("/api/document/{doc_id}")).await;
        assert_eq!(status, 404);

        // Verify the list is empty for the batch
        let (_, list_json) = app
            .get(&format!("/api/document?batch_id={batch_id}"))
            .await;
        assert!(list_json.as_array().unwrap().is_empty());
    }
}

#[cfg(test)]
mod extraction_api {
    use crate::helpers::TestApp;
    use harvex_services::ExtractionDao;

    #[tokio::test]
    async fn list_extractions_empty() {
        let app = TestApp::new();
        let batch_id = app.create_batch("Empty Ext").await;

        let (status, json) = app
            .get(&format!("/api/batch/{batch_id}/extraction"))
            .await;
        assert_eq!(status, 200);
        assert!(json.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_extractions_with_data() {
        let app = TestApp::new();
        let (batch_id, doc_id) = app
            .upload_test_file("ext.pdf", b"content", "Ext Test")
            .await;

        // Insert extraction via DAO
        let data = serde_json::json!({"vendor": "Test Corp"});
        ExtractionDao::create(
            &app.db, &doc_id, &batch_id, "invoice",
            Some("raw text"), Some(&data), 0.9, Some("test-model"), 500,
        )
        .unwrap();

        let (status, json) = app
            .get(&format!("/api/batch/{batch_id}/extraction"))
            .await;
        assert_eq!(status, 200);

        let exts = json.as_array().unwrap();
        assert_eq!(exts.len(), 1);
        assert_eq!(exts[0]["document_type"], "invoice");
        assert_eq!(exts[0]["confidence"], 0.9);
    }

    #[tokio::test]
    async fn get_extraction() {
        let app = TestApp::new();
        let (batch_id, doc_id) = app
            .upload_test_file("get_ext.pdf", b"content", "Get Ext")
            .await;

        let ext = ExtractionDao::create(
            &app.db, &doc_id, &batch_id, "receipt",
            Some("receipt text"), None, 0.75, None, 300,
        )
        .unwrap();

        let (status, json) = app
            .get(&format!("/api/batch/{batch_id}/extraction/{}", ext.id))
            .await;
        assert_eq!(status, 200);
        assert_eq!(json["id"], ext.id);
        assert_eq!(json["document_type"], "receipt");
    }

    #[tokio::test]
    async fn get_nonexistent_extraction() {
        let app = TestApp::new();
        let batch_id = app.create_batch("No Ext").await;

        let (status, _) = app
            .get(&format!("/api/batch/{batch_id}/extraction/nonexistent"))
            .await;
        assert_eq!(status, 404);
    }
}

#[cfg(test)]
mod export_api {
    use crate::helpers::TestApp;
    use harvex_services::ExtractionDao;

    #[tokio::test]
    async fn export_json() {
        let app = TestApp::new();
        let (batch_id, doc_id) = app
            .upload_test_file("exp.pdf", b"content", "Export JSON")
            .await;

        let data = serde_json::json!({"amount": 100.0, "vendor": "Acme"});
        ExtractionDao::create(
            &app.db, &doc_id, &batch_id, "invoice",
            Some("invoice text"), Some(&data), 0.9, Some("test-model"), 500,
        )
        .unwrap();

        let (status, body) = app
            .request(
                axum::http::Request::builder()
                    .uri(&format!("/api/export/json/{batch_id}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await;
        assert_eq!(status, 200);

        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["batch_id"], batch_id);
        assert_eq!(json["batch_name"], "Export JSON");
        assert_eq!(json["extractions"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn export_csv() {
        let app = TestApp::new();
        let (batch_id, doc_id) = app
            .upload_test_file("csv.pdf", b"content", "Export CSV")
            .await;

        ExtractionDao::create(
            &app.db, &doc_id, &batch_id, "invoice",
            Some("text"), None, 0.8, None, 100,
        )
        .unwrap();

        let (status, body) = app
            .request(
                axum::http::Request::builder()
                    .uri(&format!("/api/export/csv/{batch_id}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await;
        assert_eq!(status, 200);

        let csv = String::from_utf8(body).unwrap();
        assert!(csv.contains("extraction_id"));
        assert!(csv.contains("document_type"));
        assert!(csv.contains("confidence"));
    }

    #[tokio::test]
    async fn export_excel() {
        let app = TestApp::new();
        let (batch_id, doc_id) = app
            .upload_test_file("xlsx.pdf", b"content", "Export Excel")
            .await;

        ExtractionDao::create(
            &app.db, &doc_id, &batch_id, "invoice",
            Some("text"), None, 0.8, None, 100,
        )
        .unwrap();

        let (status, body) = app
            .request(
                axum::http::Request::builder()
                    .uri(&format!("/api/export/excel/{batch_id}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await;
        assert_eq!(status, 200);

        // XLSX files start with PK (ZIP magic bytes)
        assert!(body.len() > 4);
        assert_eq!(&body[0..2], b"PK");
    }

    #[tokio::test]
    async fn export_nonexistent_batch() {
        let app = TestApp::new();
        let (status, _) = app
            .request(
                axum::http::Request::builder()
                    .uri("/api/export/json/nonexistent")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await;
        assert_eq!(status, 500);
    }
}

#[cfg(test)]
mod model_api {
    use crate::helpers::TestApp;

    #[tokio::test]
    async fn get_model_info() {
        let app = TestApp::new();
        let (status, json) = app.get("/api/model").await;

        assert_eq!(status, 200);
        assert_eq!(json["model_name"], "test-model");
        assert!(json["api_url"].is_string());
        assert!(json["context_size"].is_number());
        assert!(json["temperature"].is_number());
        assert!(json["max_tokens"].is_number());
    }

    #[tokio::test]
    async fn switch_model() {
        let app = TestApp::new();
        let (status, json) = app
            .post(
                "/api/model/switch",
                &serde_json::json!({ "model_name": "new-model" }),
            )
            .await;

        assert_eq!(status, 200);
        assert_eq!(json["previous_model"], "test-model");
        assert_eq!(json["current_model"], "new-model");

        // Verify it persisted
        let (_, info) = app.get("/api/model").await;
        assert_eq!(info["model_name"], "new-model");
    }

    #[tokio::test]
    async fn update_settings() {
        let app = TestApp::new();
        let (status, json) = app
            .post(
                "/api/model/settings",
                &serde_json::json!({
                    "temperature": 0.5,
                    "max_tokens": 4096,
                    "context_size": 8192,
                }),
            )
            .await;

        assert_eq!(status, 200);
        assert_eq!(json["current"]["temperature"], 0.5);
        assert_eq!(json["current"]["max_tokens"], 4096);
        assert_eq!(json["current"]["context_size"], 8192);
    }

    #[tokio::test]
    async fn health_check_unreachable() {
        let app = TestApp::new();
        let (status, json) = app.get("/api/model/health").await;

        assert_eq!(status, 200);
        assert_eq!(json["reachable"], false);
        assert_eq!(json["model_name"], "test-model");
    }
}

#[cfg(test)]
mod batch_cascade_delete {
    use crate::helpers::TestApp;
    use harvex_services::ExtractionDao;

    #[tokio::test]
    async fn delete_batch_cascades() {
        let app = TestApp::new();

        // Upload creates batch + document
        let (batch_id, doc_id) = app
            .upload_test_file("cascade.pdf", b"cascade content", "Cascade Test")
            .await;

        // Add extraction
        ExtractionDao::create(
            &app.db, &doc_id, &batch_id, "invoice",
            Some("text"), None, 0.8, None, 100,
        )
        .unwrap();

        // Verify data exists
        let (_, docs) = app.get(&format!("/api/document?batch_id={batch_id}")).await;
        assert_eq!(docs.as_array().unwrap().len(), 1);

        let (_, exts) = app.get(&format!("/api/batch/{batch_id}/extraction")).await;
        assert_eq!(exts.as_array().unwrap().len(), 1);

        // Delete batch
        let (status, json) = app.delete(&format!("/api/batch/{batch_id}")).await;
        assert_eq!(status, 200);
        assert_eq!(json["files_removed"], 1);

        // Verify everything is gone
        let (status, _) = app.get(&format!("/api/batch/{batch_id}")).await;
        assert_eq!(status, 404);

        let (_, docs) = app.get(&format!("/api/document?batch_id={batch_id}")).await;
        assert!(docs.as_array().unwrap().is_empty());

        let (_, exts) = app.get(&format!("/api/batch/{batch_id}/extraction")).await;
        assert!(exts.as_array().unwrap().is_empty());
    }
}
