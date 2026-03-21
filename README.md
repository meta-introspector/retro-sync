# Retrosync Media Group — Enterprise Platform

## Stack
Rust · WASM (Yew/Trunk) · BTFS · BTTC · Groth16/BN254 · DDEX ERN 4.1 · Ledger

## Frameworks
ISO 9001:2015 · GMP · Six Sigma DMAIC · ITIL v4 · Zero Trust (SPIFFE/OPA) · LangSec · AGPL-3.0

## Quick Start
```bash
cp .env .env.local   # fill in real values
cargo build
cargo run --bin backend
```

## DeFi Security
- Reentrancy guard on all distribute() calls
- ZK proof required (Groth16/BN254) for every distribution
- Value cap: 1M BTT max per non-timelocked transaction
- 48h timelock on large distributions
- Immutable contract (no proxy/upgrade path)

## Compliance
- DMCA §512 notice-and-takedown: POST /api/takedown
- GDPR/CCPA data rights: /api/privacy/*
- KYC/AML + OFAC screening: /api/kyc/*
- DSA/Article 17 moderation queue: /api/moderation/*
- CWR 2.2 full record set + 50+ global collection societies: royalty_reporting.rs

## Ceremony
Before production: `cargo run --bin ceremony` then call ZKVerifier.setVerifyingKey()
