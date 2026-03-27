# Projects

Each musical work is a project. The retro-sync pipeline ingests sources, extracts metadata, renders audio, packages as DA51 CBOR shards, and embeds in steganographic NFT tiles.

## Project Schema (project.toml)

```toml
[project]
name        = "work-slug"
title       = "Human-readable title"
date        = "composition date"
license     = "PD | CC-BY | rights-managed"
description = "..."

[sources.wikidata]     # Wikidata QID + SPARQL query
[sources.wikipedia]    # Wikipedia article
[sources.youtube]      # Witnessed performances (yt-dlp IDs)
[sources.archive_org]  # Archive.org items
[sources.imslp]        # IMSLP score IDs

[tiles]                # Stego tile config (count, pattern, dimensions)
[paths]                # Output directories
[segments]             # NFT7 segment names
```

## Usage

```bash
# Verify a specific project
cargo run --release --example verify_stego -- hurrian-h6

# Default project (from retro-sync.toml [platform].default)
cargo run --release --example verify_stego
```

## Public Domain Music Roadmap

Works are ordered chronologically. All pre-1929 works are PD in the US; EU life+70 applies to later works.

| Era | Project | Work | Date | Sources |
|-----|---------|------|------|---------|
| Ancient | `hurrian-h6` | Hurrian Hymn No. 6 | ~1400 BCE | YouTube, tablet photos |
| Ancient | `seikilos` | Seikilos Epitaph | ~200 BCE | Wikipedia, IMSLP |
| Medieval | `gregorian-chant` | Musica enchiriadis | ~900 CE | Archive.org, IMSLP |
| Medieval | `hildegard` | Hildegard von Bingen — Ordo Virtutum | ~1151 | IMSLP, Archive.org |
| Ars Nova | `machaut-messe` | Machaut — Messe de Nostre Dame | 1365 | IMSLP |
| Renaissance | `josquin-missa` | Josquin — Missa Pange Lingua | ~1515 | IMSLP |
| Renaissance | `palestrina-missa` | Palestrina — Missa Papae Marcelli | 1567 | IMSLP |
| Baroque | `bach-wtc` | Bach — Well-Tempered Clavier | 1722 | IMSLP, Archive.org |
| Baroque | `vivaldi-seasons` | Vivaldi — Four Seasons | 1725 | IMSLP |
| Baroque | `handel-messiah` | Handel — Messiah | 1741 | IMSLP, Archive.org |
| Classical | `mozart-requiem` | Mozart — Requiem K.626 | 1791 | IMSLP, Archive.org |
| Classical | `beethoven-sym` | Beethoven — 9 Symphonies | 1800–1824 | IMSLP, Archive.org |
| Romantic | `chopin-nocturnes` | Chopin — Nocturnes | 1827–1846 | IMSLP |
| Romantic | `liszt-sonata` | Liszt — Piano Sonata in B minor | 1854 | IMSLP |
| Romantic | `brahms-sym` | Brahms — 4 Symphonies | 1876–1885 | IMSLP |
| Romantic | `tchaikovsky-nutcracker` | Tchaikovsky — Nutcracker | 1892 | IMSLP, Archive.org |
| Late Romantic | `debussy-prelude` | Debussy — Prélude à l'après-midi | 1894 | IMSLP |
| Late Romantic | `mahler-sym` | Mahler — Symphonies | 1888–1910 | IMSLP |
| 20th C | `stravinsky-rite` | Stravinsky — Rite of Spring | 1913 | IMSLP (PD in US) |
| 20th C | `ravel-bolero` | Ravel — Boléro | 1928 | PD in US (pre-1929) |

## Data Sources per Project

Each project pulls from multiple sources to build a complete provenance chain:

1. **Wikidata** → structured metadata (composer, opus, key, instrumentation, ISRC/ISWC)
2. **Wikipedia** → historical context, analysis, reception
3. **IMSLP** → PD scores (PDF, LilyPond where available)
4. **Archive.org** → PD recordings, scans, manuscripts
5. **YouTube** → witnessed modern performances (with aubio extraction)

All metadata becomes DA51 CBOR shards. All audio/scores become NFT7 segments in stego tiles. The full provenance DAG is queryable like SPARQL.
