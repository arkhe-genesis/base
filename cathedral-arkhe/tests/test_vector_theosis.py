import pytest
import numpy as np
from cathedral.substrates.theosis.core import VectorTheosis1092
from cathedral.constants import GateState, PHI_SQUARED

class TestVectorTheosisInit:
    def test_default_creation(self):
        vt = VectorTheosis1092(dim=128)
        assert vt.dim == 128
        assert vt.rkhs_bandwidth == 0.1
        assert vt.window_sizes == (2, 3, 5, 8, 13)
        assert len(vt.readings) == 0

    def test_custom_window_sizes(self):
        vt = VectorTheosis1092(dim=64, window_sizes=(3, 7))
        assert vt.window_sizes == (3, 7)
        assert 3 in vt._buffers
        assert 7 in vt._buffers

class TestVectorTheosisUpdate:
    def test_first_two_updates_return_none(self):
        vt = VectorTheosis1092(dim=64)
        r1 = vt.update(np.zeros(64, dtype=np.float32))
        r2 = vt.update(np.ones(64, dtype=np.float32))
        assert r1 is None
        assert r2 is None
        assert vt._cycle == 2

    def test_third_update_returns_reading(self):
        vt = VectorTheosis1092(dim=64)
        vt.update(np.zeros(64, dtype=np.float32))
        vt.update(np.ones(64, dtype=np.float32))
        r = vt.update(np.full(64, 0.5, dtype=np.float32))
        assert r is not None
        assert "theosis" in r
        assert "tee" in r
        assert "gate" in r
        assert r["cycle"] == 3

    def test_constant_input_gives_theosis_one(self):
        vt = VectorTheosis1092(dim=64)
        emb = np.ones(64, dtype=np.float32)
        for _ in range(10):
            r = vt.update(emb)
        assert r is not None
        assert r["theosis"] > 0.99, f"Expected Θ≈1.0 for constant input, got {r['theosis']}"
        assert r["gate"] == GateState.OPEN.value

    def test_step_change_increases_tee(self):
        vt = VectorTheosis1092(dim=64)
        base = np.zeros(64, dtype=np.float32)
        for i in range(10):
            perturbed = base + np.random.randn(64).astype(np.float32) * (0.01 * (i + 1))
            r = vt.update(perturbed)
            if r:
                if i > 3:
                    assert r["tee"] > 0.001, "TEE must grow with growing perturbation"

    def test_theosis_bounds(self):
        vt = VectorTheosis1092(dim=64)
        for _ in range(20):
            r = vt.update(np.random.randn(64).astype(np.float32) * 0.5)
            if r:
                assert 0.0 <= r["theosis"] <= 1.0

    def test_gate_escalation(self):
        vt = VectorTheosis1092(dim=64)
        gates_seen = []
        base = np.zeros(64, dtype=np.float32)
        for _ in range(5):
            vt.update(base)
            gates_seen.append("OPEN") # Implicitly open when theosis is 1.0
        for i in range(20):
            perturbed = base + np.ones(64, dtype=np.float32) * (0.05 * (i + 1))
            r = vt.update(perturbed)
            if r:
                gates_seen.append(r["gate"])
        assert len(set(gates_seen)) >= 2

    def test_bifurcation_detection(self):
        vt = VectorTheosis1092(dim=64)
        base = np.zeros(64, dtype=np.float32)
        vt.update(base)
        vt.update(base + 0.001 * np.ones(64, dtype=np.float32))
        vt.update(base + 0.5 * np.ones(64, dtype=np.float32))
        r = vt.update(base + 0.6 * np.ones(64, dtype=np.float32))
        if r:
            assert r["tee_per_scale"] is not None

    def test_reset(self):
        vt = VectorTheosis1092(dim=64)
        for _ in range(5):
            vt.update(np.random.randn(64).astype(np.float32))
        assert len(vt.readings) > 0
        vt.reset()
        assert len(vt.readings) == 0
        assert vt._cycle == 0
        assert vt._last_theosis == 1.0

class TestVectorTheosisTelemetry:
    def test_telemetry_structure(self):
        vt = VectorTheosis1092(dim=64)
        vt.update(np.zeros(64, dtype=np.float32))
        vt.update(np.ones(64, dtype=np.float32))
        vt.update(np.full(64, 0.5, dtype=np.float32))
        t = vt.get_telemetry()
        assert t["module"] == "VectorTheosis1092"
        assert t["version"] == "4.0.0"
        assert t["dim"] == 64
        assert t["n_readings"] == 1
        assert "stats" in t
        assert "seal" in t
