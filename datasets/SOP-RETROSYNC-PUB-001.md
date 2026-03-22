# Standard Operating Procedure: retro-sync Public Dataset Publishing

**Document ID:** SOP-RETROSYNC-PUB-001
**Version:** 1.0.0
**Effective Date:** 2026-03-22
**Review Date:** 2026-06-22
**Owner:** Monster CFT Research Group

## 1. Purpose

Define the procedure for publishing witnessed music-rights data to the
`introspector/retro-sync` HuggingFace dataset using erdfa-publish DA51 CBOR
shards with full CFT decomposition and zkperf witness chains.

## 2. Scope

Applies to all public data published from the retro-sync platform, including:
- Historical music notation (Hurrian, cuneiform, early Western)
- Eigenspace analysis results (Cl(15,0,0) decompositions)
- Witness chains (zkperf 5-layer provenance)
- CFT shard manifests

## 3. References

- SOP-FRACTRAN-QC-001 — Dataset quality control
- SOP-USA250-TRENTON-001 — NFT generation & witness procedure
- ISO 9001:2015 Clause 8.5 — Production and Service Provision
- ISO 9001:2015 Clause 8.6 — Release of Products and Services
- ITIL v4 — Service Value System
- Six Sigma DMAIC Methodology

## 4. Tools

| Tool | Location | Purpose |
|------|----------|---------|
| erdfa-cli | `nix/erdfa-publish` | DA51 CBOR shard creation + CFT decomposition |
| fixtures | `fixtures/` | Test data generators (h.6, witness chains) |
| cargo run --example | `fixtures/examples/` | Export pipeline |
| git (HF) | `datasets/public/` | HuggingFace dataset submodule |

## 5. Procedure

### Step 1: Generate Source Data

```bash
cd /mnt/data1/time-2026/03-march/20/retro-sync
nix develop --command cargo run -p fixtures --example export
```

Output: JSON to stdout with eigenspace, shard hex, witness chain.

### Step 2: Produce DA51 CBOR Shards via erdfa-publish

```bash
nix develop --command cargo run -p erdfa-publish --bin erdfa-cli -- \
  import --src fixtures/data/ --dir datasets/public/shards/ --max-depth 2
```

Each source file → CFT tower (Post → Paragraph → Line) → DA51 CBOR shards.

### Step 3: Generate Manifest

```bash
nix develop --command cargo run -p erdfa-publish --bin erdfa-cli -- \
  list datasets/public/shards/ > datasets/public/manifest.json
```

### Step 4: Quality Verification

For each shard, verify:
- [ ] DA51 magic bytes (0xDA51) present
- [ ] CBOR decodes without error
- [ ] SHA-256 matches manifest CID
- [ ] Witness chain commitment is valid
- [ ] Eigenspace percentages sum to 100%

### Step 5: Publish to HuggingFace

```bash
cd datasets/public
git add shards/ manifest.json README.md
git commit -m "release: $(date -I) — N shards, M bytes"
git push origin main
```

### Step 6: Update Parent Submodule

```bash
cd /mnt/data1/time-2026/03-march/20/retro-sync
git add datasets/public
git commit -m "chore: bump datasets/public submodule"
git push meta-introspector main
```

## 6. Output Artefacts

| Artefact | Location | Format |
|----------|----------|--------|
| CBOR shards | `datasets/public/shards/*.cbor` | DA51-tagged CBOR |
| JSON witness | `datasets/public/witnesses/*.json` | zkperf witness-chain |
| Manifest | `datasets/public/manifest.json` | ShardSet JSON |
| README | `datasets/public/README.md` | HF dataset card |

## 7. Quality Records

All publish runs logged via git commit messages with keywords
`[retro-sync, erdfa, da51, witness, iso9001]`.

## 8. Change History

| Version | Date | Change | Author |
|---------|------|--------|--------|
| 1.0.0 | 2026-03-22 | Initial release — Hurrian h.6 first customer | Dataset Engineering |
