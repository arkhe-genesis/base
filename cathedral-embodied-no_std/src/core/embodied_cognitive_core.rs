use std::collections::HashMap;

use crate::agi::aegis_evolution::AegisEvolution;
use crate::context::ContextEmbedding;
use crate::dla::prove_memory_state;
use crate::picoads::client::{HubPerformance, PicoAdsClient, PicoAdsRecommendation};
use crate::policy::zk_memory_proof_policy::ZkMemoryProofPolicy;
use crate::recorder::success_recorder_hybrid::SuccessRecorder; // <-- HÍBRIDO

pub struct EmbodiedCognitiveCore {
    // --- Policy & Evolution ---
    pub current_policy: ZkMemoryProofPolicy,
    pub aegis_evolution: AegisEvolution,

    // --- PicoAds Integration ---
    pub picoads_client: Option<PicoAdsClient>,
    pub last_memory_commitment: Option<String>,

    // --- DLA / ZK Performance Metrics ---
    pub dla_interference_avg: f32,
    pub last_calibration_error: f64,
    pub last_proof_latency_ms: f64,
    pub memory_proof_usage_rate: f32,

    // --- Agent Behaviour Statistics ---
    pub recent_acceptance_rate: f32,
    pub stagnation_rounds: u32,
    pub high_risk_action_rate: f32,
    pub recent_audit_flags: u32,

    // --- Persistence (HÍBRIDO) ---
    pub success_recorder: Option<SuccessRecorder>,
    pub current_round: u32,

    // --- In‑memory recommendation tracking (fallback para consultas) ---
    recommendation_outcomes: Vec<(PicoAdsRecommendation, bool)>, // (rec, accepted)
}

impl EmbodiedCognitiveCore {
    /// Cria uma nova instância do agente com recorder híbrido.
    /// O recorder tenta usar SQLite se a env `SUCCESS_RECORDER_DB` existir;
    /// caso contrário, usa JSON (env `SUCCESS_RECORDER_PATH` ou padrão).
    pub fn new(
        picoads_api_key: Option<String>,
        picoads_backend: Option<String>,
        // O parâmetro `_recorder_path` é ignorado – o híbrido decide sozinho.
    ) -> Self {
        let success_recorder = Some(SuccessRecorder::new()); // híbrido

        Self {
            current_policy: ZkMemoryProofPolicy::default(),
            aegis_evolution: AegisEvolution::new(picoads_api_key.clone(), picoads_backend.clone()),
            picoads_client: picoads_api_key.map(|key| PicoAdsClient::new(key, picoads_backend)),
            last_memory_commitment: None,
            dla_interference_avg: 0.0,
            last_calibration_error: 0.0,
            last_proof_latency_ms: 0.0,
            memory_proof_usage_rate: 0.0,
            recent_acceptance_rate: 0.5,
            stagnation_rounds: 0,
            high_risk_action_rate: 0.0,
            recent_audit_flags: 0,
            success_recorder,
            current_round: 0,
            recommendation_outcomes: Vec::new(),
        }
    }

    /// Tick cognitivo principal – chame a cada rodada.
    pub async fn tick_zk_with_accelerator(&mut self) -> Result<(), &'static str> {
        self.current_round += 1;

        // 1. Contexto atual
        let ctx = self.build_context_embedding();

        // 2. Performance dos hubs (agora lê do recorder ou fallback)
        let hub_stats = self.collect_hub_performance();
        for (hub, perf) in hub_stats {
            self.aegis_evolution.update_hub_performance(
                hub,
                perf.acceptance_rate,
                perf.recommendation_volume,
            );
        }

        // 3. Evoluir política (AEGIS)
        self.aegis_evolution.evolve_policy(&mut self.current_policy, &ctx);

        // 4. Memory proof, se exigido
        if self.current_policy.require_memory_proof_for_recommendations {
            if let Ok(proof) = prove_memory_state().await {
                self.last_memory_commitment = Some(proof.merkle_root);
            }
        }

