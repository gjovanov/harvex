use std::sync::RwLock;
use std::time::Instant;

use harvex_config::LlmSettings;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::prompts;

/// Response from LLM inference.
pub struct LlmResponse {
    pub structured_data: serde_json::Value,
    pub document_type: String,
    pub confidence: f64,
    pub model_used: String,
    pub processing_time_ms: i64,
}

/// LLM engine that calls an OpenAI-compatible API endpoint.
///
/// Works with Ollama, llama.cpp server, vLLM, or any OpenAI-compatible API.
pub struct LlmEngine {
    client: reqwest::Client,
    settings: RwLock<LlmSettings>,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    response_format: Option<ResponseFormat>,
}

#[derive(Serialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

impl LlmEngine {
    pub fn new(settings: LlmSettings) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to build HTTP client");

        info!(
            "LLM engine initialized: model={}, api={}",
            settings.model_name, settings.api_url
        );

        Self {
            client,
            settings: RwLock::new(settings),
        }
    }

    /// Get current model name.
    pub fn model_name(&self) -> String {
        self.settings.read().unwrap().model_name.clone()
    }

    /// Get current settings snapshot.
    pub fn settings(&self) -> LlmSettings {
        self.settings.read().unwrap().clone()
    }

    /// Switch to a different model.
    pub fn switch_model(&self, model_name: &str) {
        let mut settings = self.settings.write().unwrap();
        info!("Switching LLM model: {} -> {}", settings.model_name, model_name);
        settings.model_name = model_name.to_string();
    }

    /// Check if the LLM API is reachable.
    pub async fn health_check(&self) -> Result<bool, anyhow::Error> {
        let settings = self.settings.read().unwrap().clone();
        let url = format!("{}/models", settings.api_url);

        let mut req = self.client.get(&url);
        if !settings.api_key.is_empty() {
            req = req.bearer_auth(&settings.api_key);
        }

        match req.send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(e) => {
                warn!("LLM API health check failed: {e}");
                Ok(false)
            }
        }
    }

    /// Extract structured data from raw text using the LLM.
    pub async fn extract_structured(
        &self,
        raw_text: &str,
        document_type_hint: &str,
    ) -> Result<LlmResponse, anyhow::Error> {
        let settings = self.settings.read().unwrap().clone();
        let start = Instant::now();

        // Build the prompt
        let system_prompt = prompts::system_prompt(document_type_hint);
        let user_prompt = prompts::user_prompt(raw_text, document_type_hint);

        debug!(
            "LLM inference: model={}, doc_type={}, text_len={}",
            settings.model_name,
            document_type_hint,
            raw_text.len()
        );

        // Truncate text if it exceeds context window (rough char estimate)
        let max_chars = (settings.context_size as usize) * 3;
        let truncated_text = if user_prompt.len() > max_chars {
            format!("{}...[truncated]", &user_prompt[..max_chars])
        } else {
            user_prompt
        };

        let request = ChatRequest {
            model: settings.model_name.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".into(),
                    content: truncated_text,
                },
            ],
            temperature: settings.temperature,
            max_tokens: settings.max_tokens,
            response_format: Some(ResponseFormat {
                r#type: "json_object".into(),
            }),
        };

        let url = format!("{}/chat/completions", settings.api_url);

        let mut req = self.client.post(&url).json(&request);
        if !settings.api_key.is_empty() {
            req = req.bearer_auth(&settings.api_key);
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "LLM API returned {status}: {body}"
            ));
        }

        let chat_response: ChatResponse = response.json().await?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let elapsed_ms = start.elapsed().as_millis() as i64;

        // Parse the LLM response as JSON
        let (structured_data, confidence) = parse_llm_response(&content);

        // Let the LLM's classification override heuristics if it provided one
        let final_doc_type = structured_data
            .get("document_type")
            .and_then(|v| v.as_str())
            .unwrap_or(document_type_hint)
            .to_string();

        info!(
            "LLM inference complete: model={}, doc_type={}, confidence={:.2}, time={}ms",
            settings.model_name, final_doc_type, confidence, elapsed_ms
        );

        Ok(LlmResponse {
            structured_data,
            document_type: final_doc_type,
            confidence,
            model_used: settings.model_name,
            processing_time_ms: elapsed_ms,
        })
    }
}

/// Parse the LLM response, extracting JSON and a confidence score.
fn parse_llm_response(content: &str) -> (serde_json::Value, f64) {
    // Try direct JSON parse first
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
        let confidence = value
            .get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.8);
        return (value, confidence);
    }

    // Try to find JSON block in markdown code fence
    if let Some(json_str) = extract_json_block(content) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
            let confidence = value
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.7);
            return (value, confidence);
        }
    }

    // Try to find any JSON object in the text
    if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            let candidate = &content[start..=end];
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(candidate) {
                let confidence = value
                    .get("confidence")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.6);
                return (value, confidence);
            }
        }
    }

    // Fallback: wrap raw text in a JSON object
    warn!("Could not parse LLM response as JSON, wrapping as raw_response");
    (
        serde_json::json!({
            "raw_response": content,
            "parse_error": "LLM response was not valid JSON"
        }),
        0.3,
    )
}

/// Extract JSON from a markdown code fence like ```json ... ```.
fn extract_json_block(text: &str) -> Option<&str> {
    let start_markers = ["```json\n", "```json\r\n", "```\n", "```\r\n"];

    for marker in &start_markers {
        if let Some(start_idx) = text.find(marker) {
            let json_start = start_idx + marker.len();
            if let Some(end_idx) = text[json_start..].find("```") {
                return Some(&text[json_start..json_start + end_idx]);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_direct_json() {
        let input = r#"{"vendor_name": "Acme", "total": 100.0, "confidence": 0.95}"#;
        let (value, confidence) = parse_llm_response(input);
        assert_eq!(value["vendor_name"], "Acme");
        assert!((confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn parse_json_in_code_fence() {
        let input = "Here is the result:\n```json\n{\"vendor\": \"Test\", \"amount\": 50}\n```\n";
        let (value, confidence) = parse_llm_response(input);
        assert_eq!(value["vendor"], "Test");
        assert!(confidence > 0.0);
    }

    #[test]
    fn parse_json_embedded_in_text() {
        let input = "The extracted data is: {\"name\": \"Invoice\"} and that's it.";
        let (value, confidence) = parse_llm_response(input);
        assert_eq!(value["name"], "Invoice");
        assert!(confidence > 0.0);
    }

    #[test]
    fn parse_non_json_fallback() {
        let input = "This is just plain text with no JSON.";
        let (value, confidence) = parse_llm_response(input);
        assert!(value.get("raw_response").is_some());
        assert!((confidence - 0.3).abs() < 0.01);
    }
}
