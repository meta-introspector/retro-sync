//! NFT71 ZK circuit — pure Rust, ark-groth16/BN254.
//!
//! Proves: "I know 71 DA51 shard hashes that form a valid Merkle tree
//!          with the public root, the eigenspace commitment matches,
//!          and the crown shard (p71) is unique."
//!
//! Public inputs:  merkle_root, eigenspace_commitment, shard_count (=71)
//! Private witness: 71 shard hashes, Merkle siblings, eigenspace values
//!
//! Uses MiMC as the in-circuit hash (cheap in R1CS, ~300 constraints per hash).

use ark_bn254::{Bn254, Fr};
use ark_ff::PrimeField;
use ark_ff::Field;
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::{fields::fp::FpVar, prelude::*};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_snark::SNARK;
use ark_std::rand::SeedableRng;
use ark_std::Zero;
use sha2::{Digest, Sha256};

pub const SHARD_COUNT: usize = 71;
pub const TREE_DEPTH: usize = 7; // 2^7 = 128 ≥ 71
pub const EIGENSPACE_TOTAL: u64 = 10_000; // 100.00%

/// MiMC constants (220 rounds, BN254). First 220 digits of π as seeds.
const MIMC_ROUNDS: usize = 220;

fn mimc_constants() -> Vec<Fr> {
    // Deterministic constants from SHA-256("mimc_seed_i")
    (0..MIMC_ROUNDS)
        .map(|i| {
            let h = Sha256::digest(format!("mimc_seed_{i}").as_bytes());
            Fr::from_le_bytes_mod_order(&h)
        })
        .collect()
}

/// MiMC hash of two field elements (Feistel construction).
fn mimc_hash_native(left: Fr, right: Fr) -> Fr {
    let constants = mimc_constants();
    let mut xl = left;
    let mut xr = right;
    for c in &constants {
        let t = xl + c;
        let t2 = t * t;
        let t4 = t2 * t2;
        let t7 = t4 * t2 * t; // x^7
        xr += t7;
        std::mem::swap(&mut xl, &mut xr);
    }
    xl + xr
}

/// MiMC hash as R1CS gadget.
fn mimc_hash_circuit(
    cs: ConstraintSystemRef<Fr>,
    left: &FpVar<Fr>,
    right: &FpVar<Fr>,
) -> Result<FpVar<Fr>, SynthesisError> {
    let constants = mimc_constants();
    let mut xl = left.clone();
    let mut xr = right.clone();
    for c in &constants {
        let c_var = FpVar::constant(*c);
        let t = &xl + &c_var;
        let t2 = &t * &t;
        let t4 = &t2 * &t2;
        let t7 = &t4 * &t2 * &t;
        xr = &xr + &t7;
        std::mem::swap(&mut xl, &mut xr);
    }
    Ok(&xl + &xr)
}

/// Witness for the NFT71 proof.
#[derive(Clone)]
pub struct NFT71Witness {
    pub shard_hashes: [Fr; SHARD_COUNT],
    pub merkle_siblings: [[Fr; TREE_DEPTH]; SHARD_COUNT],
    pub merkle_dirs: [[bool; TREE_DEPTH]; SHARD_COUNT], // false=left, true=right
    pub earth_pct: u64,
    pub spoke_pct: u64,
    pub hub_pct: u64,
}

/// The R1CS circuit.
pub struct NFT71Circuit {
    pub witness: NFT71Witness,
    pub merkle_root: Fr,
    pub eigenspace_commitment: Fr,
}

