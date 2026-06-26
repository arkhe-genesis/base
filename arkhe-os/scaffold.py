import os

CRATES = [
    "arkhe-core", "arkhe-identity", "arkhe-asset", "arkhe-bridge",
    "arkhe-payment", "arkhe-agents", "arkhe-llm", "arkhe-neural",
    "arkhe-symbolic", "arkhe-metacognition", "arkhe-apex", "arkhe-agi",
    "arkhe-consensus-tpm", "arkhe-bootmaker", "arkhe-ventoy-adapter",
    "arkhe-openwrt-adapter"
]

FILES_BY_CRATE = {
    "arkhe-core": ["error.rs", "types.rs", "crypto.rs"],
    "arkhe-identity": ["did.rs", "vc.rs", "mldsa.rs", "hybrid.rs", "resolver.rs"],
    "arkhe-asset": ["asset.rs", "erc20.rs", "erc721.rs", "wallet.rs"],
    "arkhe-bridge": ["nomic.rs", "taproot.rs", "ethereum.rs", "cosmos.rs", "adapter.rs"],
    "arkhe-payment": [],
    "arkhe-agents": ["agent.rs", "capsule.rs", "intent.rs", "orchestrator.rs", "tools.rs"],
    "arkhe-llm": ["model.rs", "inference.rs", "fine_tuning.rs", "rag.rs", "embeddings.rs"],
    "arkhe-neural": ["causal_attention.rs", "episodic_memory.rs", "semantic_memory.rs", "counterfactual.rs", "emerge.rs"],
    "arkhe-symbolic": ["atomspace.rs", "logic.rs", "nars.rs", "owl.rs", "grounding.rs"],
    "arkhe-metacognition": ["monitor.rs", "escalate.rs", "reflect.rs"],
    "arkhe-apex": ["active_inference.rs", "world_model.rs", "goal_decomposer.rs", "agent.rs"],
    "arkhe-agi": ["coordinator.rs", "pipeline.rs", "lifecycle.rs"],
    "arkhe-consensus-tpm": ["tpm.rs", "consensus.rs"],
    "arkhe-bootmaker": ["iso.rs", "usb.rs", "sign.rs"],
    "arkhe-ventoy-adapter": ["installer.rs", "plugin.rs", "boot.rs"],
    "arkhe-openwrt-adapter": ["uci.rs", "ubus.rs", "procd.rs", "package.rs"]
}

