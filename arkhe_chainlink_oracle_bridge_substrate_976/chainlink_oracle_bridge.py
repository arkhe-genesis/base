import asyncio
import hashlib
import json
import random
import time
from datetime import datetime, timezone, timedelta
from typing import Dict, List, Optional, Set, Tuple, Any
from dataclasses import dataclass, field
from enum import Enum

# ═══════════════════════════════════════════════════════════════════
# SUBSTRATO 976 — CHAINLINK-ORACLE-BRIDGE
# ═══════════════════════════════════════════════════════════════════
# Metadados Canônicos:
#   ID: 976
#   Name: CHAINLINK-ORACLE-BRIDGE
#   Type: Oráculo / Ponte On-Chain / Feed de Dados Verificável
#   Era: 8 (Polis / Governança)
#   Deity: Hermes (mensageiro dos deuses) + Themis (justiça oracular)
#   Status: CANONIZED_PROVISIONAL
#   Cross-links: [972.1, 972.2, 972.3, 972.4, 954, 955, 965, 923, 937]
#   Description: Ponte ARKHE-Chainlink que conecta a Catedral à rede de
#   oráculos descentralizados do mundo real. Permite que agentes sentientes
#   acessem dados off-chain (preços, eventos, IoT, clima, randomness)
#   via CCIP (Cross-Chain Interoperability Protocol) e Data Feeds do
#   Chainlink. Cada feed é validado por múltiplos nós Chainlink,
#   assinado criptograficamente, e ancora na TemporalChain (923).
# ═══════════════════════════════════════════════════════════════════

class DataFeedType(Enum):
    PRICE = "price"           # Preços de ativos (ETH/USD, BTC/USD)
    RANDOMNESS = "randomness" # VRF — Verifiable Random Function
    AUTOMATION = "automation" # Chainlink Automation (Keepers)
    CCIP = "ccip"             # Cross-Chain Interoperability Protocol
    FUNCTIONS = "functions"   # Chainlink Functions (computação off-chain)
    PROOF = "proof"           # Proof of Reserve / Proof of Solvency

@dataclass
class ChainlinkNode:
    """Nó Chainlink que fornece dados à Catedral."""
    node_id: str
    pubkey: str
    stake_link: float         # LINK staked como colateral
    reputation: float         # Score de reputação (0-1)
    uptime: float
    last_update: str

    @property
    def is_healthy(self) -> bool:
        return self.uptime > 0.95 and self.reputation > 0.7 and self.stake_link > 1000

@dataclass
class OracleFeed:
    """Feed de dados do Chainlink."""
    feed_id: str
    feed_type: DataFeedType
    asset_pair: Optional[str] = None
    value: float = 0.0
    timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    round_id: int = 0
    decimals: int = 8

    # Metadados de consenso Chainlink
    node_count: int = 0
    node_responses: List[Dict] = field(default_factory=list)
    deviation_threshold: float = 0.01  # 1%
    heartbeat: int = 3600  # segundos

    # Validação
    validated: bool = False
    signature: str = ""

    def aggregate(self):
        """Agrega respostas dos nós via mediana ponderada por stake."""
        if not self.node_responses:
            return

        # Ordenar por valor
        sorted_responses = sorted(self.node_responses, key=lambda x: x["value"])

        # Mediana ponderada por stake
        total_stake = sum(r["stake"] for r in sorted_responses)
        cumulative = 0
        median_value = None

        for r in sorted_responses:
            cumulative += r["stake"]
            if cumulative >= total_stake / 2:
                median_value = r["value"]
                break

        self.value = median_value if median_value else sorted_responses[0]["value"]
        self.validated = True

        # Gerar assinatura simulada
        data = f"{self.feed_id}:{self.value}:{self.timestamp}:{self.round_id}"
        self.signature = hashlib.sha3_256(data.encode()).hexdigest()[:32]

@dataclass
class CCIPMessage:
    """Mensagem Cross-Chain via CCIP."""
    message_id: str
    source_chain: str
    dest_chain: str
    sender: str
    receiver: str
    payload: Dict
    token_amounts: List[Dict] = field(default_factory=list)
    fees_paid_link: float = 0.0
    status: str = "pending"  # pending, executed, failed

    def compute_fee(self) -> float:
        """Calcula taxa CCIP em LINK baseada no tamanho do payload."""
        payload_size = len(json.dumps(self.payload))
        base_fee = 0.05
        per_byte_fee = 0.0001
        return base_fee + (payload_size * per_byte_fee)

