#!/bin/bash
# SOP-TILE-002: Compose complete tile SVGs with embedded photo backgrounds
# One SVG per tile → rasterize → stego. No compositing needed.
#
# Each SVG contains:
#   1. Photo background (base64 embedded)
#   2. Cuneiform text
#   3. Interval name
#   4. Shard info
#   5. Ishtar star decoration

set -euo pipefail

PROC_ID="SOP-TILE-002"
OUT="${1:?Usage: $0 <output_svg_dir> <bg_tiles_dir>}"
BG="${2:?Need bg tiles dir}"
N=71

echo "[$PROC_ID] Composing $N tile SVGs with embedded photos"
mkdir -p "$OUT"

CUNEIFORM=(
  "𒀸𒌑𒄴𒊑" "𒄿𒊭𒅈𒌈" "𒂊𒁍𒁍" "𒉌𒀉𒃻" "𒃻𒇷𒌈"
  "𒆠𒁴𒈬" "𒁉𒌈" "𒊺𒊒" "𒊭𒅖𒊭𒌈" "𒊑𒁍𒌈"
  "𒅖𒄣" "𒋾𒌅𒅈𒃻" "𒋾𒌅𒅈𒄿" "𒊺𒅈𒁺" "𒀀𒈬𒊏𒁉"
)

INTERVALS=(
  "nīš tuḫrim" "išartum" "embūbum" "nīd qablim" "qablītum"
  "kitmum" "pītum" "šērum" "šalšatum" "rebûttum"
  "isqum" "titur qablītim" "titur išartim" "ṣerdum" "colophon"
)

for i in $(seq 1 $N); do
  pad=$(printf '%02d' "$i")
  bg_file="$BG/${pad}.png"
  svg_file="$OUT/${pad}.svg"
  
  ci=$(( (i - 1) % 15 ))
  cunei="${CUNEIFORM[$ci]}"
  interval="${INTERVALS[$ci]}"
  
  o71=$(( i % 71 ))
  o59=$(( i % 59 ))
  o47=$(( i % 47 ))

  # Base64 encode the photo
  b64=$(base64 -w0 "$bg_file")

  cat > "$svg_file" << SVGEOF
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="512" height="512" viewBox="0 0 512 512">
  <image href="data:image/png;base64,${b64}" width="512" height="512" opacity="0.7"/>
  <rect x="30" y="40" width="452" height="75" fill="#1a1510" rx="6" opacity="0.88"/>
  <text x="256" y="95" text-anchor="middle" fill="#ffd700" font-size="48" font-weight="bold" font-family="serif">${cunei}</text>
  <rect x="50" y="125" width="412" height="35" fill="#1a1510" rx="4" opacity="0.85"/>
  <text x="256" y="150" text-anchor="middle" fill="#e8d8b0" font-family="monospace" font-size="20" font-weight="bold">${interval}</text>
  <rect x="60" y="170" width="392" height="22" fill="#1a1510" rx="3" opacity="0.75"/>
  <text x="256" y="186" text-anchor="middle" fill="#a0b0a0" font-family="monospace" font-size="9">orbifold (${o71},${o59},${o47}) mod (71,59,47) · shard ${i}/71</text>
  <rect x="30" y="440" width="452" height="65" fill="#1a1510" rx="5" opacity="0.88"/>
  <text x="256" y="462" text-anchor="middle" fill="#d0c8b0" font-family="monospace" font-size="13" font-weight="bold">Hurrian Hymn h.6 · Hymn to Nikkal</text>
  <text x="256" y="480" text-anchor="middle" fill="#90b8e0" font-family="monospace" font-size="10">Urẖiya (composer) · Ammurabi (scribe) · ~1400 BC · Ugarit</text>
  <text x="256" y="496" text-anchor="middle" fill="#707880" font-family="monospace" font-size="8">DA51 · Cl(15,0,0) · zaluzi · Tablet RS 15.30 · 6-layer stego</text>
</svg>
SVGEOF

  if [ $((i % 20)) -eq 0 ]; then
    echo "[$PROC_ID] $i/$N"
  fi
done

echo "[$PROC_ID] ✅ $N SVGs in $OUT/"
