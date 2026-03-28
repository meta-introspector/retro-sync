# External API Abstraction Plan
# Each external service → local DA51 stub that speaks erdfa

## External Dependencies → Local Replacements

| External | Env Var | DA51 Replacement | Status |
|----------|---------|-----------------|--------|
| BTFS | BTFS_API_URL | Local IPFS/file store | ✅ use local fs |
| BTTC RPC | BTTC_RPC_URL | Local Foundry anvil | ✅ in flake.nix |
| Coinbase Commerce | COINBASE_COMMERCE_API_KEY | Local payment stub (erdfa shard) | TODO |
| DDEX Gateway | DDEX_API_KEY | Local CWR/ERN generator (already have) | ✅ export-cwr.py |
| CMRRA | CMRRA_API_KEY | Local mechanical licence stub | TODO |
| BBS | BBS_API_KEY | Local cue sheet generator | TODO |
| Music Reports | MUSIC_REPORTS_API_KEY | Local royalty calculator | TODO |
| ISNI | ISNI_API_KEY | Wikidata SPARQL (already have) | ✅ artist-ids.py |
| Tron | TRON_API_URL | Local Foundry anvil | ✅ same as BTTC |
| Safe (Gnosis) | SAFE_API_URL | Local multi-sig stub | TODO |
| NCMEC | NCMEC_API_KEY | Local CSAM filter stub | TODO |
| Mirror BBS | MIRROR_BBS_URL | Local mirror | TODO |
| Archive | ARCHIVE_SECRET_KEY | Local file store | ✅ use local fs |
| JWT | JWT_SECRET | Generate locally | ✅ random |

## Dev Mode Env File

All stubs enabled, no external calls:

```env
BTTC_DEV_MODE=1
BTFS_API_URL=http://localhost:5001
BTTC_RPC_URL=http://localhost:8545
VAULT_RPC_URL=http://localhost:8545
TRON_API_URL=http://localhost:8545
SAFE_API_URL=http://localhost:8545
JWT_SECRET=dev-secret-retro-sync-2026
RETROSYNC_DATA_DIR=./data
```

## Architecture

```
External API call
  → DA51 adapter (erdfa shard interface)
  → Local stub (file/anvil/wikidata)
  → Response as erdfa CBOR shard
```

Each adapter is a thin layer that:
1. Accepts the same request format as the real API
2. Processes locally (file store, anvil, SPARQL)
3. Returns response as DA51-tagged CBOR
4. Logs the interaction as a witness shard
