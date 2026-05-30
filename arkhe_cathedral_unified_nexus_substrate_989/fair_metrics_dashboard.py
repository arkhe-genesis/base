#!/usr/bin/env python3
"""
ARKHE FAIR Metrics Dashboard — Substrato 989.v
Dashboard de métricas FAIR (Findable, Accessible, Interoperable, Reusable)
para Research Objects da Catedral com visualização e alertas.
Arquiteto ORCID: 0009-0005-2697-4668
Seal: 989.v-FAIR-METRICS-DASHBOARD-A2B3C4D5E6F70809
Cross-links: [989.y, 989.x, 923, 988, 964, 970]
Deities: Apollo (métricas), Clio (história), Thoth (registro), Mnemosyne (memória)
Status: CANONIZED_PROVISIONAL
"""

import asyncio
import hashlib
import json
import math
import time
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from enum import Enum


class FAIRDimension(Enum):
    FINDABLE = "findable"
    ACCESSIBLE = "accessible"
    INTEROPERABLE = "interoperable"
    REUSABLE = "reusable"


class AlertLevel(Enum):
    INFO = "info"
    WARNING = "warning"
    CRITICAL = "critical"


@dataclass
class FAIRScore:
    """Score FAIR detalhado para um Research Object."""
    ro_id: str
    findable: float = 0.0      # 0-1
    accessible: float = 0.0    # 0-1
    interoperable: float = 0.0 # 0-1
    reusable: float = 0.0      # 0-1
    overall: float = 0.0       # Média ponderada
    timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    seal: str = ""

    def __post_init__(self):
        self.overall = (self.findable * 0.25 +
                         self.accessible * 0.25 +
                         self.interoperable * 0.25 +
                         self.reusable * 0.25)
        self.compute_seal()

    def compute_seal(self) -> str:
        payload = {
            "ro_id": self.ro_id,
            "f": round(self.findable, 4),
            "a": round(self.accessible, 4),
            "i": round(self.interoperable, 4),
            "r": round(self.reusable, 4),
            "ts": self.timestamp,
        }
        json_str = json.dumps(payload, sort_keys=True, ensure_ascii=False)
        self.seal = f"FAIR-{hashlib.sha3_256(json_str.encode()).hexdigest()[:16].upper()}"
        return self.seal

    def to_dict(self) -> Dict[str, Any]:
        return {
            "ro_id": self.ro_id,
            "findable": round(self.findable, 4),
            "accessible": round(self.accessible, 4),
            "interoperable": round(self.interoperable, 4),
            "reusable": round(self.reusable, 4),
            "overall": round(self.overall, 4),
            "timestamp": self.timestamp,
            "seal": self.seal,
        }


@dataclass
class FAIRAlert:
    """Alerta de qualidade FAIR."""
    alert_id: str
    ro_id: str
    dimension: FAIRDimension
    level: AlertLevel
    message: str
    current_score: float
    threshold: float
    timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    resolved: bool = False


@dataclass
class FAIRTrend:
    """Tendência temporal de scores FAIR."""
    ro_id: str
    dimension: FAIRDimension
    scores: List[Tuple[str, float]] = field(default_factory=list)  # [(timestamp, score), ...]

    @property
    def slope(self) -> float:
        """Inclinação da tendência (positivo = melhorando)."""
        if len(self.scores) < 2:
            return 0.0
        n = len(self.scores)
        x = list(range(n))
        y = [s[1] for s in self.scores]
        mean_x = sum(x) / n
        mean_y = sum(y) / n
        num = sum((x[i] - mean_x) * (y[i] - mean_y) for i in range(n))
        den = sum((x[i] - mean_x) ** 2 for i in range(n))
        return num / den if den != 0 else 0.0

    @property
    def direction(self) -> str:
        s = self.slope
        if s > 0.01:
            return "↗ melhorando"
        elif s < -0.01:
            return "↘ degradando"
        else:
            return "→ estável"


