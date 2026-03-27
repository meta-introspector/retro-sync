#!/usr/bin/env bash
set -euo pipefail
# deploy-hf.sh — Upload a project's stego tiles to HuggingFace
# Usage: deploy-hf.sh <project_dir>
# Requires: source ~/.agentrc first for HF auth

PROJECT_DIR="${1:?Usage: deploy-hf.sh <project_dir>}"
PROJECT_DIR=$(realpath "$PROJECT_DIR")
SLUG=$(basename "$PROJECT_DIR")
REPO=$(python3 -c "import toml; print(toml.load('retro-sync.toml')['publish']['huggingface_dataset'])" 2>/dev/null || echo "introspector/retro-sync")
STEGO_DIR="$PROJECT_DIR/output/stego"
VIEWER="$PROJECT_DIR/../../docs/index.html"

[ -d "$STEGO_DIR" ] || { echo "❌ No stego tiles in $STEGO_DIR"; exit 1; }

TILES=$(ls "$STEGO_DIR"/*.png 2>/dev/null | wc -l)
[ "$TILES" -gt 0 ] || { echo "❌ No PNG tiles found"; exit 1; }

echo "=== DEPLOY TO HUGGINGFACE ==="
echo "  Project: $SLUG"
echo "  Tiles:   $TILES"
echo "  Repo:    $REPO"
echo

python3 -c "
from huggingface_hub import HfApi
import os, sys

api = HfApi()
slug = '$SLUG'
repo = '$REPO'
stego_dir = '$STEGO_DIR'
viewer = '$VIEWER'

tiles = sorted(f for f in os.listdir(stego_dir) if f.endswith('.png'))
print(f'Uploading {len(tiles)} tiles...')

for i, f in enumerate(tiles):
    path = os.path.join(stego_dir, f)
    api.upload_file(
        path_or_fileobj=path,
        path_in_repo=f'{slug}/tiles/{f}',
        repo_id=repo,
        repo_type='dataset',
    )
    if (i+1) % 10 == 0 or i+1 == len(tiles):
        print(f'  {i+1}/{len(tiles)}')

if os.path.exists(viewer):
    api.upload_file(
        path_or_fileobj=viewer,
        path_in_repo=f'{slug}/index.html',
        repo_id=repo,
        repo_type='dataset',
    )
    print('  ✅ index.html')

print(f'\\n✅ Deployed {slug} to https://huggingface.co/datasets/{repo}')
"

echo "=== DEPLOY COMPLETE ==="
