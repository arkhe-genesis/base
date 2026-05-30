#!/usr/bin/env python3
"""
ARKHE Unified Orchestrator — Substrato 989.w
Orquestra unificada de todos os substratos 989.x/989.y/989.z
com health checks, circuit breakers, métricas e auto-healing.
Arquiteto ORCID: 0009-0005-2697-4668
Seal: 989.w-UNIFIED-ORCHESTRATOR-F3A4B5C6D7E8F901
Cross-links: [989.x, 989.y, 989.z, 989.x.1, 989.x.2, 989.x.3, 989.x.4, 970, 972, 964]
Deities: Zeus (soberania), Athena (estratégia), Hermes (mensageiro), Hephaestus (forja)
Status: CANONIZED_PROVISIONAL
"""

import asyncio
import hashlib
import json
import time
from typing import Dict, List, Optional, Any, Callable, Set
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from enum import Enum
import random


class SubstrateStatus(Enum):
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"
    OFFLINE = "offline"
    UNKNOWN = "unknown"


class CircuitState(Enum):
    CLOSED = "closed"      # Normal operation
    OPEN = "open"          # Failure threshold exceeded
    HALF_OPEN = "half_open"  # Testing recovery


@dataclass
class HealthCheck:
    substrate_id: str
    timestamp: str
    latency_ms: float
    status: SubstrateStatus
    error: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class CircuitBreaker:
    substrate_id: str
    state: CircuitState
    failure_count: int = 0
    success_count: int = 0
    last_failure: Optional[str] = None
    last_success: Optional[str] = None
    threshold: int = 5
    timeout_seconds: int = 30
    half_open_max: int = 3


@dataclass
class OrchestratorMetrics:
    total_requests: int = 0
    successful_requests: int = 0
    failed_requests: int = 0
    total_latency_ms: float = 0.0
    circuit_breaks: int = 0
    auto_heals: int = 0
    theosis: float = 0.0
    entropy: float = 0.0
    resilience: float = 0.0
    timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    @property
    def avg_latency_ms(self) -> float:
        return self.total_latency_ms / max(self.total_requests, 1)

    @property
    def success_rate(self) -> float:
        return self.successful_requests / max(self.total_requests, 1)

    @property
    def availability(self) -> float:
        return 1.0 - (self.failed_requests / max(self.total_requests, 1))