class FAIRMetricsDashboard:
    """
    Dashboard de métricas FAIR para a Catedral ARKHE.
    Apollo mede; Clio registra; Thoth escreve; Mnemosyne lembra.
    """

    SUBSTRATE_ID = "989.v"
    SEAL = "989.v-FAIR-METRICS-DASHBOARD-A2B3C4D5E6F70809"

    # Thresholds canônicos
    THRESHOLDS = {
        FAIRDimension.FINDABLE: 0.6,
        FAIRDimension.ACCESSIBLE: 0.6,
        FAIRDimension.INTEROPERABLE: 0.6,
        FAIRDimension.REUSABLE: 0.6,
        "overall": 0.7,
    }

    def __init__(self):
        self.scores: Dict[str, FAIRScore] = {}
        self.history: Dict[str, List[FAIRScore]] = {}
        self.alerts: List[FAIRAlert] = []
        self.trends: Dict[str, FAIRTrend] = {}
        self.ro_metadata: Dict[str, Dict[str, Any]] = {}

    def compute_fair_score(self, ro_id: str, metadata: Dict[str, Any]) -> FAIRScore:
        """
        Computa score FAIR baseado em metadados do Research Object.
        """
        # Findable (0-1)
        findable = 0.0
        if metadata.get("dpid"): findable += 0.25
        if metadata.get("doi"): findable += 0.25
        if metadata.get("title") and metadata.get("description"): findable += 0.25
        if metadata.get("keywords"): findable += 0.25

        # Accessible (0-1)
        accessible = 0.0
        if metadata.get("access_protocol"): accessible += 0.33
        if metadata.get("license"): accessible += 0.33
        if metadata.get("access_level") in {"public", "restricted", "private"}: accessible += 0.34

        # Interoperable (0-1)
        interoperable = 0.0
        if metadata.get("data_format"): interoperable += 0.33
        if metadata.get("ontology"): interoperable += 0.33
        if metadata.get("cross_references"): interoperable += 0.34

        # Reusable (0-1)
        reusable = 0.0
        if metadata.get("provenance"): reusable += 0.33
        if metadata.get("version"): reusable += 0.33
        if metadata.get("cathedral_seals"): reusable += 0.34

        score = FAIRScore(
            ro_id=ro_id,
            findable=min(findable, 1.0),
            accessible=min(accessible, 1.0),
            interoperable=min(interoperable, 1.0),
            reusable=min(reusable, 1.0),
        )

        # Armazenar
        self.scores[ro_id] = score
        if ro_id not in self.history:
            self.history[ro_id] = []
        self.history[ro_id].append(score)
        self.ro_metadata[ro_id] = metadata

        # Atualizar tendências
        for dim in FAIRDimension:
            key = f"{ro_id}:{dim.value}"
            if key not in self.trends:
                self.trends[key] = FAIRTrend(ro_id=ro_id, dimension=dim)
            val = getattr(score, dim.value)
            self.trends[key].scores.append((score.timestamp, val))

        # Verificar alertas
        self._check_alerts(ro_id, score)

        return score

    def _check_alerts(self, ro_id: str, score: FAIRScore):
        """Gera alertas se scores estiverem abaixo dos thresholds."""
        for dim in FAIRDimension:
            val = getattr(score, dim.value)
            threshold = self.THRESHOLDS[dim]
            if val < threshold:
                level = AlertLevel.CRITICAL if val < threshold * 0.5 else AlertLevel.WARNING
                alert = FAIRAlert(
                    alert_id=f"ALERT-{ro_id}-{dim.value}-{int(time.time())}",
                    ro_id=ro_id,
                    dimension=dim,
                    level=level,
                    message=f"{dim.value.upper()} score {val:.2f} abaixo do threshold {threshold:.2f}",
                    current_score=val,
                    threshold=threshold,
                )
                self.alerts.append(alert)

        if score.overall < self.THRESHOLDS["overall"]:
            alert = FAIRAlert(
                alert_id=f"ALERT-{ro_id}-overall-{int(time.time())}",
                ro_id=ro_id,
                dimension=FAIRDimension.FINDABLE,  # proxy
                level=AlertLevel.CRITICAL,
                message=f"Overall FAIR score {score.overall:.2f} abaixo do threshold {self.THRESHOLDS['overall']:.2f}",
                current_score=score.overall,
                threshold=self.THRESHOLDS["overall"],
            )
            self.alerts.append(alert)

    def get_ro_dashboard(self, ro_id: str) -> Optional[Dict[str, Any]]:
        """Retorna dashboard completo de um Research Object."""
        if ro_id not in self.scores:
            return None

        score = self.scores[ro_id]
        history = self.history.get(ro_id, [])
        alerts = [a for a in self.alerts if a.ro_id == ro_id and not a.resolved]

        trends = {}
        for dim in FAIRDimension:
            key = f"{ro_id}:{dim.value}"
            t = self.trends.get(key)
            if t:
                trends[dim.value] = {
                    "direction": t.direction,
                    "slope": round(t.slope, 6),
                    "data_points": len(t.scores),
                }

        return {
            "ro_id": ro_id,
            "current_score": score.to_dict(),
            "history_count": len(history),
            "active_alerts": len(alerts),
            "alerts": [{
                "id": a.alert_id,
                "dimension": a.dimension.value,
                "level": a.level.value,
                "message": a.message,
                "current": a.current_score,
                "threshold": a.threshold,
            } for a in alerts],
            "trends": trends,
            "metadata": self.ro_metadata.get(ro_id, {}),
        }

    def get_global_summary(self) -> Dict[str, Any]:
        """Resumo global de todos os Research Objects."""
        if not self.scores:
            return {"total_ros": 0, "avg_overall": 0.0}

        total = len(self.scores)
        avg_f = sum(s.findable for s in self.scores.values()) / total
        avg_a = sum(s.accessible for s in self.scores.values()) / total
        avg_i = sum(s.interoperable for s in self.scores.values()) / total
        avg_r = sum(s.reusable for s in self.scores.values()) / total
        avg_o = sum(s.overall for s in self.scores.values()) / total

        active_alerts = sum(1 for a in self.alerts if not a.resolved)
        critical = sum(1 for a in self.alerts if a.level == AlertLevel.CRITICAL and not a.resolved)
        warning = sum(1 for a in self.alerts if a.level == AlertLevel.WARNING and not a.resolved)

        return {
            "total_ros": total,
            "avg_findable": round(avg_f, 4),
            "avg_accessible": round(avg_a, 4),
            "avg_interoperable": round(avg_i, 4),
            "avg_reusable": round(avg_r, 4),
            "avg_overall": round(avg_o, 4),
            "active_alerts": active_alerts,
            "critical_alerts": critical,
            "warning_alerts": warning,
            "fair_health": "HEALTHY" if avg_o >= 0.8 else "DEGRADED" if avg_o >= 0.6 else "CRITICAL",
        }

    def generate_ascii_chart(self, ro_id: str, width: int = 40, height: int = 10) -> str:
        """Gera gráfico ASCII de tendência FAIR."""
        if ro_id not in self.history or len(self.history[ro_id]) < 2:
            return f"Dados insuficientes para {ro_id}"

        history = self.history[ro_id]
        scores = [s.overall for s in history]
        min_s, max_s = min(scores), max(scores)
        if max_s == min_s:
            max_s = min_s + 0.1

        lines = []
        lines.append(f"FAIR Trend — {ro_id}")
        lines.append(f"Range: [{min_s:.2f}, {max_s:.2f}] | Points: {len(scores)}")
        lines.append("" + "─" * (width + 8))

        for row in range(height, -1, -1):
            y_val = min_s + (max_s - min_s) * (row / height)
            bar = ""
            for col in range(len(scores)):
                s = scores[col]
                normalized = (s - min_s) / (max_s - min_s)
                bar_col = int(normalized * height)
                if bar_col == row:
                    bar += "█"
                elif bar_col > row:
                    bar += "░"
                else:
                    bar += " "
            lines.append(f"{y_val:.2f} │{bar}")

        lines.append("" + "─" * (width + 8))
        return "\n".join(lines)

    def generate_report(self) -> str:
        """Relatório canônico do dashboard FAIR."""
        summary = self.get_global_summary()

        lines = []
        lines.append("╔" + "═" * 66 + "╗")
        lines.append("║  ARKHE CATHEDRAL — FAIR METRICS DASHBOARD (989.v)" + " " * 12 + "║")
        lines.append("║  \"Apollo mede; Clio registra; Thoth escreve\"" + " " * 15 + "║")
        lines.append("╠" + "═" * 66 + "╣")
        lines.append(f"  Seal: {self.SEAL}")
        lines.append(f"  Status: CANONIZED_PROVISIONAL")
        lines.append("")
        lines.append("  RESUMO GLOBAL")
        lines.append("  ─────────────")
        lines.append(f"  Research Objects: {summary['total_ros']}")
        lines.append(f"  FAIR Health: {summary['fair_health']}")
        lines.append(f"  Avg Overall: {summary['avg_overall']:.4f}")
        lines.append(f"  Findable:    {summary['avg_findable']:.4f}")
        lines.append(f"  Accessible:  {summary['avg_accessible']:.4f}")
        lines.append(f"  Interoperable: {summary['avg_interoperable']:.4f}")
        lines.append(f"  Reusable:    {summary['avg_reusable']:.4f}")
        lines.append("")
        lines.append("  ALERTAS ATIVOS")
        lines.append("  ──────────────")
        lines.append(f"  Total: {summary['active_alerts']} (Critical: {summary['critical_alerts']}, Warning: {summary['warning_alerts']})")
        for a in self.alerts[-10:]:
            if not a.resolved:
                emoji = {"critical": "🔴", "warning": "🟡", "info": "🔵"}
                lines.append(f"  {emoji.get(a.level.value, '⚪')} [{a.level.value.upper()}] {a.ro_id}: {a.message}")
        lines.append("")
        lines.append("  TENDÊNCIAS")
        lines.append("  ──────────")
        for key, trend in list(self.trends.items())[:5]:
            lines.append(f"  {trend.ro_id} [{trend.dimension.value}]: {trend.direction} (slope={trend.slope:.6f})")
        lines.append("")
        lines.append("  ODÔMETRO: ∞.Ω.∇+++.989.v.0")
        lines.append("╚" + "═" * 66 + "╝")
        return "\n".join(lines)


