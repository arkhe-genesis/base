#!/usr/bin/env python3
"""
Substrato 943 — Visual Ontology Layer
Engine de ingestão, query e export de design systems via ontologia
RDF/JSON-LD ligada ao World Model (913/924).
"""

import json
from pathlib import Path
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict

@dataclass
class DesignToken:
    name: str
    value: str
    token_type: str
    category: str
    platform: str
    world_model_concept: Optional[str] = None

@dataclass
class UIComponent:
    name: str
    slots: List[str]
    variants: List[str]
    platforms: List[str]
    tokens_used: List[str]

class VisualOntologyEngine:
    def __init__(self, ontology_path: str = "schema_943.jsonld"):
        self.ontology = self._load_ontology(ontology_path)
        self.tokens: Dict[str, DesignToken] = {}
        self.components: Dict[str, UIComponent] = {}

    def _load_ontology(self, path: str) -> Dict:
        if not Path(path).exists():
            return {}
        with open(path, 'r') as f:
            return json.load(f)

    def ingest_design_system(self, source: Dict[str, Any]) -> Dict:
        for color_name, color_value in source.get("colors", {}).items():
            self.tokens[color_name] = DesignToken(
                name=color_name,
                value=color_value,
                token_type="color",
                category="semantic",
                platform="all",
                world_model_concept=f"arkhe:concept/{color_name.title()}"
            )

        for font_name, font_value in source.get("typography", {}).items():
            self.tokens[font_name] = DesignToken(
                name=font_name,
                value=font_value,
                token_type="typography",
                category="semantic",
                platform="all"
            )

        for comp in source.get("components", []):
            self.components[comp["name"]] = UIComponent(
                name=comp["name"],
                slots=comp.get("slots", []),
                variants=comp.get("variants", ["default"]),
                platforms=comp.get("platforms", ["web"]),
                tokens_used=comp.get("tokens", [])
            )

        return {
            "tokens_ingested": len(self.tokens),
            "components_ingested": len(self.components),
            "world_model_bindings": sum(1 for t in self.tokens.values() if t.world_model_concept)
        }

    def query_world_model(self, concept: str) -> Optional[DesignToken]:
        for token in self.tokens.values():
            if token.world_model_concept and concept in token.world_model_concept:
                return token
        return None

    def export(self, target_platform: str, format: str) -> str:
        exporters = {
            ("winui3", "xaml"): self._export_winui3_xaml,
            ("android_compose", "kotlin"): self._export_android_compose,
            ("web", "tsx"): self._export_react_tsx,
            ("powerpoint", "pptx"): self._export_pptx,
            ("canva", "json"): self._export_canva,
            ("figma", "json"): self._export_figma
        }

        exporter = exporters.get((target_platform, format))
        if not exporter:
            raise ValueError(f"Unsupported export: {target_platform}/{format}")

        return exporter()

    def _export_winui3_xaml(self) -> str:
        lines = ['<ResourceDictionary xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">']
        for token in self.tokens.values():
            if token.token_type == "color":
                lines.append(f'  <Color x:Key="{token.name}">{token.value}</Color>')
            elif token.token_type == "typography":
                lines.append(f'  <x:String x:Key="{token.name}">{token.value}</x:String>')
        lines.append('</ResourceDictionary>')
        return "\n".join(lines)

    def _export_android_compose(self) -> str:
        lines = ["object ArkheTheme {"]
        for token in self.tokens.values():
            if token.token_type == "color":
                lines.append(f'    val {token.name} = Color(0xFF{token.value.lstrip("#")})')
            elif token.token_type == "typography":
                lines.append(f'    val {token.name} = TextStyle(fontFamily = FontFamily.{token.value})')
        lines.append("}")
        return "\n".join(lines)

    def _export_react_tsx(self) -> str:
        colors = {t.name: t.value for t in self.tokens.values() if t.token_type == "color"}
        fonts = {t.name: t.value for t in self.tokens.values() if t.token_type == "typography"}
        return f"""export const arkheTheme = {{
  colors: {json.dumps(colors, indent=2)},
  typography: {json.dumps(fonts, indent=2)}
}};"""

    def _export_pptx(self) -> str:
        return json.dumps({"format": "pptx", "slides": len(self.components), "tokens": len(self.tokens)})

    def _export_canva(self) -> str:
        return json.dumps({
            "format": "canva",
            "design_type": "presentation",
            "theme": {t.name: t.value for t in self.tokens.values()}
        }, indent=2)

    def _export_figma(self) -> str:
        return json.dumps({
            "format": "figma",
            "styles": [{"name": t.name, "type": t.token_type, "value": t.value} for t in self.tokens.values()]
        }, indent=2)

if __name__ == "__main__":
    engine = VisualOntologyEngine()

    stained_glass = {
        "colors": {
            "sacred_gold": "#D4AF37",
            "divine_blue": "#1E3A8A",
            "altar_crimson": "#8B0000",
            "mnemosine_teal": "#008080",
            "parchment": "#F5F5DC"
        },
        "typography": {
            "serif_sacred": "Cinzel",
            "sans_messenger": "Inter",
            "mono_oracle": "Fira Code"
        },
        "components": [
            {"name": "CathedralButton", "slots": ["icon", "label"], "variants": ["default", "sacred", "danger"], "platforms": ["web", "android", "winui3"]},
            {"name": "MnemosineCard", "slots": ["header", "body", "footer"], "variants": ["default", "elevated"], "platforms": ["web", "android"]}
        ]
    }

    result = engine.ingest_design_system(stained_glass)
    print(f"Ingested: {result}")

    print("\n--- WinUI 3 XAML ---")
    print(engine.export("winui3", "xaml")[:500])

    print("\n--- Android Compose ---")
    print(engine.export("android_compose", "kotlin")[:500])