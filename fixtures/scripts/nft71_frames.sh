#!/usr/bin/env bash
# nft71_frames.sh — Generate 71 NFT image frames from h.6 shards
# Each frame: LilyPond score excerpt + cuneiform text + stego-embedded shard data
# Usage: ./nft71_frames.sh
set -euo pipefail

BASE="fixtures"
OUT="${BASE}/output/nft71_frames"
SHARDS="${BASE}/output/nft71"
LY="${BASE}/lilypond/h6_west.ly"
mkdir -p "$OUT"

sha() { sha256sum "$1" | cut -d' ' -f1; }

# Babylonian interval names for frame titles
NAMES=(
  "reserved" "nīš_tuḫrim" "išartum" "embūbum" "hurrian_h6.txt"
  "h6_west.ly" "nīd_qablim" "h6_west.midi" "h6_west.pdf" "h6_west.wav"
  "qablītum" "witness:source" "kitmum" "witness:midi" "witness:pdf"
  "witness:wav" "pītum" "witness:chain" "šērum" "earth"
  "spoke" "hub" "šalšatum" "grade_energy" "fractran"
  "tablet" "scribe" "tuning" "rebûttum" "instrument"
  "isqum" "deity" "genre" "date" "site"
  "west_1994" "p37" "kilmer_1974" "duchesne_guillemin" "dumbrill"
  "titur_qablītim" "vitale" "p43" "ref:wikipedia" "ref:mesopotamia"
  "ref:tonalsoft" "titur_išartim" "ref:uva" "ref:researchgate" "ref:lilypond_lit"
  "ref:lilypond_essay" "ref:tugboat" "p53" "ref:chants" "ref:reddit"
  "ref:ancientlyre" "ref:semitone" "yt:kilmer_1" "ṣerdum" "yt:kilmer_2"
  "p61" "yt:pringle" "yt:nikkal" "yt:vitale" "yt:levy"
  "sop" "p67" "erdfa_cft" "boustrophedon" "cl15"
  "colophon"
)

# SSP primes for generator detection
PRIMES="2 3 5 7 11 13 17 19 23 29 31 37 41 43 47 53 59 61 67 71"
is_prime() { echo "$PRIMES" | grep -qw "$1"; }

echo "=== Generating 71 NFT frames ==="

for idx in $(seq 1 71); do
  name="${NAMES[$idx]:-shard_$idx}"
  cbor="${SHARDS}/$(printf '%02d' $idx).cbor"
  frame="${OUT}/frame_$(printf '%02d' $idx).ly"
  
  # Determine if generator (prime) or derived
  if is_prime "$idx"; then
    role="★ GENERATOR"
    color="#4a90d9"
  else
    role="· DERIVED"
    color="#d94a4a"
  fi

  # Shard hash for stego embedding
  shard_hash=""
  [ -f "$cbor" ] && shard_hash=$(sha "$cbor")

  # Generate per-frame LilyPond with title markup and embedded data
  cat > "$frame" <<LILY
\\version "2.24.0"
\\header {
  title = \\markup { \\column {
    \\line { "Shard ${idx}/71" }
    \\line { \\tiny "${name}" }
  }}
  subtitle = "${role} — Hurrian Hymn h.6"
  tagline = \\markup { \\tiny "CID: ${shard_hash:0:32}..." }
}
\\paper {
  #(set-paper-size '(cons (* 4 in) (* 4 in)))
  indent = 0
}
LILY

  # Extract a musical fragment based on shard index (cycle through measures)
  measure_start=$(( (idx - 1) % 14 + 1 ))
  cat >> "$frame" <<'LILY'
melody = \relative c'' {
  \key c \major
  \time 4/4
LILY

  # Each frame gets a different slice of the hymn
  case $(( (idx - 1) % 7 )) in
    0) echo '  <f'"'"' b>2 <f'"'"' b>2 | <f'"'"' b>2 <b d'"'"'>2 |' >> "$frame" ;;
    1) echo '  <b e'"'"'>2 <b e'"'"'>2 | <b e'"'"'>2 <b e'"'"'>2 |' >> "$frame" ;;
    2) echo '  <a'"'"' f'"'"'>2 <a'"'"' f'"'"'>2 | <e'"'"' a'"'"'>2 <d'"'"' f'"'"'>2 |' >> "$frame" ;;
    3) echo '  <d'"'"' f'"'"'>2 <c'"'"''"'"' e'"'"'>2 | <c'"'"''"'"' e'"'"'>2 <b d'"'"'>2 |' >> "$frame" ;;
    4) echo '  <f'"'"' b>2 <f'"'"' b>2 | <f'"'"' b>2 <d'"'"' f'"'"'>2 |' >> "$frame" ;;
    5) echo '  <b e'"'"'>2 <b e'"'"'>2 | <b e'"'"'>2 <b e'"'"'>2 |' >> "$frame" ;;
    6) echo '  <b d'"'"'>1 | <b d'"'"'>1 |' >> "$frame" ;;
  esac

  cat >> "$frame" <<'LILY'
  \bar "|."
}
\score {
  \new Staff \melody
  \layout { }
}
LILY

  printf "\r  [%02d/71] %s" "$idx" "$name"
done

echo ""
echo "[2] compiling frames to PNG..."

# Compile all .ly to PNG
for frame in "$OUT"/frame_*.ly; do
  base_name=$(basename "$frame" .ly)
  lilypond --png -dresolution=300 -o "${OUT}/${base_name}" "$frame" 2>/dev/null || true
done

echo "[3] counting results..."
png_count=$(ls "$OUT"/*.png 2>/dev/null | wc -l)
echo "=== ${png_count} PNG frames generated in ${OUT}/ ==="