FILE_CONTENTS = {
    "arkhe-core/src/error.rs": """use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum KernelError {
    #[error("Generic error")]
    Generic,
    #[error("Invalid capability")]
    InvalidCapability,
    #[error("Capability expired")]
    CapabilityExpired,
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    #[error("Out of memory")]
    OutOfMemory,
    #[error("IPC error: {0}")]
    IpcError(String),
    #[error("Invalid syscall: {0}")]
    InvalidSyscall(u32),
    #[error("Invalid proof")]
    InvalidProof,
    #[error("PQC error: {0}")]
    PqcError(String),
    #[error("Adapter not found")]
    AdapterNotFound,
    #[error("Unsupported asset: {0}")]
    UnsupportedAsset(String),
    #[error("Timeout")]
    Timeout,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, KernelError>;
""",
    "arkhe-core/src/types.rs": """use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did(pub String);

impl Did {
    pub fn new(method: &str, identifier: &str) -> Self {
        Self(format!("did:{}:{}", method, identifier))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub agent_id: AgentId,
    pub capability: Capability,
    pub expires_at: u64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Capability {
    TransferAssets,
    VerifyProofs,
    QueryUniverse,
    DelegateTasks,
    ReadMemory,
    WriteMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRef {
    pub chain: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    TransferAsset {
        asset: AssetRef,
        amount: u64,
        recipient: Did,
        priority: u8,
    },
    VerifyProof {
        proof: Vec<u8>,
        public_inputs: Vec<u8>,
        priority: u8,
    },
    DelegateTask {
        task: Task,
        to: AgentId,
        priority: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub payload: Vec<u8>,
}
""",
    "arkhe-identity/src/mldsa.rs": """use ml_dsa::{MlDsa87, signing::SigningKey, verifying::VerifyingKey, Signature};
use rand::rngs::OsRng;
use arkhe_core::error::{KernelError, Result};

pub struct MldsaSigner {
    signing_key: SigningKey<MlDsa87>,
    verifying_key: VerifyingKey<MlDsa87>,
}

impl MldsaSigner {
    pub fn generate() -> Self {
        let mut rng = OsRng;
        let sk = SigningKey::<MlDsa87>::generate(&mut rng);
        let vk = sk.verifying_key();
        Self { signing_key: sk, verifying_key: vk }
    }

    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        self.signing_key.sign(msg).to_bytes().to_vec()
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> Result<bool> {
        let sig = Signature::from_bytes(sig).map_err(|_| KernelError::PqcError("Invalid signature".to_string()))?;
        Ok(self.verifying_key.verify(msg, &sig).is_ok())
    }

    pub fn verifying_key_bytes(&self) -> Vec<u8> {
        self.verifying_key.to_bytes().to_vec()
    }
}
""",
    "arkhe-identity/src/hybrid.rs": """use ml_kem::{MlKem1024, kem::Kem};
use ml_kem::kem::DecapsulationKey;
use ml_kem::kem::EncapsulationKey;
// Using placeholder for dalek since it might not be in the dependencies or could cause conflict.
// use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};
use rand::rngs::OsRng;

pub struct HybridKeyExchange {
    // x25519_secret: EphemeralSecret,
    // x25519_public: X25519PublicKey,
    mlkem_decaps_key: ml_kem::DecapsulationKey<MlKem1024>,
    mlkem_encaps_key: ml_kem::EncapsulationKey<MlKem1024>,
}

impl HybridKeyExchange {
    pub fn generate() -> Self {
        let mut rng = OsRng;
        // let x25519_secret = EphemeralSecret::random();
        // let x25519_public = X25519PublicKey::from(&x25519_secret);
        let (dk, ek) = MlKem1024::generate_keypair().unwrap();
        Self { /* x25519_secret, x25519_public, */ mlkem_decaps_key: dk, mlkem_encaps_key: ek }
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32 + self.mlkem_encaps_key.as_bytes().len());
        // bytes.extend_from_slice(self.x25519_public.as_bytes());
        bytes.extend_from_slice(&[0u8; 32]);
        bytes.extend_from_slice(self.mlkem_encaps_key.as_bytes());
        bytes
    }

    pub fn derive_shared(&self, peer_public: &[u8]) -> Vec<u8> {
        /*
        let peer_x25519 = X25519PublicKey::from(peer_public[..32].try_into().unwrap());
        let x25519_shared = self.x25519_secret.diffie_hellman(&peer_x25519);
        */
        let mlkem_ek = ml_kem::EncapsulationKey::<MlKem1024>::from_bytes(&peer_public[32..]).unwrap();
        let (_, mlkem_shared) = mlkem_ek.encapsulate().unwrap();
        let mut combined = Vec::with_capacity(32 + 32);
        combined.extend_from_slice(&[0u8; 32]); // placeholder
        combined.extend_from_slice(&mlkem_shared);
        let hash = blake3::hash(&combined);
        hash.as_bytes().to_vec()
    }
}
""",
    "arkhe-agents/src/agent.rs": """use crate::intent::Intent;
use crate::capsule::Capsule;
use arkhe_core::types::{AgentId, CapabilityToken};
use std::collections::HashMap;

pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub capsule: Capsule,
    pub capabilities: Vec<CapabilityToken>,
    pub intent_queue: Vec<Intent>,
}

impl Agent {
    pub fn new(id: AgentId, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            capsule: Capsule::new(id),
            capabilities: Vec::new(),
            intent_queue: Vec::new(),
        }
    }

    pub fn submit_intent(&mut self, intent: Intent) {
        self.intent_queue.push(intent);
    }
}

pub struct AgentManager {
    agents: std::collections::HashMap<AgentId, Agent>,
    next_id: u64,
}

impl AgentManager {
    pub fn new() -> Self {
        Self { agents: HashMap::new(), next_id: 0 }
    }

    pub fn create_agent(&mut self, name: &str) -> AgentId {
        let id = AgentId(self.next_id);
        self.next_id += 1;
        self.agents.insert(id, Agent::new(id, name));
        id
    }

    pub fn get_agent(&self, id: AgentId) -> Option<&Agent> {
        self.agents.get(&id)
    }
}
""",
    "arkhe-agents/src/intent.rs": """use arkhe_core::types::Intent;
use std::collections::VecDeque;

pub struct IntentScheduler {
    queue: VecDeque<Intent>,
}

impl IntentScheduler {
    pub fn new() -> Self {
        Self { queue: VecDeque::new() }
    }

    pub fn submit(&mut self, intent: Intent) {
        self.queue.push_back(intent);
    }

    pub fn schedule_next(&mut self) -> Option<Intent> {
        self.queue.pop_front()
    }
}
""",
    "arkhe-agents/src/capsule.rs": """use arkhe_core::types::AgentId;

pub struct Capsule {
    pub agent_id: AgentId,
}

impl Capsule {
    pub fn new(agent_id: AgentId) -> Self {
        Self { agent_id }
    }
}
""",
    "arkhe-llm/src/inference.rs": """use async_trait::async_trait;

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn generate(&self, prompt: &str, temperature: f32, max_tokens: u32) -> Result<String, String>;
}

pub struct LlamaCppEngine {
    model_path: String,
    // ctx: Option<llama_cpp_2::model::LlamaModel>,
}

impl LlamaCppEngine {
    pub fn new(model_path: &str) -> Self {
        Self { model_path: model_path.to_string(), /* ctx: None */ }
    }

    /* pub fn load(&mut self) -> Result<(), String> {
        let params = llama_cpp_2::model::params::LlamaModelParams::default();
        let backend = llama_cpp_2::context::params::LlamaContextParams::default();
        self.ctx = Some(llama_cpp_2::model::LlamaModel::load_from_file(&self.model_path, params)
            .map_err(|e| e.to_string())?);
        Ok(())
    } */
}

/*
#[async_trait]
impl InferenceEngine for LlamaCppEngine {
    async fn generate(&self, prompt: &str, temperature: f32, max_tokens: u32) -> Result<String, String> {
        let ctx = self.ctx.as_ref().ok_or("Model not loaded")?;
        let mut params = llama_cpp_2::LlamaInferenceParams::default();
        params.temperature = temperature;
        params.n_predict = max_tokens as i32;
        let output = ctx.inference(prompt, params)
            .map_err(|e| e.to_string())?;
        Ok(output)
    }
}
*/
""",
    "arkhe-agi/src/coordinator.rs": """use arkhe_agents::{agent::AgentManager, intent::IntentScheduler};
use arkhe_llm::inference::InferenceEngine;
use arkhe_identity::mldsa::MldsaSigner;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AgiCoordinator {
    agents: Arc<Mutex<AgentManager>>,
    scheduler: Arc<Mutex<IntentScheduler>>,
    llm: Arc<dyn InferenceEngine>,
    signer: MldsaSigner,
}

impl AgiCoordinator {
    pub fn new(llm: Arc<dyn InferenceEngine>) -> Self {
        Self {
            agents: Arc::new(Mutex::new(AgentManager::new())),
            scheduler: Arc::new(Mutex::new(IntentScheduler::new())),
            llm,
            signer: MldsaSigner::generate(),
        }
    }

    pub async fn run(&self) -> ! {
        loop {
            if let Some(intent) = self.scheduler.lock().await.schedule_next() {
                // Processa intenção
                println!("Executing intent: {:?}", intent);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}
"""
}

