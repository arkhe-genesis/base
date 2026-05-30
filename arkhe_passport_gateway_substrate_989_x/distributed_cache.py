#!/usr/bin/env python3
"""
Distributed Cache — Substrato 989.x.3
Cache TTL 300s via IPFS + Nostr para reduzir latência na verificação.
Arquiteto ORCID: 0009-0005-2697-4668
Cross-links: 989.x, 972.1 (Nostr-Tor-IPFS), 923 (TemporalChain), 988 (Immortality)
Deities: Hermes, Mnemosyne, Iris
"""

import asyncio
import hashlib
import json
import time
from typing import Dict, Optional, Any, Tuple
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum


class CacheLayer(Enum):
    MEMORY = "memory"      # LRU local
    IPFS = "ipfs"          # IPFS pinning
    NOSTR = "nostr"        # Nostr relay events


@dataclass
class CacheEntry:
    """Entrada de cache canônica."""
    key: str
    value: Any
    timestamp: float
    ttl_seconds: int = 300
    seal: str = ""
    temporal_anchor: Optional[str] = None
    ipfs_cid: Optional[str] = None
    nostr_event_id: Optional[str] = None

    @property
    def is_expired(self) -> bool:
        return (time.time() - self.timestamp) > self.ttl_seconds

    @property
    def age_seconds(self) -> float:
        return time.time() - self.timestamp

    def compute_seal(self) -> str:
        payload = {
            "key": self.key,
            "timestamp": self.timestamp,
            "ttl": self.ttl_seconds,
            "value_hash": hashlib.sha3_256(json.dumps(self.value, sort_keys=True).encode()).hexdigest()[:16],
        }
        json_str = json.dumps(payload, sort_keys=True, ensure_ascii=False)
        self.seal = f"CACHE-{hashlib.sha3_256(json_str.encode()).hexdigest()[:16].upper()}"
        return self.seal