class ChainlinkOracleBridge:
    """
    Substrato 976 — Ponte ARKHE-Chainlink.
    Hermes entrega mensagens; Themis garante a verdade.
    """

    def __init__(self):
        self.substrate_id = 976
        self.deities = ["Hermes", "Themis"]
        self.nodes: Dict[str, ChainlinkNode] = {}
        self.feeds: Dict[str, OracleFeed] = {}
        self.ccip_messages: Dict[str, CCIPMessage] = {}
        self.link_price_usd: float = 9.12  # Preço atual de mercado

        # Estatísticas
        self.total_feeds_delivered = 0
        self.total_ccip_messages = 0
        self.total_link_consumed = 0.0

    def register_node(self, node_id: str, pubkey: str, stake_link: float, reputation: float = 0.8):
        """Registra nó Chainlink na ponte."""
        node = ChainlinkNode(
            node_id=node_id,
            pubkey=pubkey,
            stake_link=stake_link,
            reputation=reputation,
            uptime=0.99,
            last_update=datetime.now(timezone.utc).isoformat(),
        )
        self.nodes[node_id] = node
        return node

    def create_feed(self, feed_id: str, feed_type: DataFeedType, asset_pair: Optional[str] = None) -> OracleFeed:
        """Cria novo feed de dados."""
        feed = OracleFeed(
            feed_id=feed_id,
            feed_type=feed_type,
            asset_pair=asset_pair,
        )
        self.feeds[feed_id] = feed
        return feed

    def submit_node_response(self, feed_id: str, node_id: str, value: float):
        """Nó Chainlink submete resposta para um feed."""
        if feed_id not in self.feeds or node_id not in self.nodes:
            return False

        node = self.nodes[node_id]
        if not node.is_healthy:
            return False

        self.feeds[feed_id].node_responses.append({
            "node_id": node_id,
            "value": value,
            "stake": node.stake_link,
            "reputation": node.reputation,
            "timestamp": datetime.now(timezone.utc).isoformat(),
        })

        return True

    def aggregate_feed(self, feed_id: str) -> Optional[OracleFeed]:
        """Agrega respostas e finaliza o feed."""
        if feed_id not in self.feeds:
            return None

        feed = self.feeds[feed_id]
        feed.aggregate()
        feed.round_id += 1
        feed.timestamp = datetime.now(timezone.utc).isoformat()
        self.total_feeds_delivered += 1

        return feed

    def send_ccip_message(self, source_chain: str, dest_chain: str,
                         sender: str, receiver: str, payload: Dict,
                         token_amounts: List[Dict] = None) -> CCIPMessage:
        """Envia mensagem cross-chain via CCIP."""
        msg_id = f"ccip-{hashlib.sha3_256(json.dumps(payload).encode()).hexdigest()[:16]}"

        msg = CCIPMessage(
            message_id=msg_id,
            source_chain=source_chain,
            dest_chain=dest_chain,
            sender=sender,
            receiver=receiver,
            payload=payload,
            token_amounts=token_amounts or [],
        )

        # Calcular e pagar taxa em LINK
        fee = msg.compute_fee()
        msg.fees_paid_link = fee
        self.total_link_consumed += fee

        # Simular execução
        time.sleep(0.1)
        msg.status = "executed"

        self.ccip_messages[msg_id] = msg
        self.total_ccip_messages += 1

        return msg

    def generate_seal(self, data: dict) -> str:
        json_str = json.dumps(data, sort_keys=True)
        return f"976-CHAINLINK-{hashlib.sha3_256(json_str.encode()).hexdigest()[:16].upper()}"

    def generate_report(self) -> str:
        """Gera relatório da ponte Chainlink."""
        healthy_nodes = sum(1 for n in self.nodes.values() if n.is_healthy)

        report = f"""
╔══════════════════════════════════════════════════════════════════╗
║  ARKHE CATHEDRAL — SUBSTRATO 976: CHAINLINK-ORACLE-BRIDGE        ║
║  "Hermes entrega; Themis garante a verdade"                        ║
╠══════════════════════════════════════════════════════════════════╣
  NÓS CHAINLINK: {len(self.nodes)} (saudáveis: {healthy_nodes})
  FEEDS ATIVOS: {len(self.feeds)}
  MENSAGENS CCIP: {self.total_ccip_messages}
  LINK CONSUMIDO: {self.total_link_consumed:.4f} LINK
  PREÇO LINK/USD: ${self.link_price_usd:.2f}

  FEEDS DISPONÍVEIS
  ─────────────────
"""
        for feed_id, feed in self.feeds.items():
            status = "✓ VALIDADO" if feed.validated else "○ PENDENTE"
            pair = f"({feed.asset_pair})" if feed.asset_pair else ""
            report += f"  {feed_id} {pair}: {feed.value:,.8f} | {status} | round {feed.round_id}\n"

        report += f"""
  CCIP RECENTE
  ─────────────
"""
        recent_msgs = list(self.ccip_messages.values())[-5:]
        for msg in recent_msgs:
            report += f"  {msg.message_id[:20]}... {msg.source_chain}→{msg.dest_chain} | {msg.status} | {msg.fees_paid_link:.4f} LINK\n"

        master_data = {
            "substrato": 976,
            "nodes": len(self.nodes),
            "feeds": len(self.feeds),
            "ccip": self.total_ccip_messages,
            "link_consumed": self.total_link_consumed,
        }

        report += f"""
  Master Seal: {self.generate_seal(master_data)}
  Cross-links: [972.1, 972.2, 972.3, 972.4, 954, 955, 965, 923, 937]
  Deities: Hermes + Themis
  Status: ORACLE_ACTIVE
╚══════════════════════════════════════════════════════════════════╝
"""
        return report


