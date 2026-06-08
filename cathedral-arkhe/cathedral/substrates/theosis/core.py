import numpy as np
from collections import deque
import time
from cathedral.constants import PHI_SQUARED, GateState


class VectorTheosis1092:
    def __init__(self, dim, window_sizes=(2, 3, 5, 8, 13), ema_short=0.3, ema_long=0.05, rkhs_bandwidth=0.1):
        self.dim = dim; self.window_sizes = window_sizes; self.ema_short = ema_short
        self.ema_long = ema_long; self.rkhs_bandwidth = rkhs_bandwidth
        self._buffers = {w: deque(maxlen=w + 2) for w in window_sizes}
        self._global_buffer = deque(maxlen=max(window_sizes) + 4)
        self._last_theosis = 1.0; self._ema_short_val = 0.0; self._ema_long_val = 0.0
        self._cycle = 0; self._bifurcation_detected = False; self._bifurcation_count = 0
        self.readings = []
        self._gate_thresholds = {
            "emergency_tee": 0.50, "emergency_theta": 0.01,
            "locked_tee": 0.15, "locked_theta": 0.50,
            "restricted_tee": 0.05, "restricted_theta": 0.90,
            "caution_tee": 0.01, "caution_theta": 0.98,
        }

    def _rkhs_predict(self, buffer):
        if len(buffer) < 3: return None
        states = np.array(buffer); n = len(states)
        K = np.zeros((n-1, n-1))
        for i in range(n-1):
            for j in range(n-1):
                dist = np.linalg.norm(states[i] - states[j])
                K[i, j] = np.exp(-dist**2 / (2 * self.rkhs_bandwidth**2))
        y = states[-1]
        if np.allclose(states, states[0]): return y.copy()
        try:
            alpha = np.linalg.solve(K + 1e-6 * np.eye(n-1), np.dot(states[:-1], y))
            pred = np.zeros(self.dim)
            for i in range(n-1): pred += alpha[i] * states[i]
            return pred
        except np.linalg.LinAlgError: return self._linear_predict(buffer)

    def _linear_predict(self, buffer):
        if len(buffer) < 3: return None
        states = np.array(buffer); n = len(states) - 1
        X = np.column_stack([np.arange(n, dtype=np.float64), np.ones(n)])
        try:
            coeffs, _, _, _ = np.linalg.lstsq(X, states[:-1], rcond=None)
            return coeffs[0] * n + coeffs[1]
        except: return states[-1]

    def _compute_tee(self, buffer, use_rkhs=True):
        predicted = self._rkhs_predict(buffer) if use_rkhs else self._linear_predict(buffer)
        if predicted is None: return None
        h_t = np.array(buffer[-1], dtype=np.float64)
        error = np.linalg.norm(h_t - predicted)
        scale = np.linalg.norm(h_t)
        return float(error / (scale + 1e-12))

    def _spectral_entropy(self, buffer):
        if len(buffer) < 3: return 0.0
        mat = np.array(buffer, dtype=np.float64)
        cov = np.cov(mat.T)
        if cov.ndim == 0: return 0.0
        try:
            eigvals = np.linalg.eigvalsh(cov)
            eigvals = np.abs(eigvals); eigvals = eigvals[eigvals > 1e-10]
            if len(eigvals) == 0: return 0.0
            probs = eigvals / np.sum(eigvals)
            entropy = -np.sum(probs * np.log(probs))
            max_ent = np.log(len(probs))
            return float(entropy / max_ent) if max_ent > 0 else 0.0
        except: return 0.0

    def _detect_bifurcation(self, tee_values):
        if len(tee_values) < 2: return False
        return np.var(list(tee_values.values())) > 0.1

    def _compute_theosis(self, tee, spectral_ent):
        exponent = -tee * PHI_SQUARED * (1 + spectral_ent)
        return max(0.0, min(1.0, float(np.exp(exponent))))

    def update(self, embedding, logits=None, layer_activations=None):
        vec = np.asarray(embedding, dtype=np.float32).flatten()
        if vec.shape[0] != self.dim: vec = np.pad(vec, (0, max(0, self.dim - vec.shape[0])))[:self.dim]
        self._cycle += 1
        for buf in self._buffers.values(): buf.append(vec.copy())
        self._global_buffer.append(vec.copy())
        if len(self._global_buffer) < 3: return None
        tee_values = {}
        for w, buf in self._buffers.items():
            t = self._compute_tee(buf, use_rkhs=True)
            if t is not None: tee_values[w] = t
        if not tee_values: return None
        weights = {w: 1.0 / w for w in tee_values}
        total_w = sum(weights.values())
        tee_aggregate = sum(v * weights[w] for w, v in tee_values.items()) / total_w
        tee_mu = float(np.mean(list(tee_values.values())))
        spectral_ent = self._spectral_entropy(self._global_buffer)
        theosis = self._compute_theosis(tee_aggregate, spectral_ent)
        self._bifurcation_detected = self._detect_bifurcation(tee_values)
        if self._bifurcation_detected: self._bifurcation_count += 1
        delta_theta = abs(theosis - self._last_theosis)
        self._ema_short_val = (1 - self.ema_short) * self._ema_short_val + self.ema_short * delta_theta
        self._ema_long_val = (1 - self.ema_long) * self._ema_long_val + self.ema_long * delta_theta
        refined = min(1.0, 0.7 * self._ema_short_val + 0.3 * self._ema_long_val + 0.1 * tee_aggregate)
        gate = self._compute_gate(tee_aggregate, theosis)
        reading = {
            "cycle": self._cycle, "theosis": round(theosis, 6), "tee": round(tee_aggregate, 6),
            "tee_mu": round(tee_mu, 6), "tee_per_scale": {str(w): round(v, 6) for w, v in tee_values.items()},
            "refined_fatigue": round(refined, 6), "spectral_entropy": round(spectral_ent, 6),
            "bifurcation_detected": self._bifurcation_detected, "bifurcation_count": self._bifurcation_count,
            "gate": gate.name, "timestamp": time.time(),
        }
        self._last_theosis = theosis; self.readings.append(reading)
        return reading

    def _compute_gate(self, tee, theosis):
        th = self._gate_thresholds
        if tee > th["emergency_tee"] or theosis < th["emergency_theta"]: return GateState.EMERGENCY
        if tee > th["locked_tee"] and theosis < th["locked_theta"]: return GateState.LOCKED
        if tee > th["restricted_tee"] or theosis < th["restricted_theta"]: return GateState.RESTRICTED
        if tee > th["caution_tee"] or theosis < th["caution_theta"]: return GateState.CAUTION
        return GateState.OPEN

    def get_stats(self):
        if not self.readings: return {"n_readings": 0}
        theosis = [r["theosis"] for r in self.readings]; tees = [r["tee"] for r in self.readings]
        gates = [r["gate"] for r in self.readings]
        from collections import Counter
        return {"n_readings": len(self.readings), "theosis_mean": round(float(np.mean(theosis)), 6),
                "theosis_min": round(float(np.min(theosis)), 6), "theosis_max": round(float(np.max(theosis)), 6),
                "theosis_std": round(float(np.std(theosis)), 6), "tee_mean": round(float(np.mean(tees)), 6),
                "tee_max": round(float(np.max(tees)), 6), "gate_distribution": dict(Counter(gates)),
                "last_gate": gates[-1], "bifurcations": self._bifurcation_count}

    def reset(self):
        for buf in self._buffers.values(): buf.clear()
        self._global_buffer.clear(); self._last_theosis = 1.0; self._ema_short_val = 0.0
        self._ema_long_val = 0.0; self._cycle = 0; self._bifurcation_detected = False
        self._bifurcation_count = 0; self.readings.clear()

    def get_telemetry(self):
        return {"module": "VectorTheosis1092", "version": "4.0.0", "substrate": "1091.2",
                "seal": "VECTOR-THEOSIS-1091.2-v4.0.0-2026-06-07", "dim": self.dim,
                "window_sizes": list(self.window_sizes), "rkhs_bandwidth": self.rkhs_bandwidth,
                "n_readings": len(self.readings), "stats": self.get_stats()}
