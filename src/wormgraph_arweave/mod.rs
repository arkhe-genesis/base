use anyhow::Result;
use tracing::info;

pub struct WormGraphIndexer;

impl WormGraphIndexer {
    pub fn new() -> Self {
        Self
    }

    pub fn index_vulnerability(&self, vuln: &crate::integrations::openant::Vulnerability, _source: &str) -> Result<String> {
        Ok(vuln.id.clone())
    }

    pub async fn index_with_recall(
        &mut self,
        vuln: &crate::integrations::openant::Vulnerability,
        source: &str,
    ) -> Result<String> {
        let tx_id = self.index_vulnerability(vuln, source)?;

        let bittensor = crate::integrations::bittensor::BittensorClient::new(
            crate::integrations::bittensor::BittensorConfig::default()
        )?;
        let recall = crate::integrations::bittensor::sn31_recall::RecallClient::new(std::sync::Arc::new(bittensor));

        let metadata = serde_json::to_value(vuln)?;
        let recall_id = recall.store(
            &vuln.id,
            &format!("{} - {}", vuln.title, vuln.description),
            metadata,
        ).await?;

        info!("📚 Vulnerabilidade armazenada na SN31: {}", recall_id);
        Ok(tx_id)
    }
}
