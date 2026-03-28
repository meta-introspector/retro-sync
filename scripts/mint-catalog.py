#!/usr/bin/env python3
"""mint-catalog.py — Check mint status and mint unminted works as NFTs.

Reads catalog/works.json, checks which have token_ids, mints the rest.
Supports: BTTC (existing), Solana compressed NFTs (new).

Usage: python3 scripts/mint-catalog.py [--chain bttc|solana] [--dry-run]
"""

import json, hashlib, sys, os
from datetime import datetime

CATALOG = "catalog/works.json"
MINT_LOG = "catalog/mint_log.json"
API_URL = os.environ.get("RETROSYNC_API", "https://localhost:8443")

def load_mint_log():
    if os.path.exists(MINT_LOG):
        return json.load(open(MINT_LOG))
    return {"minted": [], "pending": []}

def save_mint_log(log):
    os.makedirs(os.path.dirname(MINT_LOG), exist_ok=True)
    with open(MINT_LOG, 'w') as f:
        json.dump(log, f, indent=2)

def work_id(work):
    """Deterministic ID from title + writer."""
    key = f"{work['title']}:{work['writers'][0]['name']}" if work.get('writers') else work['title']
    return hashlib.sha256(key.encode()).hexdigest()[:16]

def build_manifest(work, project_dir):
    """Build ShardManifest from work + stego tiles."""
    stego_dir = os.path.join("projects", work["retrosync"]["project"], "output", "stego")
    tiles = sorted(f for f in os.listdir(stego_dir) if f.endswith('.png')) if os.path.isdir(stego_dir) else []
    
    # Compute shard CIDs (content-addressed from tile hashes)
    shard_cids = []
    for t in tiles:
        path = os.path.join(stego_dir, t)
        h = hashlib.sha256(open(path, 'rb').read()).hexdigest()
        shard_cids.append(f"bafk{h[:32]}")
    
    # ZK commitment
    commit_data = json.dumps({"title": work["title"], "shards": len(shard_cids)}).encode()
    zk_commit = hashlib.sha256(commit_data).hexdigest()
    
    return {
        "version": 1,
        "title": work["title"],
        "writer": work["writers"][0]["name"] if work.get("writers") else "Unknown",
        "rs_id": work["writers"][0].get("rs_id", "") if work.get("writers") else "",
        "isrc": work.get("recording", {}).get("isrc"),
        "shard_count": len(shard_cids),
        "shard_cids": shard_cids[:5],  # first 5 for preview
        "zk_commit_hash": zk_commit,
        "blade_grade": work["retrosync"].get("blade_grade"),
        "orbifold": work["retrosync"].get("orbifold"),
        "project": work["retrosync"]["project"],
    }

def main():
    chain = "bttc"
    dry_run = False
    for arg in sys.argv[1:]:
        if arg.startswith("--chain="): chain = arg.split("=")[1]
        if arg == "--dry-run": dry_run = True
    
    catalog = json.load(open(CATALOG))
    works = catalog["works"]
    mint_log = load_mint_log()
    minted_ids = set(m["work_id"] for m in mint_log["minted"])
    
    print(f"=== MINT CATALOG ({chain.upper()}) ===")
    print(f"  Works: {len(works)}")
    print(f"  Already minted: {len(minted_ids)}")
    print(f"  Dry run: {dry_run}")
    print()
    
    # Check each work
    unminted = []
    for work in works:
        wid = work_id(work)
        status = "✅ minted" if wid in minted_ids else "⬜ pending"
        if wid not in minted_ids:
            unminted.append((wid, work))
    
    print(f"  Unminted: {len(unminted)}")
    print()
    
    if not unminted:
        print("  All works minted!")
        return
    
    # Mint each unminted work
    print(f"{'#':>3} {'Work ID':>18} {'Title':<40} {'Chain':<8} {'Status'}")
    print("-" * 90)
    
    for i, (wid, work) in enumerate(unminted):
        title = work["title"][:38]
        manifest = build_manifest(work, "projects")
        
        if dry_run:
            print(f"{i+1:>3} {wid} {title:<40} {chain:<8} 🔍 dry-run (zk={manifest['zk_commit_hash'][:12]}...)")
            mint_log["pending"].append({
                "work_id": wid,
                "title": work["title"],
                "chain": chain,
                "manifest": manifest,
                "status": "pending",
            })
        else:
            # Call the API to mint
            try:
                import urllib.request
                req_data = json.dumps({"manifest": manifest, "chain": chain}).encode()
                req = urllib.request.Request(
                    f"{API_URL}/api/manifest/mint",
                    data=req_data,
                    headers={"Content-Type": "application/json"},
                    method="POST"
                )
                resp = urllib.request.urlopen(req, timeout=30)
                result = json.loads(resp.read())
                token_id = result.get("token_id", "?")
                print(f"{i+1:>3} {wid} {title:<40} {chain:<8} ✅ token={token_id}")
                mint_log["minted"].append({
                    "work_id": wid,
                    "title": work["title"],
                    "chain": chain,
                    "token_id": token_id,
                    "tx_hash": result.get("tx_hash", ""),
                    "zk_commit": manifest["zk_commit_hash"],
                    "minted_at": datetime.utcnow().isoformat(),
                })
            except Exception as e:
                print(f"{i+1:>3} {wid} {title:<40} {chain:<8} ❌ {e}")
                mint_log["pending"].append({
                    "work_id": wid,
                    "title": work["title"],
                    "chain": chain,
                    "error": str(e),
                    "status": "failed",
                })
    
    save_mint_log(mint_log)
    print(f"\n  Log: {MINT_LOG}")
    print(f"  Minted: {len(mint_log['minted'])}, Pending: {len(mint_log['pending'])}")

if __name__ == "__main__":
    main()
