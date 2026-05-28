#!/usr/bin/env python3
"""
Substrato 941 — Cognitive Effort Controller
Ajusta adaptive_thinking_level do Claude Opus 4.7 com base na
complexidade Kolmogorov (898) da tarefa e no histórico FluxMem (933).
"""

import json
import math
import time
from typing import Dict, Optional, Tuple
from dataclasses import dataclass
from enum import Enum

class EffortLevel(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    EXTRA_HIGH = "extra_high"
    MAX = "max"

@dataclass
class TaskProfile:
    task_id: str
    query: str
    modality: str
    expected_output_length: int
    reasoning_depth: int
    domain: str

@dataclass
class EffortConfig:
    level: EffortLevel
    max_tokens: int
    temperature: float
    top_p: float
    thinking_budget: Optional[int]
    cache_enabled: bool

class CognitiveEffortController:
    TOKEN_BUDGETS = {
        EffortLevel.LOW: 4096,
        EffortLevel.MEDIUM: 16384,
        EffortLevel.HIGH: 65536,
        EffortLevel.EXTRA_HIGH: 131072,
        EffortLevel.MAX: 200000
    }

    TEMPERATURES = {
        EffortLevel.LOW: 0.1,
        EffortLevel.MEDIUM: 0.3,
        EffortLevel.HIGH: 0.5,
        EffortLevel.EXTRA_HIGH: 0.7,
        EffortLevel.MAX: 0.9
    }

    def __init__(self, kolmogorov_service_url: str, fluxmem_service_url: str):
        self.kolmogorov_url = kolmogorov_service_url
        self.fluxmem_url = fluxmem_service_url
        self.history: Dict[str, EffortConfig] = {}

    def measure_complexity(self, task: TaskProfile) -> float:
        base = len(task.query) / 10000.0
        reasoning_factor = task.reasoning_depth / 5.0
        modality_entropy = {"text": 0.3, "vision": 0.5, "audio": 0.6, "multimodal": 0.9}[task.modality]
        complexity = min(1.0, (base * 0.3 + reasoning_factor * 0.4 + modality_entropy * 0.3))
        return complexity

    def query_fluxmem_optimal(self, task: TaskProfile) -> Optional[EffortLevel]:
        domain_defaults = {
            "mathematics": EffortLevel.MAX,
            "coding": EffortLevel.HIGH,
            "creative": EffortLevel.MEDIUM,
            "summarization": EffortLevel.LOW,
            "security_audit": EffortLevel.EXTRA_HIGH
        }
        return domain_defaults.get(task.domain)

    def compute_effort(self, task: TaskProfile,
                       user_override: Optional[EffortLevel] = None,
                       cost_budget_usd: Optional[float] = None) -> EffortConfig:
        if user_override:
            level = user_override
        else:
            complexity = self.measure_complexity(task)
            fluxmem_suggestion = self.query_fluxmem_optimal(task)
            if fluxmem_suggestion:
                comp_idx = int(complexity * 4)
                hist_idx = list(EffortLevel).index(fluxmem_suggestion)
                blended = int(0.6 * comp_idx + 0.4 * hist_idx)
                level = list(EffortLevel)[min(blended, 4)]
            else:
                level = list(EffortLevel)[min(int(complexity * 4), 4)]

        if cost_budget_usd is not None:
            level = self._apply_cost_guardrail(level, cost_budget_usd)

        config = EffortConfig(
            level=level,
            max_tokens=self.TOKEN_BUDGETS[level],
            temperature=self.TEMPERATURES[level],
            top_p=0.95 if level in (EffortLevel.LOW, EffortLevel.MEDIUM) else 0.99,
            thinking_budget=self.TOKEN_BUDGETS[level] // 4 if level in (EffortLevel.HIGH, EffortLevel.EXTRA_HIGH, EffortLevel.MAX) else None,
            cache_enabled=level in (EffortLevel.LOW, EffortLevel.MEDIUM)
        )
        self.history[task.task_id] = config
        return config

    def _apply_cost_guardrail(self, level: EffortLevel, budget: float) -> EffortLevel:
        costs = {
            EffortLevel.LOW: 0.10,
            EffortLevel.MEDIUM: 0.50,
            EffortLevel.HIGH: 2.00,
            EffortLevel.EXTRA_HIGH: 5.00,
            EffortLevel.MAX: 10.00
        }
        while costs[level] > budget and level != EffortLevel.LOW:
            idx = list(EffortLevel).index(level)
            level = list(EffortLevel)[max(0, idx - 1)]
        return level

    def to_anthropic_api_params(self, config: EffortConfig) -> Dict:
        params = {
            "model": "claude-opus-4-7-20260501",
            "max_tokens": config.max_tokens,
            "temperature": config.temperature,
            "top_p": config.top_p,
        }
        if config.thinking_budget:
            params["thinking"] = {
                "type": "enabled",
                "budget_tokens": config.thinking_budget
            }
        return params

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Cognitive Effort Controller 941")
    parser.add_argument("--query", required=True)
    parser.add_argument("--modality", default="text", choices=["text", "vision", "audio", "multimodal"])
    parser.add_argument("--reasoning", type=int, default=3)
    parser.add_argument("--domain", default="general")
    parser.add_argument("--budget", type=float)
    parser.add_argument("--override", choices=["low", "medium", "high", "extra_high", "max"])
    args = parser.parse_args()

    task = TaskProfile(
        task_id=f"task_{int(time.time())}",
        query=args.query,
        modality=args.modality,
        expected_output_length=1000,
        reasoning_depth=args.reasoning,
        domain=args.domain
    )

    ctrl = CognitiveEffortController(
        kolmogorov_service_url="grpc://localhost:9390",
        fluxmem_service_url="grpc://localhost:9330"
    )

    override = EffortLevel(args.override) if args.override else None
    config = ctrl.compute_effort(task, user_override=override, cost_budget_usd=args.budget)

    print(json.dumps({
        "task_id": task.task_id,
        "effort_level": config.level.value,
        "max_tokens": config.max_tokens,
        "temperature": config.temperature,
        "top_p": config.top_p,
        "thinking_budget": config.thinking_budget,
        "cache_enabled": config.cache_enabled,
        "anthropic_params": ctrl.to_anthropic_api_params(config)
    }, indent=2))