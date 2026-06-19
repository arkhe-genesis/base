# 🏛️ Cathedral ARKHE — AGI-GRADE v3.0.0

**Selo:** `CATHEDRAL-ARKHE-AGI-GRADE-v3.0.0-2026-06-19`

## Arquitetura

```mermaid
flowchart TB
    subgraph "Edge Agent"
        Scheduler[HybridScheduler]
        Registry[WorkerRegistry]
        Fallback[FallbackChain]
        CostOpt[CostOptimizer]
        Health[Healthcheck / Metrics]
    end

    subgraph "AGI Core"
        WM[WorldModel]
        MCTS[MCTS Engine]
        MCL[MetaCognitiveLoop]
        WH[HierarchicalWormhole]
        EV[EthicsVerifier]
        LLM[Ollama Client]
    end

    subgraph "Memory"
        ES[EpisodicSync]
        SQLite[(SQLite)]
    end

    subgraph "UI"
        Dashboard[Next.js Dashboard]
        SSE[SSE Stream]
        Chat[AI Chat Agent]
        Auth[Better-Auth + B20]
    end

    User --> Dashboard
    User --> Edge Agent

    Edge Agent --> Scheduler
    Scheduler --> Registry
    Scheduler --> Fallback
    Fallback --> CostOpt

    Edge Agent --> AGI Core
    AGI Core --> WM
    AGI Core --> MCTS
    AGI Core --> MCL
    AGI Core --> WH
    AGI Core --> EV
    AGI Core --> LLM

    MCL <--> ES
    ES --> SQLite

    AGI Core --> Dashboard
    Dashboard --> SSE
    Dashboard --> Chat
    Dashboard --> Auth
```

## Componentes

| Crate | Versão | Descrição |
|-------|--------|-----------|
| `cathedral-scheduler` | 1.0.0 | HybridScheduler + Worker Registry + Prometheus |
| `cathedral-episodic` | 1.0.0 | EpisodicSync com SQLite (CRDT-lite) |
| `cathedral-tee` | 1.0.0 | TEEBridge (SGX + IoNet) |
| `cathedral-fallback` | 1.0.0 | FallbackChain 3 níveis + CostOptimizer |
| `cathedral-agi` | 3.0.0 | AGI Core (WorldModel + MCTS + MetaCognitive + Wormhole + Ethics) |
| `cathedral-edge-agent` | 1.0.0 | Edge Agent (Hybrid Mode) |
| `cathedral-core` | 1.0.0 | Facade de integração |

## Como Executar

### Pré-requisitos
- Rust 1.80+
- Node.js 20+
- Ollama (com modelo `llama3.1:8b`)
- SQLite (embutido, não requer instalação)

```text
# Instalar Ollama
curl -fsSL https://ollama.com/install.sh
ollama pull llama3.1:8b

# Build
cargo build --release

# Executar Edge Agent
just run-edge

# Executar UI
just run-ui
```

## Métricas Prometheus

| Métrica | Descrição |
|---------|-----------|
| `scheduler_workers_total` | Total de workers registrados |
| `scheduler_workers_by_tier` | Workers por tier |
| `scheduler_avg_reputation` | Reputação média |
| `scheduler_tasks_scheduled_total` | Tarefas agendadas |
| `scheduler_tasks_failed_total` | Tarefas com falha |
| `scheduler_estimated_cost_usd` | Custo estimado por tarefa |

## Testes

```text
cargo test --all
```

## Licença

MIT

**Arquiteto ORCID:** 0009-0005-2697-4668
