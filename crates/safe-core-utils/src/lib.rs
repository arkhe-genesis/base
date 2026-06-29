
pub mod cgf_metrics;
pub mod cgf_orchestrator;
pub mod cgf_prompt_selector;

#[cfg(feature = "anti-vibe")]
pub mod anti_vibe;

pub use cgf_metrics::{
    CgfEngine, CgfReportX, SessionReport, EpistemicLevel,
    SAFE_CORE_CONCEPTS, CONCEPT_WEIGHTS,
};
pub use cgf_orchestrator::{
    CgfOrchestrator, CgfOrchestratorConfig, CgfRoundResult,
    CgfOrchestratorError, LlmModel,
};
pub use cgf_prompt_selector::{PromptSelector, PromptDepth};

#[cfg(feature = "anti-vibe")]
pub use anti_vibe::{
    VibeFailScenario, VIBE_FAILS, ANTI_VIBE_KEYWORDS,
    x_detect_vibe_awareness, find_relevant_scenario,
    generate_anti_vibe_prompt,
};
