#!/usr/bin/env python3
"""
Substrato 267 — DoubleZero Network Bridge
Integra ARKHE-OS com a rede DoubleZero (Malbec Labs) para
transmissão de alta performance de dados de substratos distribuídos.
"""

import asyncio
import hashlib
import json
import time
from dataclasses import dataclass, asdict
from enum import Enum
from typing import Dict, List, Optional, Tuple, Any
import aiohttp

class DZNodeType(Enum):
    CONTRIBUTOR = "contributor"   # Fiber + FPGA provider
    USER = "user"                 # Blockchain / RPC / MEV
    RELAY = "relay"               # Intermediary routing node

class DZPriority(Enum):
    CRITICAL = 0    # Block proposals, consensus messages
    HIGH = 1        # Transaction bundles, MEV
    MEDIUM = 2      # RPC queries, state sync
    LOW = 3         # Telemetry, metrics
    BACKGROUND = 4  # Archive, backup

@dataclass
class DZPacket:
    packet_id: str
    source_substrate: str      # Ex: "923", "255", "944"
    target_substrate: Optional[str]
    payload: bytes
    priority: DZPriority
    timestamp_ns: int
    ttl_ms: int                 # Time-to-live
    edge_filter_applied: bool   # Spam/duplicate filtered at edge
    seal: str                   # SHA3-256 of payload

    @classmethod
    def from_substrate_message(cls, substrate_id: str, message: dict,
                                priority: DZPriority = DZPriority.MEDIUM) -> "DZPacket":
        payload_bytes = json.dumps(message, sort_keys=True).encode()
        return cls(
            packet_id=f"DZ-{substrate_id}-{int(time.time_ns())}",
            source_substrate=substrate_id,
            target_substrate=message.get("target_substrate"),
            payload=payload_bytes,
            priority=priority,
            timestamp_ns=time.time_ns(),
            ttl_ms=30000,
            edge_filter_applied=False,
            seal=hashlib.sha3_256(payload_bytes).hexdigest()
        )