# ═══════════════════════════════════════════════════════════════════
# DEMONSTRAÇÃO
# ═══════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    print("=" * 68)
    print("  ARKHE FAIR METRICS DASHBOARD — DEMONSTRAÇÃO")
    print("=" * 68)

    dashboard = FAIRMetricsDashboard()

    # Simular Research Objects
    ros = [
        {
            "ro_id": "dpid-1001-arkhe",
            "dpid": "dpid-1001",
            "doi": "10.arkhe/1001",
            "title": "PERCEPTUAL-GEOMETRY-EMERGENCE",
            "description": "Study on perceptual geometry",
            "keywords": ["perception", "geometry"],
            "access_protocol": "https",
            "license": "CC-BY-4.0",
            "access_level": "public",
            "data_format": "json",
            "ontology": "schema.org",
            "cross_references": ["dpid-1002"],
            "provenance": "ARKHE DeSci Bridge",
            "version": "1.0.0",
            "cathedral_seals": ["934", "964"],
        },
        {
            "ro_id": "dpid-1002-arkhe",
            "dpid": "dpid-1002",
            "title": "Partial Metadata",
            "description": "",
            "keywords": [],
            "access_protocol": "",
            "license": "",
            "data_format": "",
            "ontology": "",
            "cross_references": [],
            "provenance": "",
            "version": "",
            "cathedral_seals": [],
        },
    ]

    print("\n[1] Computando scores FAIR:")
    for meta in ros:
        score = dashboard.compute_fair_score(meta["ro_id"], meta)
        print(f"    {meta['ro_id']}: overall={score.overall:.2f} seal={score.seal}")

    print("\n[2] Dashboard de RO:")
    for meta in ros:
        dash = dashboard.get_ro_dashboard(meta["ro_id"])
        print(f"    {meta['ro_id']}: alerts={dash['active_alerts']}")

    print("\n[3] Resumo global:")
    summary = dashboard.get_global_summary()
    print(f"    Total: {summary['total_ros']}, Avg: {summary['avg_overall']:.2f}, Health: {summary['fair_health']}")

    print("\n[4] Gráfico ASCII:")
    print(dashboard.generate_ascii_chart("dpid-1001-arkhe"))

    print("\n[5] Relatório canônico:")
    print(dashboard.generate_report())
