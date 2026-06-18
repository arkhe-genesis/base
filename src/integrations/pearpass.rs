// src/integrations/pearpass.rs
//! Integração com PearPass para autenticação biométrica + ZKP.

use pearpass_sdk::{PearPassClient, AuthRequest, AuthProof, ZkpProof};

pub struct PearPassAuth {
    client: PearPassClient,
}

impl PearPassAuth {
    pub fn new(api_url: &str, api_key: &str) -> Self {
        let client = PearPassClient::new(api_url, api_key);
        Self { client }
    }

    /// Inicia o fluxo de autenticação biométrica.
    pub async fn start_auth(&self, user_id: &str) -> Result<AuthRequest, String> {
        self.client.start_auth(user_id).await
            .map_err(|e| format!("Erro PearPass: {}", e))
    }

    /// Verifica a prova ZK retornada pelo dispositivo.
    pub async fn verify_proof(&self, proof: &ZkpProof) -> Result<bool, String> {
        self.client.verify_proof(proof).await
            .map_err(|e| format!("Erro na verificação ZK: {}", e))
    }

    /// Gera uma prova ZK de que o usuário possui um ORCID sem revelá-lo.
    pub async fn prove_orcid(&self, user_id: &str, orcid_hash: &[u8]) -> Result<ZkpProof, String> {
        self.client.prove_claim(user_id, "orcid", orcid_hash).await
            .map_err(|e| format!("Erro ao provar ORCID: {}", e))
    }
}