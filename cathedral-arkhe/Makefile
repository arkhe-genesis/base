# Makefile
.PHONY: help build test bench clean deploy up down logs bridge agent observer

help:
	@echo "🏛️ Cathedral ARKHE — Comandos"
	@echo "  build         Compila todos os crates"
	@echo "  test          Executa todos os testes"
	@echo "  bench         Executa benchmarks"
	@echo "  clean         Limpa artefatos"
	@echo "  deploy        Implanta com Docker Compose"
	@echo "  up            Inicia serviços Docker"
	@echo "  down          Para serviços Docker"
	@echo "  logs          Mostra logs dos serviços"
	@echo "  bridge        Executa o bridge localmente"
	@echo "  agent         Executa o prometheus-agent localmente"
	@echo "  observer      Executa o observer-5d localmente"

build:
	cargo build --workspace --release

test:
	cargo test --workspace -- --nocapture

bench:
	cargo bench --workspace

clean:
	cargo clean
	rm -rf data/
	rm -rf target/

deploy:
	chmod +x scripts/deploy.sh
	./scripts/deploy.sh

up:
	docker-compose up -d

down:
	docker-compose down

logs:
	docker-compose logs -f

bridge:
	cargo run --bin cathedral-bridge -- --addr 0.0.0.0:9002 --data-dir ./data/bridge

agent:
	cargo run --bin prometheus-agent -- --bridge-endpoint http://localhost:9002 --interval-secs 10

observer:
	cargo run --bin observer-5d -- --bridge-endpoint http://localhost:9002