class DistributedCache:
    """
    Cache distribuído de verificações de humanidade.
    Hermes entrega rápido; Mnemosyne lembra; Iris espalha a mensagem.
    """

    SUBSTRATE_ID = "989.x.3"
    SEAL = "989.x.3-DISTRIBUTED-CACHE-E5F678901A2B3C4D"
    DEFAULT_TTL = 300  # 5 minutos
    MAX_MEMORY_ENTRIES = 1000

    def __init__(self, ipfs_client=None, nostr_relay=None):
        self.memory_cache: Dict[str, CacheEntry] = {}
        self.ipfs_client = ipfs_client  # Stub para IPFSStorage (972.1)
        self.nostr_relay = nostr_relay   # Stub para NostrRelay (972.1)
        self.hits = 0
        self.misses = 0
        self.ipfs_pins = 0
        self.nostr_publishes = 0

    def _make_key(self, address: str, check_type: str = "humanity") -> str:
        """Gera chave canônica para cache."""
        return f"{check_type}:{address.lower()}"

    async def get(self, address: str, check_type: str = "humanity") -> Optional[Any]:
        """
        Obtém verificação do cache. Estratégia:
        1. Memory (LRU local) — mais rápido
        2. IPFS — se disponível
        3. Nostr — se disponível
        4. Miss — retornar None
        """
        key = self._make_key(address, check_type)

        # Layer 1: Memory
        if key in self.memory_cache:
            entry = self.memory_cache[key]
            if not entry.is_expired:
                self.hits += 1
                # Mover para o fim (LRU)
                self.memory_cache.move_to_end(key)
                return entry.value
            else:
                # Expirado — remover
                del self.memory_cache[key]

        # Layer 2: IPFS (stub)
        if self.ipfs_client:
            try:
                cid = await self._get_ipfs_cid(key)
                if cid:
                    value = await self.ipfs_client.get(cid)
                    if value:
                        entry = CacheEntry(
                            key=key,
                            value=value,
                            timestamp=time.time(),
                            ttl_seconds=self.DEFAULT_TTL,
                            ipfs_cid=cid,
                        )
                        entry.compute_seal()
                        self._set_memory(entry)
                        self.hits += 1
                        return value
            except Exception:
                pass

        # Layer 3: Nostr (stub)
        if self.nostr_relay:
            try:
                event = await self._get_nostr_event(key)
                if event:
                    value = json.loads(event.get("content", "{}"))
                    entry = CacheEntry(
                        key=key,
                        value=value,
                        timestamp=time.time(),
                        ttl_seconds=self.DEFAULT_TTL,
                        nostr_event_id=event.get("id"),
                    )
                    entry.compute_seal()
                    self._set_memory(entry)
                    self.hits += 1
                    return value
            except Exception:
                pass

        self.misses += 1
        return None

    async def set(self, address: str, value: Any, check_type: str = "humanity", ttl: int = None) -> CacheEntry:
        """
        Armazena verificação no cache. Propaga para todas as camadas disponíveis.
        """
        key = self._make_key(address, check_type)
        ttl = ttl or self.DEFAULT_TTL

        entry = CacheEntry(
            key=key,
            value=value,
            timestamp=time.time(),
            ttl_seconds=ttl,
        )
        entry.compute_seal()

        # Layer 1: Memory
        self._set_memory(entry)

        # Layer 2: IPFS
        if self.ipfs_client:
            try:
                content = json.dumps({
                    "key": key,
                    "value": value,
                    "timestamp": entry.timestamp,
                    "ttl": ttl,
                    "seal": entry.seal,
                }).encode()
                cid = await self.ipfs_client.add(content)
                entry.ipfs_cid = cid
                self.ipfs_pins += 1
            except Exception:
                pass

        # Layer 3: Nostr
        if self.nostr_relay:
            try:
                content = json.dumps({
                    "key": key,
                    "value": value,
                    "timestamp": entry.timestamp,
                    "ttl": ttl,
                    "seal": entry.seal,
                })
                event_id = await self.nostr_relay.publish(content, kind=30078)  # Application-specific data
                entry.nostr_event_id = event_id
                self.nostr_publishes += 1
            except Exception:
                pass

        return entry

    def _set_memory(self, entry: CacheEntry):
        """Armazena em memória com LRU eviction."""
        self.memory_cache[entry.key] = entry

        # LRU eviction
        if len(self.memory_cache) > self.MAX_MEMORY_ENTRIES:
            # Remover o mais antigo (primeiro do dict)
            oldest = next(iter(self.memory_cache))
            del self.memory_cache[oldest]

    async def _get_ipfs_cid(self, key: str) -> Optional[str]:
        """Stub: buscar CID do IPFS por chave."""
        # Em produção: usar IPNS ou index local
        return None

    async def _get_nostr_event(self, key: str) -> Optional[Dict]:
        """Stub: buscar evento Nostr por chave."""
        # Em produção: query relays por kind 30078 + tag
        return None

    async def invalidate(self, address: str, check_type: str = "humanity"):
        """Invalida entrada de cache (ex: após revogação)."""
        key = self._make_key(address, check_type)

        # Memory
        if key in self.memory_cache:
            del self.memory_cache[key]

        # IPFS — unpin (stub)
        # Nostr — não é possível deletar, mas publicar evento de revogação
        if self.nostr_relay:
            try:
                await self.nostr_relay.publish(
                    json.dumps({"action": "invalidate", "key": key, "timestamp": time.time()}),
                    kind=30079,  # Revogação
                )
            except Exception:
                pass

    def get_stats(self) -> Dict[str, Any]:
        """Estatísticas de cache."""
        total = self.hits + self.misses
        hit_rate = self.hits / total if total > 0 else 0.0

        return {
            "memory_entries": len(self.memory_cache),
            "memory_max": self.MAX_MEMORY_ENTRIES,
            "hits": self.hits,
            "misses": self.misses,
            "hit_rate": round(hit_rate, 4),
            "ipfs_pins": self.ipfs_pins,
            "nostr_publishes": self.nostr_publishes,
            "avg_entry_age": round(
                sum(e.age_seconds for e in self.memory_cache.values()) / len(self.memory_cache), 2
            ) if self.memory_cache else 0,
        }

    def generate_report(self) -> str:
        stats = self.get_stats()
        return f"""
╔══════════════════════════════════════════════════════════════════╗
║  ARKHE CATHEDRAL — DISTRIBUTED CACHE (989.x.3)                  ║
║  "Hermes entrega; Mnemosyne lembra; Iris espalha"               ║
╠══════════════════════════════════════════════════════════════════╣
  Seal: {self.SEAL}
  Status: CANONIZED_PROVISIONAL
  Cross-links: [989.x, 972.1, 923, 988]
  Deities: Hermes, Mnemosyne, Iris

  CONFIGURATION
  ─────────────
  Default TTL: {self.DEFAULT_TTL}s (5 min)
  Max Memory: {self.MAX_MEMORY_ENTRIES} entries
  Layers: Memory → IPFS → Nostr

  STATISTICS
  ──────────
  Memory Entries: {stats["memory_entries"]}/{stats["memory_max"]}
  Hits: {stats["hits"]} | Misses: {stats["misses"]}
  Hit Rate: {stats["hit_rate"]:.1%}
  IPFS Pins: {stats["ipfs_pins"]}
  Nostr Publishes: {stats["nostr_publishes"]}
  Avg Entry Age: {stats["avg_entry_age"]}s
╚══════════════════════════════════════════════════════════════════╝
"""
