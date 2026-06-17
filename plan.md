1. Add `SubagentSpawner`, `ContextManager`, `BlockchainNervousSystem` to `MCPServerState` in `src/mcp/server.rs`.
2. Ensure imports are correct in `src/mcp/server.rs`.
3. Add tool definitions and call handlers to `src/mcp/server.rs` using the dummy handlers we created.
4. Update `start_mcp_server` to accept the new fields.
5. Create `src/orchestrator/sandbox.rs` with `Sandbox`, `WasmSandbox`, `ProcessSandbox` implementation as requested.
6. Create `src/orchestrator/subagent_persistence.rs` with `SubagentState` and implementation.
7. Update `cathedral-agent/agent.py` to add new handlers for subagent tools.
8. Add Pre-commit check step.
