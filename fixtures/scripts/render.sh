#!/usr/bin/env bash
# render.sh — Hurrian h.6 LilyPond → MIDI → WAV pipeline with zkperf witnesses
# Each step records: input hash, tool version, output hash, timestamp
# Usage: ./render.sh [input.ly] [output_dir]
set -euo pipefail

LY="${1:-fixtures/lilypond/h6_west.ly}"
OUT="${2:-fixtures/output}"
WITNESS_DIR="${OUT}/witnesses"
mkdir -p "$OUT" "$WITNESS_DIR"

ts() { date -u +%Y%m%dT%H%M%SZ; }
sha() { sha256sum "$1" | cut -d' ' -f1; }

witness() {
  local step="$1" tool="$2" version="$3" input_hash="$4" output_file="$5"
  local output_hash exit_code
  output_hash=$(sha "$output_file")
  cat > "${WITNESS_DIR}/${step}.witness.json" <<EOF
{
  "step": "${step}",
  "timestamp": "$(ts)",
  "tool": "${tool}",
  "tool_version": "${version}",
  "input_hash": "${input_hash}",
  "output_file": "$(basename "$output_file")",
  "output_hash": "${output_hash}",
  "output_bytes": $(stat -c%s "$output_file"),
  "hostname": "$(hostname)",
  "platform": "$(uname -s)-$(uname -m)"
}
EOF
  echo "  witness: ${step} → ${output_hash:0:16}..."
}

echo "=== retro-sync h.6 render pipeline ==="
echo "input: ${LY}"

# Step 0: Source witness
INPUT_HASH=$(sha "$LY")
GIT_HASH=$(git rev-parse HEAD 2>/dev/null || echo "none")
cat > "${WITNESS_DIR}/00_source.witness.json" <<EOF
{
  "step": "00_source",
  "timestamp": "$(ts)",
  "file": "${LY}",
  "sha256": "${INPUT_HASH}",
  "git_commit": "${GIT_HASH}",
  "hostname": "$(hostname)"
}
EOF
echo "[0] source: ${INPUT_HASH:0:16}..."

# Step 1: LilyPond → PDF + MIDI
echo "[1] lilypond compile..."
LILY_VER=$(lilypond --version 2>&1 | head -1)
lilypond -o "${OUT}/h6_west" "$LY" 2>"${OUT}/lilypond.log"
witness "01_midi" "lilypond" "$LILY_VER" "$INPUT_HASH" "${OUT}/h6_west.midi"
[ -f "${OUT}/h6_west.pdf" ] && witness "01_pdf" "lilypond" "$LILY_VER" "$INPUT_HASH" "${OUT}/h6_west.pdf"

# Step 2: MIDI → WAV via fluidsynth
echo "[2] fluidsynth render..."
MIDI_HASH=$(sha "${OUT}/h6_west.midi")
FLUID_VER=$(fluidsynth --version 2>&1 | head -1)
# Use default soundfont — nix provides one
SF2="${SOUNDFONT:-/run/current-system/sw/share/soundfonts/default.sf2}"
[ ! -f "$SF2" ] && SF2=$(find /nix/store -name "*.sf2" -path "*/share/*" 2>/dev/null | head -1)
[ ! -f "$SF2" ] && SF2=$(find /usr/share -name "*.sf2" 2>/dev/null | head -1)
if [ -n "$SF2" ] && [ -f "$SF2" ]; then
  fluidsynth -ni -F "${OUT}/h6_west.wav" -r 44100 "$SF2" "${OUT}/h6_west.midi" 2>"${OUT}/fluidsynth.log"
  witness "02_wav" "fluidsynth" "$FLUID_VER" "$MIDI_HASH" "${OUT}/h6_west.wav"
  SF_HASH=$(sha "$SF2")
  echo "  soundfont: ${SF2} (${SF_HASH:0:16}...)"
else
  echo "  WARN: no soundfont found, skipping WAV render"
  echo "  set SOUNDFONT=/path/to/file.sf2 or install fluid-soundfont-gm"
fi

# Step 3: Chain commitment
echo "[3] chain commitment..."
CHAIN=$(cat "${WITNESS_DIR}"/*.witness.json | sha256sum | cut -d' ' -f1)
cat > "${WITNESS_DIR}/99_commitment.witness.json" <<EOF
{
  "step": "99_commitment",
  "timestamp": "$(ts)",
  "chain_hash": "${CHAIN}",
  "witness_count": $(ls "${WITNESS_DIR}"/*.witness.json | wc -l),
  "pipeline": "retro-sync/h6-render",
  "sop": "SOP-RETROSYNC-PUB-001"
}
EOF
echo "=== commitment: ${CHAIN:0:32}... ==="
