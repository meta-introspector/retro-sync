# Retrosync — Music Publishing Platform

Queryable NFT dataset for music publishers. Ingest from public sources, encode as DA51 CBOR shards in steganographic tiles, query like SPARQL. Demo on public domain music; production for rights-managed catalogs.

## Stack
Rust · WASM · BTFS · BTTC · Groth16/BN254 · DDEX ERN 4.1 · Ledger

## Data Sources

| Source | What | API |
|--------|------|-----|
| YouTube | Witnessed performances, audio | yt-dlp → aubio |
| Wikidata | Structured metadata (composers, works, ISRC, ISWC) | SPARQL endpoint |
| Wikipedia | Prose context, historical notes | MediaWiki API |
| Archive.org | Public domain recordings, scores, scans | Internet Archive API |
| IMSLP | PD sheet music (Petrucci Library) | Scrape/API |

## Pipeline

```
Sources (YouTube, Archive.org, IMSLP)
  → Ingest (yt-dlp, ia-download, scrape)
  → Extract (aubio notes, OCR scores)
  → Notation (LilyPond .ly, quantized)
  → Render (MIDI + WAV via lilypond)
  → Metadata (Wikidata SPARQL, Wikipedia context)
  → Package (DA51 CBOR shards — queryable like RDF)
  → Embed (NFT7 container → 6-layer bit-plane stego → PNG tiles)
  → Publish (HuggingFace, BTFS, IPFS, streaming platforms)
```

## Query Layer

DA51 shards replace Wikidata for music metadata:
- Each shard is a semantic component (KeyValue, Table, Tree, Link, etc.)
- Shards link via `input_cid` → full provenance DAG
- WASM decoder runs in browser — no server needed
- Orbifold addressing: CRT mod (71, 59, 47) for content-addressed routing

## Projects

Each musical work is a project with its own `project.toml`:

```
projects/
  hurrian-h6/          # ~1400 BCE — oldest known melody
  gregorian-chant/     # ~900 CE — Musica enchiriadis
  machaut-messe/       # 1365 — first complete polyphonic mass
  bach-wtc/            # 1722 — Well-Tempered Clavier
  beethoven-sym/       # 1800s — symphonies (PD)
  ...
```

## Quick Start
```bash
nix develop
cargo build --workspace
bun run build
cargo run --example verify_stego   # verify 71 stego tiles
```

## Compliance
- DMCA §512 notice-and-takedown: POST /api/takedown
- GDPR/CCPA data rights: /api/privacy/*
- CWR 2.2 full record set + 50+ global collection societies
- ZK proof required (Groth16/BN254) for every distribution

## License
AGPL-3.0
