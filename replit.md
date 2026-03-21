# Retrosync Media Group — Enterprise Platform

## Project Overview
A decentralized media distribution and royalty management platform for the music industry. Features peer-to-peer music distribution with zero-knowledge royalty verification built on the BTTC (BitTorrent Chain) blockchain.

## Architecture
- **Frontend**: React + TypeScript + Vite + Tailwind CSS + Shadcn UI (in `apps/web-client/`)
- **Backend**: Rust/Axum API server (in `apps/api-server/`) — not currently running as a workflow
- **WASM Frontend**: Rust/Yew alternative frontend (in `apps/wasm-frontend/`)
- **Smart Contracts**: Solidity via Foundry (in `libs/contracts/`)
- **Shared Libs**: Rust shared code, ZK circuits (in `libs/`)

## Running the App
- **Workflow**: "Start application" runs `npm run dev` → Vite serves the React frontend on port 5000
- **Host**: `0.0.0.0` with `allowedHosts: true` for Replit proxy compatibility

## Key Technologies
- React 18, TypeScript, Vite 8, Tailwind CSS, Shadcn UI
- React Router v6, TanStack Query, Framer Motion, Recharts
- Rust workspace (Cargo), Axum, Tokio
- BTTC/BTFS blockchain integration
- Zero-knowledge proofs (arkworks Groth16/BN254)
- DDEX ERN 4.1, CWR compliance protocols

## Package Management
- Frontend: npm with `--legacy-peer-deps` flag (due to Vite 8 peer dependency constraints)
- Backend/Rust: Cargo workspace

## Security Features (implemented)
- **Rate limiting**: Per-IP sliding-window middleware — 120/min general, 10/min auth, 5/min upload; IP from X-Real-IP / X-Forwarded-For
- **Wallet auth**: Challenge-response authentication (`GET /api/auth/challenge/:addr`, `POST /api/auth/verify`) — EIP-191 ECDSA, 24h JWT, single-use nonces
- **LMDB persistence**: All five stores (KYC, moderation, privacy, takedown, ZK cache) backed by heed 0.20 LMDB — survive restarts
- **Per-user auth guards**: KYC and privacy endpoints enforce `JWT sub == uid` — 403 on mismatch
- **BTFS API key**: `X-API-Key` header on all BTFS requests when `BTFS_API_KEY` env var is set
- **BTFS TLS**: HTTP blocked in production (`RETROSYNC_ENV=production`) — requires HTTPS reverse proxy
- **NCMEC CyberTipline**: CSAM reports auto-submit to NCMEC API (18 U.S.C. §2258A); gated on `NCMEC_API_KEY`
- **JWT extractor**: `auth::extract_caller` decodes Bearer JWT, checks expiry, returns wallet address
- **CORS**: Locked to `ALLOWED_ORIGINS` env var
- **Upload cap**: 100MB hard limit (`MAX_AUDIO_BYTES` env var)
- **DDEX**: XML escaping on all user inputs
- **Moderation IDs**: Cryptographically random (OS entropy)

## Key Backend Modules (apps/api-server/src/)
- `persist.rs` — generic LMDB store (put/get/append/update/delete)
- `rate_limit.rs` — per-IP sliding-window rate limiter middleware
- `wallet_auth.rs` — challenge issuance, ECDSA verify, JWT issuance
- `auth.rs` — Zero Trust middleware + `extract_caller` helper
- `kyc.rs` — KYC/AML with LMDB + per-user guard
- `moderation.rs` — DSA content queue with LMDB + NCMEC CyberTipline reporting
- `privacy.rs` — GDPR/CCPA with LMDB + per-user guard
- `takedown.rs` — DMCA §512 with LMDB
- `zk_cache.rs` — ZK proof cache with LMDB
- `btfs.rs` — BTFS upload/pin with API key auth + TLS enforcement

## Deployment
- Target: Static site
- Build: `npm run build`
- Public dir: `dist`
