// src/integrations/pix.rs
use reqwest::Client;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PixPaymentRequest {
    pub amount: f64,
    pub description: String,
    pub payer_name: Option<String>,
    pub payer_document: Option<String>,
    pub expiration_seconds: Option<u32>,
    pub callback_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PixResponse {
    pub qr_code: String,
    pub copy_paste: String,
    pub transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PixStatus {
    Created,
    Waiting,
    Paid,
    Expired,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixWebhookPayload {
    pub transaction_id: String,
    pub status: PixStatus,
    pub paid_amount: Option<f64>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub payer_document: Option<String>,
    pub payer_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}


pub struct PixGateway {
    pub client: Client,
    pub base_url: String,
    pub api_key: String,
    pub merchant_id: String,
}

impl PixGateway {
    pub fn new(base_url: &str, api_key: &str, merchant_id: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            merchant_id: merchant_id.to_string(),
        }
    }

    pub async fn create_payment(&self, request: &PixPaymentRequest) -> Result<PixResponse, String> {
        let url = format!("{}/v1/payments/pix", self.base_url);
        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("x-merchant-id", &self.merchant_id)
            .json(request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        let status = response.status();
        if status.is_success() {
             response.json::<PixResponse>().await.map_err(|e| format!("Failed to parse response: {}", e))
        } else {
             let error_text = response.text().await.unwrap_or_default();
             Err(format!("Request failed with status {}: {}", status, error_text))
        }
    }
}

pub struct OpenFinanceConsent {
    pub consent_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub scope: Vec<String>,
}

pub struct OpenFinanceClient {
    pub base_url: String,
    pub client_id: String,
    pub client_secret: String,
}

impl OpenFinanceClient {
    pub fn new(base_url: &str, client_id: &str, client_secret: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }

    pub async fn transfer_pix(
        &self,
        consent: &OpenFinanceConsent,
        pix_key: &str,
        amount: f64,
        description: &str
    ) -> Result<(), String> {
        // Implementation for open finance
        Ok(())
    }
}