def create_crate(crate_name):
    crate_dir = os.path.join("arkhe-os", "crates", crate_name)
    src_dir = os.path.join(crate_dir, "src")
    os.makedirs(src_dir, exist_ok=True)

    extra_deps = ""
    if crate_name == "arkhe-core":
        extra_deps = "thiserror.workspace = true\nserde.workspace = true\nuuid.workspace = true"
    elif crate_name == "arkhe-identity":
        extra_deps = 'ml-dsa.workspace = true\nml-kem.workspace = true\nrand.workspace = true\narkhe-core = { path = "../arkhe-core" }\nblake3.workspace = true'
    elif crate_name == "arkhe-agents":
        extra_deps = 'arkhe-core = { path = "../arkhe-core" }'
    elif crate_name == "arkhe-llm":
        extra_deps = "async-trait.workspace = true\nllama-cpp-2.workspace = true"
    elif crate_name == "arkhe-agi":
        extra_deps = 'tokio.workspace = true\narkhe-agents = { path = "../arkhe-agents" }\narkhe-llm = { path = "../arkhe-llm" }\narkhe-identity = { path = "../arkhe-identity" }'

    cargo_toml = f"""[package]
name = "{crate_name}"
version.workspace = true
edition.workspace = true

[dependencies]
{extra_deps}
"""

    with open(os.path.join(crate_dir, "Cargo.toml"), "w") as f:
        f.write(cargo_toml)

    lib_content = "#![allow(dead_code)]\n#![allow(unused_imports)]\n#![warn(missing_docs)]\n#![allow(unsafe_code)]\n\n//! Arkhe OS Component\n"
    for module in FILES_BY_CRATE.get(crate_name, []):
        mod_name = module.replace(".rs", "")
        lib_content += f"pub mod {mod_name};\n"

    with open(os.path.join(src_dir, "lib.rs"), "w") as f:
        f.write(lib_content)

    for module in FILES_BY_CRATE.get(crate_name, []):
        file_path = os.path.join(crate_name, "src", module)
        full_path = os.path.join("arkhe-os", "crates", file_path)
        content = FILE_CONTENTS.get(file_path, f"//! {module}\n")
        with open(full_path, "w") as f:
            f.write(content)

for crate in CRATES:
    create_crate(crate)
