import numpy as np
from collections import deque
import time


class Stethoscope1081:
    def __init__(self, n_layers=32, dim=4096, n_heads=32, anomaly_window=5):
        self.n_layers = n_layers; self.dim = dim; self.n_heads = n_heads
        self.anomaly_window = anomaly_window
        self._layer_norms = {i: deque(maxlen=anomaly_window * 3) for i in range(n_layers)}
        self._layer_cosines = {i: deque(maxlen=anomaly_window * 3) for i in range(n_layers - 1)}
        self._layer_rates = {i: deque(maxlen=anomaly_window * 3) for i in range(n_layers)}
        self._trajectory = []; self._readings = []; self._step = 0
        self._fft_buffer = deque(maxlen=64)

    def feed_logits_trajectory(self, logits_sequence, embedding):
        self._step += 1
        reading = {"step": self._step, "n_tokens": len(logits_sequence),
                   "per_token_metrics": [], "aggregate": {}, "anomalies": [],
                   "spectral": {}, "timestamp": time.time()}
        prev_h = None
        for t_idx, logits in enumerate(logits_sequence):
            logits_f = np.asarray(logits, dtype=np.float32).flatten()
            projected = self._project_to_dim(logits_f, self.dim)
            h = projected; norm = float(np.linalg.norm(h))
            cosine = 0.0
            if prev_h is not None:
                denom = norm * float(np.linalg.norm(prev_h)) + 1e-12
                cosine = float(np.dot(h, prev_h) / denom)
            rate = 0.0
            if prev_h is not None:
                rate = float(np.linalg.norm(h - prev_h) / (norm + 1e-12))
            probs = np.exp(logits_f - np.max(logits_f))
            probs = probs / (np.sum(probs) + 1e-12)
            entropy = float(-np.sum(probs * np.log(probs + 1e-12)))
            token_metrics = {"token_idx": t_idx, "norm": round(norm, 4),
                             "cosine_prev": round(cosine, 4), "rate": round(rate, 4),
                             "entropy": round(entropy, 4), "top_token": int(np.argmax(logits_f)),
                             "top_prob": round(float(probs[np.argmax(logits_f)]), 4)}
            reading["per_token_metrics"].append(token_metrics)
            anomaly = self._check_anomaly(t_idx, norm, cosine, rate, entropy)
            if anomaly: reading["anomalies"].append(anomaly)
            prev_h = h

        norms = [m["norm"] for m in reading["per_token_metrics"]]
        cosines = [m["cosine_prev"] for m in reading["per_token_metrics"] if m["token_idx"] > 0]
        rates = [m["rate"] for m in reading["per_token_metrics"] if m["token_idx"] > 0]
        entropies = [m["entropy"] for m in reading["per_token_metrics"]]
        reading["aggregate"] = {
            "mean_norm": round(float(np.mean(norms)), 4),
            "std_norm": round(float(np.std(norms)), 4),
            "mean_cosine": round(float(np.mean(cosines)), 4) if cosines else 0.0,
            "min_cosine": round(float(np.min(cosines)), 4) if cosines else 0.0,
            "mean_rate": round(float(np.mean(rates)), 4) if rates else 0.0,
            "max_rate": round(float(np.max(rates)), 4) if rates else 0.0,
            "mean_entropy": round(float(np.mean(entropies)), 4),
            "min_entropy": round(float(np.min(entropies)), 4),
            "entropy_decay": round(float(entropies[-1] - entropies[0]) if len(entropies) > 1 else 0.0, 4),
        }

        self._fft_buffer.append(np.mean(norms))
        if len(self._fft_buffer) >= 8:
            fft_vals = np.array(list(self._fft_buffer))
            fft_result = np.fft.rfft(fft_vals)
            freqs = np.fft.rfftfreq(len(fft_vals))
            reading["spectral"] = {
                "dominant_freq": round(float(freqs[np.argmax(np.abs(fft_result[1:])) + 1]), 4) if len(freqs) > 1 else 0.0,
                "spectral_energy": round(float(np.sum(np.abs(fft_result)**2)), 4),
            }

        self._trajectory.append(np.asarray(embedding, dtype=np.float32).flatten()[:self.dim])
        self._readings.append(reading)
        return reading

    def _project_to_dim(self, vec, target_dim):
        src_dim = vec.shape[0]
        if src_dim == target_dim: return vec
        if src_dim > target_dim:
            indices = np.linspace(0, src_dim - 1, target_dim, dtype=int)
            return vec[indices]
        indices = np.linspace(0, src_dim - 1, target_dim)
        floor_idx = np.floor(indices).astype(int)
        frac = indices - floor_idx
        next_idx = np.minimum(floor_idx + 1, src_dim - 1)
        return vec[floor_idx] * (1 - frac) + vec[next_idx] * frac

    def _check_anomaly(self, idx, norm, cosine, rate, entropy):
        anomalies = []
        if norm < 1e-3: anomalies.append(("COLLAPSE", f"norma={norm:.6f}"))
        if cosine < -0.8: anomalies.append(("OSCILLATION", f"cosine={cosine:.4f}"))
        if rate > 2.0: anomalies.append(("SPIKE", f"rate={rate:.4f}"))
        if entropy < 0.1 and entropy > 0: anomalies.append(("ENTROPY_COLLAPSE", f"entropy={entropy:.4f}"))
        if anomalies:
            return {"index": idx, "types": [a[0] for a in anomalies], "details": [a[1] for a in anomalies]}
        return None

    def get_spectral_analysis(self):
        mat = self.get_trajectory_matrix()
        if mat is None or mat.shape[0] < 3: return {"status": "INSUFFICIENT_DATA"}
        cov = np.cov(mat.T)
        try:
            eigvals = np.linalg.eigvalsh(cov)
            eigvals = np.sort(eigvals)[::-1]
            total_var = np.sum(eigvals) + 1e-12
            cum_var = np.cumsum(eigvals) / total_var
            n_eff = int(np.searchsorted(cum_var, 0.95)) + 1
            cond = float(eigvals[0] / (eigvals[-1] + 1e-12))
            return {"status": "OK", "top_5_eigenvalues": [round(float(e), 4) for e in eigvals[:5]],
                    "effective_dim_95": n_eff, "condition_number": round(cond, 2),
                    "total_variance": round(float(total_var), 4)}
        except: return {"status": "LIN_ALG_ERROR"}

    def get_trajectory_matrix(self):
        if not self._trajectory: return None
        return np.array(self._trajectory)

    def reset(self):
        for d in self._layer_norms.values(): d.clear()
        for d in self._layer_cosines.values(): d.clear()
        for d in self._layer_rates.values(): d.clear()
        self._trajectory.clear(); self._readings.clear(); self._step = 0; self._fft_buffer.clear()

    def get_telemetry(self):
        return {"module": "Stethoscope1081", "version": "3.0.0", "substrate": "1081.1",
                "seal": "STETHOSCOPE-1081.1-v3.0.0-2026-06-07",
                "n_layers": self.n_layers, "dim": self.dim, "n_heads": self.n_heads,
                "steps": self._step, "trajectory_length": len(self._trajectory),
                "anomalies_total": sum(len(r["anomalies"]) for r in self._readings),
                "spectral": self.get_spectral_analysis()}
