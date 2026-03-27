#!/usr/bin/env bash
set -euo pipefail
# midi2svg.sh — Convert MIDI files to sheet music SVG tiles via lilypond
# Usage: midi2svg.sh <midi_dir> <svg_dir> [max_tiles]

MIDI_DIR="${1:?Usage: midi2svg.sh <midi_dir> <svg_dir> [max_tiles]}"
SVG_DIR="${2:?Usage: midi2svg.sh <midi_dir> <svg_dir> [max_tiles]}"
MAX="${3:-71}"

mkdir -p "$SVG_DIR" /tmp/retro-ly

i=0
for mid in "$MIDI_DIR"/*.mid; do
  [ -f "$mid" ] || continue
  i=$((i+1))
  [ $i -gt "$MAX" ] && break
  
  base=$(basename "$mid" .mid)
  ly="/tmp/retro-ly/${base}.ly"
  out="$SVG_DIR/$(printf '%02d' $i)"
  
  # MIDI → lilypond
  midi2ly "$mid" -o "$ly" 2>/dev/null || continue
  
  # Render SVG
  lilypond --svg -dbackend=svg -o "$out" "$ly" 2>/dev/null || continue
  
  # Lilypond outputs 01-1.svg for page 1 — rename to 01.svg
  if [ -f "${out}-1.svg" ]; then
    mv "${out}-1.svg" "${out}.svg"
    rm -f "${out}"-[2-9].svg "${out}.midi" 2>/dev/null
  fi
  echo "✅ $i: $base"
done

# Fill remaining slots by cycling
HAVE=$(ls "$SVG_DIR"/*.svg 2>/dev/null | wc -l)
if [ "$HAVE" -gt 0 ] && [ "$HAVE" -lt "$MAX" ]; then
  SVGS=("$SVG_DIR"/*.svg)
  for j in $(seq $((HAVE+1)) "$MAX"); do
    src="${SVGS[$(( (j-1) % HAVE ))]}"
    cp "$src" "$SVG_DIR/$(printf '%02d' $j).svg"
  done
fi

echo "→ $(ls "$SVG_DIR"/*.svg 2>/dev/null | wc -l) SVGs in $SVG_DIR"
