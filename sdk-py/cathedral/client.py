import asyncio
import json
import uuid
import time
from dataclasses import dataclass, asdict
from enum import Enum
from typing import Any, Dict, List, Optional
import logging

import grpc
from google.protobuf.timestamp_pb2 import Timestamp

from .generated.cathedral.v1 import bridge_pb2
from .generated.cathedral.v1 import bridge_pb2_grpc

logger = logging.getLogger("cathedral-sdk")

# ============================================================
# TIPOS DE EVENTO
# ============================================================

class EventType(str, Enum):
    DESIGN_PROPOSED = "design_proposed"
    SIMULATION_COMPLETED = "simulation_completed"
    DESIGN_OPTIMIZED = "design_optimized"
    FABRICATION_PLANNED = "fabrication_planned"
    FABRICATION_COMPLETED = "fabrication_completed"
    TEST_RESULT = "test_result"
    HUMAN_REVIEW = "human_review"
    AGENT_MUTATION = "agent_mutation"

class HumanVerdict(str, Enum):
    APPROVED = "approved"
    CONDITIONAL = "conditional"
    REJECTED = "rejected"

class GovernanceMode(str, Enum):
    HUMAN_IN_THE_LOOP = "human_in_the_loop"
    AUTONOMOUS_WITH_CIRCUIT_BREAKER = "autonomous_with_circuit_breaker"
    AUTONOMOUS_FULL = "autonomous_full"

@dataclass
class DesignProposedEvent:
    design_hash: str
    parent_hashes: List[str]
    parameters: Dict[str, float]
    rationale: str
    agent_id: str

@dataclass
class SimulationCompletedEvent:
    design_hash: str
    simulator: str
    metrics: Dict[str, float]
    convergence: bool
    compute_cost_usd: float

@dataclass
class AgentMutationEvent:
    mutation_description: str
    previous_agent_hash: str
    substrate_version: str

@dataclass
class GovernanceResponse:
    verdict: str  # approved, rejected, requires_human, conditional, timeout
    rationale: str
    conditions: Optional[List[str]] = None

# ============================================================
# CLIENTE PRINCIPAL
# ============================================================

