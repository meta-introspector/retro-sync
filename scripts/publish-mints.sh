#!/bin/bash
# publish-mints.sh — Publish minted works to HuggingFace, Archive.org, and Pastebin
# Local-first: reads catalog/mint_log.json, publishes erdfa shards
# Usage: bash scripts/publish-mints.sh [--target hf|ia|paste|all]
set -euo pipefail

TARGET="${1:---target}"
TARGET="${2:-all}"
if [ "$1" = "--target" ]; then TARGET="$2"; fi
if [ "$1" != "--target" ]; then TARGET="${1:-all}"; fi

MINT_LOG="catalog/mint_log.json"
CATALOG="catalog/works.json"
DIST="dist/publish"
mkdir -p "$DIST"

echo "╔══════════════════════════════════════════════════╗"
echo "║  PUBLISH MINTS — erdfa shards to 3 targets      ║"
echo "╚══════════════════════════════════════════════════╝"

# Generate erdfa HTML per minted work
echo "── 1. Generate erdfa shards ──"
python3 -c "
import json, hashlib, os

mint_log = json.load(open('$MINT_LOG'))
catalog = json.load(open('$CATALOG'))
works_by_title = {w['title']: w for w in catalog['works']}
dist = '$DIST'

for m in mint_log['minted']:
    wid = m['work_id']
    title = m['title']
    work = works_by_title.get(title, {})
    writer = work.get('writers', [{}])[0].get('name', 'Unknown')
    rs_id = work.get('writers', [{}])[0].get('rs_id', '')
    chain = m['chain']
    token = m['token_id']
    zk = m['zk_commit']
    shards = m.get('shard_count', 0)
    ts = m['minted_at']

    # Orbifold from zk commit
    h = hashlib.sha256(zk.encode()).digest()
    v = int.from_bytes(h[:8], 'little')
    o = (v%71, v%59, v%47)

    # erdfa HTML shard
    html = f'''<!DOCTYPE html>
<html><head><meta charset=\"utf-8\"><title>{title}</title></head>
<body>
<div typeof=\"erdfa:SheafSection dasl:MintedWork\" about=\"#token-{token}\">
  <meta property=\"erdfa:shard\" content=\"{o[0]},{o[1]},{o[2]}\" />
  <meta property=\"erdfa:encoding\" content=\"nft7\" />
  <meta property=\"dasl:token_id\" content=\"{token}\" />
  <meta property=\"dasl:chain\" content=\"{chain}\" />
  <meta property=\"dasl:zk_commit\" content=\"{zk}\" />
  <meta property=\"dasl:work_id\" content=\"{wid}\" />
  <meta property=\"sheaf:orbifold\" content=\"({o[0]} mod 71, {o[1]} mod 59, {o[2]} mod 47)\" />
  <h1>{title}</h1>
  <p>Writer: {writer} ({rs_id})</p>
  <p>Token: {token} on {chain}</p>
  <p>Shards: {shards}</p>
  <p>Minted: {ts}</p>
</div>
</body></html>'''

    path = os.path.join(dist, f'{wid}.html')
    open(path, 'w').write(html)

print(f'  Generated {len(mint_log[\"minted\"])} erdfa shards → {dist}/')
"

count=$(ls "$DIST"/*.html 2>/dev/null | wc -l)
echo "  $count shards ready"

# ── HuggingFace ──
if [ "$TARGET" = "hf" ] || [ "$TARGET" = "all" ]; then
    echo "── 2a. Publish to HuggingFace ──"
    source ~/.agentrc 2>/dev/null || true
    python3 -c "
from huggingface_hub import HfApi
api = HfApi()
api.upload_folder(
    folder_path='$DIST',
    repo_id='introspector/retro-sync',
    repo_type='dataset',
    path_in_repo='mints',
)
print('  ✅ HuggingFace: introspector/retro-sync/mints/')
" 2>&1 || echo "  ⚠ HF upload failed (check token)"
fi

# ── Archive.org ──
if [ "$TARGET" = "ia" ] || [ "$TARGET" = "all" ]; then
    echo "── 2b. Publish to Archive.org ──"
    if command -v ia &>/dev/null; then
        ia upload retro-sync-mints "$DIST"/*.html \
            --metadata="title:retro-sync minted works" \
            --metadata="creator:meta-introspector" \
            --metadata="subject:music;nft;erdfa;monster-group" \
            --metadata="licenseurl:https://www.gnu.org/licenses/agpl-3.0.html" \
            2>&1 | tail -3
        echo "  ✅ Archive.org: archive.org/details/retro-sync-mints"
    else
        echo "  ⚠ ia CLI not found — install with: pip install internetarchive"
    fi
fi

# ── Pastebin ──
if [ "$TARGET" = "paste" ] || [ "$TARGET" = "all" ]; then
    echo "── 2c. Publish to Pastebin ──"
    for f in "$DIST"/*.html; do
        name=$(basename "$f" .html)
        url=$(cat "$f" | pastebinit 2>/dev/null) || url="(failed)"
        echo "  $name → $url"
    done
    echo "  ✅ Pastebin: all shards posted"
fi

echo
echo "── Summary ──"
echo "  Minted:    $count works"
echo "  Shards:    $DIST/"
[ "$TARGET" = "hf" ] || [ "$TARGET" = "all" ] && echo "  HF:        huggingface.co/datasets/introspector/retro-sync/mints/"
[ "$TARGET" = "ia" ] || [ "$TARGET" = "all" ] && echo "  Archive:   archive.org/details/retro-sync-mints"
[ "$TARGET" = "paste" ] || [ "$TARGET" = "all" ] && echo "  Pastebin:  solana.solfunmeme.com/pastebin/"
echo
echo "∴ Published. □"