class UnifiedOrchestrator:
    """
    Orquestrador unificado da Catedral ARKHE.
    Zeus governa; Athena planeja; Hermes conecta; Hephaestus forja.
    """

    SUBSTRATE_ID = "989.w"
    SEAL = "989.w-UNIFIED-ORCHESTRATOR-F3A4B5C6D7E8F901"

    # Substratos gerenciados
    MANAGED_SUBSTRATES = [
        "989.x", "989.x.1", "989.x.2", "989.x.3", "989.x.4",
        "989.y", "989.z", "970", "972", "964"
    ]

    def __init__(self):
        self.substrates: Dict[str, Any] = {}  # Instâncias de substratos
        self.health_checks: Dict[str, List[HealthCheck]] = {}
        self.circuit_breakers: Dict[str, CircuitBreaker] = {}
        self.metrics = OrchestratorMetrics()
        self.logs: List[str] = []
        self.is_running = False
        self._tasks: Set[asyncio.Task] = set()

        # Inicializar circuit breakers
        for sid in self.MANAGED_SUBSTRATES:
            self.circuit_breakers[sid] = CircuitBreaker(
                substrate_id=sid,
                state=CircuitState.CLOSED,
                threshold=5,
                timeout_seconds=30,
            )
            self.health_checks[sid] = []

    def log(self, msg: str):
        t = datetime.now(timezone.utc).isoformat()
        entry = f"[{t}] [ORCH] {msg}"
        self.logs.append(entry)
        print(f"  {entry}")

    # ───────────────────────────────────────────────────────────
    # Registro de Substratos
    # ───────────────────────────────────────────────────────────

    def register_substrate(self, substrate_id: str, instance: Any) -> bool:
        """Registra um substrato no orquestrador."""
        if substrate_id not in self.MANAGED_SUBSTRATES:
            self.log(f"⚠ Substrato {substrate_id} não está na lista gerenciada")
            return False
        self.substrates[substrate_id] = instance
        self.log(f"✓ Substrato {substrate_id} registrado")
        return True

    # ───────────────────────────────────────────────────────────
    # Health Checks
    # ───────────────────────────────────────────────────────────

    async def health_check(self, substrate_id: str) -> HealthCheck:
        """Executa health check em um substrato."""
        start = time.time()
        instance = self.substrates.get(substrate_id)

        if not instance:
            latency = (time.time() - start) * 1000
            return HealthCheck(
                substrate_id=substrate_id,
                timestamp=datetime.now(timezone.utc).isoformat(),
                latency_ms=latency,
                status=SubstrateStatus.OFFLINE,
                error="Substrato não registrado",
            )

        try:
            # Tentar chamar generate_report como proxy de saúde
            if hasattr(instance, "generate_report"):
                report = instance.generate_report()
                latency = (time.time() - start) * 1000
                return HealthCheck(
                    substrate_id=substrate_id,
                    timestamp=datetime.now(timezone.utc).isoformat(),
                    latency_ms=latency,
                    status=SubstrateStatus.HEALTHY,
                    metadata={"report_length": len(report)},
                )
            else:
                latency = (time.time() - start) * 1000
                return HealthCheck(
                    substrate_id=substrate_id,
                    timestamp=datetime.now(timezone.utc).isoformat(),
                    latency_ms=latency,
                    status=SubstrateStatus.HEALTHY,
                    metadata={"no_report_method": True},
                )
        except Exception as e:
            latency = (time.time() - start) * 1000
            return HealthCheck(
                substrate_id=substrate_id,
                timestamp=datetime.now(timezone.utc).isoformat(),
                latency_ms=latency,
                status=SubstrateStatus.UNHEALTHY,
                error=str(e),
            )

    async def run_all_health_checks(self) -> Dict[str, HealthCheck]:
        """Executa health checks em todos os substratos registrados."""
        results = {}
        for sid in self.substrates:
            check = await self.health_check(sid)
            results[sid] = check
            self.health_checks[sid].append(check)
            # Manter apenas últimos 100 checks
            self.health_checks[sid] = self.health_checks[sid][-100:]
            self._update_circuit_breaker(sid, check)
        return results

    # ───────────────────────────────────────────────────────────
    # Circuit Breaker
    # ───────────────────────────────────────────────────────────

    def _update_circuit_breaker(self, substrate_id: str, check: HealthCheck):
        """Atualiza estado do circuit breaker baseado no health check."""
        cb = self.circuit_breakers[substrate_id]

        if cb.state == CircuitState.CLOSED:
            if check.status in {SubstrateStatus.UNHEALTHY, SubstrateStatus.OFFLINE}:
                cb.failure_count += 1
                cb.last_failure = check.timestamp
                if cb.failure_count >= cb.threshold:
                    cb.state = CircuitState.OPEN
                    self.metrics.circuit_breaks += 1
                    self.log(f"🔴 Circuit OPEN para {substrate_id} ({cb.failure_count} falhas)")
            else:
                cb.success_count += 1
                cb.last_success = check.timestamp
                cb.failure_count = max(0, cb.failure_count - 1)

        elif cb.state == CircuitState.OPEN:
            # Verificar se timeout passou
            if cb.last_failure:
                last = datetime.fromisoformat(cb.last_failure.replace("Z", "+00:00"))
                if (datetime.now(timezone.utc) - last).total_seconds() > cb.timeout_seconds:
                    cb.state = CircuitState.HALF_OPEN
                    cb.failure_count = 0
                    cb.success_count = 0
                    self.log(f"🟡 Circuit HALF-OPEN para {substrate_id}")

        elif cb.state == CircuitState.HALF_OPEN:
            if check.status in {SubstrateStatus.UNHEALTHY, SubstrateStatus.OFFLINE}:
                cb.failure_count += 1
                if cb.failure_count >= cb.half_open_max:
                    cb.state = CircuitState.OPEN
                    self.log(f"🔴 Circuit OPEN (novamente) para {substrate_id}")
            else:
                cb.success_count += 1
                if cb.success_count >= cb.half_open_max:
                    cb.state = CircuitState.CLOSED
                    cb.failure_count = 0
                    self.metrics.auto_heals += 1
                    self.log(f"🟢 Circuit CLOSED para {substrate_id} (recuperado)")

    def can_execute(self, substrate_id: str) -> bool:
        """Verifica se pode executar operação no substrato."""
        cb = self.circuit_breakers.get(substrate_id)
        if not cb:
            return True
        return cb.state in {CircuitState.CLOSED, CircuitState.HALF_OPEN}

    # ───────────────────────────────────────────────────────────
    # Execução com Circuit Breaker
    # ───────────────────────────────────────────────────────────

    async def execute(self, substrate_id: str, operation: str, *args, **kwargs) -> Any:
        """Executa operação em substrato com proteção de circuit breaker."""
        self.metrics.total_requests += 1
        start = time.time()

        if not self.can_execute(substrate_id):
            self.metrics.failed_requests += 1
            latency = (time.time() - start) * 1000
            self.metrics.total_latency_ms += latency
            raise Exception(f"Circuit breaker OPEN para {substrate_id}")

        instance = self.substrates.get(substrate_id)
        if not instance:
            self.metrics.failed_requests += 1
            latency = (time.time() - start) * 1000
            self.metrics.total_latency_ms += latency
            raise Exception(f"Substrato {substrate_id} não registrado")

        try:
            method = getattr(instance, operation, None)
            if not method:
                raise Exception(f"Operação {operation} não encontrada em {substrate_id}")

            if asyncio.iscoroutinefunction(method):
                result = await method(*args, **kwargs)
            else:
                result = method(*args, **kwargs)

            self.metrics.successful_requests += 1
            latency = (time.time() - start) * 1000
            self.metrics.total_latency_ms += latency
            return result

        except Exception as e:
            self.metrics.failed_requests += 1
            latency = (time.time() - start) * 1000
            self.metrics.total_latency_ms += latency
            # Forçar health check para atualizar circuit breaker
            check = await self.health_check(substrate_id)
            self._update_circuit_breaker(substrate_id, check)
            raise

    # ───────────────────────────────────────────────────────────
    # Auto-Healing
    # ───────────────────────────────────────────────────────────

    async def auto_heal(self):
        """Tenta recuperar substratos em estado OPEN."""
        for sid, cb in self.circuit_breakers.items():
            if cb.state == CircuitState.OPEN:
                self.log(f"🩹 Tentando auto-heal de {sid}...")
                check = await self.health_check(sid)
                self._update_circuit_breaker(sid, check)
                if cb.state != CircuitState.OPEN:
                    self.log(f"✓ {sid} recuperado via auto-heal")

    # ───────────────────────────────────────────────────────────
    # Loop de Monitoramento
    # ───────────────────────────────────────────────────────────

    async def monitor_loop(self, interval_seconds: int = 10):
        """Loop contínuo de monitoramento."""
        self.is_running = True
        self.log("🔄 Monitor loop iniciado")
        while self.is_running:
            await self.run_all_health_checks()
            await self.auto_heal()
            self._compute_global_metrics()
            await asyncio.sleep(interval_seconds)

    def _compute_global_metrics(self):
        """Computa métricas globais da Catedral."""
        total = len(self.substrates)
        if total == 0:
            return

        healthy = sum(1 for checks in self.health_checks.values()
                      if checks and checks[-1].status == SubstrateStatus.HEALTHY)
        degraded = sum(1 for checks in self.health_checks.values()
                       if checks and checks[-1].status == SubstrateStatus.DEGRADED)
        unhealthy = sum(1 for checks in self.health_checks.values()
                        if checks and checks[-1].status == SubstrateStatus.UNHEALTHY)

        self.metrics.theosis = healthy / total
        self.metrics.entropy = (degraded + unhealthy) / total
        self.metrics.resilience = 1.0 - (unhealthy / total)
        self.metrics.timestamp = datetime.now(timezone.utc).isoformat()

    def stop(self):
        """Para o monitor loop."""
        self.is_running = False
        self.log("⏹ Monitor loop encerrado")

    # ───────────────────────────────────────────────────────────
    # Relatório Canônico
    # ───────────────────────────────────────────────────────────

    def generate_report(self) -> str:
        self._compute_global_metrics()
        m = self.metrics

        lines = []
        lines.append("╔" + "═" * 66 + "╗")
        lines.append("║  ARKHE CATHEDRAL — UNIFIED ORCHESTRATOR (989.w)" + " " * 14 + "║")
        lines.append("║  \"Zeus governa; Athena planeja; Hermes conecta\"" + " " * 12 + "║")
        lines.append("╠" + "═" * 66 + "╣")
        lines.append(f"  Seal: {self.SEAL}")
        lines.append(f"  Status: CANONIZED_PROVISIONAL")
        lines.append(f"  Cross-links: {self.MANAGED_SUBSTRATES}")
        lines.append("")
        lines.append("  MÉTRICAS GLOBAIS")
        lines.append("  ────────────────")
        lines.append(f"  Theosis:     {m.theosis:.4f}")
        lines.append(f"  Entropia:    {m.entropy:.4f}")
        lines.append(f"  Resiliência: {m.resilience:.4f}")
        lines.append(f"  Availability: {m.availability:.4f}")
        lines.append(f"  Success Rate: {m.success_rate:.4f}")
        lines.append(f"  Avg Latency:  {m.avg_latency_ms:.2f}ms")
        lines.append("")
        lines.append("  CIRCUIT BREAKERS")
        lines.append("  ────────────────")
        for sid, cb in self.circuit_breakers.items():
            emoji = {"closed": "🟢", "open": "🔴", "half_open": "🟡"}
            lines.append(f"  {emoji.get(cb.state.value, '⚪')} {sid}: {cb.state.value.upper()} (f:{cb.failure_count}, s:{cb.success_count})")
        lines.append("")
        lines.append("  SUBSTRATOS REGISTRADOS")
        lines.append("  ──────────────────────")
        for sid in self.substrates:
            checks = self.health_checks.get(sid, [])
            last = checks[-1].status.value if checks else "unknown"
            lines.append(f"  • {sid}: {last}")
        lines.append("")
        lines.append("  ODÔMETRO: ∞.Ω.∇+++.989.w.0")
        lines.append("╚" + "═" * 66 + "╝")
        return "\n".join(lines)


