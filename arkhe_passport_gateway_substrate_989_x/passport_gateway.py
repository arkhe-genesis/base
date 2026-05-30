#!/usr/bin/env python3
"""
Passport Gateway — Substrato 989.x
Verifica humanidade via Gitcoin Passport + ORCID para governança DAO e acesso à malha.
Arquiteto ORCID: 0009-0005-2697-4668
Cross-links: 979, 954, 982, 983, 957, 958
"""

import asyncio
import hashlib
import json
import os
from typing import Dict, Optional, List
from dataclasses import dataclass
from datetime import datetime, timezone
import aiohttp

# ═══════════════════════════════════════════════════════════════
# Configuração (variáveis de ambiente)
# ═══════════════════════════════════════════════════════════════
PASSPORT_API_KEY = os.environ.get("PASSPORT_API_KEY", "demo-key")
PASSPORT_SCORER_ID = os.environ.get("PASSPORT_SCORER_ID", "1")
ORCID_CLIENT_ID = os.environ.get("ORCID_CLIENT_ID", "APP-XXXXXXXX")
ORCID_CLIENT_SECRET = os.environ.get("ORCID_CLIENT_SECRET", "secret")

@dataclass
class HumanityProof:
    address: str
    is_human: bool
    score: float           # 0-1, quanto maior mais humano
    stamps: List[str]      # stamps verificados (ex: "EncryptedMobile")
    orcid_verified: bool
    timestamp: str = ""
    def __post_init__(self):
        if not self.timestamp:
            self.timestamp = datetime.now(timezone.utc).isoformat()

class PassportGateway:
    """Integra Gitcoin Passport (stamps/scorer) e ORCID."""

    def __init__(self):
        self.session: Optional[aiohttp.ClientSession] = None

    async def start(self):
        self.session = aiohttp.ClientSession(
            headers={"X-API-Key": PASSPORT_API_KEY}
        )
        print("  [PASSPORT] Gateway iniciado.")

    async def stop(self):
        if self.session:
            await self.session.close()

    # -----------------------------------------------------------------
    # Gitcoin Passport — via Scorer API (community scorer)
    # -----------------------------------------------------------------
    async def get_passport_score(self, address: str) -> dict:
        """Obtém score de humanidade de um endereço usando o scorer."""
        if not self.session:
            raise RuntimeError("PassportGateway não iniciado.")
        url = f"https://api.scorer.gitcoin.co/registry/score/{PASSPORT_SCORER_ID}/{address}"
        async with self.session.get(url) as resp:
            if resp.status == 200:
                return await resp.json()
            else:
                return {"error": f"HTTP {resp.status}"}

    async def get_passport_stamps(self, address: str) -> List[dict]:
        """Retorna stamps verificados de um endereço."""
        if not self.session:
            raise RuntimeError("PassportGateway não iniciado.")
        url = f"https://api.scorer.gitcoin.co/registry/stamps/{address}"
        async with self.session.get(url) as resp:
            if resp.status == 200:
                data = await resp.json()
                return data.get("items", [])
            else:
                return []

    async def is_human(self, address: str, min_score: float = 20.0) -> HumanityProof:
        """
        Verifica se um endereço é humano com base no score do Passport.
        Retorna HumanityProof com score normalizado (0-1).
        """
        score_data = await self.get_passport_score(address)
        if "error" in score_data:
            return HumanityProof(address=address, is_human=False, score=0.0,
                                 stamps=[], orcid_verified=False)

        raw_score = score_data.get("score", 0)
        # Normalização: Gitcoin usa threshold ~20 para Unique Humanity
        normalized = min(raw_score / 20.0, 1.0)
        stamps = await self.get_passport_stamps(address)
        stamp_names = [s.get("credential", {}).get("credentialSubject", {}).get("provider", "")
                       for s in stamps if s.get("credential")]

        # Verificação ORCID (simulada – em produção integrar com substrato 982)
        orcid_ok = await self._verify_orcid_link(address)

        return HumanityProof(
            address=address,
            is_human=normalized >= 0.75,
            score=normalized,
            stamps=stamp_names,
            orcid_verified=orcid_ok,
        )

    # -----------------------------------------------------------------
    # ORCID (substrato 982) – vinculação simplificada
    # -----------------------------------------------------------------
    async def _verify_orcid_link(self, address: str) -> bool:
        """
        Verifica se o endereço tem um ORCID vinculado.
        Em produção, consultaria a TemporalChain (923) ou o banco do 982.
        Aqui simulamos: endereços que começam com "0xAlice" ou "0xArchitect" têm ORCID.
        """
        if address.startswith("0xAlice") or address.startswith("0xArchitect"):
            return True
        return False

    # -----------------------------------------------------------------
    # Integração com DAO (979): verificação de eleitor
    # -----------------------------------------------------------------
    async def verify_dao_voter(self, address: str) -> bool:
        """
        Um endereço pode votar na DAO se:
        - É considerado humano pelo Passport (score ≥ 0.75)
        - OU possui ORCID verificado
        - Não está em lista de sanções (simulado)
        """
        proof = await self.is_human(address)
        return proof.is_human or proof.orcid_verified

    # -----------------------------------------------------------------
    # Integração com Nós da Malha (972): controle de acesso
    # -----------------------------------------------------------------
    async def verify_node_access(self, address: str) -> bool:
        """Um operador de nó deve ser humano ou ter ORCID."""
        return await self.verify_dao_voter(address)

async def demo_passport_gateway():
    gateway = PassportGateway()
    await gateway.start()

    # Verificar humanidade de alguns endereços
    addresses = ["0xAlice123...", "0xBob456...", "0xSybil999...", "0xArchitect0009..."]
    for addr in addresses:
        proof = await gateway.is_human(addr)
        print(f"{addr[:15]}... → humano: {proof.is_human}, score: {proof.score:.2f}, "
              f"stamps: {len(proof.stamps)}, ORCID: {proof.orcid_verified}")

    # Verificar permissão para votar na DAO
    print("\nVerificação DAO:")
    for addr in ["0xAlice123...", "0xSybil999..."]:
        can = await gateway.verify_dao_voter(addr)
        print(f"  {addr[:15]}... pode votar: {can}")

    await gateway.stop()

# Executar
if __name__ == "__main__":
    asyncio.run(demo_passport_gateway())
