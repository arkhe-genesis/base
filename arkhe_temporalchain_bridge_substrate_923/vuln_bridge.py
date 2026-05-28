#!/usr/bin/env python3
"""
Substrato 923.2 — Vulnerability Bridge
Conecta o ARKHE‑GLASSWING‑SENTINEL (944) à Vulnerability TemporalChain.
"""

import json, hashlib, os, time
from web3 import Web3
from web3.middleware import geth_poa_middleware
from eth_account import Account
from typing import Dict, Optional

class VulnerabilityBridge:
    """
    Regista vulnerabilidades na blockchain e gera provas de integridade.
    """
    def __init__(self, rpc_url: str, contract_address: str, private_key: str):
        self.w3 = Web3(Web3.HTTPProvider(rpc_url))
        # Para redes como Base ou Arbitrum, injecta middleware PoA
        self.w3.middleware_onion.inject(geth_poa_middleware, layer=0)
        self.account = Account.from_key(private_key)
        self.contract = self._load_contract(contract_address)

    def _load_contract(self, address: str):
        with open("VulnerabilityRegistry.abi", "r") as f:
            abi = json.load(f)
        return self.w3.eth.contract(address=Web3.to_checksum_address(address), abi=abi)

    def report_vulnerability(self, cve_id: str, project: str,
                             report_payload: Dict, severity: int,
                             proof: bytes = b"") -> str:
        """
        Submete uma vulnerabilidade à TemporalChain.
        Retorna o hash da transacção.
        """
        # Hash canónico do relatório
        content_str = json.dumps(report_payload, sort_keys=True)
        content_hash = Web3.keccak(text=content_str)

        # Constrói transacção
        tx = self.contract.functions.reportVulnerability(
            cve_id,
            project,
            content_hash,
            severity,
            proof
        ).build_transaction({
            'from': self.account.address,
            'nonce': self.w3.eth.get_transaction_count(self.account.address),
            'gas': 500000,
            'gasPrice': self.w3.eth.gas_price
        })

        signed = self.account.sign_transaction(tx)
        tx_hash = self.w3.eth.send_raw_transaction(signed.rawTransaction)
        receipt = self.w3.eth.wait_for_transaction_receipt(tx_hash)
        return receipt.transactionHash.hex()

    def mark_patched(self, vuln_id: int, patch_commit: str) -> str:
        """Marca uma vulnerabilidade como corrigida."""
        tx = self.contract.functions.markPatched(
            vuln_id, patch_commit
        ).build_transaction({
            'from': self.account.address,
            'nonce': self.w3.eth.get_transaction_count(self.account.address),
            'gas': 200000,
            'gasPrice': self.w3.eth.gas_price
        })
        signed = self.account.sign_transaction(tx)
        tx_hash = self.w3.eth.send_raw_transaction(signed.rawTransaction)
        receipt = self.w3.eth.wait_for_transaction_receipt(tx_hash)
        return receipt.transactionHash.hex()

    def generate_zk_proof(self, cve_id: str, project: str) -> bytes:
        """
        Gera uma prova ZK simplificada de divulgação responsável.
        Em produção, usaria circuitos ark‑bn254 do Substrato 260.2.
        Aqui, retorna um placeholder compatível com Groth16.
        """
        payload = f"ARKHE-VULN-923.2|{cve_id}|{project}|{int(time.time())}"
        return hashlib.sha3_256(payload.encode()).digest()[:64]  # simulacro
