// src/integrations/qvac.rs
//! Interface com QVAC TEE Vault para gerenciamento seguro de chaves.

use qvac_sdk::{VaultClient, VaultConfig, SignRequest, SignResponse};
use std::path::PathBuf;

pub struct QvacVault {
    client: VaultClient,
}

impl QvacVault {
    /// Inicializa o cliente QVAC com configuração TEE.
    pub fn new(tee_url: &str, attestation_cert: &PathBuf) -> Result<Self, String> {
        let config = VaultConfig::builder()
            .tee_url(tee_url)
            .attestation_cert(attestation_cert)
            .build()
            .map_err(|e| format!("Erro QVAC: {}", e))?;
        let client = VaultClient::new(config)
            .map_err(|e| format!("Erro ao conectar QVAC: {}", e))?;
        Ok(Self { client })
    }

    /// Assina uma mensagem usando a chave protegida pela TEE (ex: para pagamento).
    pub async fn sign_message(&self, key_id: &str, message: &[u8]) -> Result<Vec<u8>, String> {
        let request = SignRequest {
            key_id: key_id.to_string(),
            message: message.to_vec(),
        };
        let response: SignResponse = self.client.sign(request).await
            .map_err(|e| format!("Erro na assinatura QVAC: {}", e))?;
        Ok(response.signature)
    }

    /// Verifica a integridade do TEE (attestation).
    pub async fn verify_tee(&self) -> Result<bool, String> {
        self.client.attest().await
            .map_err(|e| format!("Erro na atestação: {}", e))
    }
}