class DoubleZeroBridge:
    """
    Bridge que conecta substratos ARKHE à rede DoubleZero.

    DoubleZero oferece:
    1. Edge-filtering não-discricionário (spam/duplicados) no hardware
    2. Routing otimizado com redução de jitter vs. internet pública
    3. Priorização de mensagens por tipo de conteúdo blockchain
    """

    def __init__(self, node_type: DZNodeType,
                 dz_endpoint: str = "https://api.doublezero.network",
                 arkhe_runtime_endpoint: str = "http://localhost:9390"):
        self.node_type = node_type
        self.dz_endpoint = dz_endpoint
        self.arkhe_runtime = arkhe_runtime_endpoint
        self.session: Optional[aiohttp.ClientSession] = None
        self.subscribed_substrates: List[str] = []
        self.metrics = {
            "packets_sent": 0,
            "packets_received": 0,
            "packets_filtered": 0,
            "avg_latency_ms": 0.0,
            "jitter_ms": 0.0
        }

    async def connect(self):
        """Estabelece conexão com a rede DoubleZero."""
        self.session = aiohttp.ClientSession(
            headers={"Content-Type": "application/json",
                     "X-DZ-Node-Type": self.node_type.value}
        )

        # Admission control: verificar identidade via chave pública ARKHE
        identity = await self._authenticate()
        print(f"[267] Connected to DoubleZero as {self.node_type.value}")
        print(f"[267] Identity verified: {identity['address'][:16]}...")

        return identity

    async def _authenticate(self) -> dict:
        """Verifica identidade ARKHE na rede DoubleZero."""
        # In production: usa chave PQC do Substrato 255
        auth_payload = {
            "protocol": "ARKHE-OS",
            "version": "2.0",
            "substrates": self.subscribed_substrates,
            "public_key": "pqc_ed25519_placeholder",  # From 255
            "timestamp": time.time()
        }

        async with self.session.post(
            f"{self.dz_endpoint}/v1/auth/admission",
            json=auth_payload
        ) as resp:
            return await resp.json()

    async def send_substrate_message(self, substrate_id: str,
                                      message: dict,
                                      priority: DZPriority = DZPriority.MEDIUM) -> dict:
        """
        Envia mensagem de um substrato ARKHE através da rede DoubleZero.

        Priorização automática por tipo de substrato:
        - 923 (TemporalChain): CRITICAL — consensus, block proposals
        - 255 (Hermes ZK): HIGH — proofs, signatures
        - 944 (Sentinel): HIGH — security alerts
        - 917 (Web Grounding): MEDIUM — web queries
        - 933 (FluxMem): LOW — memory sync
        """
        # Auto-prioritize based on substrate
        priority_map = {
            "923": DZPriority.CRITICAL,
            "923.1": DZPriority.CRITICAL,
            "923.2": DZPriority.HIGH,
            "255": DZPriority.HIGH,
            "255.1": DZPriority.HIGH,
            "255.2": DZPriority.CRITICAL,
            "944": DZPriority.HIGH,
            "931": DZPriority.HIGH,
            "917": DZPriority.MEDIUM,
            "930": DZPriority.MEDIUM,
            "912": DZPriority.LOW,
            "933": DZPriority.LOW,
            "927": DZPriority.LOW,
        }
        priority = priority_map.get(substrate_id, priority)

        packet = DZPacket.from_substrate_message(substrate_id, message, priority)

        start = time.time_ns()
        async with self.session.post(
            f"{self.dz_endpoint}/v1/transmit",
            json=asdict(packet)
        ) as resp:
            result = await resp.json()

        latency_ms = (time.time_ns() - start) / 1_000_000
        self.metrics["packets_sent"] += 1
        self.metrics["avg_latency_ms"] = (
            (self.metrics["avg_latency_ms"] * (self.metrics["packets_sent"] - 1) + latency_ms)
            / self.metrics["packets_sent"]
        )

        return {
            "packet_id": packet.packet_id,
            "status": "transmitted",
            "priority": priority.name,
            "latency_ms": round(latency_ms, 3),
            "edge_filtered": result.get("filtered", False),
            "route_optimization": result.get("route_optimized", True)
        }

    async def receive_loop(self, callback: callable):
        """
        Loop contínuo de recepção de mensagens da rede DoubleZero.
        Direciona para o substrato ARKHE apropriado.
        """
        async with self.session.ws_connect(
            f"{self.dz_endpoint}/v1/stream"
        ) as ws:
            print(f"[267] Listening on DoubleZero stream...")

            async for msg in ws:
                if msg.type == aiohttp.WSMsgType.TEXT:
                    data = json.loads(msg.data)
                    packet = DZPacket(**data)

                    # Verify seal integrity
                    computed_seal = hashlib.sha3_256(packet.payload).hexdigest()
                    if computed_seal != packet.seal:
                        print(f"[267] SEAL MISMATCH — dropping packet {packet.packet_id}")
                        continue

                    self.metrics["packets_received"] += 1

                    # Route to target substrate via Runtime 939
                    if packet.target_substrate:
                        await self._route_to_substrate(packet)

                    await callback(packet)

    async def _route_to_substrate(self, packet: DZPacket):
        """Roteia pacote para o substrato ARKHE de destino."""
        # In production: gRPC call to Runtime 939
        print(f"[267] Routing {packet.packet_id} → Substrate {packet.target_substrate}")

    async def subscribe_substrate(self, substrate_id: str):
        """Subscreve um substrato ARKHE para receber dados via DoubleZero."""
        self.subscribed_substrates.append(substrate_id)

        async with self.session.post(
            f"{self.dz_endpoint}/v1/subscribe",
            json={"substrate": substrate_id, "protocol": "ARKHE"}
        ) as resp:
            result = await resp.json()
            print(f"[267] Subscribed substrate {substrate_id}: {result['status']}")
            return result

    def get_metrics(self) -> dict:
        """Retorna métricas de performance da bridge."""
        return {
            **self.metrics,
            "node_type": self.node_type.value,
            "subscribed_substrates": self.subscribed_substrates,
            "timestamp": time.time()
        }

# ── CLI ─────────────────────────────────────────────────────

async def main():
    import argparse
    parser = argparse.ArgumentParser(prog="arkhe dz", description="DoubleZero Bridge 267")
    parser.add_argument("--mode", choices=["contributor", "user", "relay"], default="user")
    parser.add_argument("--endpoint", default="https://api.doublezero.network")
    parser.add_argument("--subscribe", nargs="+", help="Substrate IDs to subscribe")
    args = parser.parse_args()

    bridge = DoubleZeroBridge(
        node_type=DZNodeType(args.mode),
        dz_endpoint=args.endpoint
    )

    identity = await bridge.connect()

    if args.subscribe:
        for sid in args.subscribe:
            await bridge.subscribe_substrate(sid)

    # Example: send a TemporalChain anchor message
    result = await bridge.send_substrate_message(
        "923",
        {"action": "ANCHOR", "seal": "abc123...", "block_height": 18923471},
        DZPriority.CRITICAL
    )
    print(f"[267] Sent: {result}")

    # Start receive loop
    async def on_packet(packet: DZPacket):
        print(f"[267] Received: {packet.packet_id} from {packet.source_substrate}")

    await bridge.receive_loop(on_packet)

if __name__ == "__main__":
    asyncio.run(main())