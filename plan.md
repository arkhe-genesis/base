1. **Create Crates**:
   - Create `crates/safe-core-ethics/src/lib.rs` with the `lib.rs` content from the prompt, and remove the `mod` declarations in favor of inline modules or just flattening to compile. The prompt requires us to put the content into `crates/safe-core-ethics/src/lib.rs`.
   - Create `crates/safe-core-persistence/src/lib.rs` with the `lib.rs` content from the prompt.
   - Update `crates/safe-core-verifier/src/lib.rs` with the `lib.rs` content from the prompt.
   - Create `crates/safe-core-audit/src/lib.rs` with the `lib.rs` content from the prompt.
   - Create `crates/safe-core-governance/Cargo.toml` and its `src/lib.rs`, `src/governance.rs`, `src/mcp.rs`.
   - Create `crates/safe-core-bridge/Cargo.toml` and its `src/tools.rs`, `src/mcp.rs`, `src/main.rs`, `src/lib.rs`.
2. **Create Integrations**:
   - Create `docs/integration-wmux.md` and `docs/integration-dmux.md`.
   - Create `examples/mux-integration/src/main.rs`, `examples/mux-integration/wmux-config.toml`, `examples/mux-integration/dmux-hooks.sh`.
3. **Verify new files**:
   - Run `ls -la crates/safe-core-governance` to verify creation.
4. **Compile check**:
   - Run `cargo test --workspace` to ensure no regressions were introduced.
5. **Pre-commit**:
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
