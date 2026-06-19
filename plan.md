1. **Understand the problem**:
    The user wants to integrate components into `src/substrato_5002` and `src/substrato_9000`. These include `compensation_prompt_integration.rs`, `meta_controller_v2_3.rs`, and `thompson_bandit.rs` for `substrato_5002`, and `cognitive_router_integration.rs` for `substrato_9000`.

2. **Actions Taken**:
    - Created directories `src/substrato_5002` and `src/substrato_9000`.
    - Created the corresponding Rust files with the provided code.
    - Updated `src/lib.rs` to expose the newly added modules.
    - Verified compilation by running `cd orchestrator && cargo check` and `cd orchestrator && cargo test --no-run`. Both succeed. Note that `cargo check` at the root folder times out but testing the `orchestrator` crate works and is part of the workspace.

3. **Next Steps**:
    - Complete pre-commit instructions.
    - Submit the changes.
