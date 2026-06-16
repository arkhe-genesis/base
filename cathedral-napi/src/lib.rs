//! cathedral-napi – Native Node.js binding for Cathedral ARKHE
//!
//! Exposes:
//! - `prove_memory_state()` – generates a DLA memory proof
//! - `CathedralAgent` – wrapper around EmbodiedCognitiveCore for full agent control

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;
use tokio::runtime::Runtime;

// ════════════════════════════════════════════════════════════════
// 1. DLA Memory Proof (standalone function)
// ════════════════════════════════════════════════════════════════

#[derive(serde::Serialize, serde::Deserialize)]
#[napi(object)]
pub struct MemoryProof {
    pub merkle_root: String,
    pub timestamp: i64,
    pub state_count: u32,
}

/// Generate a cryptographic commitment to the current DLA memory state.
/// This can be called from TypeScript without instantiating the full agent.
#[napi]
pub async fn prove_memory_state() -> Result<MemoryProof> {
    // In production, replace with actual DLA call (FFI, HTTP, or direct engine)
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Mock merkle root (use blake3 or real DLA)
    let merkle_root = format!(
        "0x{:064x}",
        blake3::hash(format!("dla_memory_{}", timestamp).as_bytes()).as_bytes()[..8]
            .iter()
            .fold(0u64, |acc, &b| (acc << 8) | b as u64)
    );

    Ok(MemoryProof {
        merkle_root,
        timestamp,
        state_count: 47, // example
    })
}

// ════════════════════════════════════════════════════════════════
// 2. CathedralAgent – High‑level wrapper for EmbodiedCognitiveCore
// ════════════════════════════════════════════════════════════════

// We assume cathedral-embodied-no_std is a dependency of this crate
use cathedral_embodied_no_std::core::embodied_cognitive_core::EmbodiedCognitiveCore;

#[napi]
pub struct CathedralAgent {
    core: Arc<std::sync::Mutex<EmbodiedCognitiveCore>>,
    runtime: Arc<Runtime>,
}

#[napi]
impl CathedralAgent {
    /// Creates a new agent instance.
    /// Reads environment variables:
    /// - PICOADS_API_KEY
    /// - PICOADS_BACKEND_URL
    /// - SUCCESS_RECORDER_DB (optional, for SQLite persistence)
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        let picoads_api_key = std::env::var("PICOADS_API_KEY").ok();
        let picoads_backend = std::env::var("PICOADS_BACKEND_URL").ok();
        let _recorder_db = std::env::var("SUCCESS_RECORDER_DB").ok();

        let core = EmbodiedCognitiveCore::new(picoads_api_key, picoads_backend);
        let runtime = Runtime::new().map_err(|e| napi::Error::from_reason(e.to_string()))?;

        Ok(Self {
            core: Arc::new(std::sync::Mutex::new(core)),
            runtime: Arc::new(runtime),
        })
    }

    /// Execute a single cognitive tick (async from Node.js)
    #[napi]
    pub async fn tick(&self) -> napi::Result<String> {
        // Because inner structure of EmbodiedCognitiveCore uses `rusqlite` which is `!Send`,
        // and standard Mutex guard is `!Send` either, we must lock inside blocking thread
        // or just spawn blocking directly.
        let core_arc = Arc::clone(&self.core);
        let result = self.runtime.spawn_blocking(move || {
            let mut core = core_arc.lock().unwrap();
            let fut = core.tick_zk_with_accelerator();
            // Actually tick_zk_with_accelerator is async, we can block on it here
            tokio::runtime::Handle::current().block_on(fut)
        }).await;

        match result {
            Ok(Ok(_)) => Ok("tick_complete".to_string()),
            Ok(Err(e)) => Err(napi::Error::from_reason(e)),
            Err(e) => Err(napi::Error::from_reason(e.to_string())),
        }
    }

    /// Retrieve the current policy as a JSON string
    #[napi]
    pub fn get_policy(&self) -> napi::Result<String> {
        let core = self.core.lock().map_err(|e| napi::Error::from_reason(e.to_string()))?;
        serde_json::to_string(&core.current_policy)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Record that a recommendation was accepted by the user
    #[napi]
    pub fn accept_recommendation(&self, rec_id: String) -> napi::Result<()> {
        let mut core = self.core.lock().map_err(|e| napi::Error::from_reason(e.to_string()))?;
        core.accept_recommendation(&rec_id);
        Ok(())
    }

    /// Get current round number
    #[napi]
    pub fn current_round(&self) -> napi::Result<u32> {
        let core = self.core.lock().map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(core.current_round)
    }

    /// Fetch PicoAds recommendations directly from the core (returns JSON array)
    #[napi]
    pub async fn get_recommendations(
        &self,
        query: String,
        hub: Option<String>,
        max_results: Option<u32>,
    ) -> napi::Result<String> {
        let core_arc = Arc::clone(&self.core);

        let result = self.runtime.spawn_blocking(move || {
            let core = core_arc.lock().unwrap();
            let fut = core.fetch_picoads_recommendations(&query, hub.as_deref(), max_results);
            tokio::runtime::Handle::current().block_on(fut)
        }).await;

        let recs = result
            .map_err(|e| napi::Error::from_reason(e.to_string()))?
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        serde_json::to_string(&recs).map_err(|e| napi::Error::from_reason(e.to_string()))
    }
}

// Required by napi-rs
#[napi]
pub fn init() {}
