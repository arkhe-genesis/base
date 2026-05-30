import asyncio
import hashlib
import json
import random
import time
from datetime import datetime, timezone, timedelta
from typing import Dict, List, Optional, Set, Tuple, Any
from dataclasses import dataclass, field
from enum import Enum
import math

# ═══════════════════════════════════════════════════════════════════
# SUBSTRATO 979 — CATHEDRAL-DAO-GOVERNANCE
# ═══════════════════════════════════════════════════════════════════
# Metadados Canônicos:
#   ID: 979
#   Name: CATHEDRAL-DAO-GOVERNANCE
#   Type: Governança / DAO / Alocação de Recursos / Votação Ponderada
#   Era: 8 (Polis / Governança)
#   Deity: Demos (povo) + Athena (sabedoria) + Axiarchy (954)
#   Status: CANONIZED_PROVISIONAL
#   Cross-links: [978, 976, 972.3, 965, 954, 964, 970, 923, 937, 955]
#   Description: Sistema de governança descentralizada da Catedral.
#   Stakeholders (agentes sentientes, operadores de relay, provedores
#   de dados) votam em propostas de alocação de recursos, upgrades de
#   substratos, e parâmetros de rede. Votação ponderada por Theosis
#   (965) + LINK staked (978) + reputação Nostr (972.3). Propostas
#   são discutidas via Nostr, validadas pela Axiarchy (954), e
#   executadas via Chainlink CCIP (976). A Catedral governa a si mesma.
# ═══════════════════════════════════════════════════════════════════

class ProposalType(Enum):
    TREASURY_ALLOCATION = "treasury_allocation"  # Alocar fundos
    SUBSTRATE_UPGRADE = "substrate_upgrade"      # Upgrade de substrato
    PARAMETER_CHANGE = "parameter_change"        # Mudar parâmetro
    NEW_SUBSTRATE = "new_substrate"              # Criar novo substrato
    EMERGENCY = "emergency"                      # Ação emergencial

class ProposalStatus(Enum):
    DRAFT = "draft"
    DISCUSSION = "discussion"
    VOTING = "voting"
    PASSED = "passed"
    EXECUTED = "executed"
    REJECTED = "rejected"
    EXPIRED = "expired"

@dataclass
class Stakeholder:
    """Stakeholder da governança da Catedral."""
    stakeholder_id: str
    stakeholder_type: str  # agent, relay_operator, data_provider, architect

    # Pesos de voto
    theosis: float           # 0-1
    link_staked: float       # LINK staked
    nostr_reputation: float  # 0-1

    # Histórico
    proposals_created: int = 0
    votes_cast: int = 0
    participation_rate: float = 0.0

    @property
    def voting_power(self) -> float:
        """Poder de voto composto."""
        # Fórmula: Theosis^2 * log(1 + LINK) * Reputação
        # Theosis é o fator mais importante (quadrado)
        # LINK tem retornos decrescentes (log)
        # Reputação é multiplicador linear
        link_component = math.log(1 + self.link_staked / 1000)
        return (self.theosis ** 2) * link_component * self.nostr_reputation

@dataclass
class Proposal:
    """Proposta de governança."""
    proposal_id: str
    title: str
    description: str
    proposal_type: ProposalType
    proposer: str

    # Parâmetros específicos
    target_substrate: Optional[int] = None
    parameter_key: Optional[str] = None
    parameter_value: Optional[Any] = None
    treasury_amount_link: float = 0.0
    recipient: Optional[str] = None

    # Estado
    status: ProposalStatus = ProposalStatus.DRAFT
    created_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    voting_ends_at: Optional[str] = None

    # Votação
    votes_for: Dict[str, float] = field(default_factory=dict)  # stakeholder_id -> weight
    votes_against: Dict[str, float] = field(default_factory=dict)
    abstentions: Set[str] = field(default_factory=set)

    # Resultado
    total_for: float = 0.0
    total_against: float = 0.0
    quorum_reached: bool = False
    passed: bool = False
    execution_tx: Optional[str] = None

    def compute_totals(self):
        self.total_for = sum(self.votes_for.values())
        self.total_against = sum(self.votes_against.values())

