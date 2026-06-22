# Cathedral-LLM — Makefile
# Arquiteto ORCID 0009-0005-2697-4668

.PHONY: all build test lint fmt check clean install dev docs bench release docker

# Variáveis
RUSTFLAGS ?= "-C target-cpu=native"
CARGO := cargo
DOCKER := docker
PYTHON := python3

# Default target
all: fmt lint test build

# === BUILD ===

build:
	@echo "🏛️  Building Cathedral-LLM workspace..."
	$(CARGO) build --workspace

build-release:
	@echo "🏛️  Building Cathedral-LLM (release)..."
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) build --workspace --release

build-core:
	@echo "🏛️  Building cathedral-llm-core..."
	$(CARGO) build -p cathedral-llm-core --release

build-runtime:
	@echo "🏛️  Building cathedral-inference-runtime..."
	$(CARGO) build -p cathedral-inference-runtime --release

build-api:
	@echo "🏛️  Building cathedral-api..."
	$(CARGO) build -p cathedral-api --release

build-cli:
	@echo "🏛️  Building cathedral-cli..."
	$(CARGO) build -p cathedral-cli --release

# === TEST ===

test:
	@echo "🏛️  Running all tests..."
	$(CARGO) test --workspace

test-core:
	@echo "🏛️  Testing cathedral-llm-core..."
	$(CARGO) test -p cathedral-llm-core

test-runtime:
	@echo "🏛️  Testing cathedral-inference-runtime..."
	$(CARGO) test -p cathedral-inference-runtime

test-e2e:
	@echo "🏛️  Running end-to-end tests..."
	$(CARGO) test -p cathedral-tests -- --test-threads=1 --nocapture

test-zk:
	@echo "🏛️  Testing ZK proofs..."
	$(CARGO) test -p cathedral-zk

test-identity:
	@echo "🏛️  Testing identity verification..."
	$(CARGO) test -p cathedral-identity

# === LINT ===

lint:
	@echo "🏛️  Running clippy..."
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

fmt:
	@echo "🏛️  Formatting code..."
	$(CARGO) fmt --all

check:
	@echo "🏛️  Running cargo check..."
	$(CARGO) check --workspace

deny:
	@echo "🏛️  Running cargo deny..."
	cargo deny check

# === DOCUMENTATION ===

docs:
	@echo "🏛️  Building documentation..."
	$(CARGO) doc --workspace --no-deps --open

docs-private:
	@echo "🏛️  Building documentation (private items)..."
	$(CARGO) doc --workspace --document-private-items

# === BENCHMARKS ===

bench:
	@echo "🏛️  Running benchmarks..."
	$(CARGO) bench -p cathedral-benchmarks

bench-inference:
	@echo "🏛️  Benchmarking inference..."
	$(CARGO) bench -p cathedral-benchmarks -- inference

bench-zk:
	@echo "🏛️  Benchmarking ZK proofs..."
	$(CARGO) bench -p cathedral-benchmarks -- zk

# === DEVELOPMENT ===

dev:
	@echo "🏛️  Starting development environment..."
	docker-compose -f docker/docker-compose.dev.yml up -d

setup:
	@echo "🏛️  Setting up development environment..."
	@bash scripts/setup-dev.sh

bootstrap:
	@echo "🏛️  Bootstrapping Cathedral-LLM..."
	@bash scripts/bootstrap.sh

# === DOCKER ===

docker-build:
	@echo "🏛️  Building Docker images..."
	$(DOCKER) build -f docker/Dockerfile.base -t cathedral-llm:base .
	$(DOCKER) build -f docker/Dockerfile.cpu -t cathedral-llm:cpu .

docker-run:
	@echo "🏛️  Running Cathedral-LLM in Docker..."
	docker-compose -f docker/docker-compose.yml up -d

docker-stop:
	@echo "🏛️  Stopping Cathedral-LLM containers..."
	docker-compose -f docker/docker-compose.yml down

# === MODELS ===

model-download:
	@echo "🏛️  Downloading model checkpoints..."
	$(PYTHON) models/huggingface/download.py

model-convert:
	@echo "🏛️  Converting model format..."
	$(PYTHON) tools/model_converter/convert_from_hf.py

model-quantize:
	@echo "🏛️  Quantizing model..."
	$(PYTHON) tools/quantizer/quantize.py

# === DATA ===

data-generate:
	@echo "🏛️  Generating synthetic training data..."
	$(PYTHON) scripts/generate-identity-data.py
	$(PYTHON) data/synthetic/generate_reasoning_data.py
	$(PYTHON) data/synthetic/generate_ethical_data.py

# === DEPLOYMENT ===

deploy:
	@echo "🏛️  Deploying Cathedral-LLM..."
	@bash scripts/deploy-model.sh

release:
	@echo "🏛️  Creating release..."
	$(CARGO) build --workspace --release
	@bash scripts/package-release.sh

# === UTILITIES ===

clean:
	@echo "🏛️  Cleaning build artifacts..."
	$(CARGO) clean
	@rm -rf target/
	@find . -name "*.rs.bk" -delete

install:
	@echo "🏛️  Installing Cathedral-LLM CLI..."
	$(CARGO) install --path crates/cathedral-cli

update:
	@echo "🏛️  Updating dependencies..."
	$(CARGO) update

verify-zk:
	@echo "🏛️  Verifying ZK proofs..."
	$(PYTHON) scripts/verify-zk-proofs.py

# === HELP ===

help:
	@echo "🏛️  Cathedral-LLM — Available targets:"
	@echo ""
	@echo "  Build:"
	@echo "    build, build-release, build-core, build-runtime, build-api, build-cli"
	@echo ""
	@echo "  Test:"
	@echo "    test, test-core, test-runtime, test-e2e, test-zk, test-identity"
	@echo ""
	@echo "  Lint:"
	@echo "    lint, fmt, check, deny"
	@echo ""
	@echo "  Docs:"
	@echo "    docs, docs-private"
	@echo ""
	@echo "  Benchmarks:"
	@echo "    bench, bench-inference, bench-zk"
	@echo ""
	@echo "  Development:"
	@echo "    dev, setup, bootstrap"
	@echo ""
	@echo "  Docker:"
	@echo "    docker-build, docker-run, docker-stop"
	@echo ""
	@echo "  Models:"
	@echo "    model-download, model-convert, model-quantize"
	@echo ""
	@echo "  Data:"
	@echo "    data-generate"
	@echo ""
	@echo "  Deployment:"
	@echo "    deploy, release"
	@echo ""
	@echo "  Utilities:"
	@echo "    clean, install, update, verify-zk, help"
