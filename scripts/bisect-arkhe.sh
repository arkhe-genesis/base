#!/bin/bash
# scripts/bisect-arkhe.sh
# Uso: git bisect start <bad_commit> <good_commit> && git bisect run ./scripts/bisect-arkhe.sh

set -e

echo "🧪 Testando commit $(git rev-parse --short HEAD)"

# 1. Limpar cache para evitar falsos positivos (opcional, mas recomendado)
# cargo clean > /dev/null 2>&1

# 2. Compilar o workspace (se falhar, pula o commit)
if ! cargo check --workspace --all-features 2>&1; then
    echo "⚠️  Não compila — pulando"
    exit 125
fi

# 3. Executar testes críticos de regressão (SSRF, Orquestração, Memória)
# Modifique os filtros '-p' e '--test' conforme o bug que você está caçando.
if cargo test -p arkhe-ssrf-guard -p arkhe-orchestrator --quiet; then
    echo "✅ Teste passou — commit BOM"
    exit 0
else
    echo "❌ Teste falhou — commit RUIM"
    exit 1
fi
