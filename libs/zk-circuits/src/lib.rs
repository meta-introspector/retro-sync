//! ZK circuit library — Groth16 proofs over BN254.
pub mod royalty_split;
pub mod nft71;
pub use royalty_split::{generate_proof, verify, RoyaltySplitCircuit, RoyaltySplitWitness};
pub use nft71::{NFT71Circuit, NFT71Witness};
