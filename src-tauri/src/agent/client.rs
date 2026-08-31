use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
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
    message: ChatMessage,
}

pub struct LlmClient {
    http: Client,
}

impl LlmClient {
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn chat_completion(
        &self,
        base_url: &str,
        api_key: &str,
        model: &str,
        prompt: &str,
    ) -> Result<String, LlmError> {
        let url = format!(
            "{}/chat/completions",
            base_url.trim_end_matches('/')
        );

        let mut request = self.http.post(&url).json(&ChatRequest {
            model: model.to_string(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
        });

        if !api_key.is_empty() {
            request = request.bearer_auth(api_key);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::Api(format!("{status}: {body}")));
        }

        let chat: ChatResponse = response.json().await?;
        chat.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| LlmError::Api("empty response".into()))
    }

    pub async fn test_connection(
        &self,
        base_url: &str,
        api_key: &str,
        model: &str,
    ) -> Result<String, LlmError> {
        self.chat_completion(base_url, api_key, model, "Say OK")
            .await
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}
