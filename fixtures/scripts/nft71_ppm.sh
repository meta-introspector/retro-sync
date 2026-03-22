#!/usr/bin/env bash
# nft71_ppm.sh — Generate 71 PPM tile images for NFT collection
# Each tile: 64x64, cuneiform text + notes + shard metadata
# Pattern follows Roebling NFT PPM tile approach
set -euo pipefail

OUT="fixtures/output/nft71_ppm"
mkdir -p "$OUT"

FONT_CUNEI="/usr/share/fonts/truetype/noto/NotoSansCuneiform-Regular.ttf"
FONT_MONO="DejaVu-Sans-Mono"

# Cuneiform signs from the h.6 tablet (transliterated terms → Unicode cuneiform)
# These are representative signs for each Babylonian interval term
CUNEIFORM=(
  "𒀸𒌑𒄴𒊑"    # nīš tuḫrim
  "𒄿𒊭𒅈𒌈"    # išartum
  "𒂊𒁍𒁍"      # embūbum
  "𒉌𒀉𒃻"      # nīd qablim
  "𒃻𒇷𒌈"      # qablītum
  "𒆠𒁴𒈬"      # kitmum
  "𒁉𒌈"        # pītum
  "𒊺𒊒"        # šērum
  "𒊭𒅖𒊭𒌈"    # šalšatum
  "𒊑𒁍𒌈"      # rebûttum
  "𒅖𒄣"        # isqum
  "𒋾𒌅𒅈𒃻"    # titur qablītim
  "𒋾𒌅𒅈𒄿"    # titur išartim
  "𒊺𒅈𒁺"      # ṣerdum
  "𒀀𒈬𒊏𒁉"    # Ammurabi (colophon)
)

# Notation from the tablet (Dietrich & Loretz 1975)
NOTATION_L1="qáb-li-te 3  ir-bu-te 1  qáb-li-te 3  ša-aḫ-ri 1  i-šar-te 10"
NOTATION_L2="ti-ti-mi-šar-te 2  zi-ir-te 1  ša-aḫ-ri 2  ša-aš-ša-te 2  ir-bu-te 2"

# SSP interval names
INTERVALS=(
  "nīš tuḫrim" "išartum" "embūbum" "nīd qablim" "qablītum"
  "kitmum" "pītum" "šērum" "šalšatum" "rebûttum"
  "isqum" "titur qablītim" "titur išartim" "ṣerdum" "colophon"
)

# Category colors (R,G,B backgrounds)
color_for() {
  case "$1" in
    generator)      echo "#1a1a2e";;  # deep blue
    source)         echo "#2d1b2e";;  # purple
    artifact)       echo "#1b2e1b";;  # dark green
    witness)        echo "#2e2e1b";;  # olive
    eigenspace)     echo "#1b2e2e";;  # teal
    metadata)       echo "#2e1b1b";;  # dark red
    reconstruction) echo "#1b1b2e";;  # indigo
    reference)      echo "#2e2b1b";;  # brown
    youtube)        echo "#2e1b2b";;  # magenta
    pipeline)       echo "#1b2b1b";;  # forest
    reserved)       echo "#1a1a1a";;  # charcoal
    *)              echo "#1a1a1a";;
  esac
}

# Category assignments (same as nft71.rs)
PRIMES=(2 3 5 7 11 13 17 19 23 29 31 37 41 43 47 53 59 61 67 71)
is_prime() {
  for p in "${PRIMES[@]}"; do [[ "$1" == "$p" ]] && return 0; done
  return 1
}

category_for() {
  local idx=$1
  if is_prime "$idx"; then echo "generator"; return; fi
  case "$idx" in
    4|6) echo "source";;
    8|9|10) echo "artifact";;
    12|14|15|16|18) echo "witness";;
    20|21|22|24|25) echo "eigenspace";;
    26|27|28|30|32|33|34|35) echo "metadata";;
    36|38|39|40|42) echo "reconstruction";;
    44|45|46|48|49|50|51|52|54|55|56|57) echo "reference";;
    58|60|62|63|64|65) echo "youtube";;
    66|68|69|70) echo "pipeline";;
    *) echo "reserved";;
  esac
}

# Spread cuneiform text across shards — each gets 1-2 signs
cunei_for() {
  local idx=$(( ($1 - 1) % ${#CUNEIFORM[@]} ))
  echo "${CUNEIFORM[$idx]}"
}

# Spread notation across shards
notation_for() {
  local idx=$1
  if (( idx % 2 == 1 )); then
    echo "$NOTATION_L1" | cut -d' ' -f$(( (idx / 2) % 5 + 1 ))-$(( (idx / 2) % 5 + 2 ))
  else
    echo "$NOTATION_L2" | cut -d' ' -f$(( (idx / 2) % 5 + 1 ))-$(( (idx / 2) % 5 + 2 ))
  fi
}

# Interval name for this shard
interval_for() {
  local idx=$(( ($1 - 1) % ${#INTERVALS[@]} ))
  echo "${INTERVALS[$idx]}"
}

echo "=== Generating 71 NFT PPM tiles (64×64) ==="

for idx in $(seq 1 71); do
  padded=$(printf "%02d" "$idx")
  cat=$(category_for "$idx")
  bg=$(color_for "$cat")
  cunei=$(cunei_for "$idx")
  interval=$(interval_for "$idx")
  notation=$(notation_for "$idx")

  # Prime marker
  if is_prime "$idx"; then
    marker="★"
    border_color="#ffd700"
  else
    marker="·"
    border_color="#444444"
  fi

  # Generate 64x64 PPM with layered text
  convert -size 64x64 "xc:${bg}" \
    -fill "$border_color" -draw "rectangle 0,0 63,1" -draw "rectangle 0,0 1,63" \
    -draw "rectangle 0,62 63,63" -draw "rectangle 62,0 63,63" \
    -fill "#ffd700" -font "$FONT_CUNEI" -pointsize 18 \
    -gravity North -annotate +0+3 "$cunei" \
    -fill "#c9d1d9" -font "$FONT_MONO" -pointsize 7 \
    -gravity Center -annotate +0-4 "$interval" \
    -fill "#8b949e" -font "$FONT_MONO" -pointsize 6 \
    -gravity South -annotate +0+12 "${marker}${padded} ${cat}" \
    -fill "#58a6ff" -font "$FONT_MONO" -pointsize 5 \
    -gravity South -annotate +0+4 "$notation" \
    "ppm:${OUT}/${padded}.ppm"

  echo "${marker} ${padded} [${cat}] ${cunei} — ${interval}"
done

# Also generate a 8×9 mosaic (72 tiles, last one blank)
echo ""
echo "=== Generating mosaic ==="
montage "${OUT}/"*.ppm -tile 8x9 -geometry 64x64+2+2 -background '#0d1117' \
  "ppm:${OUT}/mosaic.ppm"

TOTAL=$(du -sh "$OUT" | cut -f1)
COUNT=$(ls "$OUT"/*.ppm | grep -v mosaic | wc -l)
echo ""
echo "→ ${COUNT} tiles + mosaic written to ${OUT} (${TOTAL})"
echo "→ mosaic: ${OUT}/mosaic.ppm"
