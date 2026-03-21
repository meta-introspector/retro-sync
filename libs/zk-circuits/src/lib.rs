//! ZK circuit library — Groth16 proofs over BN254.
pub mod royalty_split;
pub use royalty_split::{generate_proof, verify, RoyaltySplitCircuit, RoyaltySplitWitness};
