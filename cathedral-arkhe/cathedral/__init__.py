"""
Cathedral ARKHE — Recursive Self-Improvement Orchestration

Active Substrates:
  1094.1  GGUF Bridge v3
  1094.2  LlamaCpp Bridge v3
  1091.2  VectorTheosis v4.0.0
  1081.1  Stethoscope v3.0.0
  1085.1  Kleros v2.0.0
  1095.1  ZKML Bridge v2.0.0
  1096    Agentic Loop v1.0.0
  1097    TemporalChain v2.0.0
  1098    LoRA Fine-Tuner v1.0.0
  1099    Garak Bridge v1.0.0
"""

from cathedral._version import __version__, __version_info__
from cathedral.orchestrator.v5_1 import CathedralOrchestratorV5_1 as CathedralOrchestratorV5

__all__ = [
    "CathedralOrchestratorV5",
    "__version__",
    "__version_info__",
]
