# retro-sync — Music Publisher Onboarding

## What is retro-sync?

A music publishing platform for managing public domain catalogs. Register works, generate collection society submissions, browse metadata, and distribute to streaming platforms.

## Live

| URL | What |
|-----|------|
| [Catalog](https://solana.solfunmeme.com/retro-sync/) | Browse collections |
| [Dashboard](https://solana.solfunmeme.com/retro-sync/dashboard.html) | API explorer |
| [Dataset](https://huggingface.co/datasets/introspector/retro-sync) | Full catalog download |

## Catalog

- **35 works** — Bach Two-Part Inventions + Bartók piano pieces
- **2 artists** — with ISNI and Wikidata identifiers
- **CWR 2.2** — ready for submission to 50+ collection societies

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | /retro-sync/health | Health check |
| GET | /retro-sync/catalog/ | Browse catalog (works.json, artists.json, CWR) |
| GET | /retro-sync/api/societies | List collection societies |
| POST | /retro-sync/api/register | Register a new work |
| POST | /retro-sync/api/upload | Upload a track |
| POST | /retro-sync/api/gateway/ern/push | DDEX ERN distribution |

## Quick Start

### 1. Browse the catalog
Visit the catalog menu. Each work has title, writers, ISNI, territory info.

### 2. Download
```
git clone https://huggingface.co/datasets/introspector/retro-sync
```

### 3. Submit to societies
The CWR file at `catalog/retro-sync.cwr` covers ASCAP, BMI, SOCAN, SACEM, JASRAC, and 45+ more.

### 4. Register new works
POST to `/retro-sync/api/register` with title, writers, ISWC, territories.

### 5. Distribute
POST to `/retro-sync/api/gateway/ern/push` for DDEX ERN 4.1 delivery.

## Data Formats

- **works.json** — WorkRegistration format (title, writers, ISNI, ISWC, territories)
- **artists.json** — Artist identifiers (ISNI, VIAF, Wikidata QID)
- **retro-sync.cwr** — CWR 2.2 (108 records, 12.9KB)

## Compliance

- DMCA §512 notice-and-takedown
- GDPR/CCPA data rights
- CWR 2.2 full record set
- 50+ global collection societies supported
