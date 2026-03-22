# retro-sync — Next Steps Plan

## Phase 1: Complete h.6 Public Example (current)

### 1.1 Visual NFT Frames ← NEXT
- [ ] Generate all 71 LilyPond frames with cuneiform text overlay (TeX/LilyPond markup)
- [ ] Add Hurrian text transliteration to each frame header
- [ ] Render at 300dpi for NFT-quality images
- [ ] Proper PNG stego with decoded pixel data (not raw bytes)
- [ ] Add QR code overlay with shard CID on each frame

### 1.2 Reference Capture
- [ ] Wire moltis browser tool for zkTLS witness capture of all 25+ reference URLs
- [ ] Capture Wikipedia Hurrian songs page + all outbound references
- [ ] Capture LilyPond documentation pages (items 9-14 from pastebin)
- [ ] Each capture → erdfa shard → update dataset

### 1.3 YouTube Audio Comparison
- [ ] Use moltis browser Evaluate action with Web Audio API to capture audio streams
- [ ] Private spectral fingerprinting (never redistribute audio)
- [ ] Compare YouTube performances against our MIDI rendering
- [ ] Witness the spectral delta between interpretations
- [ ] Store comparison metrics in shards 58-65

### 1.4 Multiple Reconstructions
- [ ] h6_kilmer.ly — Kilmer 1974 (ascending scale)
- [ ] h6_dumbrill.ly — Dumbrill (Peter Pringle version)
- [ ] h6_vitale.ly — Vitale interpretation
- [ ] h6_duchesne_guillemin.ly — Duchesne-Guillemin 1984
- [ ] Cross-mode j-invariant analysis (Hub recovery from 5+ rival readings)

### 1.5 Dataset Publication
- [ ] Generate manifest: erdfa-cli list datasets/public/shards/
- [ ] Create HF dataset card (YAML frontmatter, features schema)
- [ ] Push shards to HuggingFace: datasets/public/ → git push
- [ ] Update parent submodule ref

## Phase 2: Platform Infrastructure

### 2.1 API Server
- [ ] Wire shard_db module in apps/api-server
- [ ] REST endpoints: GET /shards/:id, GET /shards/manifest, POST /shards/verify
- [ ] Serve stego'd NFT images via /nft/:collection/:index
- [ ] ZK proof verification endpoint

### 2.2 Smart Contracts
- [ ] NFT71 collection contract (ERC-721 or BTTC equivalent)
- [ ] Merkle root stored on-chain
- [ ] Groth16 verifier contract (from ark-groth16 → Solidity)
- [ ] Stego-lifting integration (Solana program from cicadia71)

### 2.3 Alife Simulation
- [ ] Run retrosync-sim with h.6 as seed organism
- [ ] 100-tick evolution, publish results (never internals)
- [ ] Honesty/deception dynamics from the 24-world study
- [ ] Phase-offset speciation analysis

## Phase 3: Customer Pipeline

### 3.1 Private Audio Processing
- [ ] Same pipeline as h.6 but with customer's copyrighted material
- [ ] NFT-gated shard distribution (only token holders can decrypt)
- [ ] Private witness chain (customer controls access)
- [ ] DMCA/GDPR compliance via existing api-server endpoints

### 3.2 Multi-Format Support
- [ ] MusicXML import → LilyPond conversion
- [ ] MIDI import → shard decomposition
- [ ] Audio fingerprinting (Chromaprint/AcoustID)
- [ ] Video frame extraction for visual NFTs

### 3.3 Cross-Platform Distribution
- [ ] BTFS pinning of all shards
- [ ] IPFS gateway for public access
- [ ] HuggingFace dataset auto-update
- [ ] NFT marketplace integration (OpenSea, Rarible metadata)

## Tools & Dependencies

| Tool | Version | Purpose |
|------|---------|---------|
| circom-chan | 2.2.3 | Circuit compiler (Rust) |
| ark-groth16 | 0.4 | ZK proving (pure Rust) |
| LilyPond | 2.24.4 | Music engraving |
| FluidSynth | 2.5.2 | MIDI → WAV rendering |
| erdfa-publish | local | Shard creation + CFT decomposition |
| moltis | local | Browser automation + witness capture |
| HME stego | erdfa-namespace | Hostile Media Embedding |
| frida-poc | local | FRI-based DA sampling |
