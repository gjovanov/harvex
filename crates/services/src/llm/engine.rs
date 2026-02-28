use std::sync::RwLock;
use std::time::Instant;

use base64::Engine as _;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Serialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: MessageContent,
}

/// Message content — either plain text or multimodal parts (for vision).
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// A single part of a multimodal message.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ImageUrl {
    url: String,
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
            "LLM engine initialized: model={}, vision_model={}, api={}",
            settings.model_name,
            if settings.vision_model_name.is_empty() {
                "(disabled)"
            } else {
                &settings.vision_model_name
            },
            settings.api_url
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

    /// Check if vision model is configured.
    pub fn has_vision(&self) -> bool {
        let s = self.settings.read().unwrap();
        !s.vision_model_name.is_empty()
    }

    /// Switch to a different model.
    pub fn switch_model(&self, model_name: &str) {
        let mut settings = self.settings.write().unwrap();
        info!(
            "Switching LLM model: {} -> {}",
            settings.model_name, model_name
        );
        settings.model_name = model_name.to_string();
    }

    /// Update LLM settings at runtime.
    pub fn update_settings(
        &self,
        api_url: Option<&str>,
        api_key: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        context_size: Option<u32>,
        vision_model_name: Option<&str>,
    ) {
        let mut settings = self.settings.write().unwrap();
        if let Some(url) = api_url {
            info!("Updating LLM api_url: {}", url);
            settings.api_url = url.to_string();
        }
        if let Some(key) = api_key {
            settings.api_key = key.to_string();
        }
        if let Some(temp) = temperature {
            settings.temperature = temp;
        }
        if let Some(mt) = max_tokens {
            settings.max_tokens = mt;
        }
        if let Some(cs) = context_size {
            settings.context_size = cs;
        }
        if let Some(vm) = vision_model_name {
            info!("Updating vision model: {}", vm);
            settings.vision_model_name = vm.to_string();
        }
    }

    /// List available models from the API (works with Ollama and OpenAI-compatible APIs).
    pub async fn list_models(&self) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let settings = self.settings.read().unwrap().clone();
        let url = format!("{}/models", settings.api_url);

        let mut req = self.client.get(&url);
        if !settings.api_key.is_empty() {
            req = req.bearer_auth(&settings.api_key);
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to list models: {status}: {body}"));
        }

        let body: serde_json::Value = response.json().await?;

        // OpenAI format: { "data": [...] }
        // Ollama format: { "models": [...] }
        let models = body
            .get("data")
            .or_else(|| body.get("models"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(models)
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
                    content: MessageContent::Text(system_prompt),
                },
                ChatMessage {
                    role: "user".into(),
                    content: MessageContent::Text(truncated_text),
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
            return Err(anyhow::anyhow!("LLM API returned {status}: {body}"));
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

    /// Extract structured data from page images using the vision LLM.
    ///
    /// Processes each page individually, then merges multi-page results
    /// using the text model.
    pub async fn extract_structured_with_vision(
        &self,
        page_images: &[Vec<u8>],
        document_type_hint: &str,
    ) -> Result<LlmResponse, anyhow::Error> {
        let settings = self.settings.read().unwrap().clone();

        if settings.vision_model_name.is_empty() {
            return Err(anyhow::anyhow!(
                "Vision model not configured (vision_model_name is empty)"
            ));
        }

        let start = Instant::now();
        let total_pages = page_images.len();
        let system_prompt = prompts::system_prompt(document_type_hint);

        info!(
            "Vision inference: model={}, pages={}, doc_type={}",
            settings.vision_model_name, total_pages, document_type_hint
        );

        let mut page_results: Vec<serde_json::Value> = Vec::new();

        for (i, image_bytes) in page_images.iter().enumerate() {
            let page_num = i + 1;
            let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);
            let data_url = format!("data:image/jpeg;base64,{b64}");

            let user_prompt =
                prompts::vision_user_prompt(document_type_hint, page_num, total_pages);

            let request = ChatRequest {
                model: settings.vision_model_name.clone(),
                messages: vec![
                    ChatMessage {
                        role: "system".into(),
                        content: MessageContent::Text(system_prompt.clone()),
                    },
                    ChatMessage {
                        role: "user".into(),
                        content: MessageContent::Parts(vec![
                            ContentPart::Text { text: user_prompt },
                            ContentPart::ImageUrl {
                                image_url: ImageUrl { url: data_url },
                            },
                        ]),
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

            debug!(
                "Vision: sending page {}/{} ({} bytes)",
                page_num,
                total_pages,
                image_bytes.len()
            );

            let response = req.send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                warn!(
                    "Vision LLM failed for page {}: {} {}",
                    page_num, status, body
                );
                continue;
            }

            let chat_response: ChatResponse = response.json().await?;
            let content = chat_response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default();

            let (page_data, _) = parse_llm_response(&content);
            debug!("Vision: page {}/{} extracted", page_num, total_pages);
            page_results.push(page_data);
        }

        if page_results.is_empty() {
            return Err(anyhow::anyhow!(
                "Vision LLM returned no results for any page"
            ));
        }

        // Single page — use directly; multi-page — merge via text model
        let (structured_data, confidence, model_used) = if page_results.len() == 1 {
            let confidence = page_results[0]
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.7);
            (
                page_results.into_iter().next().unwrap(),
                confidence,
                settings.vision_model_name.clone(),
            )
        } else {
            self.merge_page_results(&page_results, document_type_hint, &settings)
                .await?
        };

        let final_doc_type = structured_data
            .get("document_type")
            .and_then(|v| v.as_str())
            .unwrap_or(document_type_hint)
            .to_string();

        let elapsed_ms = start.elapsed().as_millis() as i64;

        info!(
            "Vision inference complete: model={}, doc_type={}, confidence={:.2}, pages={}, time={}ms",
            model_used, final_doc_type, confidence, total_pages, elapsed_ms
        );

        Ok(LlmResponse {
            structured_data,
            document_type: final_doc_type,
            confidence,
            model_used,
            processing_time_ms: elapsed_ms,
        })
    }

    /// Merge per-page extraction results into a single JSON using the text model.
    async fn merge_page_results(
        &self,
        page_results: &[serde_json::Value],
        document_type_hint: &str,
        settings: &LlmSettings,
    ) -> Result<(serde_json::Value, f64, String), anyhow::Error> {
        let system_prompt = prompts::system_prompt(document_type_hint);
        let merge_prompt = prompts::merge_pages_prompt(document_type_hint, page_results);

        info!(
            "Merging {} page results via text model: {}",
            page_results.len(),
            settings.model_name
        );

        let request = ChatRequest {
            model: settings.model_name.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: MessageContent::Text(system_prompt),
                },
                ChatMessage {
                    role: "user".into(),
                    content: MessageContent::Text(merge_prompt),
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
                "LLM merge API returned {status}: {body}"
            ));
        }

        let chat_response: ChatResponse = response.json().await?;
        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let (data, confidence) = parse_llm_response(&content);
        let model_used = format!(
            "{}+{}",
            settings.vision_model_name, settings.model_name
        );

        Ok((data, confidence, model_used))
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

    #[test]
    fn message_content_text_serializes_as_string() {
        let msg = ChatMessage {
            role: "user".into(),
            content: MessageContent::Text("hello".into()),
        };
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["content"], "hello");
    }

    #[test]
    fn message_content_parts_serializes_as_array() {
        let msg = ChatMessage {
            role: "user".into(),
            content: MessageContent::Parts(vec![
                ContentPart::Text {
                    text: "describe this".into(),
                },
                ContentPart::ImageUrl {
                    image_url: ImageUrl {
                        url: "data:image/jpeg;base64,abc123".into(),
                    },
                },
            ]),
        };
        let json = serde_json::to_value(&msg).unwrap();
        let parts = json["content"].as_array().unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["type"], "text");
        assert_eq!(parts[1]["type"], "image_url");
        assert_eq!(parts[1]["image_url"]["url"], "data:image/jpeg;base64,abc123");
    }
}
