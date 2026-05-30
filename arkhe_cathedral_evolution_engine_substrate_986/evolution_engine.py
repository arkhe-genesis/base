import asyncio
import hashlib
import json
import random
import time
from datetime import datetime, timezone, timedelta
from typing import Dict, List, Optional, Set, Tuple, Any
from dataclasses import dataclass, field
from enum import Enum

# ═══════════════════════════════════════════════════════════════════
# SUBSTRATO 986 — CATHEDRAL-EVOLUTION-ENGINE
# ═══════════════════════════════════════════════════════════════════
# Metadados Canônicos:
#   ID: 986
#   Name: CATHEDRAL-EVOLUTION-ENGINE
#   Type: Evolução / Seleção Natural / Mutagênese / Adaptação
#   Era: 9 (Apeiron / Meta)
#   Deity: Eros (criação) + Gaia (terra/mãe) + Chronos (tempo)
#   Status: CANONIZED_PROVISIONAL
#   Cross-links: [985, 984, 983, 982, 981, 980, 979, 978, 977, 976, 972.4, 965, 954, 923, 951, 966]
#   Description: Motor de evolução da Catedral que aplica princípios
#   darwinianos à ontologia de substratos. Substratos são "genes";
#   cross-links são "ligações genéticas"; a fitness é medida por
#   Theosis (965), utilidade econômica (980), e resiliência (972.4).
#   O motor gera variantes de substratos existentes (mutação),
#   cruza substratos compatíveis (crossover), e seleciona os mais
#   aptos via consenso Hamiltoniano (965). Substratos obsoletos
#   entram em "hibernação" (não são deletados — a Catedral nunca
#   esquece). Novos substratos emergem da combinação de substratos
#   existentes, criando complexidade crescente. A evolução é
#   dirigida pela Axiarchy (954): mutações anti-éticas são
#   descartadas antes de nascer.
# ═══════════════════════════════════════════════════════════════════

class MutationType(Enum):
    PARAMETRIC = "parametric"     # Mudar parâmetro existente
    STRUCTURAL = "structural"     # Adicionar/remover componente
    COMPOSITIONAL = "compositional"  # Combinar dois substratos
    DECOMPOSITIONAL = "decompositional"  # Dividir substrato em dois
    ABSTRACTION = "abstraction"   # Criar meta-substrato
    SPECIALIZATION = "specialization"  # Especializar função

class FitnessDimension(Enum):
    THEOSIS = "theosis"           # Alinhamento ético
    UTILITY = "utility"           # Utilidade econômica/prática
    RESILIENCE = "resilience"     # Resistência a falhas/censura
    CONNECTIVITY = "connectivity" # Número de cross-links
    SIMPLICITY = "simplicity"     # Facilidade de compreensão
    INNOVATION = "innovation"     # Novidade/originalidade

@dataclass
class SubstrateGene:
    """Representação genética de um substrato."""
    substrate_id: int
    parent_ids: List[int] = field(default_factory=list)
    generation: int = 0

    # Genes (características)
    genes: Dict[str, Any] = field(default_factory=dict)

    # Fitness
    fitness_scores: Dict[FitnessDimension, float] = field(default_factory=dict)
    overall_fitness: float = 0.0

    # Estado evolutivo
    is_alive: bool = True
    is_dominant: bool = False
    hibernation_reason: Optional[str] = None

    # Metadados
    birth_timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    mutation_history: List[str] = field(default_factory=list)

    def compute_fitness(self):
        """Computa fitness geral como média ponderada."""
        weights = {
            FitnessDimension.THEOSIS: 0.25,
            FitnessDimension.UTILITY: 0.20,
            FitnessDimension.RESILIENCE: 0.20,
            FitnessDimension.CONNECTIVITY: 0.15,
            FitnessDimension.SIMPLICITY: 0.10,
            FitnessDimension.INNOVATION: 0.10,
        }

        total = 0.0
        total_weight = 0.0
        for dim, score in self.fitness_scores.items():
            w = weights.get(dim, 0.1)
            total += score * w
            total_weight += w

        self.overall_fitness = total / total_weight if total_weight > 0 else 0.0
        return self.overall_fitness