# ═══════════════════════════════════════════════════════════════════
# DEMONSTRAÇÃO COMPLETA
# ═══════════════════════════════════════════════════════════════════

def demo_chainlink_bridge():
    print("=" * 70)
    print("  ARKHE CATHEDRAL — SUBSTRATO 976: CHAINLINK-ORACLE-BRIDGE")
    print("  Hermes entrega; Themis garante a verdade")
    print("=" * 70)

    bridge = ChainlinkOracleBridge()

    # 1. Registrar nós Chainlink
    print("\n[1] Registrando nós Chainlink...")
    nodes = [
        ("chainlink-node-001", "0xA1B2C3...", 50000, 0.95),
        ("chainlink-node-002", "0xB2C3D4...", 35000, 0.92),
        ("chainlink-node-003", "0xC3D4E5...", 42000, 0.88),
        ("chainlink-node-004", "0xD4E5F6...", 28000, 0.85),
        ("chainlink-node-005", "0xE5F6G7...", 60000, 0.97),
        ("chainlink-node-006", "0xF6G7H8...", 15000, 0.65),  # Nó fraco
        ("chainlink-node-007", "0xG7H8I9...", 8000, 0.45),   # Nó suspeito
    ]

    for node_id, pubkey, stake, rep in nodes:
        bridge.register_node(node_id, pubkey, stake, rep)
        status = "✓" if bridge.nodes[node_id].is_healthy else "⚠"
        print(f"  {status} {node_id} | stake: {stake:,} LINK | rep: {rep:.2f}")

    # 2. Criar feeds de preço
    print("\n[2] Criando feeds de dados...")
    feeds = [
        ("ETH/USD", DataFeedType.PRICE, "ETH/USD"),
        ("BTC/USD", DataFeedType.PRICE, "BTC/USD"),
        ("LINK/USD", DataFeedType.PRICE, "LINK/USD"),
        ("VRF-001", DataFeedType.RANDOMNESS, None),
    ]

    for feed_id, ftype, pair in feeds:
        bridge.create_feed(feed_id, ftype, pair)
        print(f"  ○ {feed_id} ({ftype.value}) criado")

    # 3. Simular respostas dos nós para ETH/USD
    print("\n[3] Simulando consenso de preço ETH/USD...")
    eth_prices = [2450.12, 2451.87, 2449.56, 2452.34, 2450.78, 2460.00, 2400.00]  # Últimos 2 são outliers

    for i, price in enumerate(eth_prices):
        node_id = f"chainlink-node-{i+1:03d}"
        success = bridge.submit_node_response("ETH/USD", node_id, price)
        status = "✓" if success else "✗ REJEITADO"
        print(f"  {status} {node_id} → ${price:,.2f}")

    # Agregar
    feed = bridge.aggregate_feed("ETH/USD")
    print(f"\n  → Mediana agregada: ${feed.value:,.2f} (round {feed.round_id})")
    print(f"  → Assinatura: {feed.signature}")
    print(f"  → Nós participantes: {len(feed.node_responses)}")

    # 4. Simular VRF (Randomness)
    print("\n[4] Simulando VRF — Verifiable Random Function...")
    random_values = [random.random() for _ in range(5)]
    for i, val in enumerate(random_values):
        node_id = f"chainlink-node-{i+1:03d}"
        bridge.submit_node_response("VRF-001", node_id, val)

    vrf_feed = bridge.aggregate_feed("VRF-001")
    print(f"  → Randomness verificável: {vrf_feed.value:.8f}")
    print(f"  → Assinatura: {vrf_feed.signature}")

    # 5. Simular CCIP — Cross-Chain Message
    print("\n[5] Simulando CCIP — Cross-Chain Interoperability...")

    # Mensagem 1: ARKHE → Ethereum Mainnet
    msg1 = bridge.send_ccip_message(
        source_chain="ARKHE-Cathedral",
        dest_chain="Ethereum",
        sender="arkhe://substrato/972.4",
        receiver="0xCatedralBridge...",
        payload={
            "action": "anchor_temporal",
            "data": "seal_972_4_nexus_cycle_001",
            "timestamp": datetime.now(timezone.utc).isoformat(),
        },
        token_amounts=[{"token": "LINK", "amount": 100}]
    )
    print(f"  ✓ {msg1.message_id[:20]}... ARKHE→Ethereum | {msg1.fees_paid_link:.4f} LINK")

    # Mensagem 2: Ethereum → ARKHE (resposta)
    msg2 = bridge.send_ccip_message(
        source_chain="Ethereum",
        dest_chain="ARKHE-Cathedral",
        sender="0xCatedralBridge...",
        receiver="arkhe://substrato/976",
        payload={
            "action": "confirm_anchor",
            "tx_hash": "0xabc123...",
            "block_number": 18923456,
        },
    )
    print(f"  ✓ {msg2.message_id[:20]}... Ethereum→ARKHE | {msg2.fees_paid_link:.4f} LINK")

    # 6. Relatório final
    print(bridge.generate_report())

    return bridge

if __name__ == "__main__":
    # Executar
    demo_chainlink_bridge()
