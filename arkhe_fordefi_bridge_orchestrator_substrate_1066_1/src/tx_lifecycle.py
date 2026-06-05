#!/usr/bin/env python3
"""
tx_lifecycle.py — Ciclo de vida de transações Fordefi v1.0.0.
Criação → Simulação Semântica → Assinatura MPC → Broadcast → Monitoramento.

Deidades: Hermes Trismegisto (mensageiro), Atena (verificação)
Selo: FORDEFI-BRIDGE-1066.1-v1.0.0-2026-06-05
"""

import json
import os
import time
from typing import Dict, Optional, Any, List
from fordefi_client import FordefiClient

TX_REGISTRY = os.path.expanduser("~/.cathedral/fordefi_transactions.json")


class TransactionLifecycle:
    """Ciclo completo de vida de transações Fordefi com validação Axiarquia."""

    def __init__(self, client: Optional[FordefiClient] = None):
        self.client = client or FordefiClient()
        self._registry = self._load_registry()

    def _load_registry(self) -> Dict:
        if os.path.exists(TX_REGISTRY):
            with open(TX_REGISTRY, "r") as f:
                return json.load(f)
        return {"transactions": {}, "metadata": {"version": "1.0.0", "source": "1066.1"}}

    def _save_registry(self):
        os.makedirs(os.path.dirname(TX_REGISTRY), exist_ok=True)
        with open(TX_REGISTRY, "w") as f:
            json.dump(self._registry, f, indent=2)

    def create(self, vault_id: str, to: str, amount: str, chain: str, data: Optional[str] = None) -> Dict:
        """Cria transação Fordefi."""
        details = {
            "type": "evm_raw_transaction" if "ethereum" in chain.lower() else "raw_transaction",
            "chain": chain,
            "to": to,
            "value": amount,
        }
        if data:
            details["data"] = data

        result = self.client.create_transaction(vault_id, "evm_transaction", details)

        if "error" in result:
            return result

        tx_id = result.get("id", f"tx_{int(time.time())}")

        self._registry["transactions"][tx_id] = {
            "vault_id": vault_id,
            "to": to,
            "amount": amount,
            "chain": chain,
            "status": "CREATED",
            "created_at": time.time(),
            "fordefi_data": result
        }
        self._save_registry()

        return {
            "tx_id": tx_id,
            "status": "CREATED",
            "vault_id": vault_id,
            "to": to,
            "amount": amount,
            "chain": chain,
            "message": "Transação criada. Próximo passo: simulate."
        }

    def simulate(self, tx_id: str) -> Dict:
        """Simula transação semanticamente (989.z.4)."""
        tx = self._registry["transactions"].get(tx_id, {})
        if not tx:
            return {"error": "Transação não encontrada", "status": "FAILED"}

        result = self.client.simulate_transaction(tx_id)

        risk_level = result.get("risk_level", "UNKNOWN")
        if risk_level == "HIGH":
            tx["status"] = "SIMULATION_FAILED"
            self._save_registry()
            return {
                "tx_id": tx_id,
                "status": "SIMULATION_FAILED",
                "risk_level": risk_level,
                "message": "[AXIARQUIA-954] Risco ALTO detectado. Transação BLOQUEADA.",
                "details": result.get("risk_details", {})
            }

        tx["status"] = "SIMULATED"
        tx["simulation"] = result
        self._save_registry()

        return {
            "tx_id": tx_id,
            "status": "SIMULATED",
            "risk_level": risk_level,
            "gas_estimate": result.get("gas_estimate", "N/A"),
            "message": "Simulação aprovada. Próximo passo: sign."
        }

    def sign(self, tx_id: str) -> Dict:
        """Assina transação via MPC em hardware enclave."""
        tx = self._registry["transactions"].get(tx_id, {})
        if tx.get("status") != "SIMULATED":
            return {"error": "Transação não simulada", "status": "FAILED"}

        result = self.client.submit_transaction(tx_id)

        if "error" in result:
            return result

        tx["status"] = "SIGNED"
        tx["signed_at"] = time.time()
        self._save_registry()

        return {
            "tx_id": tx_id,
            "status": "SIGNED",
            "message": "Transação assinada via MPC. Próximo passo: submit."
        }

    def submit(self, tx_id: str) -> Dict:
        """Broadcast da transação para a rede."""
        tx = self._registry["transactions"].get(tx_id, {})
        if tx.get("status") != "SIGNED":
            return {"error": "Transação não assinada", "status": "FAILED"}

        result = self.client.submit_transaction(tx_id)

        if "error" in result:
            return result

        tx["status"] = "SUBMITTED"
        tx["tx_hash"] = result.get("tx_hash", "unknown")
        tx["submitted_at"] = time.time()
        self._save_registry()

        return {
            "tx_id": tx_id,
            "status": "SUBMITTED",
            "tx_hash": tx["tx_hash"],
            "message": "Transação broadcasted. Use 'watch' para monitorar."
        }

    def watch(self, tx_id: str, timeout: int = 300) -> Dict:
        """Monitora confirmação da transação."""
        tx = self._registry["transactions"].get(tx_id, {})
        if not tx:
            return {"error": "Transação não encontrada", "status": "FAILED"}

        start = time.time()
        while time.time() - start < timeout:
            result = self.client.get_transaction(tx_id)
            status = result.get("status", "PENDING")

            if status in ("CONFIRMED", "FAILED", "REVERTED"):
                tx["status"] = status
                tx["confirmed_at"] = time.time() if status == "CONFIRMED" else None
                self._save_registry()
                return {
                    "tx_id": tx_id,
                    "status": status,
                    "tx_hash": tx.get("tx_hash", "unknown"),
                    "block_number": result.get("block_number", "N/A"),
                    "gas_used": result.get("gas_used", "N/A"),
                    "message": f"Transação {status}."
                }

            time.sleep(5)

        return {
            "tx_id": tx_id,
            "status": "TIMEOUT",
            "message": f"Timeout após {timeout}s. Verifique manualmente."
        }

    def get_history(self, vault_id: Optional[str] = None) -> List[Dict]:
        """Histórico de transações com ZK-proofs."""
        txs = self._registry.get("transactions", {})
        result = []
        for tx_id, data in txs.items():
            if vault_id and data.get("vault_id") != vault_id:
                continue
            result.append({
                "tx_id": tx_id,
                "vault_id": data.get("vault_id"),
                "to": data.get("to"),
                "amount": data.get("amount"),
                "chain": data.get("chain"),
                "status": data.get("status"),
                "tx_hash": data.get("tx_hash", "N/A"),
                "zk_proof": data.get("zk_proof", "PENDING")
            })
        return result


