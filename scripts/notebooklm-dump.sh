#!/bin/bash
# notebooklm-dump.sh — Assemble a <3MB text dump for Google NotebookLM
# Usage: bash scripts/notebooklm-dump.sh > dist/notebooklm-dump.txt
set -euo pipefail

MAX_BYTES=3145728  # 3MB
OUT=""

section() { OUT+=$'\n'"========================================================================"$'\n'"= $1"$'\n'"========================================================================"$'\n'; }
file_section() {
    local label="$1" path="$2"
    if [ -f "$path" ]; then
        OUT+=$'\n'"--- $label: $path ---"$'\n'
        OUT+="$(cat "$path")"$'\n'
    fi
}

# ── 1. Project overview ──────────────────────────────────────────────
section "PROJECT OVERVIEW"
file_section "README" README.md
file_section "Config" retro-sync.toml
file_section "Makefile" Makefile
file_section "Getting Started" docs/GETTING-STARTED.md 2>/dev/null
file_section "SOP Onboard" docs/SOP-ONBOARD.md 2>/dev/null
file_section "API Abstraction" docs/API-ABSTRACTION.md 2>/dev/null

# ── 2. Private docs (~/DOCS) ─────────────────────────────────────────
section "PRIVATE DOCS (~/DOCS/services/retro-sync)"
for f in ~/DOCS/services/retro-sync/*.md; do
    file_section "$(basename "$f")" "$f"
done

# ── 3. Catalog ───────────────────────────────────────────────────────
section "CATALOG"
file_section "Works" catalog/works.json
file_section "Artists" catalog/artists.json

# ── 4. Scripts ───────────────────────────────────────────────────────
section "SCRIPTS"
for f in scripts/*.sh scripts/*.py; do
    [ -f "$f" ] && file_section "$(basename "$f")" "$f"
done

# ── 5. API Server source ─────────────────────────────────────────────
section "API SERVER (Rust)"
file_section "Cargo.toml (workspace)" Cargo.toml
file_section "Cargo.toml (backend)" apps/api-server/Cargo.toml
for f in apps/api-server/src/*.rs; do
    [ -f "$f" ] && file_section "$(basename "$f")" "$f"
done

# ── 6. Libraries ─────────────────────────────────────────────────────
section "LIBRARIES"
for f in libs/*/src/*.rs; do
    [ -f "$f" ] && file_section "$f" "$f"
done

# ── 7. Smart contracts ───────────────────────────────────────────────
section "SMART CONTRACTS (Solidity)"
for f in contracts/src/*.sol; do
    [ -f "$f" ] && file_section "$(basename "$f")" "$f"
done

# ── 8. Env + ops ─────────────────────────────────────────────────────
section "OPS"
file_section ".env.dev" .env.dev
file_section "systemd" ops/retro-sync.service
file_section "flake.nix" flake.nix

# ── Truncate if over 3MB ─────────────────────────────────────────────
BYTES=${#OUT}
if [ "$BYTES" -gt "$MAX_BYTES" ]; then
    OUT="${OUT:0:$MAX_BYTES}"
    OUT+=$'\n'"[TRUNCATED at 3MB]"$'\n'
fi

echo "$OUT"
>&2 echo "notebooklm-dump: $(echo "$OUT" | wc -c) bytes"