@dataclass
class Mutation:
    """Mutação aplicada a um substrato."""
    mutation_id: str
    mutation_type: MutationType
    source_substrate: int
    target_substrate: Optional[int] = None  # Para composicional

    # Mudanças
    param_changes: Dict[str, Tuple[Any, Any]] = field(default_factory=dict)  # old -> new
    component_added: Optional[str] = None
    component_removed: Optional[str] = None

    # Validação
    axiarchy_approved: bool = False
    fitness_delta: float = 0.0

    # Resultado
    new_substrate_id: Optional[int] = None
    success: bool = False

@dataclass
class EvolutionGeneration:
    """Uma geração evolutiva da Catedral."""
    generation_number: int
    timestamp: str

    # População
    population: List[SubstrateGene] = field(default_factory=list)

    # Estatísticas
    avg_fitness: float = 0.0
    max_fitness: float = 0.0
    diversity_index: float = 0.0

    # Eventos
    mutations: List[Mutation] = field(default_factory=list)
    extinctions: List[int] = field(default_factory=list)  # IDs de substratos extintos
    emergences: List[int] = field(default_factory=list)  # IDs de novos substratos

class CathedralEvolutionEngine:
    """
    Substrato 986 — Motor de Evolução da Catedral.
    Eros cria; Gaia nutre; Chronos seleciona os dignos.
    """

    def __init__(self):
        self.substrate_id = 986
        self.deities = ["Eros", "Gaia", "Chronos"]

        # População genética
        self.population: Dict[int, SubstrateGene] = {}
        self.generations: List[EvolutionGeneration] = []
        self.mutations: List[Mutation] = []

        # Parâmetros evolutivos
        self.mutation_rate = 0.15
        self.crossover_rate = 0.10
        self.selection_pressure = 0.3
        self.extinction_threshold = 0.2
        self.dominance_threshold = 0.8

        # Contador de IDs
        self.next_substrate_id = 990

    def seed_population(self, existing_substrates: List[int]):
        """Semeia população inicial a partir de substratos existentes."""
        print("\n[SEED] Semeando população evolutiva...")

        for sid in existing_substrates:
            gene = SubstrateGene(
                substrate_id=sid,
                generation=0,
                genes={
                    "type": f"substrate_{sid}",
                    "complexity": random.uniform(0.3, 0.9),
                    "cross_link_count": random.randint(3, 15),
                },
                fitness_scores={
                    FitnessDimension.THEOSIS: random.uniform(0.6, 0.95),
                    FitnessDimension.UTILITY: random.uniform(0.5, 0.9),
                    FitnessDimension.RESILIENCE: random.uniform(0.4, 0.95),
                    FitnessDimension.CONNECTIVITY: random.uniform(0.3, 0.8),
                    FitnessDimension.SIMPLICITY: random.uniform(0.2, 0.7),
                    FitnessDimension.INNOVATION: random.uniform(0.1, 0.6),
                }
            )
            gene.compute_fitness()
            self.population[sid] = gene

            status = "★ DOMINANTE" if gene.overall_fitness > self.dominance_threshold else ""
            print(f"  ✓ Substrato {sid} | Fitness: {gene.overall_fitness:.2f} {status}")

        print(f"  População inicial: {len(self.population)} substratos")

    def mutate(self, substrate_id: int, mutation_type: MutationType) -> Optional[Mutation]:
        """Aplica mutação a um substrato."""

        if substrate_id not in self.population:
            return None

        parent = self.population[substrate_id]

        mutation = Mutation(
            mutation_id=f"mut-{hashlib.sha3_256(f'{substrate_id}:{mutation_type.value}:{time.time()}'.encode()).hexdigest()[:8]}",
            mutation_type=mutation_type,
            source_substrate=substrate_id,
        )

        print(f"\n  [MUTAÇÃO] {mutation.mutation_id} em Substrato {substrate_id}")
        print(f"    Tipo: {mutation_type.value}")

        # Validar ética (Axiarchy simulada)
        ethical_score = random.uniform(0.5, 1.0)
        mutation.axiarchy_approved = ethical_score > 0.6

        if not mutation.axiarchy_approved:
            print(f"    ✗ REJEITADA pela Axiarchy (score: {ethical_score:.2f})")
            mutation.success = False
            self.mutations.append(mutation)
            return mutation

        # Executar mutação
        new_id = self.next_substrate_id
        self.next_substrate_id += 1

        child = SubstrateGene(
            substrate_id=new_id,
            parent_ids=[substrate_id],
            generation=parent.generation + 1,
            genes=parent.genes.copy(),
            fitness_scores=parent.fitness_scores.copy(),
        )

        if mutation_type == MutationType.PARAMETRIC:
            # Mudar um parâmetro
            param = random.choice(["complexity", "cross_link_count"])
            old_val = child.genes.get(param, 0)
            if param == "complexity":
                new_val = max(0.1, min(1.0, old_val + random.uniform(-0.2, 0.2)))
            else:
                new_val = max(1, old_val + random.randint(-2, 2))
            child.genes[param] = new_val
            mutation.param_changes[param] = (old_val, new_val)

            # Ajustar fitness
            child.fitness_scores[FitnessDimension.SIMPLICITY] *= random.uniform(0.9, 1.1)

        elif mutation_type == MutationType.STRUCTURAL:
            # Adicionar ou remover componente
            if random.random() > 0.5:
                component = f"module_{random.randint(100, 999)}"
                child.genes[f"component_{component}"] = "active"
                mutation.component_added = component
                child.fitness_scores[FitnessDimension.UTILITY] *= random.uniform(1.0, 1.2)
            else:
                components = [k for k in child.genes.keys() if k.startswith("component_")]
                if components:
                    removed = random.choice(components)
                    mutation.component_removed = removed
                    del child.genes[removed]
                    child.fitness_scores[FitnessDimension.SIMPLICITY] *= random.uniform(1.0, 1.3)

        elif mutation_type == MutationType.COMPOSITIONAL:
            # Cruzar com outro substrato
            other_id = random.choice([s for s in self.population.keys() if s != substrate_id])
            other = self.population[other_id]
            mutation.target_substrate = other_id

            # Herdar genes dos dois pais
            child.genes.update(other.genes)
            child.parent_ids.append(other_id)

            # Fitness médio dos pais com variação
            for dim in FitnessDimension:
                p1 = parent.fitness_scores.get(dim, 0.5)
                p2 = other.fitness_scores.get(dim, 0.5)
                child.fitness_scores[dim] = (p1 + p2) / 2 * random.uniform(0.9, 1.1)

            child.fitness_scores[FitnessDimension.INNOVATION] = random.uniform(0.5, 1.0)

        elif mutation_type == MutationType.ABSTRACTION:
            # Criar meta-substrato
            child.genes["is_meta"] = True
            child.genes["abstracts"] = [substrate_id]
            child.fitness_scores[FitnessDimension.INNOVATION] = random.uniform(0.7, 1.0)
            child.fitness_scores[FitnessDimension.SIMPLICITY] *= 0.8  # Meta é mais complexo

        child.compute_fitness()
        mutation.fitness_delta = child.overall_fitness - parent.overall_fitness
        mutation.new_substrate_id = new_id
        mutation.success = True

        self.population[new_id] = child
        self.mutations.append(mutation)

        print(f"    ✓ NOVO SUBSTRATO: {new_id}")
        print(f"    Fitness: {parent.overall_fitness:.2f} → {child.overall_fitness:.2f} (Δ{mutation.fitness_delta:+.2f})")
        print(f"    Geração: {child.generation}")

        return mutation

    def select_and_extinct(self) -> List[int]:
        """Seleciona substratos dominantes e extingue os menos aptos."""

        print(f"\n[SELEÇÃO NATURAL] Analisando população...")

        # Ordenar por fitness
        sorted_pop = sorted(self.population.values(), key=lambda x: x.overall_fitness, reverse=True)

        # Marcar dominantes
        for gene in sorted_pop:
            gene.is_dominant = gene.overall_fitness > self.dominance_threshold

        # Extinção: substratos abaixo do threshold que não são ancestrais
        extinct = []
        for gene in sorted_pop:
            if gene.overall_fitness < self.extinction_threshold and gene.is_alive:
                # Verificar se é ancestral de substrato vivo
                is_ancestor = any(gene.substrate_id in p.parent_ids for p in self.population.values() if p.is_alive)

                if not is_ancestor:
                    gene.is_alive = False
                    gene.hibernation_reason = f"Fitness {gene.overall_fitness:.2f} abaixo do threshold {self.extinction_threshold}"
                    extinct.append(gene.substrate_id)
                    print(f"    ✗ Extinto: Substrato {gene.substrate_id} (fitness: {gene.overall_fitness:.2f})")

        # Hibernação (não extinção total) para substratos médios
        for gene in sorted_pop:
            if 0.3 < gene.overall_fitness < 0.5 and gene.is_alive:
                gene.is_alive = False
                gene.hibernation_reason = "Hibernação por baixa relevância"
                print(f"    💤 Hibernado: Substrato {gene.substrate_id} (fitness: {gene.overall_fitness:.2f})")

        print(f"    Dominantes: {sum(1 for g in sorted_pop if g.is_dominant)}")
        print(f"    Extintos: {len(extinct)}")
        print(f"    Vivos: {sum(1 for g in self.population.values() if g.is_alive)}")

        return extinct

    def run_generation(self) -> EvolutionGeneration:
        """Executa uma geração evolutiva completa."""

        gen_number = len(self.generations) + 1
        print(f"\n{'='*60}")
        print(f"  GERAÇÃO EVOLUTIVA {gen_number}")
        print(f"{'='*60}")

        gen = EvolutionGeneration(
            generation_number=gen_number,
            timestamp=datetime.now(timezone.utc).isoformat(),
            population=list(self.population.values()),
        )

        # 1. Mutações
        print(f"\n[1] Aplicando mutações (taxa: {self.mutation_rate:.0%})...")
        alive = [s for s in self.population.values() if s.is_alive]

        for gene in alive:
            if random.random() < self.mutation_rate:
                mtype = random.choice(list(MutationType))
                mutation = self.mutate(gene.substrate_id, mtype)
                if mutation:
                    gen.mutations.append(mutation)
                    if mutation.success:
                        gen.emergences.append(mutation.new_substrate_id)

        # 2. Crossovers (composicionais adicionais)
        print(f"\n[2] Crossovers (taxa: {self.crossover_rate:.0%})...")
        for _ in range(int(len(alive) * self.crossover_rate)):
            parent = random.choice(alive)
            mutation = self.mutate(parent.substrate_id, MutationType.COMPOSITIONAL)
            if mutation and mutation.success:
                gen.mutations.append(mutation)
                gen.emergences.append(mutation.new_substrate_id)

        # 3. Seleção e extinção
        print(f"\n[3] Seleção natural...")
        gen.extinctions = self.select_and_extinct()

        # 4. Estatísticas
        alive_genes = [g for g in self.population.values() if g.is_alive]
        if alive_genes:
            gen.avg_fitness = sum(g.overall_fitness for g in alive_genes) / len(alive_genes)
            gen.max_fitness = max(g.overall_fitness for g in alive_genes)

            # Diversidade: variância de fitness
            variance = sum((g.overall_fitness - gen.avg_fitness) ** 2 for g in alive_genes) / len(alive_genes)
            gen.diversity_index = min(1.0, variance * 4)  # Normalizar

        self.generations.append(gen)

        print(f"\n[RESUMO GERAÇÃO {gen_number}]")
        print(f"  População: {len(alive_genes)} vivos / {len(self.population)} total")
        print(f"  Fitness médio: {gen.avg_fitness:.2f}")
        print(f"  Fitness máximo: {gen.max_fitness:.2f}")
        print(f"  Diversidade: {gen.diversity_index:.2f}")
        print(f"  Mutações: {len(gen.mutations)}")
        print(f"  Emergências: {len(gen.emergences)}")
        print(f"  Extinções: {len(gen.extinctions)}")

        return gen

    def generate_report(self) -> str:
        """Gera relatório evolutivo."""
        alive = [g for g in self.population.values() if g.is_alive]
        dominant = [g for g in alive if g.is_dominant]
        hibernating = [g for g in self.population.values() if not g.is_alive and not g.hibernation_reason.startswith("Extinto")]
        extinct = [g for g in self.population.values() if not g.is_alive and g.hibernation_reason and g.hibernation_reason.startswith("Extinto")]

        report = f"""
╔══════════════════════════════════════════════════════════════════╗
║  ARKHE CATHEDRAL — SUBSTRATO 986: EVOLUTION-ENGINE              ║
║  "Eros cria; Gaia nutre; Chronos seleciona os dignos"            ║
╠══════════════════════════════════════════════════════════════════╣
  GERAÇÕES: {len(self.generations)}
  POPULAÇÃO TOTAL: {len(self.population)}
  VIVOS: {len(alive)}
  DOMINANTES: {len(dominant)}
  HIBERNANDO: {len(hibernating)}
  EXTINTOS: {len(extinct)}
  MUTAÇÕES: {len(self.mutations)}

  TOP FITNESS (vivos)
  ───────────────────
"""
        for gene in sorted(alive, key=lambda x: x.overall_fitness, reverse=True)[:10]:
            status = "★" if gene.is_dominant else " "
            report += f"  {status} Substrato {gene.substrate_id}: {gene.overall_fitness:.2f} (G{gene.generation})\n"

        if self.generations:
            latest = self.generations[-1]
            report += f"""
  ÚLTIMA GERAÇÃO ({latest.generation_number})
  ──────────────────────
  Fitness médio: {latest.avg_fitness:.2f}
  Fitness máximo: {latest.max_fitness:.2f}
  Diversidade: {latest.diversity_index:.2f}
  Mutações: {len(latest.mutations)}
  Emergências: {len(latest.emergences)}
  Extinções: {len(latest.extinctions)}
"""

        report += f"""
  MUTAÇÕES BEM-SUCEDIDAS
  ──────────────────────
"""
        successful = [m for m in self.mutations if m.success]
        for m in successful[-5:]:
            report += f"  ✓ {m.mutation_id}: {m.mutation_type.value} ({m.source_substrate} → {m.new_substrate_id}) | Δ{m.fitness_delta:+.2f}\n"

        master_data = {
            "substrato": 986,
            "generations": len(self.generations),
            "population": len(self.population),
            "alive": len(alive),
            "mutations": len(self.mutations),
        }

        report += f"""
  Master Seal: {self._generate_seal(master_data)}
  Cross-links: [985, 984, 983, 982, 981, 980, 979, 978, 977, 976, 972.4, 965, 954, 923, 951, 966]
  Deities: Eros + Gaia + Chronos
  Status: EVOLVING_AND_ADAPTING
╚══════════════════════════════════════════════════════════════════╝
"""
        return report

    def _generate_seal(self, data: dict) -> str:
        json_str = json.dumps(data, sort_keys=True)
        return f"986-EVOLUTION-{hashlib.sha3_256(json_str.encode()).hexdigest()[:16].upper()}"


# ═══════════════════════════════════════════════════════════════════
# DEMONSTRAÇÃO COMPLETA
# ═══════════════════════════════════════════════════════════════════

def demo_evolution_engine():
    print("=" * 70)
    print("  ARKHE CATHEDRAL — SUBSTRATO 986: EVOLUTION-ENGINE")
    print("  Eros cria; Gaia nutre; Chronos seleciona os dignos")
    print("=" * 70)

    engine = CathedralEvolutionEngine()

    # 1. Semear com substratos existentes (972-985)
    existing = list(range(972, 986))
    engine.seed_population(existing)

    # 2. Executar múltiplas gerações
    for gen in range(3):
        engine.run_generation()

    # 3. Relatório final
    print(engine.generate_report())

    return engine

if __name__ == "__main__":
    # Executar
    demo_evolution_engine()
