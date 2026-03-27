# retro-sync Onboarding SOP
# ISO 9001:2015 — Clause 8.5 Production and service provision

## Purpose
Create a new NFT tile series from public domain MIDI files.

## Inputs
- Search term (e.g. "bach invention")
- MIDI classical dataset (datasets/midi-classical-data/)
- retro-sync.toml platform config

## Process Steps

| Step | Tool | Input | Output | Verification |
|------|------|-------|--------|-------------|
| 1. Search | scripts/onboard.sh | search term | project.toml | file exists |
| 2. Copy MIDI | scripts/onboard.sh | MIDI dataset | project/midi/*.mid | count matches -n |
| 3. MIDI→SVG | scripts/midi2svg.sh | *.mid | *.svg (sheet music) | 71 SVGs exist |
| 4. Stego embed | make stego PROJECT=x | *.svg + payload | *.png (stego tiles) | 71 PNGs exist |
| 5. Verify | make verify PROJECT=x | *.png | NFT7 segments | all magic bytes match |
| 6. QA | make qa PROJECT=x | *.png + *.svg | PSNR + OCR report | PSNR>30, OCR readable |
| 7. Deploy | make deploy PROJECT=x | *.png | HuggingFace | HTTP 200 |

## Acceptance Criteria
- [ ] 71 stego PNG tiles generated
- [ ] NFT7 payload recoverable from all 71 tiles
- [ ] PSNR > 30 dB (stego invisible)
- [ ] OCR recovers title text from stego PNG
- [ ] All MIDIs are public domain

## Records
- project.toml (config)
- output/stego/*.png (artifacts)
- output/qa_report.txt (verification)