class CathedralClient:
    def __init__(
        self,
        bridge_endpoint: str = "localhost:50051",
        project_id: str = "default",
        agent_id: str = "default-agent",
        batch_size: int = 50,
        flush_interval_ms: int = 5000,
        governance_mode: GovernanceMode = GovernanceMode.AUTONOMOUS_WITH_CIRCUIT_BREAKER,
        loop: Optional[asyncio.AbstractEventLoop] = None,
    ):
        self.bridge_endpoint = bridge_endpoint
        self.project_id = project_id
        self.agent_id = agent_id
        self.batch_size = batch_size
        self.flush_interval_ms = flush_interval_ms
        self.governance_mode = governance_mode
        self.loop = loop or asyncio.get_event_loop()

        self._event_queue: List[bridge_pb2.Event] = []
        self._flush_task: Optional[asyncio.Task] = None
        self._channel: Optional[grpc.aio.Channel] = None
        self._stub: Optional[bridge_pb2_grpc.CathedralBridgeStub] = None
        self._running = False

    async def __aenter__(self):
        self._channel = grpc.aio.insecure_channel(self.bridge_endpoint)
        self._stub = bridge_pb2_grpc.CathedralBridgeStub(self._channel)
        self._running = True
        self._flush_task = asyncio.create_task(self._background_flusher())
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        self._running = False
        if self._flush_task:
            self._flush_task.cancel()
            try:
                await self._flush_task
            except asyncio.CancelledError:
                pass

        # Flush remaining events
        if self._event_queue:
            await self._flush()

        if self._channel:
            await self._channel.close()

    # ============================================================
    # EMISSÃO DE EVENTOS (PÚBLICO)
    # ============================================================

    async def emit_design_proposed(
        self,
        design_hash: str,
        parent_hashes: List[str],
        parameters: Dict[str, float],
        rationale: str,
    ) -> None:
        event_dict = asdict(DesignProposedEvent(
            design_hash=design_hash,
            parent_hashes=parent_hashes,
            parameters=parameters,
            rationale=rationale,
            agent_id=self.agent_id,
        ))

        event = self._build_event(
            bridge_pb2.EventType.DESIGN_PROPOSED,
            design_hash,
            parent_hashes,
            event_dict,
            "aerospace",
            0.0
        )
        await self._emit_event(event)

    async def emit_simulation_completed(
        self,
        design_hash: str,
        simulator: str,
        metrics: Dict[str, float],
        convergence: bool,
        compute_cost_usd: float,
    ) -> None:
        event_dict = asdict(SimulationCompletedEvent(
            design_hash=design_hash,
            simulator=simulator,
            metrics=metrics,
            convergence=convergence,
            compute_cost_usd=compute_cost_usd,
        ))

        event = self._build_event(
            bridge_pb2.EventType.SIMULATION_COMPLETED,
            design_hash,
            [],
            event_dict,
            "simulation",
            compute_cost_usd
        )
        await self._emit_event(event)

    async def emit_agent_mutation(
        self,
        mutation_description: str,
        previous_agent_hash: str,
    ) -> None:
        event_dict = asdict(AgentMutationEvent(
            mutation_description=mutation_description,
            previous_agent_hash=previous_agent_hash,
            substrate_version="5003.8",
        ))

        import hashlib
        design_hash = hashlib.blake2b(mutation_description.encode()).hexdigest()

        # Mutação de agente requer governança síncrona
        response = await self.request_governance(bridge_pb2.EventType.AGENT_MUTATION, event_dict)
        if response.verdict == "rejected":
            raise RuntimeError(f"Agent mutation rejected: {response.rationale}")
        if response.verdict == "requires_human":
            logger.warning(f"Agent mutation requires human review: {response.rationale}")
        if response.verdict == "conditional":
            logger.info(f"Agent mutation approved with conditions: {response.conditions}")

        event = self._build_event(
            bridge_pb2.EventType.AGENT_MUTATION,
            design_hash,
            [previous_agent_hash],
            event_dict,
            "meta",
            0.0
        )
        await self._emit_event(event)

    # ============================================================
    # GOVERNANÇA SÍNCRONA
    # ============================================================

    async def request_governance(self, event_type: bridge_pb2.EventType, payload: Dict[str, Any]) -> GovernanceResponse:
        if self.governance_mode == GovernanceMode.AUTONOMOUS_FULL:
            return GovernanceResponse(verdict="approved", rationale="Autonomous full mode")

        if self.governance_mode == GovernanceMode.AUTONOMOUS_WITH_CIRCUIT_BREAKER:
            risk = self._estimate_risk(event_type)
            if risk < 0.5:
                return GovernanceResponse(verdict="approved", rationale="Low risk decision")

        if not self._stub:
            raise RuntimeError("Client must be used as an async context manager")

        request = bridge_pb2.GovernanceRequest(
            request_id=str(uuid.uuid4()),
            project_id=self.project_id,
            agent_id=self.agent_id,
            event_type=event_type,
            proposed_state_json=json.dumps(payload),
            current_state_json="{}",
            agent_risk_score=self._estimate_risk(event_type),
            domain="aerospace"
        )

        try:
            response = await self._stub.RequestGovernance(request, timeout=5.0)

            verdict_str = "unknown"
            if response.verdict == bridge_pb2.GovernanceVerdict.APPROVED:
                verdict_str = "approved"
            elif response.verdict == bridge_pb2.GovernanceVerdict.REJECTED:
                verdict_str = "rejected"
            elif response.verdict == bridge_pb2.GovernanceVerdict.REQUIRES_HUMAN:
                verdict_str = "requires_human"
            elif response.verdict == bridge_pb2.GovernanceVerdict.CONDITIONAL:
                verdict_str = "conditional"
            elif response.verdict == bridge_pb2.GovernanceVerdict.TIMEOUT:
                verdict_str = "timeout"

            return GovernanceResponse(
                verdict=verdict_str,
                rationale=response.rationale,
                conditions=list(response.conditions) if response.conditions else None,
            )
        except grpc.aio.AioRpcError as e:
            if e.code() == grpc.StatusCode.DEADLINE_EXCEEDED:
                return GovernanceResponse(
                    verdict="timeout",
                    rationale="Governance request timed out",
                )
            logger.exception("Governance request failed")
            return GovernanceResponse(
                verdict="rejected",
                rationale=f"Governance request failed: {e.details()}",
            )

    # ============================================================
    # INTERNO
    # ============================================================

    def _estimate_risk(self, event_type: bridge_pb2.EventType) -> float:
        risk_map = {
            bridge_pb2.EventType.AGENT_MUTATION: 0.85,
            bridge_pb2.EventType.FABRICATION_PLANNED: 0.70,
            bridge_pb2.EventType.SIMULATION_COMPLETED: 0.30,
            bridge_pb2.EventType.DESIGN_PROPOSED: 0.20,
        }
        return risk_map.get(event_type, 0.10)

    def _build_event(self, event_type: bridge_pb2.EventType, design_hash: str, parent_hashes: List[str], payload: Dict[str, Any], domain: str, cost: float) -> bridge_pb2.Event:
        now = time.time()
        timestamp = Timestamp(seconds=int(now), nanos=int((now % 1) * 1e9))

        metadata = bridge_pb2.EventMetadata(
            domain=domain,
            confidence=0.8,
            compute_cost_usd=cost
        )

        return bridge_pb2.Event(
            event_id=str(uuid.uuid4()),
            timestamp=timestamp,
            event_type=event_type,
            design_hash=design_hash,
            parent_hashes=parent_hashes,
            payload_json=json.dumps(payload),
            metadata=metadata
        )

    async def _emit_event(self, event: bridge_pb2.Event) -> None:
        self._event_queue.append(event)
        if len(self._event_queue) >= self.batch_size:
            await self._flush()

    async def _flush(self) -> None:
        if not self._event_queue or not self._stub:
            return

        batch = self._event_queue[:]
        self._event_queue.clear()

        request = bridge_pb2.IngestRequest(
            project_id=self.project_id,
            agent_id=self.agent_id,
            events=batch,
            batch_id=str(uuid.uuid4())
        )

        try:
            response = await self._stub.Ingest(request, timeout=5.0)
            if response.success:
                logger.debug(f"Batch of {len(batch)} events sent successfully")
            else:
                logger.error(f"Failed to send batch: {response.message}")
        except Exception as e:
            logger.exception("Failed to send batch")
            # Em produção, poderíamos colocar de volta na fila ou persistir localmente

    async def _background_flusher(self) -> None:
        while self._running:
            await asyncio.sleep(self.flush_interval_ms / 1000.0)
            if self._event_queue:
                await self._flush()
