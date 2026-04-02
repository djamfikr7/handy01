use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8765";

#[derive(Serialize)]
pub struct TranscribeRequest {
    pub audio_base64: String,
    #[serde(default)]
    pub context: String,
}

#[derive(Deserialize, Debug)]
pub struct TranscribeResponse {
    pub text: String,
    pub confidence: f64,
    pub timestamp_start: f64,
    pub timestamp_end: f64,
}

#[derive(Serialize)]
pub struct CorrectRequest {
    pub raw_text: String,
    pub confidence: f64,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub use_cloud: bool,
}

#[derive(Deserialize, Debug)]
pub struct CorrectResponse {
    pub corrected_text: String,
    pub source: String,
    pub latency_ms: f64,
}

#[derive(Deserialize, Debug)]
pub struct HealthResponse {
    pub status: String,
    pub whisper_loaded: bool,
    pub correction_loaded: bool,
}

pub struct SidecarClient {
    client: Client,
    base_url: String,
}

impl SidecarClient {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
        }
    }

    pub async fn health_check(&self) -> Result<HealthResponse, String> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Health check failed: {}", e))?;

        response
            .json::<HealthResponse>()
            .await
            .map_err(|e| format!("Failed to parse health response: {}", e))
    }

    pub async fn transcribe(&self, request: &TranscribeRequest) -> Result<TranscribeResponse, String> {
        let url = format!("{}/transcribe", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| format!("Transcription request failed: {}", e))?;

        response
            .json::<TranscribeResponse>()
            .await
            .map_err(|e| format!("Failed to parse transcription response: {}", e))
    }

    pub async fn correct(&self, request: &CorrectRequest) -> Result<CorrectResponse, String> {
        let url = format!("{}/correct", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| format!("Correction request failed: {}", e))?;

        response
            .json::<CorrectResponse>()
            .await
            .map_err(|e| format!("Failed to parse correction response: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = SidecarClient::new(None);
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn test_client_custom_url() {
        let client = SidecarClient::new(Some("http://localhost:9999".to_string()));
        assert_eq!(client.base_url, "http://localhost:9999");
    }

    #[tokio::test]
    async fn test_health_check_fails_without_server() {
        let client = SidecarClient::new(None);
        let result = client.health_check().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transcribe_fails_without_server() {
        let client = SidecarClient::new(None);
        let request = TranscribeRequest {
            audio_base64: "test".to_string(),
            context: String::new(),
        };
        let result = client.transcribe(&request).await;
        assert!(result.is_err());
    }
}
