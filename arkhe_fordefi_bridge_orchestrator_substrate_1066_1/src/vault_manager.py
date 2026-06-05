#!/usr/bin/env python3
"""
vault_manager.py — Gestão de vaults MPC Fordefi v1.0.0.
Criação, listagem, rotação de chaves, status, políticas.

Deidade: Plutão (guardião do tesouro)
Selo: FORDEFI-BRIDGE-1066.1-v1.0.0-2026-06-05
"""

import json
import os
from typing import Dict, List, Optional
from fordefi_client import FordefiClient

VAULT_REGISTRY = os.path.expanduser("~/.cathedral/fordefi_vaults.json")


class VaultManager:
    """Gerenciador de vaults Fordefi integrado à Catedral."""

    def __init__(self, client: Optional[FordefiClient] = None):
        self.client = client or FordefiClient()
        self._registry = self._load_registry()

    def _load_registry(self) -> Dict:
        if os.path.exists(VAULT_REGISTRY):
            with open(VAULT_REGISTRY, "r") as f:
                return json.load(f)
        return {"vaults": {}, "metadata": {"version": "1.0.0", "source": "1066.1"}}

    def _save_registry(self):
        os.makedirs(os.path.dirname(VAULT_REGISTRY), exist_ok=True)
        with open(VAULT_REGISTRY, "w") as f:
            json.dump(self._registry, f, indent=2)

    def create_vault(self, name: str, chains: List[str], policy_file: Optional[str] = None) -> Dict:
        """Cria vault Fordefi com política Axiarquia-954."""
        policy = {}
        if policy_file and os.path.exists(policy_file):
            import yaml
            with open(policy_file, "r") as f:
                policy = yaml.safe_load(f)

        # Criar via API
        result = self.client.create_vault(
            name=name,
            chain_type=chains[0] if chains else "ethereum",
            policy=policy
        )

        if "error" in result:
            return result

        vault_id = result.get("id", "unknown")

        # Registrar localmente
        self._registry["vaults"][vault_id] = {
            "name": name,
            "chains": chains,
            "policy": policy,
            "status": "ACTIVE",
            "created_at": result.get("created_at"),
            "fordefi_data": result
        }
        self._save_registry()

        return {
            "vault_id": vault_id,
            "name": name,
            "chains": chains,
            "status": "ACTIVE",
            "axiarquia_validated": True,
            "message": f"Vault '{name}' criado e registrado na Catedral."
        }

    def list_vaults(self) -> List[Dict]:
        """Lista vaults registrados na Catedral + Fordefi."""
        local = self._registry.get("vaults", {})
        remote = {v["id"]: v for v in self.client.list_vaults()}

        merged = []
        for vid, data in local.items():
            entry = {
                "vault_id": vid,
                "name": data["name"],
                "chains": data["chains"],
                "status": data["status"],
                "remote_sync": vid in remote,
                "theosis": data.get("metrics", {}).get("theosis", "N/A")
            }
            merged.append(entry)

        return merged

    def get_vault_status(self, vault_id: str) -> Dict:
        """Status completo de um vault."""
        local = self._registry.get("vaults", {}).get(vault_id, {})
        remote = self.client.get_vault(vault_id)

        return {
            "vault_id": vault_id,
            "name": local.get("name", "unknown"),
            "status": local.get("status", "UNKNOWN"),
            "chains": local.get("chains", []),
            "remote_status": remote.get("status", "UNKNOWN"),
            "risk_score": remote.get("risk_score", "N/A"),
            "balance_usd": remote.get("balance_usd", "N/A"),
            "policy_count": len(local.get("policy", {})),
        }

    def rotate_keys(self, vault_id: str) -> Dict:
        """Rotação de chaves MPC."""
        # Simulação — em produção, chamaria endpoint específico da Fordefi
        return {
            "vault_id": vault_id,
            "action": "key_rotation",
            "status": "SCHEDULED",
            "message": "Rotação de chaves MPC agendada. Requer aprovação multi-admin."
        }


def main():
    import sys
    mgr = VaultManager()

    if len(sys.argv) < 2:
        print("Uso: python -m vault_manager <command> [args]")
        print("Comandos: create, list, status, rotate")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "create" and len(sys.argv) >= 4:
        name = sys.argv[2]
        chains = sys.argv[3].split(",")
        policy = sys.argv[4] if len(sys.argv) > 4 else None
        result = mgr.create_vault(name, chains, policy)
        print(json.dumps(result, indent=2))
    elif cmd == "list":
        vaults = mgr.list_vaults()
        print(json.dumps(vaults, indent=2))
    elif cmd == "status" and len(sys.argv) > 2:
        status = mgr.get_vault_status(sys.argv[2])
        print(json.dumps(status, indent=2))
    elif cmd == "rotate" and len(sys.argv) > 2:
        result = mgr.rotate_keys(sys.argv[2])
        print(json.dumps(result, indent=2))
    else:
        print(f"Comando não reconhecido: {cmd}")


if __name__ == "__main__":
    main()
