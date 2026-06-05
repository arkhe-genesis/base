#!/usr/bin/env python3
"""
fordefi_client.py — Cliente HTTP/API para Fordefi v1.0.0.
MPC Wallet institutional: vaults, transactions, policies, webhooks, CARE.

Deidades: Hermes Trismegisto (mensageiro), Plutão (riqueza)
Selo: FORDEFI-BRIDGE-1066.1-v1.0.0-2026-06-05
"""

import os
import json
import time
import hmac
import hashlib
import base64
import requests
from typing import Dict, Optional, List, Any
from urllib.parse import urljoin

FORDEFI_BASE_URL = os.environ.get("FORDEFI_API_URL", "https://api.fordefi.com/api/v1")
FORDEFI_API_KEY = os.environ.get("FORDEFI_API_KEY", "")
FORDEFI_API_SECRET = os.environ.get("FORDEFI_API_SECRET", "")


class FordefiClient:
    """Cliente para a API Fordefi institutional."""

    def __init__(self, api_key: str = FORDEFI_API_KEY, api_secret: str = FORDEFI_API_SECRET, base_url: str = FORDEFI_BASE_URL):
        self.api_key = api_key
        self.api_secret = api_secret
        self.base_url = base_url
        self.session = requests.Session()
        self.session.headers.update({
            "Content-Type": "application/json",
            "Accept": "application/json",
        })

    def _sign_request(self, method: str, path: str, body: str = "") -> str:
        """Assina a requisição com HMAC-SHA256 usando API secret."""
        timestamp = str(int(time.time() * 1000))
        message = f"{timestamp}{method.upper()}{path}{body}"
        signature = hmac.new(
            self.api_secret.encode(),
            message.encode(),
            hashlib.sha256
        ).hexdigest()
        return timestamp, signature

    def _request(self, method: str, path: str, data: Optional[Dict] = None) -> Dict[str, Any]:
        """Executa requisição assinada à API Fordefi."""
        url = urljoin(self.base_url, path)
        body = json.dumps(data) if data else ""
        timestamp, signature = self._sign_request(method, path, body)

        headers = {
            "X-API-Key": self.api_key,
            "X-Timestamp": timestamp,
            "X-Signature": signature,
        }

        try:
            if method.upper() == "GET":
                resp = self.session.get(url, headers=headers, timeout=30)
            elif method.upper() == "POST":
                resp = self.session.post(url, headers=headers, data=body, timeout=30)
            elif method.upper() == "PUT":
                resp = self.session.put(url, headers=headers, data=body, timeout=30)
            elif method.upper() == "DELETE":
                resp = self.session.delete(url, headers=headers, timeout=30)
            else:
                raise ValueError(f"Método HTTP não suportado: {method}")

            resp.raise_for_status()
            return resp.json() if resp.text else {}
        except requests.exceptions.RequestException as e:
            return {"error": str(e), "status": "FAILED"}

    # === Vault Management ===
    def list_vaults(self) -> List[Dict]:
        return self._request("GET", "/vaults").get("vaults", [])

    def get_vault(self, vault_id: str) -> Dict:
        return self._request("GET", f"/vaults/{vault_id}")

    def create_vault(self, name: str, chain_type: str, policy: Optional[Dict] = None) -> Dict:
        payload = {
            "name": name,
            "chain_type": chain_type,
            "policy": policy or {}
        }
        return self._request("POST", "/vaults", payload)

    # === Transaction Lifecycle ===
    def create_transaction(self, vault_id: str, tx_type: str, details: Dict) -> Dict:
        payload = {
            "vault_id": vault_id,
            "type": tx_type,
            "details": details
        }
        return self._request("POST", "/transactions", payload)

    def get_transaction(self, tx_id: str) -> Dict:
        return self._request("GET", f"/transactions/{tx_id}")

    def list_transactions(self, vault_id: Optional[str] = None) -> List[Dict]:
        path = "/transactions"
        if vault_id:
            path += f"?vault_id={vault_id}"
        return self._request("GET", path).get("transactions", [])

    def simulate_transaction(self, tx_id: str) -> Dict:
        return self._request("POST", f"/transactions/{tx_id}/simulate")

    def submit_transaction(self, tx_id: str) -> Dict:
        return self._request("POST", f"/transactions/{tx_id}/submit")

    # === Policy Engine ===
    def list_policies(self, vault_id: str) -> List[Dict]:
        return self._request("GET", f"/vaults/{vault_id}/policies").get("policies", [])

    def apply_policy(self, vault_id: str, policy: Dict) -> Dict:
        return self._request("POST", f"/vaults/{vault_id}/policies", policy)

    # === CARE Engine ===
    def list_care_triggers(self) -> List[Dict]:
        return self._request("GET", "/care/triggers").get("triggers", [])

    def create_care_trigger(self, vault_id: str, trigger: str, action: str) -> Dict:
        payload = {
            "vault_id": vault_id,
            "trigger": trigger,
            "action": action
        }
        return self._request("POST", "/care/triggers", payload)

    def get_care_logs(self, trigger_id: str) -> List[Dict]:
        return self._request("GET", f"/care/triggers/{trigger_id}/logs").get("logs", [])

    # === Risk & Monitoring ===
    def get_risk_score(self, vault_id: str) -> Dict:
        return self._request("GET", f"/vaults/{vault_id}/risk")

    def list_alerts(self) -> List[Dict]:
        return self._request("GET", "/alerts").get("alerts", [])


def main():
    import sys
    client = FordefiClient()

    if len(sys.argv) < 2:
        print("Uso: python -m fordefi_client <command> [args]")
        print("Comandos: list-vaults, get-vault, create-vault, list-tx, get-tx, risk-score")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "list-vaults":
        vaults = client.list_vaults()
        print(json.dumps(vaults, indent=2))
    elif cmd == "get-vault" and len(sys.argv) > 2:
        vault = client.get_vault(sys.argv[2])
        print(json.dumps(vault, indent=2))
    elif cmd == "list-tx":
        txs = client.list_transactions()
        print(json.dumps(txs, indent=2))
    elif cmd == "risk-score" and len(sys.argv) > 2:
        score = client.get_risk_score(sys.argv[2])
        print(json.dumps(score, indent=2))
    else:
        print(f"Comando não reconhecido: {cmd}")


if __name__ == "__main__":
    main()