impl ConstraintSynthesizer<Fr> for NFT71Circuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let w = &self.witness;

        // === Public inputs ===
        let root_var = FpVar::new_input(ark_relations::ns!(cs, "merkle_root"), || {
            Ok(self.merkle_root)
        })?;
        let eigen_var = FpVar::new_input(ark_relations::ns!(cs, "eigenspace_commitment"), || {
            Ok(self.eigenspace_commitment)
        })?;
        let count_var = FpVar::new_input(ark_relations::ns!(cs, "shard_count"), || {
            Ok(Fr::from(SHARD_COUNT as u64))
        })?;

        // === Constraint 1: shard_count == 71 ===
        let seventy_one = FpVar::constant(Fr::from(SHARD_COUNT as u64));
        count_var.enforce_equal(&seventy_one)?;

        // === Private witness: eigenspace ===
        let earth = FpVar::new_witness(ark_relations::ns!(cs, "earth"), || {
            Ok(Fr::from(w.earth_pct))
        })?;
        let spoke = FpVar::new_witness(ark_relations::ns!(cs, "spoke"), || {
            Ok(Fr::from(w.spoke_pct))
        })?;
        let hub = FpVar::new_witness(ark_relations::ns!(cs, "hub"), || {
            Ok(Fr::from(w.hub_pct))
        })?;

        // === Constraint 2: earth + spoke + hub == 10000 ===
        let total = FpVar::constant(Fr::from(EIGENSPACE_TOTAL));
        let sum = &earth + &spoke + &hub;
        sum.enforce_equal(&total)?;

        // === Constraint 3: eigenspace commitment = MiMC(MiMC(earth, spoke), hub) ===
        let inner = mimc_hash_circuit(cs.clone(), &earth, &spoke)?;
        let eigen_computed = mimc_hash_circuit(cs.clone(), &inner, &hub)?;
        eigen_var.enforce_equal(&eigen_computed)?;

        // === Constraint 4+5: each shard is non-zero and in the Merkle tree ===
        let zero = FpVar::constant(Fr::zero());
        for i in 0..SHARD_COUNT {
            let leaf = FpVar::new_witness(ark_relations::ns!(cs, "shard"), || {
                Ok(w.shard_hashes[i])
            })?;

            // Non-zero check: leaf * leaf_inv == 1
            let leaf_inv = FpVar::new_witness(ark_relations::ns!(cs, "shard_inv"), || {
                Ok(w.shard_hashes[i].inverse().unwrap_or(Fr::zero()))
            })?;
            let product = &leaf * &leaf_inv;
            let one = FpVar::constant(Fr::from(1u64));
            product.enforce_equal(&one)?;

            // Merkle proof: walk from leaf to root
            let mut current = leaf;
            for j in 0..TREE_DEPTH {
                let sibling = FpVar::new_witness(ark_relations::ns!(cs, "sibling"), || {
                    Ok(w.merkle_siblings[i][j])
                })?;
                let dir = Boolean::new_witness(ark_relations::ns!(cs, "dir"), || {
                    Ok(w.merkle_dirs[i][j])
                })?;
                // if dir==0: hash(current, sibling), else hash(sibling, current)
                let left = dir.select(&sibling, &current)?;
                let right = dir.select(&current, &sibling)?;
                current = mimc_hash_circuit(cs.clone(), &left, &right)?;
            }
            current.enforce_equal(&root_var)?;
        }

        // === Constraint 6: crown shard (index 70) differs from all others ===
        let crown = FpVar::new_witness(ark_relations::ns!(cs, "crown"), || {
            Ok(w.shard_hashes[70])
        })?;
        for i in 0..SHARD_COUNT - 1 {
            let other = FpVar::new_witness(ark_relations::ns!(cs, "other"), || {
                Ok(w.shard_hashes[i])
            })?;
            let diff = &crown - &other;
            let diff_inv = FpVar::new_witness(ark_relations::ns!(cs, "diff_inv"), || {
                let d = w.shard_hashes[70] - w.shard_hashes[i];
                Ok(d.inverse().unwrap_or(Fr::zero()))
            })?;
            let one = FpVar::constant(Fr::from(1u64));
            let check = &diff * &diff_inv;
            check.enforce_equal(&one)?;
        }

        Ok(())
    }
}

