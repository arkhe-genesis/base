use serde::{Deserialize, Serialize};
use anyhow::Result;

pub struct OpenAntClient;

impl OpenAntClient {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub location: String,
    pub cwe_id: Option<String>,
    pub verified: bool,
    pub exploitation_details: Option<String>,
    pub remediation: Option<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::High => write!(f, "high"),
            Severity::Medium => write!(f, "medium"),
            Severity::Low => write!(f, "low"),
            Severity::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityProof {
    pub result_hash: String,
    pub signature: String,
    pub attestor_public_key: String,
    pub timestamp: u64,
    pub openant_version: String,
}

impl OpenAntClient {
    pub async fn analyze_with_bitsec(
        &self,
        code: &str,
        language: &str,
    ) -> Result<Vec<Vulnerability>> {
        let bittensor = crate::integrations::bittensor::BittensorClient::new(
            crate::integrations::bittensor::BittensorConfig::default()
        )?;
        let bitsec = crate::integrations::bittensor::sn60_bitsec::BitsecClient::new(std::sync::Arc::new(bittensor));

        let response = bitsec.analyze_code(code, language, true).await?;

        let mut vulns = Vec::new();
        for v in response.vulnerabilities {
            vulns.push(Vulnerability {
                id: v.id,
                title: v.title,
                description: v.description,
                severity: match v.severity.as_str() {
                    "critical" => Severity::Critical,
                    "high" => Severity::High,
                    "medium" => Severity::Medium,
                    "low" => Severity::Low,
                    _ => Severity::Info,
                },
                location: v.location,
                cwe_id: v.cwe_id,
                verified: false,
                exploitation_details: None,
                remediation: v.remediation,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }

        Ok(vulns)
    }
}
