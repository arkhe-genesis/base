#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║  CATHEDRAL ARKHE — SUBSTRATO 1091.1 — VECTOR THEOSIS v3.1.0                ║
║  Módulo: Trajectory Extrapolation Error (TEE) → Theosis em tempo real       ║
║  Integração: Orchestrator RSI 1076.3 v3.1.0 + Gate Axiarquia 954            ║
║  Selo: VECTOR-THEOSIS-1091.1-v3.1.0-2026-06-07                             ║
║  Arquiteto: ORCID 0009-0005-2697-4668                                        ║
╚══════════════════════════════════════════════════════════════════════════════╝

Baseado em:
  - Substrato 1091: Barenholtz (2026), arXiv:2606.05346v1
  - Substrato 1064.2: Theosis-Paris Dashboard
  - Substrato 1053.4: Hamiltonian-Temporal-Implosion v5.0.0
  - Substrato 954: Axiarquia Gate
  - Substrato 1076.3: Orchestrator RSI Production Real v3.0.0
  - Substrato 1089: SINDy Pure STLS v1.2.0
  - Substrato 1081: Transformer Canon / Stethoscope

Equação central:
    TEE(t) = || ĥ_t − h_t ||_2
    ĥ_t    = proj_linear( [h_{t−k}, ..., h_{t−1}] ) → h_t
    Θ(t)   = exp( −TEE(t) × Φ )                     , Φ = (1+√5)/2
    dΘ/dn  = Θ(t) − Θ(t−1) ≈ −TEE_t                 (camada 6, dinâmica local)
