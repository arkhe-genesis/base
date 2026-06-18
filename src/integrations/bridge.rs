// src/integrations/bridge.rs
//! Integração com bridges cross-chain via WDK.

use wdk_sdk::{BridgeClient, BridgeConfig, TransferRequest, TransferStatus};

pub struct CrossChainBridge {
    client: BridgeClient,
}

impl CrossChainBridge {
    pub fn new(config: BridgeConfig) -> Self {
        Self {
            client: BridgeClient::new(config),
        }
    }

    /// Transfere USDC da chain A para a chain B.
    pub async fn transfer_usdc(
        &self,
        from_chain: &str,
        to_chain: &str,
        amount: u64,
        recipient: &str,
    ) -> Result<String, String> {
        let request = TransferRequest {
            from_chain: from_chain.to_string(),
            to_chain: to_chain.to_string(),
            asset: "USDC".to_string(),
            amount,
            recipient: recipient.to_string(),
        };
        let response = self.client.transfer(request).await
            .map_err(|e| format!("Erro na bridge: {}", e))?;
        Ok(response.transaction_id)
    }

    /// Consulta o status de uma transferência.
    pub async fn get_transfer_status(&self, tx_id: &str) -> Result<TransferStatus, String> {
        self.client.get_status(tx_id).await
            .map_err(|e| format!("Erro ao obter status: {}", e))
    }
}