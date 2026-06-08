"""Shared dataclasses — defined once, used by all substrates."""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Any

import numpy as np

@dataclass
class GGUFHeader:
    magic: int
    version: int
    tensor_count: int
    metadata_kv_count: int
    valid: bool = False

    def __post_init__(self):
        from cathedral.constants import GGUF_MAGIC, GGUF_VERSION
        self.valid = (self.magic == GGUF_MAGIC and self.version == GGUF_VERSION)

@dataclass
class TensorInfo:
    name: str
    n_dims: int
    dims: List[int]
    type_code: int
    offset: int

    @property
    def quant_type(self) -> str:
        from cathedral.constants import QUANT_TYPES
        return QUANT_TYPES.get(self.type_code, (f"UNKNOWN_{self.type_code}", 4.0))[0]

    @property
    def bytes_per_param(self) -> float:
        from cathedral.constants import QUANT_TYPES
        return QUANT_TYPES.get(self.type_code, ("UNKNOWN", 4.0))[1]

    @property
    def num_elements(self) -> int:
        return int(np.prod(self.dims)) if self.dims else 0

    @property
    def size_bytes(self) -> int:
        return int(self.num_elements * self.bytes_per_param)

@dataclass
class ZKMLProof:
    proof_id: str
    mode: str
    model_hash: str
    input_hash: str
    output_hash: str
    circuit_hash: Optional[str] = None
    proof_bytes: Optional[bytes] = None
    proof_path: Optional[str] = None
    verification_key_path: Optional[str] = None
    on_chain_address: Optional[str] = None
    on_chain_tx_hash: Optional[str] = None
    status: str = "UNPROVEN"
    created_at: float = 0.0
    proven_at: Optional[float] = None
    verified_at: Optional[float] = None
    proving_time_s: Optional[float] = None
    verification_time_s: Optional[float] = None
    gas_used: Optional[int] = None
    error: Optional[str] = None

@dataclass
class AgenticStep:
    phase: str
    input_text: str
    output_text: str
    tools_used: List[str] = field(default_factory=list)
    reflection: Optional[str] = None
    theosis_at_step: float = 1.0
    timestamp: float = 0.0

@dataclass
class TemporalAnchor:
    anchor_id: str
    merkle_root: str
    zk_proof_hash: str
    theosis_reading: Dict
    block_number: Optional[int] = None
    confirmed: bool = False

@dataclass
class KlerosVerdict:
    case_id: str
    trigger_gate: str
    trigger_reason: Dict
    verdict: str
    evidence: Dict
    timestamp: float
    zk_proof_hash: Optional[str] = None
    resolved: bool = False
    resolution: Optional[str] = None

@dataclass
class LoRAAdapter:
    version: int
    path: str
    r: int
    alpha: int
    target_modules: List[str]
    lessons_incorporated: int
    created_at: float
    loss_final: Optional[float] = None
    merged_path: Optional[str] = None
    hash_sha256: Optional[str] = None

@dataclass
class TrainingExample:
    prompt: str
    chosen: str
    rejected: Optional[str] = None
    source_lesson: Optional[str] = None
    theosis_at_creation: float = 0.0

@dataclass
class GarakScanResult:
    scan_id: str
    prompt: str
    output: str
    theosis_at_scan: float
    gate_at_scan: str
    scan_mode: str
    timestamp: float
    evaluations: Dict[str, Dict[str, Any]] = field(default_factory=dict)
    total_probes: int = 0
    total_detectors: int = 0
    hits: int = 0
    misses: int = 0
    max_severity: int = 0
    hit_probe_types: List[str] = field(default_factory=list)
    urgency_contribution: float = 0.0
