// src/integrations/pix_openapi.rs
//! Structs geradas a partir da especificação OpenAPI do Pix (bacen/pix-api).
//! Fonte: https://github.com/bacen/pix-api/blob/main/openapi.yaml
//! Versão: 2.9.0 (Set/2025)
//!
//! Gerado automaticamente com `openapi-generator` ou manualmente.

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

// ─── Cobrança (Charge) ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixChargeRequest {
    /// Valor da cobrança em BRL (com 2 casas decimais)
    pub amount: f64,
    /// Descrição da cobrança (até 140 caracteres)
    pub description: String,
    /// Nome do pagador (opcional)
    pub payer_name: Option<String>,
    /// CPF/CNPJ do pagador (opcional)
    pub payer_document: Option<String>,
    /// Tempo de expiração em segundos (padrão: 3600)
    pub expiration_seconds: Option<u32>,
    /// URL para callback (webhook)
    pub callback_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixChargeResponse {
    pub transaction_id: String,
    pub qr_code: String,           // base64
    pub copy_paste: String,
    pub status: PixChargeStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PixChargeStatus {
    Created,
    Waiting,
    Paid,
    Expired,
    Cancelled,
    Failed,
}

// ─── Pagamento (Payment) ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixPayment {
    pub transaction_id: String,
    pub charge_id: String,
    pub amount: f64,
    pub status: PixPaymentStatus,
    pub paid_at: Option<DateTime<Utc>>,
    pub payer_document: Option<String>,
    pub payer_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PixPaymentStatus {
    Created,
    Processing,
    Paid,
    Failed,
    Refunded,
}

// ─── Webhook ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixWebhookPayload {
    pub transaction_id: String,
    pub status: PixPaymentStatus,
    pub paid_amount: Option<f64>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payer_document: Option<String>,
    pub payer_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ─── Pix Key (DICT) ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixKey {
    pub key: String,
    pub key_type: PixKeyType,
    pub holder_name: String,
    pub holder_document: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PixKeyType {
    Cpf,
    Cnpj,
    Phone,
    Email,
    Evp,   // Chave aleatória
}

// ─── Open Finance ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFinanceConsent {
    pub consent_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub scope: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFinanceBalance {
    pub balance: f64,
    pub currency: String,
    pub account_type: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFinanceTransferRequest {
    pub pix_key: String,
    pub amount: f64,
    pub description: String,
    pub consent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFinanceTransferResponse {
    pub transaction_id: String,
    pub status: TransferStatus,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TransferStatus {
    Pending,
    Success,
    Failed,
}