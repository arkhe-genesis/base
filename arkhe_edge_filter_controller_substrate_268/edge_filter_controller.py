#!/usr/bin/env python3
"""
Substrato 268 — Edge Filter Controller
Camada de filtragem não-discricionária executada em hardware FPGA
na rede DoubleZero. Remove spam e duplicados antes de entregar
mensagens aos substratos ARKHE.
"""

import hashlib
import time
from collections import deque
from dataclasses import dataclass
from enum import Enum
from typing import Dict, Set, List, Optional

class FilterAction(Enum):
    PASS = "pass"
    DROP_SPAM = "drop_spam"
    DROP_DUPLICATE = "drop_duplicate"
    DROP_TTL_EXPIRED = "drop_ttl_expired"
    DROP_MALFORMED = "drop_malformed"
    QUARANTINE = "quarantine"

@dataclass
class FilterResult:
    packet_id: str
    action: FilterAction
    reason: str
    processing_time_ns: int
    rules_matched: List[str]

class EdgeFilterController:
    """
    Controller de filtragem edge que opera em hardware FPGA na rede
    DoubleZero. Implementa regras não-discricionárias de filtragem
    para proteger substratos ARKHE contra spam e duplicados.

    Características:
    - Stateless para alta performance (milhões de pps)
    - Bloom filter para detecção de duplicados
    - Regex patterns para detecção de spam
    - TTL validation
    - Seal integrity verification
    """

    def __init__(self,
                 bloom_capacity: int = 10_000_000,
                 bloom_fp_rate: float = 0.001,
                 ttl_max_ms: int = 30000,
                 spam_patterns: List[str] = None):
        self.seen_packets: Set[str] = set()  # In FPGA: Bloom filter
        self.bloom_capacity = bloom_capacity
        self.bloom_fp_rate = bloom_fp_rate
        self.ttl_max_ms = ttl_max_ms
        self.spam_patterns = spam_patterns or [
            r"\b(viagra|casino|crypto scam)\b",
            r"\{\s*\}",  # Empty JSON objects (flood)
            r"\x00{10,}",   # Null byte padding
        ]
        self.metrics = {
            "total_processed": 0,
            "passed": 0,
            "dropped_spam": 0,
            "dropped_duplicate": 0,
            "dropped_ttl": 0,
            "dropped_malformed": 0,
            "quarantined": 0,
            "avg_processing_ns": 0
        }

    def filter_packet(self, packet: dict) -> FilterResult:
        """
        Aplica filtros não-discricionários a um pacote.
        Retorna PASS ou DROP com razão.
        """
        start = time.time_ns()
        rules_matched = []

        # 1. Malformed check
        if not self._is_wellformed(packet):
            return self._result(packet, FilterAction.DROP_MALFORMED,
                              "Malformed packet structure", start, ["malformed_check"])

        # 2. TTL check
        if packet.get("ttl_ms", 0) > self.ttl_max_ms or packet.get("ttl_ms", 0) <= 0:
            return self._result(packet, FilterAction.DROP_TTL_EXPIRED,
                              f"TTL {packet.get('ttl_ms')} out of bounds", start, ["ttl_check"])

        # 3. Seal integrity check
        if not self._verify_seal(packet):
            return self._result(packet, FilterAction.DROP_MALFORMED,
                              "Seal verification failed", start, ["seal_check"])

        # 4. Duplicate check (Bloom filter in FPGA)
        packet_hash = hashlib.sha3_256(
            f"{packet['source_substrate']}:{packet['timestamp_ns']}:{packet['payload'][:64]}".encode()
        ).hexdigest()

        if packet_hash in self.seen_packets:
            return self._result(packet, FilterAction.DROP_DUPLICATE,
                              f"Duplicate packet hash: {packet_hash[:16]}...", start, ["duplicate_check"])

        self.seen_packets.add(packet_hash)

        # 5. Spam pattern check
        payload_str = str(packet.get("payload", ""))
        for pattern in self.spam_patterns:
            import re
            if re.search(pattern, payload_str, re.IGNORECASE):
                return self._result(packet, FilterAction.DROP_SPAM,
                                  f"Spam pattern matched: {pattern}", start, ["spam_check"])

        # 6. Rate limiting per substrate
        if not self._check_rate_limit(packet.get("source_substrate", "unknown")):
            return self._result(packet, FilterAction.QUARANTINE,
                              "Rate limit exceeded", start, ["rate_limit"])

        # PASS
        return self._result(packet, FilterAction.PASS,
                          "All filters passed", start, ["all_clear"])

    def _is_wellformed(self, packet: dict) -> bool:
        required = ["packet_id", "source_substrate", "payload", "timestamp_ns", "seal"]
        return all(k in packet for k in required)

    def _verify_seal(self, packet: dict) -> bool:
        payload = packet.get("payload", b"")
        if isinstance(payload, str):
            payload = payload.encode()
        computed = hashlib.sha3_256(payload).hexdigest()
        return computed == packet.get("seal", "")

    def _check_rate_limit(self, substrate_id: str) -> bool:
        # In production: token bucket per substrate
        # Simplified: always pass for demo
        return True

    def _result(self, packet: dict, action: FilterAction, reason: str,
                start_ns: int, rules: List[str]) -> FilterResult:
        processing_time = time.time_ns() - start_ns

        # Update metrics
        self.metrics["total_processed"] += 1
        metric_key = {
            FilterAction.PASS: "passed",
            FilterAction.DROP_SPAM: "dropped_spam",
            FilterAction.DROP_DUPLICATE: "dropped_duplicate",
            FilterAction.DROP_TTL_EXPIRED: "dropped_ttl",
            FilterAction.DROP_MALFORMED: "dropped_malformed",
            FilterAction.QUARANTINE: "quarantined"
        }[action]
        self.metrics[metric_key] += 1

        # Rolling average
        n = self.metrics["total_processed"]
        self.metrics["avg_processing_ns"] = (
            (self.metrics["avg_processing_ns"] * (n - 1) + processing_time) / n
        )

        return FilterResult(
            packet_id=packet.get("packet_id", "unknown"),
            action=action,
            reason=reason,
            processing_time_ns=processing_time,
            rules_matched=rules
        )

    def get_metrics(self) -> dict:
        total = self.metrics["total_processed"]
        if total == 0:
            return self.metrics

        return {
            **self.metrics,
            "pass_rate": self.metrics["passed"] / total,
            "drop_rate": (total - self.metrics["passed"]) / total,
            "avg_processing_us": self.metrics["avg_processing_ns"] / 1000
        }

