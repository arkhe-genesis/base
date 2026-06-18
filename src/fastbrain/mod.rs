use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::info;

pub struct FastBrain;

impl FastBrain {
    pub fn new() -> Self {
        Self
    }

    /// Inferência usando SN96 (Verathos) com verificação ZK
    pub async fn infer_with_verathos(
        &self,
        prompt: &str,
        verify_zk: bool,
    ) -> Result<String> {
        let bittensor = crate::integrations::bittensor::BittensorClient::new(
            crate::integrations::bittensor::BittensorConfig::default()
        )?;
        let verathos = crate::integrations::bittensor::sn96_verathos::VerathosClient::new(Arc::new(bittensor));

        if verify_zk {
            let (text, proof) = verathos.infer_with_zk(prompt, Some(2000), Some(0.7)).await?;
            // Verifica a prova
            if verathos.verify_zk_proof(&proof).await? {
                info!("✅ Prova ZK verificada com sucesso");
                Ok(text)
            } else {
                Err(anyhow!("Falha na verificação da prova ZK"))
            }
        } else {
            verathos.infer(prompt, Some(2000), Some(0.7)).await
        }
    }
}
