//! ZK Trusted Setup Ceremony — Retrosync Media Group
//!
//! Runs a real Groth16 powers-of-tau ceremony for `RoyaltySplitCircuit`
//! using arkworks. Outputs a JSON file containing the verifying key
//! in a format ready for ZKVerifier.sol `setVerifyingKey()`.
//!
//! Usage (testnet — single party):
//!   cargo run --bin ceremony -- --output vk.json
//!
//! Usage (mainnet — multi-party MPC):
//!   See docs/ceremony.md for SnarkJS MPC phase2 instructions.
//!   The MPC output can be converted to the same JSON format.
//!
//! Security note: single-party setup leaks the "toxic waste" (τ).
//! For mainnet: use a multi-party ceremony so no single participant
//! knows the full trapdoor. The JSON output format is identical.

use ark_bn254::{Bn254, Fr, G1Affine, G2Affine};
use ark_ec::AffineRepr;
use ark_ff::PrimeField;
use ark_groth16::Groth16;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use ark_std::{rand::SeedableRng, Zero};
use serde_json::{json, Value};
use std::path::PathBuf;

// ── Duplicate the circuit here so ceremony is self-contained ────────────
// In production this would be `use zk_circuits::RoyaltySplitCircuit;`
// but ceremony is a standalone binary to avoid circular workspace deps.

const MAX_ARTISTS: usize = 16;
const BASIS_POINTS: u64 = 10_000;

#[derive(Clone, Default)]
struct ArtistWitness {
    bps: u16,
}

#[derive(Clone)]
struct CeremonyCircuit {
    #[allow(dead_code)] // shape is carried by witnesses.len(); kept for documentation
    n_artists: usize,
    band: u8,
    witnesses: Vec<ArtistWitness>,
}

impl CeremonyCircuit {
    fn blank(n_artists: usize) -> Self {
        Self {
            n_artists,
            band: 0,
            witnesses: vec![ArtistWitness::default(); n_artists],
        }
    }
}

impl ConstraintSynthesizer<Fr> for CeremonyCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        use ark_r1cs_std::fields::fp::FpVar;

        // Public input 1: band ∈ {0,1,2}
        let band_var = FpVar::<Fr>::new_input(ark_relations::ns!(cs, "band"), || {
            Ok(Fr::from(self.band as u64))
        })?;
        let one = FpVar::constant(Fr::from(1u64));
        let two = FpVar::constant(Fr::from(2u64));
        let zero = FpVar::constant(Fr::zero());
        let b1 = &band_var - &one;
        let b2 = &band_var - &two;
        let prod = &band_var * &b1;
        (&prod * &b2).enforce_equal(&zero)?;

        // Public input 2: basis points == 10,000
        let bp_var = FpVar::<Fr>::new_input(ark_relations::ns!(cs, "basis_points"), || {
            Ok(Fr::from(BASIS_POINTS))
        })?;

        // Private: sum of bps == bp_var
        // ns! requires a string literal; per-slot identity is not needed for
        // correctness — the constraint index provides uniqueness.
        let mut sum = FpVar::constant(Fr::zero());
        for w in self.witnesses.iter() {
            let bps_var = FpVar::<Fr>::new_witness(ark_relations::ns!(cs, "bps"), || {
                Ok(Fr::from(w.bps as u64))
            })?;
            sum = sum + bps_var;
        }
        sum.enforce_equal(&bp_var)?;
        Ok(())
    }
}

// ── G1/G2 serialisation helpers ─────────────────────────────────────────

fn g1_to_hex(p: &G1Affine) -> [String; 2] {
    let (x, y) = if p.is_zero() {
        (vec![0u8; 32], vec![0u8; 32])
    } else {
        let mut xb = Vec::new();
        let mut yb = Vec::new();
        p.x()
            .unwrap()
            .into_bigint()
            .serialize_uncompressed(&mut xb)
            .ok();
        p.y()
            .unwrap()
            .into_bigint()
            .serialize_uncompressed(&mut yb)
            .ok();
        (xb, yb)
    };
    [
        format!("0x{}", hex::encode(&x)),
        format!("0x{}", hex::encode(&y)),
    ]
}

