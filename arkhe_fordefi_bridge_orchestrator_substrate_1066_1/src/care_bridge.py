#!/usr/bin/env python3
"""
care_bridge.py — Bridge para Continuous Automated Response Engine (CARE) da Fordefi v1.0.0.
Triggers e ações automatizadas baseadas em eventos on-chain.

Deidade: Plutão (riqueza automatizada), Hermes (mensageiro de eventos)
Selo: FORDEFI-BRIDGE-1066.1-v1.0.0-2026-06-05
"""

import json
import os
import time
from typing import Dict, List, Optional, Callable
from fordefi_client import FordefiClient

CARE_REGISTRY = os.path.expanduser("~/.cathedral/fordefi_care.json")


class CAREBridge:
    """Bridge para o CARE Engine da Fordefi."""

    def __init__(self, client: Optional[FordefiClient] = None):
        self.client = client or FordefiClient()
        self._triggers = self._load_triggers()
        self._callbacks: List[Callable[[Dict], None]] = []

    def _load_triggers(self) -> Dict:
        if os.path.exists(CARE_REGISTRY):
            with open(CARE_REGISTRY, "r") as f:
                return json.load(f)
        return {"triggers": {}, "version": "1.0.0", "source": "1066.1"}

    def _save_triggers(self):
        os.makedirs(os.path.dirname(CARE_REGISTRY), exist_ok=True)
        with open(CARE_REGISTRY, "w") as f:
            json.dump(self._triggers, f, indent=2)

    def create_trigger(self, vault_id: str, name: str, condition: str, action: str, enabled: bool = True) -> Dict:
        """Cria trigger CARE automatizado."""
        self._trigger_counter = getattr(self, "_trigger_counter", 0) + 1
        trigger_id = f"care_{self._trigger_counter}_{vault_id[:8]}"

        self._triggers["triggers"][trigger_id] = {
            "vault_id": vault_id,
            "name": name,
            "condition": condition,
            "action": action,
            "enabled": enabled,
            "created_at": time.time(),
            "fired_count": 0,
            "last_fired": None,
            "status": "ACTIVE" if enabled else "DISABLED"
        }
        self._save_triggers()

        # Registrar na API Fordefi (simulado)
        result = self.client.create_care_trigger(vault_id, condition, action)

        return {
            "trigger_id": trigger_id,
            "vault_id": vault_id,
            "name": name,
            "condition": condition,
            "action": action,
            "status": "ACTIVE",
            "message": f"Trigger CARE '{name}' criado para vault {vault_id}."
        }

    def list_triggers(self, vault_id: Optional[str] = None) -> List[Dict]:
        """Lista triggers CARE ativos."""
        triggers = self._triggers.get("triggers", {})
        result = []
        for tid, data in triggers.items():
            if vault_id and data.get("vault_id") != vault_id:
                continue
            result.append({
                "trigger_id": tid,
                "name": data["name"],
                "condition": data["condition"],
                "action": data["action"],
                "status": data["status"],
                "fired_count": data["fired_count"],
                "last_fired": data["last_fired"]
            })
        return result

    def disable_trigger(self, trigger_id: str) -> Dict:
        """Desabilita trigger CARE."""
        if trigger_id not in self._triggers["triggers"]:
            return {"error": "Trigger não encontrado", "status": "FAILED"}

        self._triggers["triggers"][trigger_id]["enabled"] = False
        self._triggers["triggers"][trigger_id]["status"] = "DISABLED"
        self._save_triggers()

        return {
            "trigger_id": trigger_id,
            "status": "DISABLED",
            "message": "Trigger desabilitado."
        }

    def get_logs(self, trigger_id: str) -> List[Dict]:
        """Retorna logs de execução do trigger."""
        return self.client.get_care_logs(trigger_id)

    def simulate_trigger(self, trigger_id: str, event_data: Dict) -> Dict:
        """Simula execução de trigger com dados de evento."""
        trigger = self._triggers["triggers"].get(trigger_id, {})
        if not trigger:
            return {"error": "Trigger não encontrado", "status": "FAILED"}

        condition = trigger["condition"]
        action = trigger["action"]

        # Parse condição simples: "price_drop>10%", "balance<1000", "gas>100"
        condition_met = self._evaluate_condition(condition, event_data)

        if condition_met:
            trigger["fired_count"] += 1
            trigger["last_fired"] = time.time()
            self._save_triggers()

            return {
                "trigger_id": trigger_id,
                "condition": condition,
                "condition_met": True,
                "action": action,
                "action_executed": True,
                "event_data": event_data,
                "timestamp": time.time(),
                "message": f"Trigger fired: {action}"
            }

        return {
            "trigger_id": trigger_id,
            "condition": condition,
            "condition_met": False,
            "action_executed": False,
            "event_data": event_data,
            "message": "Condição não satisfeita."
        }

    def _evaluate_condition(self, condition: str, event_data: Dict) -> bool:
        """Avalia condição simples contra dados de evento."""
        try:
            # Parse: "price_drop>10%" → price_drop > 10
            if ">" in condition:
                key, val = condition.split(">")
                key = key.strip()
                val = float(val.strip().replace("%", ""))
                return event_data.get(key, 0) > val
            elif "<" in condition:
                key, val = condition.split("<")
                key = key.strip()
                val = float(val.strip().replace("%", ""))
                return event_data.get(key, 0) < val
            elif "==" in condition:
                key, val = condition.split("==")
                return str(event_data.get(key.strip(), "")) == val.strip()
            return False
        except Exception:
            return False


def main():
    import sys
    care = CAREBridge()

    if len(sys.argv) < 2:
        print("Uso: python -m care_bridge <command> [args]")
        print("Comandos: create, list, disable, logs, simulate")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "create" and len(sys.argv) >= 6:
        result = care.create_trigger(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5])
        print(json.dumps(result, indent=2))
    elif cmd == "list":
        vault = sys.argv[2] if len(sys.argv) > 2 else None
        result = care.list_triggers(vault)
        print(json.dumps(result, indent=2))
    elif cmd == "disable" and len(sys.argv) > 2:
        result = care.disable_trigger(sys.argv[2])
        print(json.dumps(result, indent=2))
    elif cmd == "logs" and len(sys.argv) > 2:
        result = care.get_logs(sys.argv[2])
        print(json.dumps(result, indent=2))
    elif cmd == "simulate" and len(sys.argv) > 3:
        event = json.loads(sys.argv[3])
        result = care.simulate_trigger(sys.argv[2], event)
        print(json.dumps(result, indent=2))
    else:
        print(f"Comando não reconhecido: {cmd}")


if __name__ == "__main__":
    main()
