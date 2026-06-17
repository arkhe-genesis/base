// orchestrator/src/lib.rs

#[cfg(feature = "deployment-sim")]
pub mod geometry;

#[cfg(feature = "deployment-sim")]
pub mod simulation;

#[cfg(feature = "deployment-sim")]
pub mod cuda {
    pub mod geometric_reward_model;
}

#[cfg(feature = "deployment-sim")]
pub mod governance {
    pub mod geometric_policy_engine;
}

#[cfg(feature = "deployment-sim")]
pub mod integration {
    pub mod hpe_simulation_adapter;
    pub mod hpe_geometry_adapter;
}