fn g2_to_hex(p: &G2Affine) -> [[String; 2]; 2] {
    let zero_coord = || ["0x00".into(), "0x00".into()];
    if p.is_zero() {
        return [zero_coord(), zero_coord()];
    }
    let mut xb = Vec::new();
    let mut yb = Vec::new();
    p.x()
        .unwrap()
        .c0
        .into_bigint()
        .serialize_uncompressed(&mut xb)
        .ok();
    p.y()
        .unwrap()
        .c0
        .into_bigint()
        .serialize_uncompressed(&mut yb)
        .ok();
    let mut xb1 = Vec::new();
    let mut yb1 = Vec::new();
    p.x()
        .unwrap()
        .c1
        .into_bigint()
        .serialize_uncompressed(&mut xb1)
        .ok();
    p.y()
        .unwrap()
        .c1
        .into_bigint()
        .serialize_uncompressed(&mut yb1)
        .ok();
    [
        [
            format!("0x{}", hex::encode(&xb)),
            format!("0x{}", hex::encode(&xb1)),
        ],
        [
            format!("0x{}", hex::encode(&yb)),
            format!("0x{}", hex::encode(&yb1)),
        ],
    ]
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let output = args
        .windows(2)
        .find(|w| w[0] == "--output")
        .map(|w| PathBuf::from(&w[1]))
        .unwrap_or_else(|| PathBuf::from("vk.json"));
    let n_artists: usize = args
        .windows(2)
        .find(|w| w[0] == "--artists")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(MAX_ARTISTS);

    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║  Retrosync ZK Trusted Setup Ceremony                  ║");
    println!("║  Circuit:  RoyaltySplitCircuit (band + splits)        ║");
    println!("║  Curve:    BN254  ·  Protocol: Groth16                ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    println!();
    println!("Artists in circuit: {}", n_artists);
    println!("WARNING: single-party setup. For mainnet use MPC ceremony.");
    println!("See docs/ceremony.md for multi-party instructions.");
    println!();
    println!("[1/3] Generating circuit constraints...");

    // Use a fixed RNG seed so testnet setups are reproducible.
    // For mainnet: use OsRng (remove seed).
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(0xDEAD_BEEF_CAFE_BABE);

    let circuit = CeremonyCircuit::blank(n_artists);
    println!("[2/3] Running Groth16 trusted setup (groth16::generate_random_parameters)...");

    let (_pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)
        .map_err(|e| anyhow::anyhow!("Trusted setup failed: {:?}", e))?;

    println!("[3/3] Serialising verifying key to JSON...");

    // Prepare IC points (1 + n_public_inputs = 1 + 2 = 3)
    let ic_json: Vec<Value> = vk
        .gamma_abc_g1
        .iter()
        .map(|p| {
            let coords = g1_to_hex(p);
            json!({ "x": coords[0], "y": coords[1] })
        })
        .collect();

    let alpha = g1_to_hex(&vk.alpha_g1);
    let beta = g2_to_hex(&vk.beta_g2);
    let gamma = g2_to_hex(&vk.gamma_g2);
    let delta = g2_to_hex(&vk.delta_g2);

    let vk_json = json!({
        "meta": {
            "circuit":        "RoyaltySplitCircuit",
            "curve":          "BN254",
            "protocol":       "groth16",
            "n_artists":      n_artists,
            "public_inputs":  ["band (u8, 0-2)", "basis_points_sum (u64, must = 10000)"],
            "ceremony_type":  "single_party_testnet",
            "generated_at":   chrono::Utc::now().to_rfc3339(),
            "warning":        "Single-party setup: toxic waste not destroyed. Use MPC for mainnet."
        },
        "alpha": { "x": alpha[0], "y": alpha[1] },
        "beta":  { "x": beta[0],  "y": beta[1]  },
        "gamma": { "x": gamma[0], "y": gamma[1]  },
        "delta": { "x": delta[0], "y": delta[1]  },
        "ic":    ic_json,
        "deployment": {
            "step1": "Deploy ZKVerifier.sol (already done by Foundry script)",
            "step2": "Call ZKVerifier.setVerifyingKey(band, alpha, beta, gamma, delta, ic)",
            "step3": "Set BTTC_DEV_MODE=0 in .env",
            "step4": "Run integration test: cargo test --package backend test_full_pipeline"
        }
    });

    std::fs::write(&output, serde_json::to_string_pretty(&vk_json)?)?;

    println!();
    println!("✅  Verifying key written to: {}", output.display());
    println!();
    println!("NEXT STEPS:");
    println!("  1. Deploy contracts:  forge script script/Deploy.s.sol:DeployScript \\");
    println!("                          --rpc-url $BTTC_RPC_URL --ledger --broadcast");
    println!("  2. Set VK on-chain:   cast send $ZK_VERIFIER_ADDR 'setVerifyingKey(...)' $(cat vk.json | jq ...)");
    println!("  3. Update .env:       ROYALTY_CONTRACT_ADDR=<deployed address>");
    println!("  4. Disable dev mode:  BTTC_DEV_MODE=0  LEDGER_DEV_MODE=0");
    println!("  5. Connect Ledger and run: cargo run --bin backend");
    println!();
    println!("For mainnet MPC ceremony: see docs/ceremony.md");

    Ok(())
}
