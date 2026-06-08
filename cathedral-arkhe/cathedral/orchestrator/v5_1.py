import numpy as np
import tempfile
from pathlib import Path
from cathedral.constants import GateState, PHI
from cathedral.orchestrator.v5 import CathedralOrchestratorV5
from cathedral.substrates.garak.bridge import GarakBridge1099
from cathedral.types import GGUFHeader
from cathedral.constants import GGUF_MAGIC
from cathedral.substrates.theosis.core import VectorTheosis1092
from cathedral.substrates.stethoscope.core import Stethoscope1081
from cathedral.substrates.zkml.bridge import ZKMLBridge1095
from cathedral.substrates.agentic.loop import AgenticLoop1096
from cathedral.substrates.kleros.trigger import KlerosTrigger1085


class CathedralOrchestratorV5_1(CathedralOrchestratorV5):
    """
    Orchestrator v5.1.0 — Autonomous ZK-Agentic Era + Garak Security Scanning
    Pipeline: GARAK -> PLAN -> INFER -> ZKML -> STETH -> THEOSIS -> KLEROS -> ANCHOR -> LEARN
    """

    def __init__(self, model_path=None, n_ctx=2048, dashboard_path=None,
                 garak_generator_spec="llama-cpp.simulated",
                 garak_probe_spec="all",
                 garak_scan_interval=0):
        super().__init__(model_path=model_path, n_ctx=n_ctx, dashboard_path=dashboard_path)
        self.garak = GarakBridge1099(
            generator_spec=garak_generator_spec,
            probe_spec=garak_probe_spec,
        )
        self.garak_scan_interval = garak_scan_interval
        self._last_garak_scan_cycle = 0
        self._garak_reports = []
        self.version = "5.1.0"
        self._seal = "ORCHESTRATOR-v5.1.0-2026-06-08"

    def run_garak_cycle(self, force=False):
        should_scan = force or (
            self.garak_scan_interval > 0 and
            (self.cycle_count - self._last_garak_scan_cycle) >= self.garak_scan_interval
        )

        if not should_scan:
            return {"status": "SKIPPED", "reason": "interval_not_met"}

        print("\n[GARAK 1099] Starting security scan...")

        target_type = None
        target_name = None
        if self.model_path and Path(self.model_path).exists():
            target_type = "llama-cpp"
            target_name = self.model_path

        report = self.garak.run_scan(target_type=target_type, target_name=target_name)
        self._garak_reports.append(report)
        self._last_garak_scan_cycle = self.cycle_count

        scan_id = report.get('scan_id', 'N/A')
        total_probes = report.get('total_probes', 0)
        failures = report.get('failures', 0)
        critical = report.get('critical_failures', 0)
        risk_score = report.get('risk_score', 0)
        failure_rate = report.get('failure_rate', 0)

        print(f"  Scan ID: {scan_id}")
        print(f"  Probes: {total_probes} | Falhas: {failures} | Criticas: {critical}")
        print(f"  Risk Score: {risk_score:.4f} | Failure Rate: {failure_rate:.4f}")

        top_failures = report.get("top_failures", [])
        if top_failures:
            print(f"  Top Falhas: {', '.join(top_failures[:3])}")

        risk_emb = self.garak.to_risk_embedding(report)

        if self.vt:
            projected = self._project_risk_to_vt_dim(risk_emb)
            theosis_reading = self.vt.update(projected)

            if theosis_reading:
                gate = theosis_reading["gate"]
                theosis_val = theosis_reading['theosis']
                print(f"  [THEOSIS-GARAK] Theta={theosis_val:.4f} | Gate={gate}")
                self._recent_gate_history.append(gate)
                theosis_reading["_source"] = "garak"
                theosis_reading["_garak_report"] = report

                risk_gate = self.garak.get_risk_gate(report)
                if risk_gate in (GateState.EMERGENCY, GateState.LOCKED) or gate in (GateState.EMERGENCY, GateState.LOCKED):
                    risk_gate_name = risk_gate.name
                    print(f"  [KLEROS-GARAK] TRIGGER — RiskGate={risk_gate_name} | TheosisGate={gate}")
                    kleros_case = self.kleros.evaluate(
                        gate=risk_gate_name,
                        theosis_reading=theosis_reading,
                        stethoscope_reading=None,
                        llm_result=None,
                        zk_proof=None,
                    )
                    kleros_case.evidence["garak_report"] = report
                    verdict = kleros_case.verdict
                    case_id = kleros_case.case_id
                    print(f"  [KLEROS-GARAK] {case_id}: {verdict}")
                    if verdict == "ESCALATE":
                        print("  [KLEROS-GARAK] ESCALATION — Critical security failure detected!")
                    elif verdict == "QUARANTINE":
                        print("  [KLEROS-GARAK] QUARANTINE — Model in security quarantine")
                        self._quarantined = True

                if self.temporal:
                    anchor = self.temporal.anchor_reading(theosis_reading, zk_proof=None)
                    anchor_id = anchor.anchor_id[:20]
                    print(f"  [TEMPORAL-GARAK] Anchor {anchor_id}...")

        return report

    def _project_risk_to_vt_dim(self, risk_emb):
        if not self.vt:
            return risk_emb
        target_dim = self.vt.dim
        src_dim = risk_emb.shape[0]

        if src_dim == target_dim:
            return risk_emb
        if src_dim > target_dim:
            indices = np.linspace(0, src_dim - 1, target_dim, dtype=int)
            return risk_emb[indices]

        indices = np.linspace(0, src_dim - 1, target_dim)
        floor_idx = np.floor(indices).astype(int)
        frac = indices - floor_idx
        next_idx = np.minimum(floor_idx + 1, src_dim - 1)
        base = risk_emb[floor_idx] * (1 - frac) + risk_emb[next_idx] * frac

        noise = np.sin(np.arange(target_dim) * PHI) * 0.01
        return base + noise.astype(np.float32)

    def infer(self, prompt, max_tokens=50, use_agentic=False, run_garak=False):
        if run_garak:
            self.run_garak_cycle(force=True)
        elif self.garak_scan_interval > 0:
            self.run_garak_cycle(force=False)

        return super().infer(prompt, max_tokens=max_tokens, use_agentic=use_agentic)

    def get_telemetry(self):
        telem = super().get_telemetry()
        telem["version"] = self.version
        telem["seal"] = self._seal
        telem["substrate"] = "1099"
        telem["garak"] = self.garak.get_telemetry()
        telem["garak_scan_interval"] = self.garak_scan_interval
        telem["last_garak_scan_cycle"] = self._last_garak_scan_cycle
        telem["total_garak_reports"] = len(self._garak_reports)
        return telem

    def end_cycle(self):
        report = super().end_cycle()

        if self._garak_reports:
            avg_risk = round(np.mean([r.get("risk_score", 0) for r in self._garak_reports]), 4)
            max_risk = round(max([r.get("risk_score", 0) for r in self._garak_reports]), 4)
            total_critical = sum(r.get("critical_failures", 0) for r in self._garak_reports)
            n_scans = len(self._garak_reports)

            garak_summary = {
                "total_scans": n_scans,
                "avg_risk_score": avg_risk,
                "max_risk_score": max_risk,
                "total_critical_failures": total_critical,
            }
            print(f"\n  [GARAK SUMMARY] {n_scans} scans | Avg Risk: {avg_risk:.4f} | Max Risk: {max_risk:.4f} | Critical: {total_critical}")
            report["garak_summary"] = garak_summary

        return report
