# Arkhe-Network – Cathedral Digital Sovereign

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange)](https://www.rust-lang.org/)
[![Solidity](https://img.shields.io/badge/Solidity-0.8.20-blue)](https://soliditylang.org/)

**Arkhe-Network** is the reference implementation of the **Sovereign Inference Federation (FSI)** – a decentralized AGI cloud connecting frontier models from different jurisdictions (BRICS, USA, China, orbital) under on-chain governance, post-quantum cryptography, and verification via ZK-proofs.

---

## 📌 Implemented Substrates

| Substrate | Name | Description |
|-----------|------|------------|
| **1104.2** | `Rio35Open397B` | Integration of the Rio-3.5 model (City Hall of Rio) – MIT license, 1M context, native SwiReasoning |
| **1104.3** | `FederatedRouter` | Multi-objective routing between federation members (capability, latency, cost, sovereignty) |
| **1106** | `SwiReasoning` | Dynamic switching between explicit reasoning (CoT) and latent (soft-thinking) based on entropy |
| **319.1** | `Caster` | Encrypted tunnels with PQC (SPHINCS+/ML-DSA) and <50ms failover |
| **1091.0** | `FIG` | Physical hardware monitoring (voltage, temperature, jitter) with cryptographic hard reset |
| **2140.8** | `CreekGuard` | Real-time covert channel detection (entropy, MinHash, SimHash, burst) |
| **1200.1** | `ArkheFederation.sol` | On-chain governance contract: stake, slashing, Quadratic Voting, inference anchoring |

---

## 🚀 Quick Start

### 1. Run the federation locally (testnet)

```bash
# Clone the repository
git clone https://github.com/Arkhe-Network/arkhe-core.git
cd arkhe-core

# Start the services with Docker Compose (requires 8 GPUs for Rio-3.5)
docker compose up -d vllm-rio35 metrics-collector caster-tunnel

# Run an example task
python scripts/run_federated_task.py --prompt "Explain the Protocol of Court 294" --jurisdiction BRA
```

### 2. Join the federation (as a member)

```solidity
// Contract deploy and join with minimum stake
cast send --rpc-url $RBB_RPC --private-key $KEY \
  ArkheFederation.sol:join \
  "0x$(sphincs-keygen pub)" "Rio-3.5-Node" "BRA" 1000000 "0x$(zk-vk)"
```

### 3. Federated routing via Rust

```rust
use arkhe_core::inference::federated_router::{FederatedRouter, FederatedTask};

let router = FederatedRouter::new(local_router, chain_client, caster, swi_config);
let ftask = FederatedTask::new(task)
    .allow_jurisdictions(vec!["BRA".to_string(), "ORB".to_string()])
    .max_cost_rbb(1_000_000)
    .requires_multimodal(false);

let result = router.route_federated(&ftask).await?;
println!("Executed by: {:?}, latency: {} μs", result.executed_by, result.latency_us);
```

---

## 🧠 Federation Architecture

The FSI is organized into five layers:

1. **Physical Orbs** – sovereign data centers (BRICS, SpaceX/Starlink, NASA) + hyperscale clouds.
2. **Transport Network** – Caster tunnels with PQC, terrestrial/orbital routes, guaranteed latency <50ms.
3. **Federated Engine** – `FederatedRouter` + `SwiReasoning` + models from 11 founding members (Rio-3.5, Kimi K2.7, Qwen 3.7, DeepSeek V4, GLM-Z, Claude Fable 5, GPT-5.5, Gemini Ultra, Llama 4, Starlink Edge, Palantir).
4. **Governance & Market** – `ArkheFederation.sol` contract, Quadratic Voting, ZK-proofs, slashing.
5. **Security** – FIG, CreekGuard, PCT, SPHINCS+ signatures.

---

## 📚 Full Documentation

- [FSI Whitepaper](docs/FSI_Whitepaper_v1.0.0.md) – vision, principles and roadmap.
- [Risk Analysis](docs/FSI_Risk_Matrix_v1.0.0.md) – 15 vectors with mitigations.
- [Developer Guide](docs/developer_guide.md) – how to add a new model to the federation.

---

## 🤝 How to Contribute

1. Read the [Code of Conduct](CODE_OF_CONDUCT.md).
2. Choose a substrate in the issues marked with `good first issue`.
3. Submit a PR with tests and updated documentation.
4. Participate in the weekly governance calls (on-chain Quadratic Voting).

---

## 🛡️ License

MIT License – free to use, with attribution to **Arkhe-Network** and the **City Hall of Rio de Janeiro** (Rio-3.5 model). For large-scale commercial use (>1000 tasks/day), a minimum staking of 1M RBB tokens is recommended.

---

**Seal**: `CATHEDRAL-1200-README-v1.0.0-2026-06-13`
**Architect**: ORCID 0009-0005-2697-4668