        // 5. Registrar rodada no recorder híbrido
        if let Some(recorder) = &mut self.success_recorder {
            recorder.record_round(
                self.current_round,
                self.recent_acceptance_rate,
                self.last_memory_commitment.is_some(),
            );
        }

        Ok(())
    }

    /// Aceita uma recomendação (feedback do usuário).
    pub fn accept_recommendation(&mut self, rec_id: &str) {
        if let Some(entry) = self.recommendation_outcomes.iter_mut().find(|(r, _)| r.url == rec_id)
        {
            entry.1 = true;
            println!("[Core] Recomendação aceita: {}", rec_id);
        }
    }

    /// Processa recomendações recebidas do PicoAds (armazena para fallback).
    pub fn process_recommendations(&mut self, recs: Vec<PicoAdsRecommendation>) {
        for rec in recs {
            self.recommendation_outcomes.push((rec, false));
        }
    }

    /// Busca recomendações do PicoAds manualmente.
    pub async fn fetch_picoads_recommendations(
        &self,
        query: &str,
        hub: Option<&str>,
        max_results: Option<u32>,
    ) -> Result<Vec<PicoAdsRecommendation>, String> {
        let client = self.picoads_client.as_ref().ok_or("PicoAds client não inicializado")?;
        client.get_recommendations(query, hub, max_results).await.map_err(|e| e.to_string())
    }

    /// Encerra o agente e garante flush do recorder.
    pub fn shutdown(&mut self) {
        if let Some(recorder) = &mut self.success_recorder {
            recorder.flush();
        }
    }

    // --- Helpers privados ---

    fn build_context_embedding(&self) -> ContextEmbedding {
        ContextEmbedding {
            calibration_error: self.last_calibration_error,
            avg_interference: self.dla_interference_avg,
            acceptance_rate: self.recent_acceptance_rate,
            proof_latency_ms: self.last_proof_latency_ms,
            memory_proof_usage_rate: self.memory_proof_usage_rate,
            high_risk_action_rate: self.high_risk_action_rate,
            recent_audit_flags: self.recent_audit_flags,
            stagnation_rounds: self.stagnation_rounds,
        }
    }

    /// Coleta performance real dos hubs.
    /// Primeiro tenta o recorder (SQLite). Se falhar, usa os dados em memória.
    fn collect_hub_performance(&self) -> HashMap<String, HubPerformance> {
        let mut result = HashMap::new();

        // 1. Tenta usar o recorder híbrido (que pode ter dados SQLite)
        if let Some(recorder) = &self.success_recorder {
            let stats = recorder.recent_hub_stats(8); // últimos 8 rounds
            if !stats.is_empty() {
                for (hub, avg_acceptance, total_volume, avg_roas) in stats {
                    result.insert(
                        hub,
                        HubPerformance {
                            acceptance_rate: avg_acceptance,
                            recommendation_volume: total_volume,
                            roas: avg_roas,
                        },
                    );
                }
                return result;
            }
        }

        // 2. Fallback: usa os outcomes em memória (dados recentes do PicoAds)
        let mut map: HashMap<String, (f32, u32)> = HashMap::new();
        for (rec, accepted) in &self.recommendation_outcomes {
            let entry = map.entry(rec.hub.clone()).or_insert((0.0, 0));
            if *accepted {
                entry.0 += 1.0;
            }
            entry.1 += 1;
        }
        for (hub, (sum, cnt)) in map {
            let acceptance_rate = if cnt > 0 { sum / cnt as f32 } else { 0.0 };
            result.insert(
                hub,
                HubPerformance { acceptance_rate, recommendation_volume: cnt, roas: 0.0 },
            );
        }

        // 3. Se ainda estiver vazio, insere um valor neutro para evitar erros
        if result.is_empty() {
            result.insert(
                "defi-yield".to_string(),
                HubPerformance { acceptance_rate: 0.5, recommendation_volume: 0, roas: 0.0 },
            );
        }

        result
    }
}