# ═══════════════════════════════════════════════════════════════════
# DEMONSTRAÇÃO
# ═══════════════════════════════════════════════════════════════════

async def demo():
    print("=" * 68)
    print("  ARKHE UNIFIED ORCHESTRATOR — DEMONSTRAÇÃO")
    print("=" * 68)

    orch = UnifiedOrchestrator()

    # Simular substratos (stubs)
    class StubSubstrate:
        def __init__(self, sid):
            self.sid = sid
        def generate_report(self):
            return f"Report from {self.sid}"

    for sid in ["989.x", "989.y", "989.z", "970", "972"]:
        orch.register_substrate(sid, StubSubstrate(sid))

    # Executar health checks
    print("\n[1] Health checks iniciais:")
    checks = await orch.run_all_health_checks()
    for sid, check in checks.items():
        print(f"    {sid}: {check.status.value} ({check.latency_ms:.2f}ms)")

    # Simular falha
    print("\n[2] Simulando falha em 989.z...")
    class BrokenSubstrate:
        def generate_report(self):
            raise Exception("Kernel panic!")
    orch.register_substrate("989.z", BrokenSubstrate())

    checks = await orch.run_all_health_checks()
    for sid, check in checks.items():
        print(f"    {sid}: {check.status.value}")

    # Tentar execução com circuit breaker
    print("\n[3] Tentando executar em 989.z (deve falhar):")
    try:
        await orch.execute("989.z", "generate_report")
    except Exception as e:
        print(f"    ✗ {e}")

    # Relatório
    print("\n[4] Relatório canônico:")
    print(orch.generate_report())


if __name__ == "__main__":
    asyncio.run(demo())