def main():
    import sys
    lifecycle = TransactionLifecycle()

    if len(sys.argv) < 2:
        print("Uso: python -m tx_lifecycle <command> [args]")
        print("Comandos: create, simulate, sign, submit, watch, history")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "create" and len(sys.argv) >= 6:
        result = lifecycle.create(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5])
        print(json.dumps(result, indent=2))
    elif cmd == "simulate" and len(sys.argv) > 2:
        result = lifecycle.simulate(sys.argv[2])
        print(json.dumps(result, indent=2))
    elif cmd == "sign" and len(sys.argv) > 2:
        result = lifecycle.sign(sys.argv[2])
        print(json.dumps(result, indent=2))
    elif cmd == "submit" and len(sys.argv) > 2:
        result = lifecycle.submit(sys.argv[2])
        print(json.dumps(result, indent=2))
    elif cmd == "watch" and len(sys.argv) > 2:
        result = lifecycle.watch(sys.argv[2])
        print(json.dumps(result, indent=2))
    elif cmd == "history":
        vault = sys.argv[2] if len(sys.argv) > 2 else None
        result = lifecycle.get_history(vault)
        print(json.dumps(result, indent=2))
    else:
        print(f"Comando não reconhecido: {cmd}")


if __name__ == "__main__":
    main()
