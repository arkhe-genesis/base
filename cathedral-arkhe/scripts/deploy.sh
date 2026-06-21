#!/bin/bash
# scripts/deploy.sh — Deploy completo do Cathedral ARKHE

set -e

echo "🏛️ Cathedral ARKHE — Deploy"

mkdir -p data/{bridge,postgres,prometheus,grafana}
mkdir -p config/grafana/{dashboards,datasources}

command -v docker >/dev/null 2>&1 || { echo "Docker não encontrado"; return 1; }
command -v docker-compose >/dev/null 2>&1 || { echo "docker-compose não encontrado"; return 1; }

echo "📦 Construindo imagens Docker..."
docker-compose build

echo "🚀 Iniciando serviços..."
docker-compose up -d

echo "⏳ Aguardando serviços ficarem saudáveis..."
sleep 10

docker-compose ps

echo "✅ Cathedral ARKHE implantado com sucesso!"
echo "   Bridge:   http://localhost:9002"
echo "   Grafana:  http://localhost:3000 (admin/cathedral)"
echo "   Prometheus: http://localhost:9090"
