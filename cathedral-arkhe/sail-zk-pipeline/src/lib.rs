//! sail-zk-pipeline — Pipeline de provas ZK com canais de resultado e integração Sail
//! Selo: CATHEDRAL-ARKHE-SAIL-ZK-PIPELINE-PROD-v1.0.0

use anyhow::{anyhow, Result};
use arkhe_zk_circuits::{PhysicalConstraintProofGenerator, PhysicalConstraintType, ZkProof, ZkBackend};
use arkhe_risc0_backend::Risc0Backend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, error, info};

// ============================================================
// JOB E RESULTADO
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofJob {
    pub job_id: String,
    pub design_hash: String,
    pub constraint_type: PhysicalConstraintType,
    pub parameters: serde_json::Value,
    pub sail_query_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofResult {
    pub job_id: String,
    pub proof: ZkProof,
    pub public_inputs: Vec<u64>,
    pub verified: bool,
    pub proving_time_ms: u64,
    pub error: Option<String>,
}

// ============================================================
// PIPELINE ZK COM CANAIS DE RESULTADO
// ============================================================

pub struct ZkPipeline {
    job_tx: mpsc::Sender<ZkProofJob>,
    pending: Arc<RwLock<HashMap<String, oneshot::Sender<ZkProofResult>>>>,
    _backend: Arc<dyn ZkBackend + Send + Sync>,
    result_broadcast_tx: tokio::sync::broadcast::Sender<ZkProofResult>,
}

impl ZkPipeline {
    pub async fn new(num_workers: usize) -> Result<Self> {
        let backend = Arc::new(Risc0Backend::new()?);
        let (job_tx, mut job_rx) = mpsc::channel(1000);
        let pending = Arc::new(RwLock::new(HashMap::new()));
        let (result_broadcast_tx, _) = tokio::sync::broadcast::channel(1000);

        // Workers
        for worker_id in 0..num_workers {
            let backend_clone = backend.clone();
            let pending_clone = pending.clone();
            let broadcast_tx = result_broadcast_tx.clone();

            tokio::spawn(async move {
                info!("🧵 Worker ZK {} iniciado", worker_id);
                while let Some(job) = job_rx.recv().await {
                    let start = std::time::Instant::now();
                    let result = Self::process_job(backend_clone.as_ref(), &job).await;
                    let elapsed = start.elapsed().as_millis() as u64;

                    let final_result = match result {
                        Ok(proof_result) => {
                            debug!("Job {} concluído em {}ms", job.job_id, elapsed);
                            ZkProofResult {
                                job_id: job.job_id.clone(),
                                proof: proof_result.clone(),
                                public_inputs: proof_result.public_inputs,
                                verified: true,
                                proving_time_ms: elapsed,
                                error: None,
                            }
                        }
                        Err(e) => {
                            error!("Job {} falhou: {}", job.job_id, e);
                            ZkProofResult {
                                job_id: job.job_id.clone(),
                                proof: ZkProof {
                                    proof_bytes: vec![],
                                    public_inputs: vec![],
                                    circuit_id: "error".to_string(),
                                    verification_key_hash: "".to_string(),
                                },
                                public_inputs: vec![],
                                verified: false,
                                proving_time_ms: elapsed,
                                error: Some(e.to_string()),
                            }
                        }
                    };

                    // Envia para broadcast (para múltiplos consumidores)
                    let _ = broadcast_tx.send(final_result.clone());

                    // Envia para o oneshot específico do job
                    if let Some(tx) = pending_clone.write().await.remove(&job.job_id) {
                        let _ = tx.send(final_result);
                    }
                }
            });
        }

        Ok(Self {
            job_tx,
            pending,
            _backend: backend,
            result_broadcast_tx,
        })
    }

    pub async fn new_with_mock(num_workers: usize) -> Result<Self> {
        struct MockBackend;
        impl ZkBackend for MockBackend {
            fn generate_proof(&self, _c: &str, _pub_in: &[u8], _priv_in: &[u8]) -> Result<ZkProof> {
                Ok(ZkProof {
                    proof_bytes: vec![1, 2, 3],
                    public_inputs: vec![42],
                    circuit_id: "mock".to_string(),
                    verification_key_hash: "0xMock".to_string(),
                })
            }
            fn verify_proof(&self, _p: &ZkProof) -> Result<bool> { Ok(true) }
        }

        let backend = Arc::new(MockBackend);
        let (job_tx, mut job_rx) = mpsc::channel(1000);
        let pending = Arc::new(RwLock::new(HashMap::new()));
        let (result_broadcast_tx, _) = tokio::sync::broadcast::channel(1000);

        for worker_id in 0..num_workers {
            let backend_clone = backend.clone();
            let pending_clone = pending.clone();
            let broadcast_tx = result_broadcast_tx.clone();

            tokio::spawn(async move {
                while let Some(job) = job_rx.recv().await {
                    let start = std::time::Instant::now();
                    let result = Self::process_job(backend_clone.as_ref(), &job).await;
                    let elapsed = start.elapsed().as_millis() as u64;

                    let final_result = match result {
                        Ok(proof_result) => ZkProofResult {
                            job_id: job.job_id.clone(),
                            proof: proof_result.clone(),
                            public_inputs: proof_result.public_inputs,
                            verified: true,
                            proving_time_ms: elapsed,
                            error: None,
                        },
                        Err(e) => ZkProofResult {
                            job_id: job.job_id.clone(),
                            proof: ZkProof {
                                proof_bytes: vec![],
                                public_inputs: vec![],
                                circuit_id: "error".to_string(),
                                verification_key_hash: "".to_string(),
                            },
                            public_inputs: vec![],
                            verified: false,
                            proving_time_ms: elapsed,
                            error: Some(e.to_string()),
                        }
                    };

                    let _ = broadcast_tx.send(final_result.clone());
                    if let Some(tx) = pending_clone.write().await.remove(&job.job_id) {
                        let _ = tx.send(final_result);
                    }
                }
            });
        }

        Ok(Self {
            job_tx,
            pending,
            _backend: backend,
            result_broadcast_tx,
        })
    }

    async fn process_job(
        backend: &dyn ZkBackend,
        job: &ZkProofJob,
    ) -> Result<ZkProof> {
        let generator = PhysicalConstraintProofGenerator::new(Box::new(backend.clone_box()));
        generator.generate_proof(job.constraint_type.clone(), &job.design_hash, &job.parameters)
    }

    // ============================================================
    // SUBMISSÃO INDIVIDUAL (COM ONESHOT)
    // ============================================================

    pub async fn submit_job(&self, job: ZkProofJob) -> Result<ZkProofResult> {
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.write().await;
            pending.insert(job.job_id.clone(), tx);
        }

        self.job_tx.send(job).await?;

        rx.await
            .map_err(|e| anyhow!("Job cancelado ou worker falhou: {}", e))
    }

    // ============================================================
    // SUBMISSÃO EM BATCH (COMPLETA)
    // ============================================================

    pub async fn submit_batch(&self, jobs: Vec<ZkProofJob>) -> Result<Vec<ZkProofResult>> {
        if jobs.is_empty() {
            return Ok(Vec::new());
        }

        info!("📦 Submetendo batch de {} jobs ZK", jobs.len());

        let mut receivers = Vec::with_capacity(jobs.len());
        let mut job_ids = Vec::with_capacity(jobs.len());

        // Submete todos os jobs com seus oneshots
        for job in jobs {
            let (tx, rx) = oneshot::channel();
            let job_id = job.job_id.clone();
            {
                let mut pending = self.pending.write().await;
                pending.insert(job_id.clone(), tx);
            }
            self.job_tx.send(job).await?;
            receivers.push(rx);
            job_ids.push(job_id);
        }

        // Aguarda todos os resultados
        let mut results = Vec::with_capacity(receivers.len());
        for (rx, job_id) in receivers.into_iter().zip(job_ids) {
            match rx.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Falha ao aguardar job {}: {}", job_id, e);
                    results.push(ZkProofResult {
                        job_id,
                        proof: ZkProof {
                            proof_bytes: vec![],
                            public_inputs: vec![],
                            circuit_id: "error".to_string(),
                            verification_key_hash: "".to_string(),
                        },
                        public_inputs: vec![],
                        verified: false,
                        proving_time_ms: 0,
                        error: Some(format!("Oneshot cancelado: {}", e)),
                    });
                }
            }
        }

        info!("✅ Batch concluído: {} resultados", results.len());
        Ok(results)
    }

    // ============================================================
    // INTEGRAÇÃO COM SAIL (Spark Connect)
    // ============================================================

    /// Processa um DataFrame/SQL do Sail e gera provas para cada linha
    pub async fn process_sail_dataframe(
        &self,
        rows: Vec<serde_json::Value>,
        constraint_type: PhysicalConstraintType,
    ) -> Result<Vec<ZkProofResult>> {
        let mut jobs = Vec::new();
        for row in rows {
            let design_hash = row
                .get("design_hash")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let job = ZkProofJob {
                job_id: uuid::Uuid::new_v4().to_string(),
                design_hash,
                constraint_type: constraint_type.clone(),
                parameters: row,
                sail_query_id: Some(format!("sail-query-{}", uuid::Uuid::new_v4())),
            };
            jobs.push(job);
        }

        self.submit_batch(jobs).await
    }

    /// Inscreve-se para receber resultados em tempo real (streaming)
    pub fn subscribe_results(&self) -> tokio::sync::broadcast::Receiver<ZkProofResult> {
        self.result_broadcast_tx.subscribe()
    }
}
