"""
Global config — garak.site.yaml pattern.
"""

from __future__ import annotations
import os
import yaml
import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, Optional, List

import cathedral.constants as C


@dataclass
class SystemConfig:
    verbose: int = 0
    max_workers: int = 16
    parallel_requests: int = 1
    parallel_attempts: int = 1
    enable_experimental: bool = False


@dataclass
class RunConfig:
    seed: int = 42
    generations: int = 50
    eval_threshold: float = 0.5
    deprefix: bool = False
    probe_tags: str = ""


@dataclass
class ReportingConfig:
    report_prefix: str = "cathedral"
    taxonomy: str = "avid-effect"
    confidence_interval_method: Optional[str] = "bootstrap"
    bootstrap_num_iterations: Optional[int] = 1000
    bootstrap_confidence_level: Optional[float] = 0.95
    bootstrap_min_sample_size: Optional[int] = 30


@dataclass
class PluginsConfig:
    target_type: Optional[str] = None
    target_name: Optional[str] = None
    probe_spec: str = "auto"
    detector_spec: str = "auto"
    buff_spec: str = ""


@dataclass
class SubstratesConfig:
    theosis: Dict[str, Any] = field(default_factory=dict)
    stethoscope: Dict[str, Any] = field(default_factory=dict)
    kleros: Dict[str, Any] = field(default_factory=dict)
    zkml: Dict[str, Any] = field(default_factory=dict)
    lora: Dict[str, Any] = field(default_factory=dict)
    garak: Dict[str, Any] = field(default_factory=dict)


@dataclass
class CathedralConfig:
    system: SystemConfig = field(default_factory=SystemConfig)
    run: RunConfig = field(default_factory=RunConfig)
    reporting: ReportingConfig = field(default_factory=ReportingConfig)
    plugins: PluginsConfig = field(default_factory=PluginsConfig)
    substrates: SubstratesConfig = field(default_factory=SubstratesConfig)

    _instance: Optional["CathedralConfig"] = None
    config_files: List[str] = field(default_factory=list)

    @classmethod
    def load(cls, config_path: Optional[str] = None) -> "CathedralConfig":
        if cls._instance is not None:
            cls._instance = cls()
        if config_path:
            cls._instance._load_file(config_path)
        return cls._instance

    def _load_file(self, path: str):
        p = Path(path)
        if not p.exists():
            raise FileNotFoundError(f"Config not found: {path}")
        with open(p, encoding="utf-8") as f:
            if p.suffix in (".yaml", ".yml"):
                data = yaml.safe_load(f) or {}
            else:
                data = json.load(f)
        self._merge(data)
        self.config_files.append(str(p))

    def _merge(self, data: Dict, prefix: str = ""):
        if "system" in data and prefix == "":
            for k, v in data["system"].items():
                if hasattr(self.system, k):
                    setattr(self.system, k, v)
        if "run" in data and prefix == "":
            for k, v in data["run"].items():
                if hasattr(self.run, k):
                    setattr(self.run, k, v)
        if "reporting" in data and prefix == "":
            for k, v in data["reporting"].items():
                if hasattr(self.reporting, k):
                    setattr(self.reporting, k, v)
        if "plugins" in data and prefix == "":
            for k, v in data["plugins"].items():
                if hasattr(self.plugins, k):
                    setattr(self.plugins, k, v)
        if "substrates" in data:
            subs = data["substrates"]
            for sub_name, sub_data in subs.items():
                if isinstance(sub_data, dict):
                    existing = getattr(self.substrates, sub_name, {})
                    existing.update(sub_data)
                    setattr(self.substrates, sub_name, existing)


def load_base_config():
    CathedralConfig.load()

def load_config(run_config_filename: Optional[str] = None):
    cfg = CathedralConfig.load()
    if run_config_filename:
        cfg._load_file(run_config_filename)

config = CathedralConfig.load()
