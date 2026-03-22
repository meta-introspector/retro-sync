# Hurrian Hymn h.6 — Multi-Layered NFT Pipeline

## Overview

The world's oldest surviving notated music (~1400 BC, Ugarit) encoded as a
71-shard NFT collection with ZK proofs, steganographic embedding, and
multi-layer witnessing. This is the public "first customer" test case for
the retro-sync platform.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    SOURCE LAYER                          │
│  Tablet RS 15.30 + 15.49 + 17.387 (cuneiform)          │
│  → Dietrich & Loretz 1975 transcription                 │
│  → 14 Babylonian interval terms → 15 SSP primes         │
│  → Cl(15,0,0) boustrophedon → eigenspace decomposition  │
└──────────────────────┬──────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────┐
│                   NOTATION LAYER                         │
│  fixtures/lilypond/h6_west.ly (West 1994 reconstruction)│
│  Future: h6_kilmer.ly, h6_dumbrill.ly, h6_vitale.ly     │
└──────────────────────┬──────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────┐
│                   RENDERING LAYER                        │
│  LilyPond 2.24.4 → PDF score + MIDI (606 bytes)        │
│  FluidSynth 2.5.2 + FluidR3_GM2 → WAV (8.4 MB, 44.1k) │
│  Each step witnessed: input hash → tool hash → output    │
│  Chain commitment: 5cee0046...                           │
└──────────────────────┬──────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────┐
│                    SHARD LAYER                            │
│  71 DA51 CBOR shards (11.2 MB total)                    │
│  20 generators (primes ≤71) + 51 derived (composites)   │
│  Layout:                                                 │
│    ★ Primes: SSP intervals + CFT structure               │
│    · 4-10:   source text, .ly, MIDI, PDF, WAV           │
│    · 12-18:  witness chain (5 witnesses)                 │
│    · 20-25:  eigenspace (earth/spoke/hub/grade/fractran) │
│    · 26-35:  metadata (tablet, scribe, tuning, etc.)    │
│    · 36-42:  reconstructions (West, Kilmer, DG, etc.)   │
│    · 44-57:  references (Wikipedia, scholarly, LilyPond) │
│    · 58-65:  YouTube (private audio comparison)          │
│    · 66-70:  pipeline (SOP, erdfa, boustrophedon, Cl15) │
│    ★ 71:     crown/colophon (full provenance)            │
└──────────────────────┬──────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────┐
│                     ZK LAYER                             │
│  Groth16/BN254 proof (pure Rust, ark-groth16)           │
│  MiMC Merkle tree (depth 7, 128 leaves)                 │
│  Public inputs: merkle_root, eigenspace_commit, count=71│
│  Constraints: 122,667 non-linear + 136,591 linear       │
│  Prove: 2.9s | Verify: 0.002s                           │
│  Also: circom spec (circuits/nft71.circom, circom-chan)  │
└──────────────────────┬──────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────┐
│                    IMAGE LAYER                            │
│  71 PNG frames (LilyPond score excerpts + metadata)     │
│  Each frame titled with shard index, interval name, CID │
│  HME steganography: DA51 CBOR embedded in LSBs          │
│  Round-trip verified: extract → matches original shard   │
└──────────────────────┬──────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────┐
│                  REFERENCE LAYER                         │
│  25+ URLs for zkTLS capture (Wikipedia, scholarly, docs) │
│  8 YouTube URLs for private audio comparison             │
│  Moltis browser + witness_download for capture           │
│  Each capture → erdfa shard → dataset                    │
└──────────────────────┬──────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────┐
│                 DISTRIBUTION LAYER                        │
│  HuggingFace dataset: introspector/retro-sync            │
│  BTFS/IPFS: DA51 CBOR shards                            │
│  NFT mint: 71 tokens per collection                      │
│  Cross-platform: stego survives re-upload/compression    │
└─────────────────────────────────────────────────────────┘
```

## Commands

```bash
# Render pipeline (LilyPond → MIDI → WAV with witnesses)
nix shell nixpkgs#lilypond nixpkgs#fluidsynth nixpkgs#soundfont-fluid \
  --command bash fixtures/scripts/render.sh

# Generate 71 shards with real data
cargo run -p fixtures --example nft71

# Generate ZK proof (pure Rust)
cargo run -p fixtures --example nft71_prove --release

# Generate NFT frames
nix shell nixpkgs#lilypond --command bash fixtures/scripts/nft71_frames.sh

# Embed shards into frames via steganography
cargo run -p fixtures --example nft71_stego

# Import to erdfa dataset
cargo run -p erdfa-publish --bin erdfa-cli -- import \
  --src fixtures/data/ --dir datasets/public/shards/ --max-depth 2
```

## File Map

```
fixtures/
├── data/
│   ├── hurrian_h6.txt          # Source notation + interval mapping
│   ├── references.txt          # 25+ URLs for zkTLS capture
│   └── youtube_sources.txt     # YouTube URLs for private comparison
├── lilypond/
│   └── h6_west.ly              # West 1994 reconstruction
├── scripts/
│   ├── render.sh               # .ly→MIDI→WAV witness pipeline
│   └── nft71_frames.sh         # 71 PNG frame generator
├── examples/
│   ├── smoke.rs                # h.6 eigenspace smoke test
│   ├── nft71.rs                # 71-shard NFT encoder (real data)
│   ├── nft71_witness.rs        # Merkle tree witness for circom
│   ├── nft71_prove.rs          # Groth16 prover (pure Rust)
│   └── nft71_stego.rs          # HME steganographic embedder
├── src/
│   ├── hurrian_h6.rs           # SSP mapping, boustrophedon, eigenspace
│   ├── witness.rs              # zkperf witness chain
│   └── lib.rs
└── output/                     # Generated artifacts (not committed)
    ├── h6_west.{midi,pdf,wav}
    ├── witnesses/
    ├── nft71/                  # 71 CBOR shards + manifest
    ├── nft71_frames/           # 71 PNG frames
    └── nft71_stego/            # 71 stego'd images + manifest

circuits/
├── nft71.circom                # Circuit spec (circom-chan 2.2.3)
├── circomlib -> ...            # Poseidon, SHA-256, comparators
└── build/                      # R1CS, WASM, C++, proof JSON

libs/zk-circuits/src/
├── nft71.rs                    # Native Rust circuit (MiMC, ark-groth16)
├── royalty_split.rs            # Existing royalty circuit
└── lib.rs
```

## Next Steps

See PLAN.md for the roadmap.
