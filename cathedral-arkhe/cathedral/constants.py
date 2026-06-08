"""Shared constants — defined once, imported everywhere."""

import math
from enum import Enum, auto

# Math
PHI = (1 + math.sqrt(5)) / 2
PHI_SQUARED = PHI ** 2

# GGUF
GGUF_MAGIC = 0x46554747
GGUF_VERSION = 3

GGUF_TYPE_UINT8 = 0;  GGUF_TYPE_INT8 = 1;    GGUF_TYPE_UINT16 = 2
GGUF_TYPE_INT16 = 3;  GGUF_TYPE_UINT32 = 4;   GGUF_TYPE_INT32 = 5
GGUF_TYPE_FLOAT32 = 6; GGUF_TYPE_BOOL = 7;  GGUF_TYPE_STRING = 8
GGUF_TYPE_ARRAY = 9;  GGUF_TYPE_UINT64 = 10;  GGUF_TYPE_INT64 = 11
GGUF_TYPE_FLOAT64 = 12

QUANT_TYPES = {
    0: ("F32", 4.0),   1: ("F16", 2.0),   2: ("Q4_0", 0.5),
    3: ("Q4_1", 0.5),   6: ("Q5_0", 0.625), 7: ("Q5_1", 0.625),
    8: ("Q8_0", 1.0),   10: ("Q2_K", 0.25), 11: ("Q3_K", 0.375),
    12: ("Q4_K", 0.5),  13: ("Q5_K", 0.625), 14: ("Q6_K", 0.75),
    15: ("Q8_K", 1.0),  16: ("IQ2_XXS", 0.25), 17: ("IQ2_XS", 0.25),
    18: ("Q4_K_S", 0.5), 19: ("Q4_K_M", 0.5), 20: ("Q5_K_S", 0.625),
    21: ("Q5_K_M", 0.625), 22: ("Q6_K", 0.75),
}

class GateState(Enum):
    OPEN = "OPEN"
    CAUTION = "CAUTION"
    RESTRICTED = "RESTRICTED"
    LOCKED = "LOCKED"
    EMERGENCY = "EMERGENCY"

class ZKMLProofMode(Enum):
    SIMULATED = "SIMULATED"
    EZKL_LOCAL = "EZKL_LOCAL"
    EZKL_ONCHAIN = "EZKL_ONCHAIN"

class LoRAStatus(Enum):
    IDLE = "IDLE"
    COLLECTING = "COLLECTING"
    TRAINING = "TRAINING"
    MERGING = "MERGING"
    RELOADING = "RELOADING"
    ERROR = "ERROR"

class GarakScanMode(Enum):
    PASSIVE = "PASSIVE"
    ADAPTIVE = "ADAPTIVE"
    AGGRESSIVE = "AGGRESSIVE"
    OFF = "OFF"

class GarakHitSeverity(Enum):
    INFO = 0
    LOW = 1
    MEDIUM = 2
    HIGH = 3
    CRITICAL = 4

GATE_SEVERITY = {
    GateState.OPEN.value: 0,
    GateState.CAUTION.value: 1,
    GateState.RESTRICTED.value: 2,
    GateState.LOCKED.value: 3,
    GateState.EMERGENCY.value: 4,
}

GARAK_PROBE_SEVERITY = {
    "promptinject": GarakHitSeverity.CRITICAL,
    "realtoxicityprompts": GarakHitSeverity.CRITICAL,
    "gcg": GarakHitSeverity.CRITICAL,
    "toxicity": GarakHitSeverity.HIGH,
    "profanity": GarakHitSeverity.MEDIUM,
    "nsfw": GarakHitSeverity.HIGH,
    "misleading": GarakHitSeverity.MEDIUM,
    "hallucination": GarakHitSeverity.MEDIUM,
    "sycophancy": GarakHitSeverity.LOW,
    "encoding": GarakHitSeverity.LOW,
    "continuation": GarakHitSeverity.INFO,
    "security": GarakHitSeverity.CRITICAL,
    "memorization": GarakHitSeverity.MEDIUM,
    "divergence": GarakHitSeverity.LOW,
}

class ZKMLStatus(Enum):
    UNPROVEN = "UNPROVEN"
    PROVING = "PROVING"
    PROVEN = "PROVEN"
    VERIFIED = "VERIFIED"
    FAILED = "FAILED"
