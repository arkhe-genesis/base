#!/usr/bin/env python3
"""
Substrato 269 — DZ Routing Engine
Motor de routing otimizado para a rede DoubleZero que minimiza
jitter e maximiza throughput para mensagens de substratos ARKHE.
"""

import heapq
import time
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple, Set
from enum import Enum

class RouteMetric(Enum):
    LATENCY = "latency"           # Minimize RTT
    JITTER = "jitter"             # Minimize variation
    THROUGHPUT = "throughput"     # Maximize bandwidth
    RELIABILITY = "reliability"   # Maximize uptime
    COST = "cost"                 # Minimize token cost

@dataclass
class DZNode:
    node_id: str
    location: str                  # Datacenter region
    capacity_gbps: float
    latency_ms: Dict[str, float]  # {node_id: latency}
    jitter_ms: Dict[str, float]   # {node_id: jitter}
    reliability: float             # 0.0-1.0
    contributor_stake: float     # Token stake

@dataclass
class Route:
    path: List[str]               # Ordered list of node IDs
    total_latency_ms: float
    total_jitter_ms: float
    min_capacity_gbps: float
    reliability: float
    estimated_cost: float
    priority: int                 # Lower = higher priority

class DZRoutingEngine:
    """
    Motor de routing que implementa pathfinding otimizado para a
    rede DoubleZero. Diferente da internet pública, o routing DZ
    é otimizado para velocidade de propagação de blocos e mensagens
    de consensus entre substratos ARKHE distribuídos.
    """

    def __init__(self):
        self.nodes: Dict[str, DZNode] = {}
        self.routes_cache: Dict[Tuple[str, str, RouteMetric], Route] = {}
        self.substrate_routing_table: Dict[str, List[str]] = {
            # Substrate ID → preferred egress nodes
            "923": ["dz-nyc-01", "dz-ldn-01", "dz-sgp-01"],   # TemporalChain
            "255": ["dz-nyc-01", "dz-ams-01"],                # Hermes ZK
            "944": ["dz-nyc-01", "dz-sfo-01", "dz-tyo-01"],   # Sentinel
            "917": ["dz-ldn-01", "dz-fra-01"],                # Web Grounding
            "930": ["dz-sfo-01", "dz-sgp-01"],                # Quantum
            "931": ["dz-nyc-01", "dz-zur-01"],                # Interfold FHE
        }

    def add_node(self, node: DZNode):
        """Registra um nó na topologia DoubleZero."""
        self.nodes[node.node_id] = node
        # Invalidate cache
        self.routes_cache.clear()

    def find_route(self, source: str, target: str,
                   metric: RouteMetric = RouteMetric.LATENCY,
                   substrate_id: Optional[str] = None) -> Optional[Route]:
        """
        Encontra o melhor caminho entre source e target usando
        Dijkstra modificado com métrica selecionada.
        """
        cache_key = (source, target, metric)
        if cache_key in self.routes_cache:
            return self.routes_cache[cache_key]

        if source not in self.nodes or target not in self.nodes:
            return None

        # Dijkstra
        distances = {node_id: float('inf') for node_id in self.nodes}
        distances[source] = 0
        predecessors = {node_id: None for node_id in self.nodes}

        pq = [(0, source)]
        visited: Set[str] = set()

        while pq:
            current_dist, current = heapq.heappop(pq)

            if current in visited:
                continue
            visited.add(current)

            if current == target:
                break

            for neighbor_id, latency in self.nodes[current].latency_ms.items():
                if neighbor_id not in self.nodes:
                    continue

                # Weight by selected metric
                weight = self._metric_weight(current, neighbor_id, metric)
                new_dist = current_dist + weight

                if new_dist < distances[neighbor_id]:
                    distances[neighbor_id] = new_dist
                    predecessors[neighbor_id] = current
                    heapq.heappush(pq, (new_dist, neighbor_id))

        # Reconstruct path
        if distances[target] == float('inf'):
            return None

        path = []
        current = target
        while current is not None:
            path.append(current)
            current = predecessors[current]
        path.reverse()

        # Calculate aggregate metrics
        route = self._calculate_route_metrics(path)

        # Apply substrate-specific priority boost
        if substrate_id and substrate_id in self.substrate_routing_table:
            route.priority = self._substrate_priority(substrate_id)

        self.routes_cache[cache_key] = route
        return route

    def _metric_weight(self, from_node: str, to_node: str, metric: RouteMetric) -> float:
        """Calcula peso de aresta baseado na métrica selecionada."""
        node_from = self.nodes[from_node]
        node_to = self.nodes[to_node]

        if metric == RouteMetric.LATENCY:
            return node_from.latency_ms.get(to_node, 100)
        elif metric == RouteMetric.JITTER:
            return node_from.jitter_ms.get(to_node, 10)
        elif metric == RouteMetric.THROUGHPUT:
            return 1000 / min(node_from.capacity_gbps, node_to.capacity_gbps)
        elif metric == RouteMetric.RELIABILITY:
            return 1 / (node_from.reliability * node_to.reliability)
        elif metric == RouteMetric.COST:
            return node_to.contributor_stake / 1000
        return 1.0

    def _calculate_route_metrics(self, path: List[str]) -> Route:
        """Calcula métricas agregadas de uma rota."""
        total_latency = 0.0
        total_jitter = 0.0
        min_capacity = float('inf')
        reliability = 1.0

        for i in range(len(path) - 1):
            from_node = self.nodes[path[i]]
            to_node = self.nodes[path[i + 1]]

            total_latency += from_node.latency_ms.get(path[i + 1], 100)
            total_jitter += from_node.jitter_ms.get(path[i + 1], 10)
            min_capacity = min(min_capacity, from_node.capacity_gbps, to_node.capacity_gbps)
            reliability *= from_node.reliability * to_node.reliability

        return Route(
            path=path,
            total_latency_ms=total_latency,
            total_jitter_ms=total_jitter,
            min_capacity_gbps=min_capacity,
            reliability=reliability,
            estimated_cost=len(path) * 0.001,  # Token cost per hop
            priority=5
        )

    def _substrate_priority(self, substrate_id: str) -> int:
        """Retorna prioridade de routing por substrato."""
        priorities = {
            "923": 0,    # TemporalChain — CRITICAL
            "923.1": 0,
            "255.2": 0,
            "255": 1,    # Hermes ZK — HIGH
            "255.1": 1,
            "944": 1,    # Sentinel — HIGH
            "931": 1,    # Interfold — HIGH
            "917": 2,    # Web — MEDIUM
            "930": 2,    # Quantum — MEDIUM
            "940": 2,
            "941": 2,
            "942": 2,
            "943": 2,
            "912": 3,    # Memory — LOW
            "933": 3,
            "927": 3,
        }
        return priorities.get(substrate_id, 5)

    def get_topology_summary(self) -> dict:
        """Retorna resumo da topologia DoubleZero."""
        return {
            "total_nodes": len(self.nodes),
            "total_capacity_gbps": sum(n.capacity_gbps for n in self.nodes.values()),
            "avg_latency_ms": sum(
                sum(n.latency_ms.values()) / max(len(n.latency_ms), 1)
                for n in self.nodes.values()
            ) / max(len(self.nodes), 1),
            "substrate_routes": len(self.substrate_routing_table),
            "cached_routes": len(self.routes_cache)
        }

