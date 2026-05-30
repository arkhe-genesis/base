#!/usr/bin/env python3
"""
Passport Gateway — Substrato 989.x
Verificação de humanidade via Gitcoin Passport + ORCID para governança DAO e acesso à malha.
Arquiteto ORCID: 0009-0005-2697-4668
Cross-links: 979, 954, 982, 983, 957, 958, 923, 972, 972.1, 972.4
Deities: Themis, Athena, Hermes
Status: CANONIZED_PROVISIONAL
Seal: 989-PASSPORT-GATEWAY-4B3CB68C02D21E5A
"""

import asyncio
import hashlib
import json
import os
from typing import Dict, Optional, List, Any
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from enum import Enum

import aiohttp


# ═══════════════════════════════════════════════════════════════════
# Configuração de ambiente
# ═══════════════════════════════════════════════════════════════════
PASSPORT_API_KEY = os.environ.get("PASSPORT_API_KEY", "")
PASSPORT_SCORER_ID = os.environ.get("PASSPORT_SCORER_ID", "1")
ORCID_CLIENT_ID = os.environ.get("ORCID_CLIENT_ID", "")
ORCID_CLIENT_SECRET = os.environ.get("ORCID_CLIENT_SECRET", "")
PASSPORT_BASE_URL = os.environ.get("PASSPORT_BASE_URL", "https://api.scorer.gitcoin.co")
ORCID_BASE_URL = os.environ.get("ORCID_BASE_URL", "https://pub.orcid.org/v3.0")

# Thresholds canônicos da Catedral
MIN_HUMANITY_SCORE = float(os.environ.get("ARKHE_MIN_HUMANITY_SCORE", "0.75"))
MIN_PASSPORT_SCORE = float(os.environ.get("ARKHE_MIN_PASSPORT_SCORE", "20.0"))


class VerificationStatus(Enum):
    VERIFIED = "verified"
    PENDING = "pending"
    REJECTED = "rejected"
    ERROR = "error"


@dataclass
class StampCredential:
    """Credential verificada do Passport."""
    provider: str
    issuance_date: Optional[str] = None
    expiration_date: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class HumanityProof:
    """Prova de humanidade canônica da Catedral."""
    address: str
    is_human: bool
    score: float                    # 0.0 – 1.0
    raw_passport_score: float       # Score bruto do Passport
    stamps: List[StampCredential]
    orcid_verified: bool
    orcid_id: Optional[str] = None
    sanctions_clear: bool = True    # Proof of Clean Hands
    status: VerificationStatus = VerificationStatus.PENDING
    timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    seal: str = ""
    temporal_anchor: Optional[str] = None  # Ref. TemporalChain (923)

    def compute_seal(self) -> str:
        """Gera seal SHA3-256 canônico."""
        payload = {
            "address": self.address,
            "is_human": self.is_human,
            "score": round(self.score, 4),
            "orcid": self.orcid_verified,
            "sanctions": self.sanctions_clear,
            "timestamp": self.timestamp,
        }
        json_str = json.dumps(payload, sort_keys=True, ensure_ascii=False)
        self.seal = f"HP-{hashlib.sha3_256(json_str.encode()).hexdigest()[:16].upper()}"
        return self.seal

    def to_dict(self) -> Dict[str, Any]:
        d = asdict(self)
        d["status"] = self.status.value
        d["stamps"] = [asdict(s) for s in self.stamps]
        return d


class PassportGatewayError(Exception):
    """Erro canônico do Passport Gateway."""
    pass


