use std::sync::Arc;

use cathedral_episodic::EpisodicSync;
use tracing::info;

use crate::{
    ethics::EthicsVerifier,
    llm_client::OllamaClient,
    mcts::MCTSEngine,
    meta_cognitive::{MetaCognitiveLoop, MetaState},
    world_model::{WorldModel, WorldState},
    wormhole::HierarchicalWormhole,
};

pub struct AGICore {
    world_model: WorldModel,
    mcts: MCTSEngine,
    meta_loop: MetaCognitiveLoop,
    wormhole: HierarchicalWormhole,
    ethics: EthicsVerifier,
    llm: Arc<OllamaClient>,
}

impl AGICore {
    pub fn new(llm: Arc<OllamaClient>, episodic_memory: Option<Arc<EpisodicSync>>) -> Self {
        let mut meta_loop = MetaCognitiveLoop::new(5);
        if let Some(mem) = episodic_memory {
            meta_loop = meta_loop.with_memory(mem);
        }

        let world_model = WorldModel::new(llm.clone(), 128, 5);

        Self {
            world_model,
            mcts: MCTSEngine::new(llm.clone(), 1.4, 50, 5),
            meta_loop,
            wormhole: HierarchicalWormhole::new(3),
            ethics: EthicsVerifier::new(0.7),
            llm,
        }
    }

    pub async fn process(&mut self, user_input: &str) -> anyhow::Result<String> {
        info!("🤖 AGI Core processing: {}", user_input);

        // 1. World Model com inferência real
        let (base_output, world_state) = self.world_model.step(user_input).await?;

        // 2. Compressão hierárquica
        let abstraction = self.wormhole.compress(&base_output);

        // 3. MCTS com LLM para geração de ações
        let mcts_result = self.mcts.search(&world_state, &abstraction).await;

        // 4. Verificação ética
        let ethics_result = self.ethics.verify(&mcts_result);

        // 5. Meta-cognição com memória episódica
        self.meta_loop.update(&mcts_result, &ethics_result, user_input).await;

        // 6. Construção da resposta final usando o LLM
        let final_prompt = format!(
            "You are an AI assistant. Based on the following reasoning, provide a final answer to the user.\n\nUser input: {}\n\nReasoning steps: {:?}\n\nEthics passed: {}\nConfidence: {:.2}\n\nFinal answer:",
            user_input,
            mcts_result.action_history,
            ethics_result.passed,
            self.meta_loop.current_state().confidence
        );

        let final_output = self.llm.generate(&final_prompt, 512, 0.3).await?;

        // 7. Resultado consolidado
        let result = format!(
            "{}\n\n---\n[Abstraction: {}]\n[MCTS nodes: {}]\n[Ethics: {}]\n[Confidence: {:.2}]\n[Memories retrieved: {}]",
            final_output,
            abstraction,
            mcts_result.nodes_explored,
            if ethics_result.passed { "✅" } else { "❌" },
            self.meta_loop.current_state().confidence,
            self.meta_loop.current_state().memory_retrieved_count
        );

        info!("✅ AGI Core completed processing");

        Ok(result)
    }

    pub fn meta_state(&self) -> &MetaState {
        self.meta_loop.current_state()
    }

    pub fn world_state(&self) -> Option<WorldState> {
        self.world_model.current_state()
    }

    pub fn reset(&mut self) {
        self.world_model = WorldModel::new(self.llm.clone(), 128, 5);
        self.meta_loop = MetaCognitiveLoop::new(5);
        // Não resetar a memória episódica
    }
}
