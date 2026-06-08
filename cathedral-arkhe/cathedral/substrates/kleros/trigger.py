import time
from datetime import datetime, timezone
from cathedral.types import KlerosVerdict


class KlerosTrigger1085:
    def __init__(self, escalation_tee=0.50, quarantine_tee=0.20, dismiss_recovery=0.3):
        self.escalation_tee = escalation_tee; self.quarantine_tee = quarantine_tee
        self.dismiss_recovery = dismiss_recovery
        self.cases = []; self._case_counter = 0; self._active_quarantine = False
        self._quarantine_since = None; self._temporal_chain = None

    def set_temporal_chain(self, tc): self._temporal_chain = tc

    def evaluate(self, gate, theosis_reading, stethoscope_reading=None,
                 llm_result=None, zk_proof=None):
        self._case_counter += 1
        case_id = f"KLR-{self._case_counter:06d}-{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}"
        tee = theosis_reading.get("tee", 0.0); theosis = theosis_reading.get("theosis", 0.0)
        fatigue = theosis_reading.get("refined_fatigue", 0.0)
        spec_div = theosis_reading.get("spectral_divergence", 0.0)
        bifurcation = theosis_reading.get("bifurcation_detected", False)
        anomalies = stethoscope_reading.get("anomalies", []) if stethoscope_reading else []
        agg = stethoscope_reading.get("aggregate", {}) if stethoscope_reading else {}

        urgency = 0.0
        urgency += min(tee / 1.0, 1.0) * 0.30
        urgency += (1 - theosis) * 0.20
        urgency += fatigue * 0.15
        urgency += min(len(anomalies) / 5.0, 1.0) * 0.15
        urgency += min(agg.get("max_rate", 0) / 3.0, 1.0) * 0.10
        urgency += (0.2 if bifurcation else 0.0)

        if urgency >= 0.70 or tee >= self.escalation_tee: verdict = "ESCALATE"
        elif urgency >= 0.40 or tee >= self.quarantine_tee: verdict = "QUARANTINE"
        elif urgency >= 0.20: verdict = "MONITOR"
        else:
            recent_gates = theosis_reading.get("_recent_gates", [])
            verdict = "DISMISS" if len(recent_gates) >= 2 and recent_gates[-2] == "OPEN" else "MONITOR"

        evidence = {"urgency_score": round(urgency, 4), "tee": tee, "theosis": theosis,
                    "fatigue": fatigue, "spectral_divergence": spec_div, "bifurcation": bifurcation,
                    "n_anomalies": len(anomalies),
                    "anomaly_types": list(set(a["types"][0] for a in anomalies)) if anomalies else [],
                    "stethoscope_aggregate": agg,
                    "llm_status": llm_result.get("status", "N/A") if llm_result else "N/A"}

        case = KlerosVerdict(case_id=case_id, trigger_gate=gate,
                             trigger_reason={"urgency": urgency,
                                             "primary_factor": self._primary_factor(tee, theosis, len(anomalies), bifurcation)},
                             verdict=verdict, evidence=evidence, timestamp=time.time(),
                             zk_proof_hash=zk_proof.proof_id if zk_proof else None)
        self.cases.append(case)
        if verdict == "QUARANTINE": self._active_quarantine = True; self._quarantine_since = time.time()
        if self._temporal_chain: self._temporal_chain.anchor_reading(theosis_reading, zk_proof)
        return case

    def _primary_factor(self, tee, theosis, n_anomalies, bifurcation):
        if tee >= self.escalation_tee: return "TEE_CRITICAL"
        if theosis < 0.1: return "THEOSIS_COLLAPSE"
        if bifurcation: return "BIFURCATION"
        if n_anomalies >= 3: return "MULTI_ANOMALY"
        if tee >= self.quarantine_tee: return "TEE_ELEVATED"
        return "FATIGUE_ACCUMULATION"

    def check_quarantine(self):
        if not self._active_quarantine: return {"in_quarantine": False}
        duration = time.time() - (self._quarantine_since or time.time())
        return {"in_quarantine": True, "duration_seconds": round(duration, 2),
                "recommendation": "Await resolution" if duration < 300 else "Auto-resolve: TIMEOUT"}

    def get_telemetry(self):
        from collections import Counter
        verdicts = [c.verdict for c in self.cases]
        return {"module": "KlerosTrigger1085", "version": "2.0.0", "substrate": "1085.1",
                "seal": "KLEROS-TRIGGER-1085.1-v2.0.0-2026-06-07",
                "total_cases": len(self.cases),
                "unresolved": sum(1 for c in self.cases if not c.resolved),
                "verdict_distribution": dict(Counter(verdicts)) if verdicts else {},
                "in_quarantine": self._active_quarantine}