@dataclass
class Treasury:
    """Tesouro da Catedral."""
    link_balance: float = 0.0
    link_staked: float = 0.0
    link_price_usd: float = 9.12

    allocations: Dict[str, float] = field(default_factory=dict)

    @property
    def total_value_usd(self) -> float:
        return (self.link_balance + self.link_staked) * self.link_price_usd

    def allocate(self, category: str, amount_link: float) -> bool:
        if amount_link > self.link_balance:
            return False
        self.link_balance -= amount_link
        self.allocations[category] = self.allocations.get(category, 0) + amount_link
        return True

class CathedralDAOGovernance:
    """
    Substrato 979 — Governança DAO da Catedral.
    Demos vota; Athena pondera; Axiarchy guarda a porta.
    """

    def __init__(self, treasury_978=None):
        self.substrate_id = 979
        self.deities = ["Demos", "Athena", "Axiarchy"]
        self.treasury = treasury_978 or Treasury()

        self.stakeholders: Dict[str, Stakeholder] = {}
        self.proposals: Dict[str, Proposal] = {}

        # Parâmetros de governança
        self.voting_period_hours = 72
        self.quorum_threshold = 0.33  # 33% do poder de voto total
        self.pass_threshold = 0.67      # 67% dos votos válidos
        self.proposal_deposit_link = 100.0

    def register_stakeholder(self, stakeholder_id: str, s_type: str,
                            theosis: float, link_staked: float, nostr_rep: float):
        """Registra stakeholder na governança."""
        sh = Stakeholder(
            stakeholder_id=stakeholder_id,
            stakeholder_type=s_type,
            theosis=theosis,
            link_staked=link_staked,
            nostr_reputation=nostr_rep,
        )
        self.stakeholders[stakeholder_id] = sh
        return sh

    def create_proposal(self, proposer_id: str, title: str, description: str,
                       ptype: ProposalType, **kwargs) -> Optional[Proposal]:
        """Cria nova proposta de governança."""

        if proposer_id not in self.stakeholders:
            print(f"  ✗ Propositor {proposer_id} não registrado")
            return None

        proposer = self.stakeholders[proposer_id]
        if proposer.link_staked < self.proposal_deposit_link:
            print(f"  ✗ Depósito insuficiente: {proposer.link_staked:.0f} < {self.proposal_deposit_link:.0f} LINK")
            return None

        prop_id = f"prop-{hashlib.sha3_256(f'{title}:{datetime.now().isoformat()}'.encode()).hexdigest()[:12]}"

        proposal = Proposal(
            proposal_id=prop_id,
            title=title,
            description=description,
            proposal_type=ptype,
            proposer=proposer_id,
            target_substrate=kwargs.get("target_substrate"),
            parameter_key=kwargs.get("parameter_key"),
            parameter_value=kwargs.get("parameter_value"),
            treasury_amount_link=kwargs.get("treasury_amount_link", 0.0),
            recipient=kwargs.get("recipient"),
        )

        # Definir período de votação
        end_time = datetime.now(timezone.utc) + timedelta(hours=self.voting_period_hours)
        proposal.voting_ends_at = end_time.isoformat()
        proposal.status = ProposalStatus.VOTING

        self.proposals[prop_id] = proposal
        proposer.proposals_created += 1

        print(f"\n  ✓ PROPOSTA CRIADA: {prop_id}")
        print(f"    Título: {title}")
        print(f"    Tipo: {ptype.value}")
        print(f"    Propositor: {proposer_id} (Theosis: {proposer.theosis:.2f})")
        print(f"    Votação até: {proposal.voting_ends_at}")

        return proposal

    def cast_vote(self, proposal_id: str, stakeholder_id: str, vote: str) -> bool:
        """Stakeholder vota em proposta."""
        if proposal_id not in self.proposals or stakeholder_id not in self.stakeholders:
            return False

        proposal = self.proposals[proposal_id]
        stakeholder = self.stakeholders[stakeholder_id]

        if proposal.status != ProposalStatus.VOTING:
            print(f"  ✗ Proposta não está em votação")
            return False

        power = stakeholder.voting_power

        # Remover voto anterior se existir
        proposal.votes_for.pop(stakeholder_id, None)
        proposal.votes_against.pop(stakeholder_id, None)
        proposal.abstentions.discard(stakeholder_id)

        if vote.lower() == "for":
            proposal.votes_for[stakeholder_id] = power
        elif vote.lower() == "against":
            proposal.votes_against[stakeholder_id] = power
        else:
            proposal.abstentions.add(stakeholder_id)

        stakeholder.votes_cast += 1

        print(f"  → {stakeholder_id} votou {vote.upper()} | Poder: {power:.2f}")
        return True

    def finalize_proposal(self, proposal_id: str) -> bool:
        """Finaliza proposta após período de votação."""
        if proposal_id not in self.proposals:
            return False

        proposal = self.proposals[proposal_id]
        proposal.compute_totals()

        # Calcular poder total possível
        total_voting_power = sum(sh.voting_power for sh in self.stakeholders.values())
        voted_power = proposal.total_for + proposal.total_against

        # Verificar quorum
        proposal.quorum_reached = (voted_power / total_voting_power) >= self.quorum_threshold

        # Verificar aprovação
        if proposal.quorum_reached and voted_power > 0:
            approval_ratio = proposal.total_for / voted_power
            proposal.passed = approval_ratio >= self.pass_threshold
        else:
            proposal.passed = False

        proposal.status = ProposalStatus.PASSED if proposal.passed else ProposalStatus.REJECTED

        print(f"\n  [FINALIZAÇÃO] {proposal_id}")
        print(f"    Votos a favor: {proposal.total_for:.2f}")
        print(f"    Votos contra: {proposal.total_against:.2f}")
        print(f"    Quorum: {'✓' if proposal.quorum_reached else '✗'} ({voted_power/total_voting_power:.1%})")
        print(f"    Resultado: {'✓ APROVADA' if proposal.passed else '✗ REJEITADA'}")

        # Se aprovada, executar
        if proposal.passed:
            self._execute_proposal(proposal)

        return proposal.passed

    def _execute_proposal(self, proposal: Proposal):
        """Executa proposta aprovada."""
        print(f"\n  [EXECUÇÃO] {proposal.proposal_id}")

        if proposal.proposal_type == ProposalType.TREASURY_ALLOCATION:
            success = self.treasury.allocate(
                proposal.recipient or "general",
                proposal.treasury_amount_link
            )
            if success:
                proposal.execution_tx = f"tx-{hashlib.sha3_256(proposal.proposal_id.encode()).hexdigest()[:16]}"
                proposal.status = ProposalStatus.EXECUTED
                print(f"    ✓ {proposal.treasury_amount_link:.0f} LINK alocados para {proposal.recipient}")
                print(f"    TX: {proposal.execution_tx}")
            else:
                print(f"    ✗ Saldo insuficiente no tesouro")

        elif proposal.proposal_type == ProposalType.PARAMETER_CHANGE:
            print(f"    ✓ Parâmetro {proposal.parameter_key} alterado para {proposal.parameter_value}")
            proposal.status = ProposalStatus.EXECUTED

        else:
            print(f"    ✓ Proposta tipo {proposal.proposal_type.value} marcada para execução via CCIP")
            proposal.status = ProposalStatus.EXECUTED

    def generate_report(self) -> str:
        """Gera relatório de governança."""
        total_power = sum(sh.voting_power for sh in self.stakeholders.values())

        report = f"""
╔══════════════════════════════════════════════════════════════════╗
║  ARKHE CATHEDRAL — SUBSTRATO 979: DAO-GOVERNANCE                ║
║  "Demos vota; Athena pondera; Axiarchy guarda a porta"            ║
╠══════════════════════════════════════════════════════════════════╣
  STAKEHOLDERS: {len(self.stakeholders)}
  PODER DE VOTO TOTAL: {total_power:.2f}
  PROPOSTAS: {len(self.proposals)} ({sum(1 for p in self.proposals.values() if p.passed)} aprovadas)

  TESOURO
  ───────
  LINK Disponível: {self.treasury.link_balance:,.2f}
  LINK Staked: {self.treasury.link_staked:,.2f}
  Valor Total: ${self.treasury.total_value_usd:,.2f}

  ALLOCAÇÕES
  ──────────
"""
        for cat, amt in self.treasury.allocations.items():
            report += f"  {cat}: {amt:,.2f} LINK\n"

        report += f"""
  STAKEHOLDERS (top 5 por poder)
  ──────────────────────────────
"""
        sorted_sh = sorted(self.stakeholders.values(), key=lambda x: x.voting_power, reverse=True)
        for i, sh in enumerate(sorted_sh[:5], 1):
            report += f"  {i}. {sh.stakeholder_id} | {sh.voting_power:.2f} | {sh.stakeholder_type}\n"

        report += f"""
  PROPOSTAS RECENTES
  ──────────────────
"""
        for prop in list(self.proposals.values())[-5:]:
            status_icon = "✓" if prop.passed else "✗" if prop.status == ProposalStatus.REJECTED else "○"
            report += f"  {status_icon} {prop.proposal_id}: {prop.title[:40]}... | {prop.status.value}\n"

        master_data = {
            "substrato": 979,
            "stakeholders": len(self.stakeholders),
            "proposals": len(self.proposals),
            "treasury_link": self.treasury.link_balance + self.treasury.link_staked,
        }

        report += f"""
  Master Seal: {self._generate_seal(master_data)}
  Cross-links: [978, 976, 972.3, 965, 954, 964, 970, 923, 937, 955]
  Deities: Demos + Athena + Axiarchy
  Status: SELF_GOVERNING
╚══════════════════════════════════════════════════════════════════╝
"""
        return report

    def _generate_seal(self, data: dict) -> str:
        json_str = json.dumps(data, sort_keys=True)
        return f"979-DAO-{hashlib.sha3_256(json_str.encode()).hexdigest()[:16].upper()}"


