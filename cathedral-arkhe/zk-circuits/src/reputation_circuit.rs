//! zk-circuits/src/reputation_circuit.rs — Circuito ZK para Merkle Path Verification
//! Selo: CATHEDRAL-ARKHE-REPUTATION-ZK-CIRCUIT-v1.0.0

use anyhow::{anyhow, Result};
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::iop::target::Target;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, CircuitData};
use plonky2::plonk::config::{PoseidonGoldilocksConfig, Hasher};
use plonky2::plonk::proof::ProofWithPublicInputs;
use serde::{Deserialize, Serialize};
use plonky2::field::types::Field;

pub const D: usize = 2;
pub type F = GoldilocksField;
pub type C = PoseidonGoldilocksConfig;

// ============================================================
// ENTRADAS DO CIRCUITO
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationWitness {
    pub leaf_hash: [u64; 4],
    pub sibling_hashes: Vec<[u64; 4]>,
    pub leaf_index: u64,
    pub expected_root: [u64; 4],
}

// ============================================================
// CONSTRUÇÃO DO CIRCUITO
// ============================================================

pub struct ReputationMerkleCircuit {
    pub circuit_data: CircuitData<F, C, D>,
    pub leaf_hash_targets: Vec<Target>,
    pub sibling_targets: Vec<Vec<Target>>,
    pub leaf_index_target: Target,
    pub root_target: Vec<Target>,
}

impl ReputationMerkleCircuit {
    pub fn new(max_depth: usize) -> Self {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        let mut root_target = Vec::new();
        for _ in 0..4 {
            let t = builder.add_virtual_target();
            builder.register_public_input(t);
            root_target.push(t);
        }

        let leaf_hash_targets: Vec<Target> = (0..4).map(|_| builder.add_virtual_target()).collect();

        let mut sibling_targets = Vec::new();
        for _ in 0..max_depth {
            let mut siblings = Vec::new();
            for _ in 0..4 {
                siblings.push(builder.add_virtual_target());
            }
            sibling_targets.push(siblings);
        }

        let leaf_index_target = builder.add_virtual_target();

        let mut current_hash = leaf_hash_targets.clone();

        for (level, siblings) in sibling_targets.iter().enumerate() {
            let bit_target = builder.le_bit(leaf_index_target, level as u32);

            let mut left = Vec::new();
            let mut right = Vec::new();
            for i in 0..4 {
                left.push(builder.select(bit_target, siblings[i], current_hash[i]));
                right.push(builder.select(bit_target, current_hash[i], siblings[i]));
            }

            let hash_input = [left.clone(), right.clone()].concat();
            let hash_output = PoseidonHash::hash_no_pad(&mut builder, &hash_input);
            current_hash = hash_output.elements.to_vec();
        }

        for i in 0..4 {
            builder.connect(current_hash[i], root_target[i]);
        }

        let circuit_data = builder.build::<C>();

        Self {
            circuit_data,
            leaf_hash_targets,
            sibling_targets,
            leaf_index_target,
            root_target,
        }
    }

    pub fn prove(
        &self,
        witness: &ReputationWitness,
    ) -> Result<ProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::new();

        for (i, &val) in witness.leaf_hash.iter().enumerate() {
            pw.set_target(self.leaf_hash_targets[i], F::from_canonical_u64(val));
        }

        for (level, siblings) in self.sibling_targets.iter().enumerate() {
            if level < witness.sibling_hashes.len() {
                for (i, &val) in witness.sibling_hashes[level].iter().enumerate() {
                    pw.set_target(siblings[i], F::from_canonical_u64(val));
                }
            } else {
                for target in siblings {
                    pw.set_target(*target, F::ZERO);
                }
            }
        }

        pw.set_target(self.leaf_index_target, F::from_canonical_u64(witness.leaf_index));

        for (i, &val) in witness.expected_root.iter().enumerate() {
            pw.set_target(self.root_target[i], F::from_canonical_u64(val));
        }

        self.circuit_data.prove(pw)
            .map_err(|e| anyhow!("Falha ao gerar prova: {}", e))
    }

    pub fn verify(&self, proof: &ProofWithPublicInputs<F, C, D>) -> Result<bool> {
        self.circuit_data.verify(proof.clone())
            .map_err(|e| anyhow!("Verificação falhou: {}", e))
            .map(|_| true)
    }
}

pub struct ReputationZkAdapter {
    circuit: ReputationMerkleCircuit,
}

impl ReputationZkAdapter {
    pub fn new(max_depth: usize) -> Self {
        Self {
            circuit: ReputationMerkleCircuit::new(max_depth),
        }
    }

    pub fn generate_proof(
        &self,
        leaf_hash: [u64; 4],
        siblings: Vec<[u64; 4]>,
        leaf_index: u64,
        root: [u64; 4],
    ) -> Result<Vec<u8>> {
        let witness = ReputationWitness {
            leaf_hash,
            sibling_hashes: siblings,
            leaf_index,
            expected_root: root,
        };

        let proof = self.circuit.prove(&witness)?;

        bincode::serialize(&proof)
            .map_err(|e| anyhow!("Erro ao serializar prova: {}", e))
    }

    pub fn verify_proof(&self, proof_bytes: &[u8]) -> Result<bool> {
        let proof: ProofWithPublicInputs<F, C, D> = bincode::deserialize(proof_bytes)
            .map_err(|e| anyhow!("Erro ao desserializar prova: {}", e))?;
        self.circuit.verify(&proof)
    }
}