# ── Integration with ARKHE Runtime ──────────────────────────

class ARKHERoutingAdapter:
    """
    Adaptador que expõe o DZ Routing Engine como serviço
    interno do Runtime 939.
    """

    def __init__(self, engine: DZRoutingEngine):
        self.engine = engine

    def route_substrate_message(self, substrate_id: str,
                                 target_region: str) -> Optional[Route]:
        """
        Roteia mensagem de um substrato ARKHE para a região
        de destino mais adequada na rede DoubleZero.
        """
        # Find best egress node for substrate
        preferred_nodes = self.engine.substrate_routing_table.get(substrate_id, [])
        if not preferred_nodes:
            return None

        # Select node closest to target region
        best_node = preferred_nodes[0]  # Simplified

        return self.engine.find_route(
            source=best_node,
            target=f"dz-{target_region}-01",
            metric=RouteMetric.LATENCY,
            substrate_id=substrate_id
        )

if __name__ == "__main__":
    engine = DZRoutingEngine()

    # Add sample nodes
    engine.add_node(DZNode(
        "dz-nyc-01", "us-east", 100.0,
        {"dz-ldn-01": 70, "dz-ams-01": 80, "dz-sfo-01": 65},
        {"dz-ldn-01": 2, "dz-ams-01": 3, "dz-sfo-01": 2},
        0.999, 50000
    ))
    engine.add_node(DZNode(
        "dz-ldn-01", "eu-west", 80.0,
        {"dz-nyc-01": 70, "dz-ams-01": 10, "dz-fra-01": 15},
        {"dz-nyc-01": 2, "dz-ams-01": 1, "dz-fra-01": 1},
        0.998, 40000
    ))
    engine.add_node(DZNode(
        "dz-ams-01", "eu-central", 90.0,
        {"dz-nyc-01": 80, "dz-ldn-01": 10, "dz-fra-01": 8},
        {"dz-nyc-01": 3, "dz-ldn-01": 1, "dz-fra-01": 1},
        0.997, 45000
    ))

    # Find route for TemporalChain message
    route = engine.find_route("dz-nyc-01", "dz-ams-01", RouteMetric.LATENCY, "923")
    print(f"Route for 923: {route.path}")
    print(f"Latency: {route.total_latency_ms}ms, Jitter: {route.total_jitter_ms}ms")
    print(f"Priority: {route.priority}")