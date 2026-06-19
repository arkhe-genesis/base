# ============================================================
# Cathedral ARKHE — AGI-GRADE v3.0.0
# Comandos de build, teste e execução
# ============================================================

# --- Build ---
build:
	cargo build --release

build-core:
	cargo build --release -p cathedral-core

build-agi:
	cargo build --release -p cathedral-agi

build-edge:
	cargo build --release -p cathedral-edge-agent

build-all:
	cargo build --release --workspace

# --- Run ---
run-edge:
	RUST_LOG=info cargo run --release -p cathedral-edge-agent

run-agi:
	cargo run --release -p cathedral-agi

run-ui:
	cd ui && npm install && npm start &

# --- Test ---
test:
	cargo test --all

test-agi:
	cargo test -p cathedral-agi

test-scheduler:
	cargo test -p cathedral-scheduler

test-episodic:
	cargo test -p cathedral-episodic

test-fallback:
	cargo test -p cathedral-fallback

test-tee:
	cargo test -p cathedral-tee

# --- Training ---
train-sft:
	python3 training/orchestrator.py --stage sft

train-dpo:
	python3 training/orchestrator.py --stage dpo

train-grpo:
	python3 training/orchestrator.py --stage grpo

eval:
	python3 training/run_eval.py --model model.gguf --holdout data/eval/holdout.jsonl

# --- Clean ---
clean:
	cargo clean
	rm -rf ui/.next
	rm -rf data/episodic.db

# --- Utils ---
check:
	cargo check --workspace

fmt:
	cargo fmt --all

lint:
	cargo clippy --all -- -D warnings
