use axum::body::Body;
use axum::Router;
use http_body_util::BodyExt;
use tower::ServiceExt;

use harvex_api::state::AppState;
use harvex_config::*;
use harvex_db::DbPool;

/// Test application fixture with in-memory database and temp upload directory.
pub struct TestApp {
    pub router: Router,
    pub db: DbPool,
    pub upload_dir: tempfile::TempDir,
}

impl TestApp {
    pub fn new() -> Self {
        let db = DbPool::new_in_memory().expect("Failed to create in-memory DB");
        let upload_dir = tempfile::tempdir().expect("Failed to create temp dir");

        let config = Settings {
            server: ServerSettings {
                host: "127.0.0.1".into(),
                port: 0,
            },
            database: DatabaseSettings {
                path: ":memory:".into(),
            },
            storage: StorageSettings {
                upload_dir: upload_dir.path().to_string_lossy().to_string(),
                max_file_size_mb: 10,
            },
            processing: ProcessingSettings { max_concurrent: 1 },
            llm: LlmSettings {
                api_url: "http://localhost:99999/v1".into(), // unreachable on purpose
                api_key: String::new(),
                model_name: "test-model".into(),
                context_size: 2048,
                temperature: 0.1,
                max_tokens: 1024,
            },
        };

        let state = AppState::new(config, db.clone());
        let router = harvex_api::build_router(state);

        Self {
            router,
            db,
            upload_dir,
        }
    }

    /// Send a request and return (status_code, response_bytes).
    pub async fn request(&self, req: axum::http::Request<Body>) -> (u16, Vec<u8>) {
        let response = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("Request failed");

        let status = response.status().as_u16();
        let body = response
            .into_body()
            .collect()
            .await
            .expect("Failed to read body")
            .to_bytes()
            .to_vec();

        (status, body)
    }

    /// Send a request and return (status_code, parsed JSON).
    pub async fn json_request(
        &self,
        req: axum::http::Request<Body>,
    ) -> (u16, serde_json::Value) {
        let (status, body) = self.request(req).await;
        let json: serde_json::Value = serde_json::from_slice(&body)
            .unwrap_or_else(|_| serde_json::json!({ "raw": String::from_utf8_lossy(&body) }));
        (status, json)
    }

    /// Helper: GET request returning JSON.
    pub async fn get(&self, path: &str) -> (u16, serde_json::Value) {
        let req = axum::http::Request::builder()
            .uri(path)
            .body(Body::empty())
            .unwrap();
        self.json_request(req).await
    }

    /// Helper: POST request with JSON body.
    pub async fn post(&self, path: &str, body: &serde_json::Value) -> (u16, serde_json::Value) {
        let req = axum::http::Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap();
        self.json_request(req).await
    }

    /// Helper: DELETE request returning JSON.
    pub async fn delete(&self, path: &str) -> (u16, serde_json::Value) {
        let req = axum::http::Request::builder()
            .method("DELETE")
            .uri(path)
            .body(Body::empty())
            .unwrap();
        self.json_request(req).await
    }

    /// Helper: Create a batch and return its ID.
    pub async fn create_batch(&self, name: &str) -> String {
        let (status, json) = self
            .post(
                "/api/batch",
                &serde_json::json!({ "name": name }),
            )
            .await;
        assert_eq!(status, 200, "Create batch failed: {json}");
        json["id"].as_str().unwrap().to_string()
    }

    /// Helper: Upload a test file via multipart and return (batch_id, doc_id).
    pub async fn upload_test_file(
        &self,
        filename: &str,
        content: &[u8],
        batch_name: &str,
    ) -> (String, String) {
        let boundary = "----TestBoundary12345";
        let mut body = Vec::new();

        // batch_name field
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"batch_name\"\r\n\r\n",
        );
        body.extend_from_slice(batch_name.as_bytes());
        body.extend_from_slice(b"\r\n");

        // file field
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"files[]\"; filename=\"{filename}\"\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body.extend_from_slice(content);
        body.extend_from_slice(b"\r\n");

        // closing boundary
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/document/upload")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        let (status, json) = self.json_request(req).await;
        assert_eq!(status, 200, "Upload failed: {json}");

        let batch_id = json["batch"]["id"].as_str().unwrap().to_string();
        let doc_id = json["documents"][0]["id"].as_str().unwrap().to_string();
        (batch_id, doc_id)
    }
}
