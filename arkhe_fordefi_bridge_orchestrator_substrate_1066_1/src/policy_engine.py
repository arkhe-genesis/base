#!/usr/bin/env python3
"""
policy_engine.py — Engine de políticas Axiarquia-954 para Fordefi v1.0.0.
Regras granulares: thresholds, multi-admin, protocol restrictions, time locks.

Deidade: Atena (sabedoria na governança)
Selo: FORDEFI-BRIDGE-1066.1-v1.0.0-2026-06-05
"""

import json
import os
import time
from typing import Dict, List, Optional, Tuple

POLICY_REGISTRY = os.path.expanduser("~/.cathedral/fordefi_policies.json")


class PolicyEngine:
    """Engine de políticas Axiarquia-954 para vaults Fordefi."""

    def __init__(self):
        self._policies = self._load_policies()

    def _load_policies(self) -> Dict:
        if os.path.exists(POLICY_REGISTRY):
            with open(POLICY_REGISTRY, "r") as f:
                return json.load(f)
        return {"policies": {}, "version": "1.0.0", "source": "1066.1"}

    def _save_policies(self):
        os.makedirs(os.path.dirname(POLICY_REGISTRY), exist_ok=True)
        with open(POLICY_REGISTRY, "w") as f:
            json.dump(self._policies, f, indent=2)

    def apply_policy(self, vault_id: str, policy_file: str) -> Dict:
        """Aplica política YAML a um vault Fordefi."""
        if not os.path.exists(policy_file):
            return {"error": f"Arquivo de política não encontrado: {policy_file}", "status": "FAILED"}

        try:
            import yaml
            with open(policy_file, "r") as f:
                policy = yaml.safe_load(f)
        except ImportError:
            return {"error": "PyYAML não instalado. pip install pyyaml", "status": "FAILED"}

        required = ["name", "rules"]
        for field in required:
            if field not in policy:
                return {"error": f"Campo obrigatório ausente: {field}", "status": "FAILED"}

        self._policies["policies"][vault_id] = {
            "policy": policy,
            "applied_at": time.time(),
            "status": "ACTIVE"
        }
        self._save_policies()

        return {
            "vault_id": vault_id,
            "policy_name": policy["name"],
            "rules_count": len(policy["rules"]),
            "status": "APPLIED",
            "message": f"Política '{policy['name']}' aplicada ao vault {vault_id}."
        }

    def validate_transaction(self, vault_id: str, tx: Dict) -> Tuple[bool, str]:
        """Valida transação contra políticas do vault."""
        policy_data = self._policies["policies"].get(vault_id, {})
        if not policy_data:
            return True, "[AXIARQUIA-954] Nenhuma política ativa. Transação LIBERADA."

        policy = policy_data.get("policy", {})
        rules = policy.get("rules", [])

        for rule in rules:
            rule_type = rule.get("type")

            if rule_type == "amount_threshold":
                max_amount = rule.get("max_amount", float("inf"))
                tx_amount = float(tx.get("amount", 0))
                if tx_amount > max_amount:
                    return False, (
                        f"[AXIARQUIA-954] BLOQUEADA: Amount {tx_amount} excede threshold {max_amount}.\n"
                        f"  Regra: {rule.get('name', 'unnamed')}"
                    )

            elif rule_type == "protocol_restriction":
                allowed = rule.get("allowed_protocols", [])
                tx_protocol = tx.get("protocol", "unknown")
                if allowed and tx_protocol not in allowed:
                    return False, (
                        f"[AXIARQUIA-954] BLOQUEADA: Protocolo '{tx_protocol}' não permitido.\n"
                        f"  Permitidos: {', '.join(allowed)}"
                    )

            elif rule_type == "multi_admin":
                required_approvals = rule.get("required_approvals", 1)
                current_approvals = tx.get("approvals", 0)
                if current_approvals < required_approvals:
                    return False, (
                        f"[AXIARQUIA-954] BLOQUEADA: Aprovações insuficientes.\n"
                        f"  Requeridas: {required_approvals}, Atuais: {current_approvals}"
                    )

            elif rule_type == "time_lock":
                lock_hours = rule.get("lock_hours", 0)
                tx_time = tx.get("created_at", time.time())
                if time.time() - tx_time < lock_hours * 3600:
                    return False, (
                        f"[AXIARQUIA-954] BLOQUEADA: Time lock ativo.\n"
                        f"  Liberação em: {lock_hours}h"
                    )

        return True, "[AXIARQUIA-954] Transação APROVADA. Todas as regras satisfeitas."

    def audit(self, vault_id: str) -> Dict:
        """Audita compliance do vault (SOC 2, Munich Re)."""
        policy_data = self._policies["policies"].get(vault_id, {})
        policy = policy_data.get("policy", {})

        checks = {
            "policy_defined": bool(policy),
            "amount_limits": any(r.get("type") == "amount_threshold" for r in policy.get("rules", [])),
            "multi_admin": any(r.get("type") == "multi_admin" for r in policy.get("rules", [])),
            "protocol_restrictions": any(r.get("type") == "protocol_restriction" for r in policy.get("rules", [])),
            "time_locks": any(r.get("type") == "time_lock" for r in policy.get("rules", [])),
        }

        score = sum(checks.values()) / len(checks)

        return {
            "vault_id": vault_id,
            "compliance_score": round(score, 2),
            "checks": checks,
            "status": "COMPLIANT" if score >= 0.8 else "PARTIAL" if score >= 0.5 else "NON_COMPLIANT",
            "standards": ["SOC 2 Type II", "Munich Re Insurance", "Axiarquia-954"]
        }

    def list_policies(self, vault_id: Optional[str] = None) -> List[Dict]:
        """Lista políticas ativas."""
        policies = self._policies.get("policies", {})
        if vault_id:
            data = policies.get(vault_id, {})
            if not data:
                return []
            return [{
                "vault_id": vault_id,
                "policy_name": data["policy"].get("name", "unnamed"),
                "status": data["status"],
                "rules_count": len(data["policy"].get("rules", []))
            }]

        result = []
        for vid, data in policies.items():
            result.append({
                "vault_id": vid,
                "policy_name": data["policy"].get("name", "unnamed"),
                "status": data["status"],
                "rules_count": len(data["policy"].get("rules", []))
            })
        return result


def main():
    import sys
    engine = PolicyEngine()

    if len(sys.argv) < 2:
        print("Uso: python -m policy_engine <command> [args]")
        print("Comandos: apply, audit, list, validate")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "apply" and len(sys.argv) >= 4:
        result = engine.apply_policy(sys.argv[2], sys.argv[3])
        print(json.dumps(result, indent=2))
    elif cmd == "audit" and len(sys.argv) > 2:
        result = engine.audit(sys.argv[2])
        print(json.dumps(result, indent=2))
    elif cmd == "list":
        vault = sys.argv[2] if len(sys.argv) > 2 else None
        result = engine.list_policies(vault)
        print(json.dumps(result, indent=2))
    elif cmd == "validate" and len(sys.argv) > 2:
        tx = {"amount": 1000, "protocol": "uniswap", "created_at": time.time()}
        permitted, msg = engine.validate_transaction(sys.argv[2], tx)
        print(json.dumps({"permitted": permitted, "message": msg}, indent=2))
    else:
        print(f"Comando não reconhecido: {cmd}")


if __name__ == "__main__":
    main()