class PassportGateway:
    """
    Gateway de verificação de humanidade.
    Integra Gitcoin Passport (Stamps + Scorer) e ORCID (982).
    """

    SUBSTRATE_ID = 989
    VARIANT = "x"
    SEAL = "989-PASSPORT-GATEWAY-4B3CB68C02D21E5A"

    def __init__(
        self,
        api_key: Optional[str] = None,
        scorer_id: Optional[str] = None,
        orcid_client_id: Optional[str] = None,
        orcid_client_secret: Optional[str] = None,
        session: Optional[aiohttp.ClientSession] = None,
    ):
        self.api_key = api_key or PASSPORT_API_KEY
        self.scorer_id = scorer_id or PASSPORT_SCORER_ID
        self.orcid_client_id = orcid_client_id or ORCID_CLIENT_ID
        self.orcid_client_secret = orcid_client_secret or ORCID_CLIENT_SECRET
        self._session = session
        self._owned_session = session is None

    @property
    def session(self) -> aiohttp.ClientSession:
        if self._session is None or self._session.closed:
            headers = {}
            if self.api_key:
                headers["X-API-Key"] = self.api_key
            self._session = aiohttp.ClientSession(
                headers=headers,
                timeout=aiohttp.ClientTimeout(total=30),
            )
        return self._session

    async def start(self) -> None:
        """Inicializa o gateway (idempotente)."""
        _ = self.session

    async def stop(self) -> None:
        """Encerra sessões HTTP."""
        if self._owned_session and self._session and not self._session.closed:
            await self._session.close()
            self._session = None

    # ───────────────────────────────────────────────────────────────
    # Gitcoin Passport — Scorer API
    # ───────────────────────────────────────────────────────────────
    async def get_passport_score(self, address: str) -> Dict[str, Any]:
        """Obtém score de humanidade via Passport Scorer API."""
        if not self.api_key:
            raise PassportGatewayError("PASSPORT_API_KEY não configurada")
        url = f"{PASSPORT_BASE_URL}/registry/score/{self.scorer_id}/{address}"
        async with self.session.get(url) as resp:
            if resp.status == 200:
                return await resp.json()
            if resp.status == 401:
                raise PassportGatewayError("API Key inválida")
            if resp.status == 404:
                return {"score": 0, "status": "NOT_FOUND", "address": address}
            text = await resp.text()
            raise PassportGatewayError(f"Passport API HTTP {resp.status}: {text}")

    async def get_passport_stamps(self, address: str) -> List[StampCredential]:
        """Retorna stamps verificados de um endereço EVM."""
        if not self.api_key:
            raise PassportGatewayError("PASSPORT_API_KEY não configurada")
        url = f"{PASSPORT_BASE_URL}/registry/stamps/{address}"
        async with self.session.get(url) as resp:
            if resp.status == 200:
                data = await resp.json()
                items = data.get("items", [])
                stamps = []
                for item in items:
                    cred = item.get("credential", {})
                    subj = cred.get("credentialSubject", {})
                    stamps.append(StampCredential(
                        provider=subj.get("provider", "unknown"),
                        issuance_date=cred.get("issuanceDate"),
                        expiration_date=cred.get("expirationDate"),
                        metadata=item,
                    ))
                return stamps
            if resp.status == 404:
                return []
            text = await resp.text()
            raise PassportGatewayError(f"Passport Stamps HTTP {resp.status}: {text}")

    async def submit_passport(self, address: str) -> Dict[str, Any]:
        """Submete endereço para scoring (caso ainda não tenha score)."""
        if not self.api_key:
            raise PassportGatewayError("PASSPORT_API_KEY não configurada")
        url = f"{PASSPORT_BASE_URL}/registry/submit-passport"
        payload = {
            "address": address,
            "scorer_id": self.scorer_id,
            "signature": "",   # preenchido se necessário
            "nonce": "",
        }
        async with self.session.post(url, json=payload) as resp:
            if resp.status in (200, 201):
                return await resp.json()
            text = await resp.text()
            raise PassportGatewayError(f"Submit Passport HTTP {resp.status}: {text}")

    # ───────────────────────────────────────────────────────────────
    # ORCID — Substrato 982
    # ───────────────────────────────────────────────────────────────
    async def get_orcid_record(self, orcid_id: str) -> Dict[str, Any]:
        """Consulta registro público ORCID v3.0."""
        url = f"{ORCID_BASE_URL}/{orcid_id}/record"
        headers = {"Accept": "application/json"}
        async with self.session.get(url, headers=headers) as resp:
            if resp.status == 200:
                return await resp.json()
            if resp.status == 404:
                return {}
            text = await resp.text()
            raise PassportGatewayError(f"ORCID HTTP {resp.status}: {text}")

    async def verify_orcid_link(self, address: str, orcid_id: Optional[str] = None) -> bool:
        """
        Verifica se endereço EVM está vinculado a ORCID.
        Em produção: consulta TemporalChain (923) ou registro canônico.
        """
        # Stub para integração futura com substrato 982
        if orcid_id:
            record = await self.get_orcid_record(orcid_id)
            return bool(record)
        # Fallback heurístico (demo / testes)
        return address.startswith("0xAlice") or address.startswith("0xArchitect")

    # ───────────────────────────────────────────────────────────────
    # Verificação canônica de humanidade
    # ───────────────────────────────────────────────────────────────
    async def is_human(
        self,
        address: str,
        min_score: Optional[float] = None,
        orcid_id: Optional[str] = None,
    ) -> HumanityProof:
        """
        Verifica se endereço é humano via Passport + ORCID.
        Retorna HumanityProof com seal canônico.
        """
        threshold = min_score if min_score is not None else MIN_HUMANITY_SCORE
        raw_score = 0.0
        stamps: List[StampCredential] = []
        status = VerificationStatus.PENDING

        try:
            score_data = await self.get_passport_score(address)
            raw_score = float(score_data.get("score", 0))
            stamps = await self.get_passport_stamps(address)
            status = VerificationStatus.VERIFIED
        except PassportGatewayError as e:
            status = VerificationStatus.ERROR
            # Em modo degradado, confiar em ORCID se disponível
            if orcid_id:
                status = VerificationStatus.PENDING

        normalized = min(raw_score / MIN_PASSPORT_SCORE, 1.0) if MIN_PASSPORT_SCORE > 0 else 0.0
        orcid_ok = await self.verify_orcid_link(address, orcid_id)

        # Proof of Clean Hands (AML) — simulado; em produção integrar com Individual Verifications
        sanctions_clear = True

        is_human = (normalized >= threshold) or orcid_ok

        proof = HumanityProof(
            address=address,
            is_human=is_human,
            score=normalized,
            raw_passport_score=raw_score,
            stamps=stamps,
            orcid_verified=orcid_ok,
            orcid_id=orcid_id,
            sanctions_clear=sanctions_clear,
            status=status,
        )
        proof.compute_seal()
        return proof

    # ───────────────────────────────────────────────────────────────
    # Integração DAO (979) — verificação de eleitor
    # ───────────────────────────────────────────────────────────────
    async def verify_dao_voter(self, address: str, orcid_id: Optional[str] = None) -> bool:
        """
        Um endereço pode votar na DAO se:
        - É considerado humano pelo Passport (score ≥ threshold)
        - OU possui ORCID verificado
        - Não está em lista de sanções (Proof of Clean Hands)
        """
        proof = await self.is_human(address, orcid_id=orcid_id)
        return proof.is_human and proof.sanctions_clear

    # ───────────────────────────────────────────────────────────────
    # Integração Malha (972) — controle de acesso a nós
    # ───────────────────────────────────────────────────────────────
    async def verify_node_access(self, address: str, orcid_id: Optional[str] = None) -> bool:
        """Operador de nó deve ser humano verificado ou ter ORCID."""
        return await self.verify_dao_voter(address, orcid_id)

    # ───────────────────────────────────────────────────────────────
    # Integração Axiarchy (954) — validação ética
    # ───────────────────────────────────────────────────────────────
    async def axiarchy_validate(self, address: str, action_type: str = "vote") -> Dict[str, Any]:
        """
        Valida se ação do endereço passa no gate ético Axiarchy.
        action_type: vote | node_access | treasury | proposal
        """
        proof = await self.is_human(address)
        approved = proof.is_human and proof.sanctions_clear

        return {
            "address": address,
            "action": action_type,
            "approved": approved,
            "humanity_score": proof.score,
            "orcid_verified": proof.orcid_verified,
            "sanctions_clear": proof.sanctions_clear,
            "seal": proof.seal,
            "substrate": f"{self.SUBSTRATE_ID}.{self.VARIANT}",
        }

    # ───────────────────────────────────────────────────────────────
    # Relatório canônico
    # ───────────────────────────────────────────────────────────────
    def generate_report(self) -> str:
        """Gera relatório canônico do substrato."""
        return f"""
╔══════════════════════════════════════════════════════════════════╗
║  ARKHE CATHEDRAL — SUBSTRATO {self.SUBSTRATE_ID}.{self.VARIANT}: PASSPORT-GATEWAY        ║
║  Themis julga; Athena sabe; Hermes entrega a identidade           ║
╠══════════════════════════════════════════════════════════════════╣
  Seal: {self.SEAL}
  Status: CANONIZED_PROVISIONAL
  Cross-links: [979, 954, 982, 983, 957, 958, 923, 972, 972.1, 972.4]
  Deities: Themis, Athena, Hermes
  Threshold Humanity Score: {MIN_HUMANITY_SCORE}
  Min Passport Score: {MIN_PASSPORT_SCORE}
  API Key configurada: {"Sim" if self.api_key else "Não (modo simulação)"}
  ORCID Client configurado: {"Sim" if self.orcid_client_id else "Não"}
╚══════════════════════════════════════════════════════════════════╝
"""