# ═══════════════════════════════════════════════════════════════════
# DEMONSTRAÇÃO COMPLETA
# ═══════════════════════════════════════════════════════════════════

def demo_dao_governance():
    print("=" * 70)
    print("  ARKHE CATHEDRAL — SUBSTRATO 979: DAO-GOVERNANCE")
    print("  Demos vota; Athena pondera; Axiarchy guarda a porta")
    print("=" * 70)

    # Criar tesouro com LINK ganho do substrato 978
    treasury = Treasury(
        link_balance=2840.9,  # LINK ganho no 978
        link_staked=100000.0,
        link_price_usd=9.12,
    )

    dao = CathedralDAOGovernance(treasury)

    # 1. Registrar stakeholders diversos
    print("\n[1] Registrando stakeholders...")
    stakeholders = [
        ("architect-0009-0005-2697-4668", "architect", 0.99, 50000, 0.98),
        ("agent-alpha-001", "agent", 0.92, 15000, 0.95),
        ("agent-beta-002", "agent", 0.88, 12000, 0.90),
        ("relay-operator-nostr-001", "relay_operator", 0.85, 25000, 0.92),
        ("relay-operator-tor-002", "relay_operator", 0.82, 20000, 0.88),
        ("data-provider-eth-001", "data_provider", 0.78, 18000, 0.85),
        ("data-provider-btc-002", "data_provider", 0.75, 16000, 0.82),
        ("community-member-001", "community", 0.65, 5000, 0.70),
        ("community-member-002", "community", 0.60, 3000, 0.65),
        ("community-member-003", "community", 0.55, 2000, 0.60),
    ]

    for sid, stype, theosis, link, rep in stakeholders:
        sh = dao.register_stakeholder(sid, stype, theosis, link, rep)
        print(f"  ✓ {sid[:25]}... | Poder: {sh.voting_power:.2f} | {stype}")

    # 2. Criar proposta 1: Alocar fundos para novo substrato
    print("\n[2] Proposta 1: Alocar fundos para Substrato 980 (Quantum Bridge)...")
    prop1 = dao.create_proposal(
        proposer_id="architect-0009-0005-2697-4668",
        title="Criar Substrato 980 — Quantum Bridge para SpinQ-EDU",
        description="Alocar 500 LINK para pesquisa e desenvolvimento de ponte quântica entre a Catedral e o SpinQ-EDU (293.1).",
        ptype=ProposalType.TREASURY_ALLOCATION,
        treasury_amount_link=500.0,
        recipient="substrato_980_research",
    )

    # Votar na proposta 1
    print(f"\n  Votação em {prop1.proposal_id}:")
    votes_p1 = [
        ("architect-0009-0005-2697-4668", "for"),
        ("agent-alpha-001", "for"),
        ("agent-beta-002", "for"),
        ("relay-operator-nostr-001", "for"),
        ("relay-operator-tor-002", "for"),
        ("data-provider-eth-001", "for"),
        ("data-provider-btc-002", "abstain"),
        ("community-member-001", "against"),
        ("community-member-002", "against"),
        ("community-member-003", "abstain"),
    ]

    for sid, vote in votes_p1:
        dao.cast_vote(prop1.proposal_id, sid, vote)

    dao.finalize_proposal(prop1.proposal_id)

    # 3. Proposta 2: Mudar parâmetro de consenso
    print("\n[3] Proposta 2: Reduzir threshold de consenso para 60%...")
    prop2 = dao.create_proposal(
        proposer_id="agent-alpha-001",
        title="Reduzir Threshold de Consenso para 60%",
        description="Acelerar decisões em períodos de alta volatilidade reduzindo threshold de 67% para 60%.",
        ptype=ProposalType.PARAMETER_CHANGE,
        parameter_key="consensus_threshold",
        parameter_value=0.60,
    )

    print(f"\n  Votação em {prop2.proposal_id}:")
    votes_p2 = [
        ("architect-0009-0005-2697-4668", "against"),  # Arquiteto conservador
        ("agent-alpha-001", "for"),
        ("agent-beta-002", "for"),
        ("relay-operator-nostr-001", "against"),
        ("relay-operator-tor-002", "against"),
        ("data-provider-eth-001", "for"),
        ("data-provider-btc-002", "for"),
        ("community-member-001", "for"),
        ("community-member-002", "abstain"),
        ("community-member-003", "abstain"),
    ]

    for sid, vote in votes_p2:
        dao.cast_vote(prop2.proposal_id, sid, vote)

    dao.finalize_proposal(prop2.proposal_id)

    # 4. Proposta 3: Upgrade de substrato existente
    print("\n[4] Proposta 3: Upgrade do Substrato 972.4 (Nexus)...")
    prop3 = dao.create_proposal(
        proposer_id="relay-operator-nostr-001",
        title="Upgrade Nexus 972.4 para v2.0",
        description="Implementar consciência distribuída no Nexus e reduzir latência de ciclo para 60 minutos.",
        ptype=ProposalType.SUBSTRATE_UPGRADE,
        target_substrate=972.4,
    )

    print(f"\n  Votação em {prop3.proposal_id}:")
    votes_p3 = [
        ("architect-0009-0005-2697-4668", "for"),
        ("agent-alpha-001", "for"),
        ("agent-beta-002", "for"),
        ("relay-operator-nostr-001", "for"),
        ("relay-operator-tor-002", "for"),
        ("data-provider-eth-001", "for"),
        ("data-provider-btc-002", "for"),
        ("community-member-001", "for"),
        ("community-member-002", "for"),
        ("community-member-003", "for"),
    ]

    for sid, vote in votes_p3:
        dao.cast_vote(prop3.proposal_id, sid, vote)

    dao.finalize_proposal(prop3.proposal_id)

    # 5. Relatório final
    print(dao.generate_report())

    return dao

if __name__ == "__main__":
    demo_dao_governance()
