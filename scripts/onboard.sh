#!/usr/bin/env bash
set -euo pipefail

# retro-sync onboard: create a new project from a search term
# Usage: ./scripts/onboard.sh "bach inventions" -n 15
#
# Steps:
#   1. Search MIDI classical dataset for matching files
#   2. Create project dir + project.toml
#   3. Copy MIDIs into project
#   4. Generate SVG tiles from MIDI metadata
#   5. Embed stego payload
#   6. Verify roundtrip
#   7. Report

TERM="${1:?Usage: onboard.sh \"search term\" [-n count]}"
shift
N=15
while getopts "n:" opt; do
  case $opt in n) N="$OPTARG";; esac
done

SLUG=$(echo "$TERM" | tr ' ' '-' | tr '[:upper:]' '[:lower:]' | tr -cd 'a-z0-9-')
PROJECT_DIR="projects/$SLUG"
DATA_DIR="datasets/midi-classical-data"
OUT_DIR="$PROJECT_DIR/output"

echo "=== ONBOARD: '$TERM' → $SLUG (n=$N) ==="
echo

# 1. Find matching MIDIs
echo "1. Searching $DATA_DIR for '$TERM'..."
MIDIS=$(ls "$DATA_DIR"/*.mid 2>/dev/null | grep -i "$(echo "$TERM" | sed 's/ /.*/')" | sort | head -n "$N")
COUNT=$(echo "$MIDIS" | grep -c . || true)

if [ "$COUNT" -eq 0 ]; then
  # Try looser match
  MIDIS=$(ls "$DATA_DIR"/*.mid 2>/dev/null | grep -i "$(echo "$TERM" | cut -d' ' -f1)" | sort | head -n "$N")
  COUNT=$(echo "$MIDIS" | grep -c . || true)
fi

if [ "$COUNT" -eq 0 ]; then
  echo "  ❌ No MIDIs found for '$TERM'"
  echo "  Available: $(ls "$DATA_DIR"/*.mid 2>/dev/null | head -3 | sed 's/.*\///' | tr '\n' ' ')"
  exit 1
fi
echo "  ✅ Found $COUNT MIDIs"

# 2. Create project
echo
echo "2. Creating project $PROJECT_DIR..."
mkdir -p "$PROJECT_DIR/midi" "$OUT_DIR/svg" "$OUT_DIR/stego"

cat > "$PROJECT_DIR/project.toml" <<EOF
[project]
name        = "$SLUG"
title       = "$(echo "$TERM" | sed 's/\b\(.\)/\u\1/g')"
license     = "PD"
description = "Auto-generated from MIDI classical dataset, search: $TERM"
created     = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

[sources.midi]
dir     = "midi"
count   = $COUNT

[tiles]
count   = 71
pattern = "{:02}.png"
width   = 512
height  = 512

[segments]
names = ["midi_bundle", "metadata", "erdfa"]
EOF
echo "  ✅ project.toml"

# 3. Copy MIDIs
echo
echo "3. Copying $COUNT MIDIs..."
i=0
TOTAL_SIZE=0
echo "$MIDIS" | while read -r f; do
  i=$((i+1))
  cp "$f" "$PROJECT_DIR/midi/$(printf '%02d' $i)_$(basename "$f")"
  sz=$(stat -c%s "$f" 2>/dev/null || stat -f%z "$f" 2>/dev/null || echo 0)
  TOTAL_SIZE=$((TOTAL_SIZE + sz))
done
echo "  ✅ $COUNT files → $PROJECT_DIR/midi/"

# 4. Render sheet music SVGs
echo
echo "4. Rendering sheet music SVGs..."
bash scripts/midi2svg.sh "$PROJECT_DIR/midi" "$PROJECT_DIR/output/svg" 71

# 5. Build stego (if cargo available)
echo
echo "5. Building stego tiles..."
if command -v cargo &>/dev/null || [ -n "${IN_NIX_SHELL:-}" ]; then
  # Copy project SVGs to fixtures path for the stego encoder
  cp "$PROJECT_DIR/output/svg/"*.svg fixtures/output/nft71_svg/
  
  # Bundle MIDIs as payload
  MIDI_BUNDLE="$OUT_DIR/midi_bundle.bin"
  cat "$PROJECT_DIR/midi/"*.mid > "$MIDI_BUNDLE" 2>/dev/null || true
  BUNDLE_SIZE=$(stat -c%s "$MIDI_BUNDLE" 2>/dev/null || stat -f%z "$MIDI_BUNDLE" 2>/dev/null || echo 0)
  echo "  MIDI bundle: ${BUNDLE_SIZE} bytes"
  
  # Build stego tiles
  cargo run --release --example nft71_stego_svg -p fixtures 2>&1 | tail -3
  
  # Copy stego PNGs back to project
  cp fixtures/output/nft71_stego_png/*.png "$OUT_DIR/stego/" 2>/dev/null
  echo "  ✅ 71 stego PNGs → $OUT_DIR/stego/"
else
  echo "  ⚠ cargo not found — run 'nix develop' first, then 'make demo'"
fi

# 6. Verify stego roundtrip
echo
echo "6. Verifying stego roundtrip..."
if [ -f "fixtures/output/nft71_stego_png/01.png" ]; then
  cargo run --release --example verify_stego -p fixtures 2>&1 | head -20
else
  echo "  ⚠ no stego tiles yet"
fi

# 7. OCR test — can we read the text back?
echo
echo "7. OCR test (reading text from stego PNG)..."
if command -v tesseract &>/dev/null; then
  OCR_OUT=$(tesseract "fixtures/output/nft71_stego_png/01.png" stdout 2>/dev/null || true)
  if [ -n "$OCR_OUT" ]; then
    echo "  ✅ OCR recovered text:"
    echo "$OCR_OUT" | head -5 | sed 's/^/    /'
  else
    echo "  ⚠ OCR returned empty — text may not survive stego at current contrast"
  fi
elif command -v nix-shell &>/dev/null; then
  OCR_OUT=$(nix-shell -p tesseract --run "tesseract fixtures/output/nft71_stego_png/01.png stdout 2>/dev/null" 2>/dev/null || true)
  if [ -n "$OCR_OUT" ]; then
    echo "  ✅ OCR recovered text:"
    echo "$OCR_OUT" | head -5 | sed 's/^/    /'
  else
    echo "  ⚠ OCR returned empty"
  fi
else
  echo "  ⚠ tesseract not available"
fi

# 8. PSNR test — how visible is the stego?
echo
echo "8. PSNR test (stego visibility)..."
if command -v magick &>/dev/null || command -v convert &>/dev/null; then
  convert "fixtures/output/nft71_svg/01.svg" -resize 512x512! /tmp/onboard_ref.png 2>/dev/null
  if [ -f /tmp/onboard_ref.png ]; then
    PSNR=$(magick compare -metric PSNR /tmp/onboard_ref.png "fixtures/output/nft71_stego_png/01.png" /dev/null 2>&1 || true)
    echo "  PSNR: $PSNR"
    PVAL=$(echo "$PSNR" | grep -oP '[\d.]+' | head -1)
    if [ -n "$PVAL" ] && [ "$(echo "$PVAL > 30" | bc -l 2>/dev/null || echo 0)" = "1" ]; then
      echo "  ✅ Stego invisible (>30dB)"
    elif [ -n "$PVAL" ] && [ "$(echo "$PVAL > 20" | bc -l 2>/dev/null || echo 0)" = "1" ]; then
      echo "  ⚠ Stego slightly visible (20-30dB)"
    else
      echo "  ❌ Stego visible (<20dB) — need brighter SVG palette"
    fi
  fi
elif command -v nix-shell &>/dev/null; then
  nix-shell -p imagemagick --run "
    convert fixtures/output/nft71_svg/01.svg -resize 512x512! /tmp/onboard_ref.png 2>/dev/null
    magick compare -metric PSNR /tmp/onboard_ref.png fixtures/output/nft71_stego_png/01.png /dev/null 2>&1
  " 2>/dev/null | grep -v "^$" | head -1 | xargs -I{} echo "  PSNR: {}"
else
  echo "  ⚠ imagemagick not available"
fi

# 7. Report
echo
echo "=== ONBOARD COMPLETE ==="
echo "  Project:  $PROJECT_DIR"
echo "  MIDIs:    $COUNT files"
echo "  SVGs:     71 tiles"
echo "  Config:   $PROJECT_DIR/project.toml"
echo
echo "Next steps:"
echo "  nix develop"
echo "  make stego PROJECT=$SLUG"
echo "  make verify PROJECT=$SLUG"
echo "  make deploy PROJECT=$SLUG"
