# Retrosync — Music Publishing Platform

Queryable NFT dataset for music publishers. Ingest from public sources, encode as DA51 CBOR shards in steganographic tiles, query like SPARQL. Demo on public domain music; production for rights-managed catalogs.

## Key Features

- **Invisible stego NFTs** — 71-tile collections with music encoded in PNG bit-planes, decoded in-browser via WASM
- **Queryable metadata** — DA51 CBOR shards replace databases; query like SPARQL, no server needed
- **On-chain ZK royalties** — Groth16/BN254 verified splits + soulbound attribution
- **CWR 2.2 + DDEX ERN** — Automated collection society submissions for 50+ global societies
- **Local-first** — Full dev mode with API stubs, no external service dependencies
- **Onboard in minutes** — `bash scripts/onboard.sh "chopin nocturne" -n 12`

## Live

| URL | What |
|-----|------|
| [Catalog](https://solana.solfunmeme.com/retro-sync/) | Browse collections |
| [HF Space](https://huggingface.co/spaces/introspector/retro-sync) | Viewer + MIDI player |
| [HF Dataset](https://huggingface.co/datasets/introspector/retro-sync) | Full dataset download |
| [Archive.org](https://archive.org/details/retro-sync-mints) | 35 minted works |

## Catalog

35+ public-domain works (Bach Two-Part Inventions + Bartók selections). New projects can be onboarded in minutes via `onboard.sh`.

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

## Quick Start
```bash
nix develop                                    # all deps: rust, lilypond, aubio, foundry
cargo build --workspace                        # build everything
bash scripts/onboard.sh "bach invention" -n 15 # onboard a composer
bash scripts/stego-build.sh projects/bach-invention  # generate stego tiles
python3 scripts/mint-catalog.py                # mint all works locally
bash scripts/publish-mints.sh --target all     # publish to HF + Archive.org + Pastebin
```

## Compliance
- DMCA §512 notice-and-takedown: POST /api/takedown
- GDPR/CCPA data rights: /api/privacy/*
- CWR 2.2 full record set + 50+ global collection societies
- ZK proof required (Groth16/BN254) for every distribution

## Contributing

Pull requests welcome for:
- New public-domain composer projects
- Collection society integrations
- DDEX/CWR format improvements
- Accessibility and i18n

See `docs/CUSTOMER-ONBOARDING.md` for publisher integration guide.

## License
AGPL-3.0
