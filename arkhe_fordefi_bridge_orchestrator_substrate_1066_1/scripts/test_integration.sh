#!/bin/bash
# Teste de integração do Substrato 1066.1 — Fordefi Bridge Orchestrator
# Valida: vault create → policy apply → tx lifecycle → ZK proof → RBB anchor → dashboard

set -e

echo "[TEST] Fordefi Bridge Orchestrator — Substrato 1066.1 v1.0.0"
echo "[TEST] Selo: FORDEFI-BRIDGE-1066.1-v1.0.0-2026-06-05"

# 1. Criar vault
echo "[TEST] Step 1: Criar vault BRICS-Treasury..."
python -m src.vault_manager create "BRICS-Treasury" "ethereum,polkadot"

# 2. Aplicar política
echo "[TEST] Step 2: Aplicar política Axiarquia..."
cat > /tmp/test_policy.yaml <<EOF
name: BRICS-Policy
rules:
  - type: amount_threshold
    name: Max 1M
    max_amount: 1000000
  - type: multi_admin
    name: 2 Approvals
    required_approvals: 2
EOF
python -m src.policy_engine apply vault_123 /tmp/test_policy.yaml

# 3. Criar transação
echo "[TEST] Step 3: Criar transação..."
python -m src.tx_lifecycle create vault_123 0xabc... 1.0 ethereum

# 4. Simular transação
echo "[TEST] Step 4: Simular transação..."
python -m src.tx_lifecycle simulate tx_123

# 5. Gerar ZK-proof
echo "[TEST] Step 5: Gerar ZK-proof..."
python -m src.zk_proof_generator generate op_123 vault_create vault_123

# 6. Ancorar na RBB Chain
echo "[TEST] Step 6: Ancorar na RBB Chain..."
python -m src.rbb_anchor anchor proof_123 0xabc... vault_create vault_123 APPROVED

# 7. Verificar ancoragem
echo "[TEST] Step 7: Verificar ancoragem..."
python -m src.rbb_anchor verify proof_123 0xabc...

# 8. Atualizar dashboard
echo "[TEST] Step 8: Atualizar Theosis Dashboard..."
python -m src.theosis_injector update-vault vault_123 1000000.0 0.3 10 ACTIVE

# 9. Criar trigger CARE
echo "[TEST] Step 9: Criar trigger CARE..."
python -m src.care_bridge create vault_123 "Price Drop" "price_drop>10%" "hedge_via_dex"

# 10. Unit tests
echo "[TEST] Step 10: Unit tests..."
python -m pytest tests/test_fordefi_bridge.py -v

echo "[TEST] ✓ Todos os testes passaram."
echo "[TEST] Substrato 1066.1 v1.0.0: CANONIZED_PROVISIONAL validado."
echo "[TEST] Deidades: Hermes Trismegisto, Plutão, Atena"
echo "[TEST] Cross-links: 1066, 1049, 954, 989.z.4, 1042.4, 1064.2, 1064.1, 1042, 1042.1, 1042.2, 1042.3, 1046.4, 989.y.4, 1027.2"