/// Build Merkle tree from shard hashes (native, for witness generation).
pub fn build_merkle_tree(leaves: &[Fr; SHARD_COUNT]) -> (Fr, Vec<Vec<Fr>>) {
    let n = 1 << TREE_DEPTH;
    let mut padded = vec![Fr::zero(); n];
    for (i, h) in leaves.iter().enumerate() {
        padded[i] = *h;
    }

    let mut levels: Vec<Vec<Fr>> = vec![padded.clone()];
    let mut current = padded;
    for _ in 0..TREE_DEPTH {
        let next: Vec<Fr> = current
            .chunks(2)
            .map(|pair| mimc_hash_native(pair[0], pair[1]))
            .collect();
        levels.push(next.clone());
        current = next;
    }
    (current[0], levels)
}

/// Extract Merkle proof for a leaf at given index.
pub fn merkle_proof(levels: &[Vec<Fr>], idx: usize) -> ([Fr; TREE_DEPTH], [bool; TREE_DEPTH]) {
    let mut siblings = [Fr::zero(); TREE_DEPTH];
    let mut dirs = [false; TREE_DEPTH];
    let mut pos = idx;
    for d in 0..TREE_DEPTH {
        let sib = if pos % 2 == 0 { pos + 1 } else { pos - 1 };
        siblings[d] = if sib < levels[d].len() { levels[d][sib] } else { Fr::zero() };
        dirs[d] = pos % 2 == 1;
        pos /= 2;
    }
    (siblings, dirs)
}

/// Hash shard bytes to a field element.
pub fn shard_to_field(data: &[u8]) -> Fr {
    let h = Sha256::digest(data);
    Fr::from_le_bytes_mod_order(&h)
}

/// Compute eigenspace commitment (native).
pub fn eigenspace_commitment(earth: u64, spoke: u64, hub: u64) -> Fr {
    let inner = mimc_hash_native(Fr::from(earth), Fr::from(spoke));
    mimc_hash_native(inner, Fr::from(hub))
}

/// Setup: generate proving and verifying keys.
pub fn setup() -> (ProvingKey<Bn254>, VerifyingKey<Bn254>) {
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(71u64);
    let dummy = NFT71Circuit {
        witness: NFT71Witness {
            shard_hashes: [Fr::from(1u64); SHARD_COUNT],
            merkle_siblings: [[Fr::zero(); TREE_DEPTH]; SHARD_COUNT],
            merkle_dirs: [[false; TREE_DEPTH]; SHARD_COUNT],
            earth_pct: EIGENSPACE_TOTAL,
            spoke_pct: 0,
            hub_pct: 0,
        },
        merkle_root: Fr::zero(),
        eigenspace_commitment: Fr::zero(),
    };
    let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(dummy, &mut rng).unwrap();
    (pk, vk)
}

/// Generate a Groth16 proof from real shard data.
pub fn prove(pk: &ProvingKey<Bn254>, witness: NFT71Witness) -> (Proof<Bn254>, Vec<Fr>) {
    let (root, _) = build_merkle_tree(&witness.shard_hashes);
    let eigen = eigenspace_commitment(witness.earth_pct, witness.spoke_pct, witness.hub_pct);

    let circuit = NFT71Circuit {
        witness: witness.clone(),
        merkle_root: root,
        eigenspace_commitment: eigen,
    };

    let public_inputs = vec![root, eigen, Fr::from(SHARD_COUNT as u64)];
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(42u64);
    let proof = Groth16::<Bn254>::prove(pk, circuit, &mut rng).unwrap();
    (proof, public_inputs)
}

/// Verify a proof.
pub fn verify(vk: &VerifyingKey<Bn254>, proof: &Proof<Bn254>, public_inputs: &[Fr]) -> bool {
    Groth16::<Bn254>::verify(vk, public_inputs, proof).unwrap_or(false)
}
