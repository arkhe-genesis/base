use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use serde_json::json;

pub struct OllamaClient {
    client: Client,
    model: String,
    base_url: String,
    timeout_secs: u64,
}

impl OllamaClient {
    pub fn new(model: &str) -> Self {
        Self {
            client: Client::builder().timeout(Duration::from_secs(120)).build().unwrap(),
            model: model.to_string(),
            base_url: "http://localhost:11434".to_string(),
            timeout_secs: 120,
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    pub async fn generate(
        &self,
        prompt: &str,
        max_tokens: usize,
        temperature: f32,
    ) -> Result<String> {
        let body = json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "num_predict": max_tokens,
                "temperature": temperature,
                "top_p": 0.9,
                "top_k": 40,
            }
        });

        let res =
            self.client.post(format!("{}/api/generate", self.base_url)).json(&body).send().await?;

        if !res.status().is_success() {
            anyhow::bail!("Ollama error: {}", res.status());
        }

        let json: serde_json::Value = res.json().await?;
        Ok(json["response"].as_str().unwrap_or("").to_string())
    }

    pub async fn generate_chat(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
    ) -> Result<String> {
        let body = json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
            "options": {
                "num_predict": max_tokens,
                "temperature": 0.2,
            }
        });

        let res =
            self.client.post(format!("{}/api/chat", self.base_url)).json(&body).send().await?;

        let json: serde_json::Value = res.json().await?;
        Ok(json["message"]["content"].as_str().unwrap_or("").to_string())
    }

    pub async fn healthcheck(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: &str) -> Self {
        Self { role: "user".to_string(), content: content.to_string() }
    }

    pub fn assistant(content: &str) -> Self {
        Self { role: "assistant".to_string(), content: content.to_string() }
    }

    pub fn system(content: &str) -> Self {
        Self { role: "system".to_string(), content: content.to_string() }
    }
}
