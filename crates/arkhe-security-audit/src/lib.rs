//! Arkhe Security Audit — Pipeline de 6 fases da Cloudflare
//!
//! 1. Reconnaissance — Mapeia arquitetura e superfícies de ataque
//! 2. Hunt — Agentes paralelos caçam vulnerabilidades
//! 3. Validate — Agentes tentam refutar cada achado
//! 4. Report — Gera relatórios humanos e detalhados
//! 5. Structured Output — findings.json validado por schema
//! 6. Independent Verification — Agentes frescos verificam cada alegação

pub mod hunt;
pub mod types;

pub use hunt::HuntPhase;
pub use types::{AttackClass, Finding, Severity};

/// Orquestrador que executa as 6 fases em sequência.
pub struct AuditOrchestrator {
    target_dir: String,
    llm: std::sync::Arc<dyn hunt::InferenceEngine>,
}

impl AuditOrchestrator {
    pub fn new(target_dir: &str, llm: std::sync::Arc<dyn hunt::InferenceEngine>) -> Self {
        Self { target_dir: target_dir.to_string(), llm }
    }

    pub async fn run(&self) -> Result<Vec<Finding>, anyhow::Error> {
        tracing::info!("🔄 Iniciando auditoria de segurança (6 fases)");

        // Fase 1: Reconhecimento (Mocked)
        let architecture = "Mocked architecture".to_string();

        // Fase 2: Caça
        let hunt = HuntPhase::new(self.llm.clone());
        let findings = hunt.run(&architecture).await?;

        // Fase 3-6 (Mocked for now)

        Ok(findings)
    }
}