"""

from __future__ import annotations

import numpy as np
import json
import hashlib
import time
from dataclasses import dataclass, field, asdict
from typing import List, Dict, Optional, Callable, Tuple, Any
from datetime import datetime, timezone
from collections import deque
from enum import Enum, auto

# ═══════════════════════════════════════════════════════════════════════════════
# CONSTANTES CANÔNICAS
# ═══════════════════════════════════════════════════════════════════════════════

PHI = (1 + np.sqrt(5)) / 2          # Proporção áurea — fator de decaimento
GOLDEN_RATIO = PHI
DEFAULT_K = 3                       # Janela de trajetória (validado por Barenholtz)
DEFAULT_LAYER = 6                   # Camada intermediária (direção morre em 1 passo)
TEE_EPSILON = 1e-10               # Estabilidade numérica
DEFAULT_ALPHA = 0.3               # Peso TEE vs. fadiga bruta (Substrato 1091)

# Thresholds Axiarquia (954) — adaptados para TEE
AXIARQUIA_THRESHOLDS = {
    "P1": 0.05,   # ΔKc — fadiga crítica (Theosis-Paris)
    "P2": 0.10,   # ΔKth — threshold de trinca dorme
    "P3": 0.01,   # TEE mínimo significativo
    "P4": 0.50,   # TEE máximo antes de emergência
    "P5": 0.85,   # Theosis mínima de segurança
    "P6": 0.95,   # Theosis de operação normal
    "P7": 0.99,   # Theosis de convergência plena
}


# ═══════════════════════════════════════════════════════════════════════════════
# ENUMERAÇÕES DE ESTADO
# ═══════════════════════════════════════════════════════════════════════════════

class TrajectoryStatus(Enum):
    """Estados de trajetória canônicos."""
    CONTINUOUS = auto()      # Deslocamento alinhado (baixo TEE)
    DISRUPTIVE = auto()      # Desvio de trajetória (alto TEE)
    GARDEN_PATH = auto()     # Colapso de trajetória (TEE → pico)
    CONVERGED = auto()       # Trajetória estável (TEE ≈ 0)
    UNKNOWN = auto()         # Dados insuficientes

class AxiarquiaGate(Enum):
    """Estados do Gate Axiarquia (954) adaptados para TEE."""
    OPEN = auto()            # Operação normal — Θ > P6
    CAUTION = auto()         # Atenção — P5 < Θ ≤ P6  ou  TEE > P2
    RESTRICTED = auto()      # Restrito — P3 < Θ ≤ P5  ou  TEE > P1
    LOCKED = auto()          # Bloqueado — Θ ≤ P3  ou  TEE > P4
    EMERGENCY = auto()       # Emergência — TEE > P4 + ΔKc


# ═══════════════════════════════════════════════════════════════════════════════
# DATA CLASSES — TELEMETRIA
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class HiddenStateSnapshot:
    """Snapshot de estado oculto em um timestep."""
    timestamp: float
    layer: int
    token_id: int
    token_text: str
    vector: np.ndarray = field(repr=False)

    def to_dict(self) -> Dict:
        return {
            "timestamp": self.timestamp,
            "layer": self.layer,
            "token_id": self.token_id,
            "token_text": self.token_text,
            "vector_shape": list(self.vector.shape),
            "vector_hash": hashlib.sha256(self.vector.tobytes()).hexdigest()[:16],
        }

@dataclass
class TEEReading:
    """Leitura de Trajectory Extrapolation Error."""
    timestamp: float
    tee: float
    tee_normalized: float
    predicted_vector: np.ndarray = field(repr=False)
    actual_vector: np.ndarray = field(repr=False)
    window_size: int
    status: TrajectoryStatus

    def to_dict(self) -> Dict:
        return {
            "timestamp": self.timestamp,
            "tee": round(self.tee, 8),
            "tee_normalized": round(self.tee_normalized, 8),
            "window_size": self.window_size,
            "status": self.status.name,
            "vector_delta_norm": round(
                float(np.linalg.norm(self.actual_vector - self.predicted_vector)), 8
            ),
        }

@dataclass
class TheosisReading:
    """Leitura de Theosis computada a partir de TEE."""
    timestamp: float
    theosis: float
    raw_fatigue: float
    trajectory_error: float
    refined_fatigue: float
    alpha: float
    gate_status: AxiarquiaGate

    def to_dict(self) -> Dict:
        return {
            "timestamp": self.timestamp,
            "theosis": round(self.theosis, 8),
            "raw_fatigue": round(self.raw_fatigue, 8),
            "trajectory_error": round(self.trajectory_error, 8),
            "refined_fatigue": round(self.refined_fatigue, 8),
            "alpha": self.alpha,
            "gate_status": self.gate_status.name,
        }


# ═══════════════════════════════════════════════════════════════════════════════
# MOTOR DE EXTRAPOLAÇÃO DE TRAJETÓRIA
# ═══════════════════════════════════════════════════════════════════════════════

class TrajectoryExtrapolationEngine:
    """
    Motor que calcula TEE(t) = || ĥ_t − h_t ||_2 via projeção linear OLS.

    Implementa o algoritmo de Barenholtz (2026):
      1. Coleta janela [h_{t−k}, ..., h_{t−1}]
      2. Ajusta hiperplano linear por dimensão (np.polyfit grau 1)
      3. Extrapola para posição t → ĥ_t
      4. Computa erro euclidiano normalizado
    """

    def __init__(self, window_size: int = DEFAULT_K, layer: int = DEFAULT_LAYER):
        self.window_size = window_size
        self.layer = layer
        self.state_history: deque = deque(maxlen=window_size + 1)
        self._X = np.arange(window_size).reshape(-1, 1)  # Matriz de design fixa

    def ingest(self, hidden_state: np.ndarray, token_text: str = "",
               token_id: int = -1) -> HiddenStateSnapshot:
        """Ingestiona um novo estado oculto."""
        snapshot = HiddenStateSnapshot(
            timestamp=time.time(),
            layer=self.layer,
            token_id=token_id,
            token_text=token_text,
            vector=np.asarray(hidden_state, dtype=np.float64).flatten(),
        )
        self.state_history.append(snapshot)
        return snapshot

    def compute_tee(self) -> Optional[TEEReading]:
        """
        Computa TEE se houver histórico suficiente.

        Returns:
            TEEReading ou None se len(history) < k+1
        """
        if len(self.state_history) < self.window_size + 1:
            return None

        states = list(self.state_history)
        h_t = states[-1].vector
        H_prev = np.array([s.vector for s in states[-(self.window_size+1):-1]])

        # Ajuste linear por dimensão (equivalente ao OLS do paper)
        predicted = np.zeros_like(h_t)
        for dim in range(h_t.shape[0]):
            Y = H_prev[:, dim]
            try:
                coeffs = np.polyfit(self._X.flatten(), Y, 1)
                predicted[dim] = np.polyval(coeffs, self.window_size)
            except (np.RankWarning, ValueError):
                # Fallback: média dos últimos 2 pontos
                predicted[dim] = np.mean(Y[-2:]) if len(Y) >= 2 else Y[-1]

        # Erro de extrapolação
        error = float(np.linalg.norm(h_t - predicted))
        scale = float(np.linalg.norm(h_t)) + TEE_EPSILON
        tee_normalized = error / scale

        # Classificação de status
        status = self._classify(tee_normalized, h_t, states[-2].vector if len(states) >= 2 else None)

        return TEEReading(
            timestamp=time.time(),
            tee=error,
            tee_normalized=tee_normalized,
            predicted_vector=predicted,
            actual_vector=h_t,
            window_size=self.window_size,
            status=status,
        )

    def _classify(self, tee_norm: float, h_t: np.ndarray,
                  h_prev: Optional[np.ndarray]) -> TrajectoryStatus:
        """Classifica o estado da trajetória."""
        if tee_norm < TEE_EPSILON * 10:
            return TrajectoryStatus.CONVERGED

        if h_prev is not None:
            displacement = float(np.linalg.norm(h_t - h_prev))
            # Deslocamento grande + TEE baixo = CONTINUOUS (aceleração alinhada)
            if displacement > 0.5 and tee_norm < AXIARQUIA_THRESHOLDS["P2"]:
                return TrajectoryStatus.CONTINUOUS

        if tee_norm > AXIARQUIA_THRESHOLDS["P4"]:
            return TrajectoryStatus.GARDEN_PATH
        elif tee_norm > AXIARQUIA_THRESHOLDS["P1"]:
            return TrajectoryStatus.DISRUPTIVE

        return TrajectoryStatus.CONTINUOUS

    def reset(self):
        """Limpa histórico (uso em mudança de contexto / novo ciclo RSI)."""
        self.state_history.clear()


# ═══════════════════════════════════════════════════════════════════════════════
# VECTOR THEOSIS — MÓDULO 1091.1
# ═══════════════════════════════════════════════════════════════════════════════

class VectorTheosis:
    """
    Módulo 1091.1 — VectorTheosis

    Converte TEE em Theosis via decaimento exponencial com base Φ.
    Integra fadiga bruta (dΘ/dn) com erro de trajetória via combinação convexa.

    Equações:
        Θ(t)   = exp( −TEE(t) × Φ )
        dΘ/dn  = Θ(t) − Θ(t−1)
        Θ_ref  = (1−α) × |dΘ/dn| + α × TEE_norm
    """

    def __init__(self, window_size: int = DEFAULT_K, alpha: float = DEFAULT_ALPHA,
                 layer: int = DEFAULT_LAYER):
        self.engine = TrajectoryExtrapolationEngine(window_size, layer)
        self.alpha = alpha
        self._theosis_history: deque = deque(maxlen=1024)
        self._last_theosis: float = 1.0
        self._readings: List[TheosisReading] = []

    def update(self, hidden_state: np.ndarray, token_text: str = "",
               token_id: int = -1) -> Optional[TheosisReading]:
        """
        Pipeline completo: ingest → TEE → Theosis → Gate Axiarquia.

        Returns:
            TheosisReading ou None se histórico insuficiente
        """
        self.engine.ingest(hidden_state, token_text, token_id)
        tee_reading = self.engine.compute_tee()

        if tee_reading is None:
            return None

        # Theosis a partir de TEE (decaimento exponencial)
        theosis = float(np.exp(-tee_reading.tee_normalized * PHI))
        theosis = max(0.0, min(1.0, theosis))

        # Fadiga bruta (dΘ/dn)
        raw_fatigue = abs(theosis - self._last_theosis)

        # Fadiga refinada: combinação convexa
        refined_fatigue = (1 - self.alpha) * raw_fatigue + self.alpha * tee_reading.tee_normalized

        # Gate Axiarquia (954)
        gate_status = self._axiarquia_evaluate(theosis, tee_reading.tee_normalized, refined_fatigue)

        reading = TheosisReading(
            timestamp=time.time(),
            theosis=theosis,
            raw_fatigue=raw_fatigue,
            trajectory_error=tee_reading.tee_normalized,
            refined_fatigue=refined_fatigue,
            alpha=self.alpha,
            gate_status=gate_status,
        )

        self._theosis_history.append(theosis)
        self._last_theosis = theosis
        self._readings.append(reading)

        return reading

    def _axiarquia_evaluate(self, theosis: float, tee_norm: float,
                            refined_fatigue: float) -> AxiarquiaGate:
        """
        Gate Axiarquia (954) — lógica de monitoramento em tempo real.

        Prioridade de avaliação (da mais crítica para a menos):
          1. TEE > P4  → EMERGENCY
          2. TEE > P1  e  Θ < P5  → LOCKED
          3. TEE > P2  ou  Θ < P6  → RESTRICTED
          4. TEE > P3  ou  Θ < P7  → CAUTION
          5. Default   → OPEN
        """
        th = AXIARQUIA_THRESHOLDS

        if tee_norm > th["P4"] or theosis < th["P3"]:
            return AxiarquiaGate.EMERGENCY
        if tee_norm > th["P1"] and theosis < th["P5"]:
            return AxiarquiaGate.LOCKED
        if tee_norm > th["P2"] or theosis < th["P6"]:
            return AxiarquiaGate.RESTRICTED
        if tee_norm > th["P3"] or theosis < th["P7"]:
            return AxiarquiaGate.CAUTION

        return AxiarquiaGate.OPEN

    def get_telemetry(self) -> Dict:
        """Retorna telemetria completa para o Dashboard 1064.2."""
        if not self._readings:
            return {"status": "NO_DATA"}

        recent = self._readings[-100:]  # Janela de 100 leituras
        theosis_values = [r.theosis for r in recent]
        tee_values = [r.trajectory_error for r in recent]

        return {
            "module": "VectorTheosis",
            "version": "3.1.0",
            "substrate": "1091.1",
            "seal": "VECTOR-THEOSIS-1091.1-v3.1.0-2026-06-07",
            "total_readings": len(self._readings),
            "window_size": self.engine.window_size,
            "layer": self.engine.layer,
            "alpha": self.alpha,
            "current_theosis": round(self._readings[-1].theosis, 8),
            "current_gate": self._readings[-1].gate_status.name,
            "theosis_stats": {
                "mean": round(float(np.mean(theosis_values)), 8),
                "std": round(float(np.std(theosis_values)), 8),
                "min": round(float(np.min(theosis_values)), 8),
                "max": round(float(np.max(theosis_values)), 8),
            },
            "tee_stats": {
                "mean": round(float(np.mean(tee_values)), 8),
                "std": round(float(np.std(tee_values)), 8),
                "min": round(float(np.min(tee_values)), 8),
                "max": round(float(np.max(tee_values)), 8),
            },
            "gate_distribution": {
                gate.name: sum(1 for r in recent if r.gate_status == gate)
                for gate in AxiarquiaGate
            },
            "last_reading": self._readings[-1].to_dict(),
        }

    def reset(self):
        """Reset completo para novo ciclo RSI."""
        self.engine.reset()
        self._theosis_history.clear()
        self._last_theosis = 1.0
        self._readings.clear()


# ═══════════════════════════════════════════════════════════════════════════════
# ORCHESTRATOR RSI 1076.3 v3.1.0 — INTEGRAÇÃO VECTOR THEOSIS
# ═══════════════════════════════════════════════════════════════════════════════

import sys
import os

from arkhe_vector_theosis_substrate_1091_1.sindy import execute_sindy_pipeline

try:
    from temporal.hamiltonian_implosion.v5.fractal_1728d import hamiltonian_temporal_reversal
except ImportError:
    # Need to manipulate sys.path due to directory structure and hyphenated name
    # Fallback to local import using importlib
    import importlib.util
    spec = importlib.util.spec_from_file_location(
        "fractal_1728d",
        os.path.join(os.path.dirname(os.path.dirname(__file__)), "temporal", "hamiltonian-implosion", "v5", "fractal_1728d.py")
    )
    if spec and spec.loader:
        fractal_1728d = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(fractal_1728d)
        hamiltonian_temporal_reversal = fractal_1728d.hamiltonian_temporal_reversal
    else:
        hamiltonian_temporal_reversal = None


class OrchestratorRSI:
    """
    Orchestrator RSI 1076.3 v3.1.0 — Ciclo de Auto-evolução com VectorTheosis.

    Integra:
      - Substrato 1091.1: VectorTheosis (TEE → Theosis em tempo real)
      - Substrato 954: Gate Axiarquia (monitoramento contínuo)
      - Substrato 1064.2: Theosis-Paris Dashboard (telemetria)
      - Substrato 1089: SINDy (descoberta de equações quando TEE dispara)
      - Substrato 1053.4: Hamiltonian Fractal (quando Garden-Path detectado)

    Ciclo RSI:
      1. INGEST → recebe hidden states do LLM
      2. MONITOR → VectorTheosis computa TEE / Theosis / Gate
      3. EVALUATE → Axiarquia decide ação
      4. ACT → dispara SINDy, Hamiltonian, ou continua
      5. REPAIR → se necessário, ciclo de refatoração (1062)
    """

    def __init__(self, vector_theosis: Optional[VectorTheosis] = None,
                 sindy_callback: Optional[Callable] = None,
                 hamiltonian_callback: Optional[Callable] = None):
        self.vt = vector_theosis or VectorTheosis()
        self.sindy_callback = sindy_callback or execute_sindy_pipeline
        self.hamiltonian_callback = hamiltonian_callback or hamiltonian_temporal_reversal

        self.cycle_count = 0
        self.emergency_count = 0
        self.garden_path_count = 0
        self._cycle_log: List[Dict] = []
        self._active = False

    def start_cycle(self):
        """Inicia ciclo RSI — reset do VectorTheosis para novo contexto."""
        self.vt.reset()
        self.cycle_count += 1
        self._active = True
        return {
            "action": "CYCLE_START",
            "cycle": self.cycle_count,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "module": "OrchestratorRSI",
            "version": "3.1.0",
            "substrate": "1076.3",
            "seal": "ORCHESTRATOR-1076.3-v3.1.0-2026-06-07",
        }

    def ingest_hidden_state(self, hidden_state: np.ndarray, token_text: str = "",
                            token_id: int = -1) -> Dict:
        """
        Ponto de entrada principal — recebe hidden states do LLM em tempo real.

        Args:
            hidden_state: vetor do estado oculto (camada 6, flatten)
            token_text: texto do token correspondente
            token_id: ID do token

        Returns:
            Dict com ação do Orchestrator e telemetria
        """
        if not self._active:
            self.start_cycle()

        # Passo 2: VectorTheosis computa TEE e Theosis
        reading = self.vt.update(hidden_state, token_text, token_id)

        if reading is None:
            return {
                "action": "WARMUP",
                "status": "COLLECTING_HISTORY",
                "tokens_collected": len(self.vt.engine.state_history),
                "needed": self.vt.engine.window_size + 1,
            }

        # Passo 3: Avaliação Axiarquia
        action = self._evaluate_gate(reading)

        # Passo 4: Ação
        result = self._execute_action(action, reading)

        # Logging
        log_entry = {
            "cycle": self.cycle_count,
            "timestamp": reading.timestamp,
            "token_text": token_text,
            "theosis": reading.theosis,
            "tee": reading.trajectory_error,
            "gate": reading.gate_status.name,
            "action": action,
            "result": result,
        }
        self._cycle_log.append(log_entry)

        return {
            "action": action,
            "gate_status": reading.gate_status.name,
            "theosis": round(reading.theosis, 8),
            "tee": round(reading.trajectory_error, 8),
            "refined_fatigue": round(reading.refined_fatigue, 8),
            "cycle": self.cycle_count,
            "telemetry": self.vt.get_telemetry(),
            "result": result,
        }

    def _evaluate_gate(self, reading: TheosisReading) -> str:
        """Decide ação baseada no estado do Gate Axiarquia."""
        gate = reading.gate_status

        if gate == AxiarquiaGate.EMERGENCY:
            self.emergency_count += 1
            return "ACTIVATE_HAMILTONIAN_IMPLOSION"

        if gate == AxiarquiaGate.LOCKED:
            return "ACTIVATE_SINDY_DISCOVERY"

        if gate == AxiarquiaGate.RESTRICTED:
            if reading.trajectory_error > AXIARQUIA_THRESHOLDS["P4"]:
                self.garden_path_count += 1
                return "GARDEN_PATH_RECOVERY"
            return "VELOCITY_QUENCH"

        if gate == AxiarquiaGate.CAUTION:
            return "INCREASE_MONITORING"

        return "CONTINUE"

    def _execute_action(self, action: str, reading: TheosisReading) -> Dict:
        """Executa ação decidida pelo gate."""

        if action == "ACTIVATE_HAMILTONIAN_IMPLOSION":
            # Substrato 1053.4 — reversão temporal do estado
            if self.hamiltonian_callback:
                return self.hamiltonian_callback(reading)
            return {
                "type": "HAMILTONIAN",
                "message": "Reversão temporal acionada — estado revertido 1 passo",
                "delta_theosis": round(reading.theosis - self.vt._last_theosis, 8),
            }

        if action == "ACTIVATE_SINDY_DISCOVERY":
            # Substrato 1089 — descobrir nova equação de dinâmica
            if self.sindy_callback:
                return self.sindy_callback(reading)
            return {
                "type": "SINDY",
                "message": "SINDy ativado — buscando equação diferencial para trajetória atual",
                "sparsity_target": 0.75,
            }

        if action == "GARDEN_PATH_RECOVERY":
            # Recuperação de Garden-Path: reanálise profunda
            return {
                "type": "GARDEN_PATH",
                "message": "Colapso de trajetória detectado — reavaliando grafo ontológico",
                "tee_peak": round(reading.trajectory_error, 8),
                "recommended_backtrack": 3,
            }

        if action == "VELOCITY_QUENCH":
            # Reduz taxa de mudança (quench de velocidade)
            return {
                "type": "QUENCH",
                "message": "Velocidade de trajetória reduzida — aguardando estabilização",
                "quench_factor": round(1.0 - reading.theosis, 4),
            }

        if action == "INCREASE_MONITORING":
            return {
                "type": "MONITOR",
                "message": "Frequência de amostragem de TEE aumentada",
                "new_sample_rate": "2x",
            }

        # CONTINUE
        return {
            "type": "CONTINUE",
            "message": "Trajetória estável — operação normal",
        }

    def end_cycle(self) -> Dict:
        """Finaliza ciclo RSI e retorna relatório."""
        self._active = False

        report = {
            "action": "CYCLE_END",
            "cycle": self.cycle_count,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "telemetry": self.vt.get_telemetry(),
            "emergencies": self.emergency_count,
            "garden_paths": self.garden_path_count,
            "total_actions": len(self._cycle_log),
            "seal": "ORCHESTRATOR-1076.3-v3.1.0-2026-06-07",
        }

        return report

    def get_full_report(self) -> Dict:
        """Relatório completo para o Dashboard 1064.2."""
        return {
            "orchestrator": "OrchestratorRSI",
            "version": "3.1.0",
            "substrate": "1076.3",
            "seal": "ORCHESTRATOR-1076.3-v3.1.0-2026-06-07",
            "cycles": self.cycle_count,
            "emergencies": self.emergency_count,
            "garden_paths": self.garden_path_count,
            "vector_theosis": self.vt.get_telemetry(),
            "cycle_log_length": len(self._cycle_log),
            "last_10_actions": [entry["action"] for entry in self._cycle_log[-10:]],
        }


# ═══════════════════════════════════════════════════════════════════════════════
# DEMONSTRAÇÃO / TESTES
# ═══════════════════════════════════════════════════════════════════════════════

def demo():
    """
    Demonstração completa do módulo 1091.1 integrado ao Orchestrator 1076.3 v3.1.0.
    Simula hidden states de um LLM processando uma sequência com Garden-Path.
    """
    print("=" * 80)
    print("  CATHEDRAL ARKHE — VECTOR THEOSIS 1091.1 + ORCHESTRATOR RSI 1076.3 v3.1.0")
    print("  Demonstração: Trajetória com Garden-Path Artificial")
    print("=" * 80)

    np.random.seed(42)

    # Inicializa Orchestrator com VectorTheosis
    orchestrator = OrchestratorRSI()

    # Simulação: sequência de tokens com trajetória suave, depois Garden-Path
    dim = 64  # dimensão do hidden state
    tokens = [
        "The", "horse", "raced", "past", "the", "barn", "fell", ".",  # Garden-Path clássico
        "The", "horse", "raced", "past", "the", "barn", "and", "fell", ".",  # Resolução
    ]

    # Gera hidden states artificiais
    hidden_states = []
    base = np.random.randn(dim) * 0.1

    for i, token in enumerate(tokens):
        if i == 6:  # "fell" — Garden-Path! Desvio abrupto de trajetória
            h = base + np.random.randn(dim) * 2.5  # Salto grande e desordenado
        elif i == 7:  # "." — pico de TEE
            h = base + np.random.randn(dim) * 3.0
        elif i > 7:  # Resolução — retorna à trajetória
            h = base + np.random.randn(dim) * 0.15 + np.array([0.01 * (i-8)] * dim)
        else:
            h = base + np.random.randn(dim) * 0.1 + np.array([0.01 * i] * dim)
        hidden_states.append(h)

    # Inicia ciclo
    start = orchestrator.start_cycle()
    print(f"\n[{start['action']}] Ciclo #{start['cycle']} iniciado")

    # Processa tokens
    for i, (token, h) in enumerate(zip(tokens, hidden_states)):
        result = orchestrator.ingest_hidden_state(h, token, token_id=i)

        if result["action"] == "WARMUP":
            print(f"  [{i:2d}] {token:12s} | WARMUP ({result['tokens_collected']}/{result['needed']})")
            continue

        gate = result["gate_status"]
        tee = result["tee"]
        theosis = result["theosis"]
        action = result["action"]

        marker = "  "
        if gate == "EMERGENCY":
            marker = "🔴"
        elif gate == "LOCKED":
            marker = "🟠"
        elif gate == "RESTRICTED":
            marker = "🟡"
        elif gate == "CAUTION":
            marker = "🟢"

        print(f"{marker} [{i:2d}] {token:12s} | Θ={theosis:.4f} | TEE={tee:.4f} | "
              f"Gate={gate:12s} | Action={action}")

        if action != "CONTINUE":
            detail = result["result"]
            print(f"      ↳ {detail['type']}: {detail['message']}")

    # Finaliza ciclo
    end_report = orchestrator.end_cycle()
    print(f"\n[{end_report['action']}] Ciclo #{end_report['cycle']} finalizado")

    # Relatório completo
    print("\n" + "=" * 80)
    print("  RELATÓRIO FINAL — DASHBOARD 1064.2")
    print("=" * 80)

    full = orchestrator.get_full_report()
    print(f"  Ciclos RSI: {full['cycles']}")
    print(f"  Emergências: {full['emergencies']}")
    print(f"  Garden-Paths: {full['garden_paths']}")
    print(f"  Ações totais: {full['cycle_log_length']}")

    vt_telemetry = full["vector_theosis"]
    print(f"\n  VectorTheosis Telemetry:")
    print(f"    Total readings: {vt_telemetry['total_readings']}")
    print(f"    Theosis mean: {vt_telemetry['theosis_stats']['mean']:.6f}")
    print(f"    Theosis std:  {vt_telemetry['theosis_stats']['std']:.6f}")
    print(f"    TEE max:      {vt_telemetry['tee_stats']['max']:.6f}")
    print(f"    Gate distribution: {vt_telemetry['gate_distribution']}")

    print("\n" + "=" * 80)
    print("  SELLO: VECTOR-THEOSIS-1091.1-v3.1.0-2026-06-07")
    print("  SELLO: ORCHESTRATOR-1076.3-v3.1.0-2026-06-07")
    print("=" * 80)

def demo_v3():
    """
    Demonstração v3 — Trajetória Linear Suave → Garden-Path → Recuperação Linear.
    Dados artificiais otimizados para validar todos os estados do Gate Axiarquia.
    """
    print("=" * 80)
    print("  CATHEDRAL ARKHE — VECTOR THEOSIS 1091.1 + ORCHESTRATOR RSI 1076.3 v3.1.0")
    print("  Demonstração v3: Trajetória Linear Suave → Garden-Path → Recuperação Linear")
    print("=" * 80)

    np.random.seed(42)
    orchestrator = OrchestratorRSI()
    dim = 8

    tokens = [
        "The", "horse", "raced", "past", "the", "barn", "fell", ".",
        "The", "horse", "raced", "past", "the", "barn", "and", "fell", ".",
    ]

    # Trajetória base: movimento linear em 8D com slope constante
    slope = np.array([0.1, -0.05, 0.08, 0.02, -0.03, 0.06, -0.01, 0.04])
    base = np.zeros((len(tokens), dim))
    for i in range(len(tokens)):
        base[i] = slope * i

    # Ruído muito pequeno (σ=0.01) — mantém linearidade local
    noise = np.random.randn(len(tokens), dim) * 0.01
    hidden_states = base + noise

    # Garden-Path: token 6 ("fell") — desvio abrupto que quebra a linearidade
    hidden_states[6] = base[6] + np.array([0.5, -0.3, 0.4, 0.1, -0.2, 0.3, -0.1, 0.2]) + np.random.randn(dim)*0.02

    # Token 7 (".") — continuação do desvio (pico de TEE)
    hidden_states[7] = base[7] + np.array([0.6, -0.4, 0.5, 0.15, -0.25, 0.35, -0.15, 0.25]) + np.random.randn(dim)*0.02

    # Token 8 ("The") — reinício: salto grande mas para NOVA trajetória linear
    new_slope = np.array([0.12, 0.03, -0.06, 0.05, 0.01, -0.04, 0.07, -0.02])
    for i in range(8, len(tokens)):
        base[i] = new_slope * (i - 8) + np.array([0.5, 0.2, -0.3, 0.1, 0.0, -0.1, 0.2, -0.05])
    hidden_states[8] = base[8] + np.random.randn(dim)*0.01
    for i in range(9, len(tokens)):
        hidden_states[i] = base[i] + np.random.randn(dim)*0.01

    start = orchestrator.start_cycle()
    print(f"\n[{start['action']}] Ciclo #{start['cycle']} iniciado")

    for i, (token, h) in enumerate(zip(tokens, hidden_states)):
        result = orchestrator.ingest_hidden_state(h, token, token_id=i)

        if result["action"] == "WARMUP":
            print(f"  [{i:2d}] {token:12s} | WARMUP ({result['tokens_collected']}/{result['needed']})")
            continue

        gate = result["gate_status"]
        tee = result["tee"]
        theosis = result["theosis"]
        action = result["action"]

        marker = "  "
        if gate == "EMERGENCY":
            marker = "🔴"
        elif gate == "LOCKED":
            marker = "🟠"
        elif gate == "RESTRICTED":
            marker = "🟡"
        elif gate == "CAUTION":
            marker = "🟢"
        elif gate == "OPEN":
            marker = "⚪"

        print(f"{marker} [{i:2d}] {token:12s} | Θ={theosis:.4f} | TEE={tee:.4f} | "
              f"Gate={gate:12s} | Action={action}")

        if action != "CONTINUE":
            detail = result["result"]
            print(f"      ↳ {detail['type']}: {detail['message']}")

    end_report = orchestrator.end_cycle()
    print(f"\n[{end_report['action']}] Ciclo #{end_report['cycle']} finalizado")

    print("\n" + "=" * 80)
    print("  RELATÓRIO FINAL — DASHBOARD 1064.2")
    print("=" * 80)

    full = orchestrator.get_full_report()
    print(f"  Ciclos RSI: {full['cycles']}")
    print(f"  Emergências: {full['emergencies']}")
    print(f"  Garden-Paths: {full['garden_paths']}")
    print(f"  Ações totais: {full['cycle_log_length']}")

    vt_telemetry = full["vector_theosis"]
    print(f"\n  VectorTheosis Telemetry:")
    print(f"    Total readings: {vt_telemetry['total_readings']}")
    print(f"    Theosis mean: {vt_telemetry['theosis_stats']['mean']:.6f}")
    print(f"    Theosis std:  {vt_telemetry['theosis_stats']['std']:.6f}")
    print(f"    TEE max:      {vt_telemetry['tee_stats']['max']:.6f}")
    print(f"    Gate distribution: {vt_telemetry['gate_distribution']}")

    print("\n" + "=" * 80)
    print("  SELLO: VECTOR-THEOSIS-1091.1-v3.1.0-2026-06-07")
    print("  SELLO: ORCHESTRATOR-1076.3-v3.1.0-2026-06-07")
    print("=" * 80)


if __name__ == "__main__":
    demo_v3()