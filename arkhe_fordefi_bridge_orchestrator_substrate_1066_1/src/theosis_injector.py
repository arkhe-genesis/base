#!/usr/bin/env python3
"""
theosis_injector.py — Injeção de métricas Fordefi no Theosis-Paris Dashboard (1064.2) v1.0.0.
Conecta operações Fordefi ao dashboard de fadiga arquitetural da Catedral.

Deidade: Atena (visão panorâmica), Nemesis (equilíbrio de riscos)
Selo: FORDEFI-BRIDGE-1066.1-v1.0.0-2026-06-05
"""

import json
import os
import time
from typing import Dict, Optional, List

METRICS_REGISTRY = os.path.expanduser("~/.cathedral/fordefi_metrics.json")


class TheosisInjector:
    """Injeta métricas Fordefi no Dashboard Theosis-Paris."""

    def __init__(self):
        self._metrics = self._load_metrics()

    def _load_metrics(self) -> Dict:
        if os.path.exists(METRICS_REGISTRY):
            with open(METRICS_REGISTRY, "r") as f:
                return json.load(f)
        return {
            "vaults": {},
            "transactions": {},
            "risk_scores": {},
            "global": {
                "total_volume_usd": 0.0,
                "active_vaults": 0,
                "pending_transactions": 0,
                "average_risk_score": 0.0,
                "last_update": time.time()
            }
        }

    def _save_metrics(self):
        os.makedirs(os.path.dirname(METRICS_REGISTRY), exist_ok=True)
        with open(METRICS_REGISTRY, "w") as f:
            json.dump(self._metrics, f, indent=2)

    def update_vault_metrics(self, vault_id: str, balance_usd: float, risk_score: float,
                            tx_count: int, status: str) -> Dict:
        """Atualiza métricas de vault."""
        self._metrics["vaults"][vault_id] = {
            "balance_usd": balance_usd,
            "risk_score": risk_score,
            "tx_count": tx_count,
            "status": status,
            "last_update": time.time()
        }

        # Recalcular globais
        vaults = self._metrics["vaults"]
        self._metrics["global"]["active_vaults"] = len([v for v in vaults.values() if v["status"] == "ACTIVE"])
        self._metrics["global"]["total_volume_usd"] = sum(v["balance_usd"] for v in vaults.values())
        if vaults:
            self._metrics["global"]["average_risk_score"] = sum(v["risk_score"] for v in vaults.values()) / len(vaults)
        self._metrics["global"]["last_update"] = time.time()

        self._save_metrics()

        return {
            "vault_id": vault_id,
            "metrics_updated": True,
            "global_active_vaults": self._metrics["global"]["active_vaults"],
            "global_volume_usd": self._metrics["global"]["total_volume_usd"],
            "global_risk_score": self._metrics["global"]["average_risk_score"]
        }

    def update_transaction_metrics(self, tx_id: str, status: str, gas_used: Optional[int] = None,
                                   block_number: Optional[int] = None) -> Dict:
        """Atualiza métricas de transação."""
        self._metrics["transactions"][tx_id] = {
            "status": status,
            "gas_used": gas_used,
            "block_number": block_number,
            "timestamp": time.time()
        }

        self._metrics["global"]["pending_transactions"] = len(
            [t for t in self._metrics["transactions"].values() if t["status"] in ("CREATED", "SIGNED", "SUBMITTED")]
        )
        self._save_metrics()

        return {
            "tx_id": tx_id,
            "status": status,
            "pending_count": self._metrics["global"]["pending_transactions"]
        }

    def get_dashboard_data(self) -> Dict:
        """Retorna dados formatados para o Dashboard 1064.2."""
        return {
            "source": "Fordefi Bridge Orchestrator (1066.1)",
            "timestamp": time.time(),
            "global": self._metrics["global"],
            "vaults": self._metrics["vaults"],
            "alerts": self._generate_alerts()
        }

    def _generate_alerts(self) -> List[Dict]:
        """Gera alertas baseados em métricas."""
        alerts = []
        for vault_id, data in self._metrics["vaults"].items():
            if data["risk_score"] > 0.7:
                alerts.append({
                    "type": "HIGH_RISK",
                    "vault_id": vault_id,
                    "risk_score": data["risk_score"],
                    "message": f"Vault {vault_id} com risco ALTO ({data['risk_score']:.2f})"
                })
            if data["balance_usd"] > 1000000:
                alerts.append({
                    "type": "HIGH_VALUE",
                    "vault_id": vault_id,
                    "balance_usd": data["balance_usd"],
                    "message": f"Vault {vault_id} com saldo > $1M"
                })
        return alerts


def main():
    import sys
    injector = TheosisInjector()

    if len(sys.argv) < 2:
        print("Uso: python -m theosis_injector <command> [args]")
        print("Comandos: update-vault, update-tx, dashboard")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "update-vault" and len(sys.argv) >= 6:
        result = injector.update_vault_metrics(sys.argv[2], float(sys.argv[3]), float(sys.argv[4]), int(sys.argv[5]), "ACTIVE")
        print(json.dumps(result, indent=2))
    elif cmd == "update-tx" and len(sys.argv) >= 4:
        gas = int(sys.argv[4]) if len(sys.argv) > 4 else None
        block = int(sys.argv[5]) if len(sys.argv) > 5 else None
        result = injector.update_transaction_metrics(sys.argv[2], sys.argv[3], gas, block)
        print(json.dumps(result, indent=2))
    elif cmd == "dashboard":
        result = injector.get_dashboard_data()
        print(json.dumps(result, indent=2))
    else:
        print(f"Comando não reconhecido: {cmd}")


if __name__ == "__main__":
    main()
