#!/usr/bin/env bash
set -euo pipefail
# deploy-space.sh — Deploy project to HuggingFace Spaces (static site)
# Usage: deploy-space.sh <project_dir>
# Env: SPACE_DIR (default: ../retro-sync-space relative to repo root)

PROJECT_DIR="${1:?Usage: deploy-space.sh <project_dir>}"
PROJECT_DIR=$(realpath "$PROJECT_DIR")
SLUG=$(basename "$PROJECT_DIR")
REPO_ROOT=$(realpath "$(dirname "$0")/..")
SPACE_DIR="${SPACE_DIR:-$(realpath "$REPO_ROOT/../retro-sync-space")}"
DOCS_DIR="$REPO_ROOT/docs"

# Validate inputs
[ -d "$PROJECT_DIR/output/stego" ] || { echo "❌ No stego tiles"; exit 1; }
[ -d "$SPACE_DIR/.git" ] || { echo "❌ Space repo not at $SPACE_DIR"; exit 1; }

echo "=== DEPLOY SPACE: $SLUG ==="

# 1. Reset Space to clean state
cd "$SPACE_DIR"
git checkout main 2>/dev/null || true
git rebase --abort 2>/dev/null || true

# 2. Write README
cat > README.md <<EOF
---
title: "retro-sync"
emoji: 🎵
colorFrom: indigo
colorTo: yellow
sdk: static
pinned: true
license: agpl-3.0
short_description: "Decode MIDIs from steganographic tiles"
---
EOF

# 3. Setup LFS
git lfs install 2>/dev/null
git lfs track "*.png" 2>/dev/null

# 4. Copy viewer
cp "$DOCS_DIR/index.html" . 2>/dev/null || cp "$PROJECT_DIR/../../docs/index.html" .

# 5. Copy WASM
mkdir -p pkg
cp "$DOCS_DIR/pkg/"* pkg/ 2>/dev/null || true

# 6. Copy tiles
mkdir -p tiles
rm -f tiles/*.png
cp "$PROJECT_DIR/output/stego/"*.png tiles/
TILES=$(ls tiles/*.png | wc -l)

# 7. Commit and push
git add -A
git commit -m "deploy: $SLUG — $TILES tiles $(date -u +%Y%m%d)" 2>/dev/null || echo "(no changes)"
git push origin main --force 2>&1 | tail -5

echo
echo "✅ $TILES tiles → https://huggingface.co/spaces/introspector/retro-sync"
