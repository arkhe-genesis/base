"""Orchestrator factory by version."""

from typing import Optional, Any
from pathlib import Path

# Assuming orchestrator files exist
from cathedral.orchestrator.v4 import CathedralOrchestratorV4 as V4
from cathedral.orchestrator.v4_1 import CathedralOrchestratorV4_1 as V4_1
from cathedral.orchestrator.v5 import CathedralOrchestratorV5 as V5
from cathedral.orchestrator.v5_1 import CathedralOrchestratorV5_1 as V5_1

_VERSION_MAP = {
    "4.0.0": V4,
    "4.1.0": V4_1,
    "5.0.0": V5,
    "5.1.0": V5_1,
}

LATEST = "5.1.0"

def create_orchestrator(version: Optional[str] = None,
                        model_path: Optional[str] = None,
                        **kwargs) -> Any:
    v = version or LATEST
    if v not in _VERSION_MAP:
        available = ", ".join(sorted(_VERSION_MAP.keys(), reverse=True))
        raise ValueError(f"Version '{v}' not available. Available: {available}")
    cls = _VERSION_MAP[v]
    if model_path is not None:
        kwargs["model_path"] = model_path
    return cls(**kwargs)