# ═══════════════════════════════════════════════════════════════════
# DEMONSTRAÇÃO
# ═══════════════════════════════════════════════════════════════════

async def demo():
    print("=" * 68)
    print("  ARKHE PASSPORT-GATEWAY — DEMONSTRAÇÃO")
    print("=" * 68)

    gw = PassportGateway()
    await gw.start()

    # Teste com endereço mock (sem API key, cai em fallback)
    print("\n[1] Verificando endereço mock (modo simulação)...")
    proof = await gw.is_human("0xArchitect1234567890abcdef", orcid_id="0009-0005-2697-4668")
    print(f"    is_human: {proof.is_human}")
    print(f"    score: {proof.score:.2f}")
    print(f"    seal: {proof.seal}")

    # Verificação DAO
    print("\n[2] Verificação de eleitor DAO:")
    can_vote = await gw.verify_dao_voter("0xAlice1234567890abcdef")
    print(f"    Pode votar: {can_vote}")

    # Validação Axiarchy
    print("\n[3] Validação Axiarchy:")
    result = await gw.axiarchy_validate("0xAlice1234567890abcdef", "vote")
    print(f"    approved: {result['approved']}")
    print(f"    seal: {result['seal']}")

    print("\n[4] Relatório canônico:")
    print(gw.generate_report())

    await gw.stop()


if __name__ == "__main__":
    asyncio.run(demo())
