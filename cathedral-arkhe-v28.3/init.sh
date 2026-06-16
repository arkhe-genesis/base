#!/bin/bash

set -e

echo "Inicializando a stack do Cathedral ARKHE v28.3..."

# 1. Verificar dependências
if ! command -v docker-compose &> /dev/null
then
    echo "Erro: docker-compose não encontrado. Instale o Docker e o docker-compose."
    exit 1
fi

if ! command -v cargo &> /dev/null
then
    echo "Erro: cargo (Rust) não encontrado."
    exit 1
fi

# 2. Iniciar os serviços base
echo "Iniciando serviços (banco de dados, Redis, VectorDB, Telemetry)..."
cd runtime
docker-compose up -d temporal-chain vector-db redis jaeger

# 3. Compilar o agente orquestrador
echo "Compilando o orquestrador (MultiAgentOrchestrator)..."
cd ..
cargo build --release --manifest-path orchestrator/Cargo.toml

echo "Stack iniciada com sucesso!"
echo "Para testar, use 'docker-compose logs -f' dentro de 'runtime'."
