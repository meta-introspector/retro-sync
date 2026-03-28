# retro-sync — Getting Started

## What is this?

A music publishing platform that encodes public domain music into steganographic NFT tiles. 
Browse the catalog, play MIDIs decoded from PNG images, register works with collection societies.

## Quick Start (5 minutes)

### 1. Browse the catalog
Visit: https://solana.solfunmeme.com/retro-sync/menu.html

### 2. Play music from tiles
Click any collection → tiles load → click "🎵 Reconstruct" → play individual tracks

### 3. Add a new collection
```bash
git clone https://github.com/meta-introspector/retro-sync
cd retro-sync
nix develop

# Search for any composer in the MIDI dataset
bash scripts/onboard.sh "chopin nocturne" -n 15

# Build stego tiles (needs pypng)
nix-shell -p python3Packages.pypng imagemagick --run \
  "bash scripts/stego-build.sh projects/chopin-nocturne"

# Generate catalog entry
python3 scripts/catalog-gen.py
python3 scripts/artist-ids.py

# Export CWR for collection societies
python3 scripts/export-cwr.py
```

### 4. Deploy
```bash
# To local web server
bash scripts/deploy-space.sh projects/chopin-nocturne

# To HuggingFace
bash scripts/publish-catalog.sh
```

## Architecture

```
MIDI files
  → onboard.sh (search + project setup)
  → midi2svg.sh (lilypond sheet music SVGs)
  → stego-build.sh (embed MIDIs in PNG tiles)
  → catalog-gen.py (WorkRegistration JSON)
  → artist-ids.py (Wikidata QID → RS-ID + ISNI)
  → export-cwr.py (CWR 2.2 for societies)
  → publish-catalog.sh (HuggingFace)
```

## URLs

| What | URL |
|------|-----|
| Catalog menu | /retro-sync/menu.html |
| Bach Inventions | /retro-sync/bach-invention/ |
| Bartók | /retro-sync/bartok/ |
| Catalog JSON | /retro-sync/catalog/works.json |
| Artists JSON | /retro-sync/catalog/artists.json |
| CWR export | /retro-sync/catalog/retro-sync.cwr |
| HF Space | https://huggingface.co/spaces/introspector/retro-sync |

## API (when backend is running)

| Endpoint | What |
|----------|------|
| POST /api/register | Register a work |
| POST /api/upload | Upload a track |
| GET /api/societies | List collection societies |
| POST /api/gateway/ern/push | DDEX ERN push |
| GET /api/manifest/:id | NFT manifest lookup |
| POST /api/manifest/mint | Mint NFT |

## File Formats

| File | Format | What |
|------|--------|------|
| project.toml | TOML | Project config (title, sources, segments) |
| works.json | JSON | WorkRegistration catalog (API-compatible) |
| artists.json | JSON | Artist IDs (RS-ID, QID, ISNI, VIAF) |
| retro-sync.cwr | CWR 2.2 | Society submission file |
| *.png | PNG+stego | 512×512 tiles with NFT7 payload |
| *.svg | SVG | Sheet music (lilypond rendered) |