# ── Hardware Abstraction Layer (FPGA) ───────────────────────

class FPGAFilterOffload:
    """
    Abstraction layer para offload de filtragem para FPGA.
    Em produção, compila regras para bitstream e carrega no hardware.
    """

    def __init__(self, fpga_device: str = "/dev/fpga0"):
        self.device = fpga_device
        self.loaded = False

    def compile_rules(self, rules: List[str]) -> bytes:
        """Compila regras de filtragem para bitstream FPGA."""
        # Placeholder: em produção, usa Vitis HLS / OpenCL
        return b"FPGA_BITSTREAM_PLACEHOLDER"

    def load_bitstream(self, bitstream: bytes):
        """Carrega bitstream no dispositivo FPGA."""
        print(f"[268] Loading bitstream to {self.device}...")
        self.loaded = True

    def filter_batch(self, packets: List[dict]) -> List[FilterResult]:
        """Processa batch de pacotes em hardware FPGA."""
        if not self.loaded:
            raise RuntimeError("FPGA not loaded")
        # In production: DMA transfer + kernel execution
        return [EdgeFilterController().filter_packet(p) for p in packets]

if __name__ == "__main__":
    # Demo
    controller = EdgeFilterController()

    test_packets = [
        {"packet_id": "p1", "source_substrate": "923", "payload": b"valid",
         "timestamp_ns": 1234567890, "ttl_ms": 1000, "seal": hashlib.sha3_256(b"valid").hexdigest()},
        {"packet_id": "p2", "source_substrate": "923", "payload": b"valid",  # Duplicate
         "timestamp_ns": 1234567890, "ttl_ms": 1000, "seal": hashlib.sha3_256(b"valid").hexdigest()},
        {"packet_id": "p3", "source_substrate": "923", "payload": b"spam viagra casino",
         "timestamp_ns": 1234567891, "ttl_ms": 1000, "seal": hashlib.sha3_256(b"spam viagra casino").hexdigest()},
    ]

    for p in test_packets:
        result = controller.filter_packet(p)
        print(f"{result.packet_id}: {result.action.value} — {result.reason}")

    print(f"\nMetrics: {controller.get_metrics()}")