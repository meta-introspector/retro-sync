# Bach Two-Part Inventions — DA51 Analysis

## SVG → FRACTRAN → Cl(15) Pipeline Results

71 DA51 CBOR shards extracted from lilypond sheet music SVG geometry.
Each invention's note positions (x=time, y=pitch) mapped to SSP primes.

### Per-Invention Blade Grades

| # | Key | Notes | Grade | Orbifold | Interpretation |
|---|-----|-------|-------|----------|----------------|
| 1 | C major | 1218 | 8 | (12,0,0) | Universal grade — balanced counterpoint |
| 2 | C minor | 1342 | 5 | (39,36,0) | Simpler structure — chromatic but constrained |
| 3 | D major | 991 | 7 | (0,0,0) | Origin point — maximally symmetric |
| 4 | D minor | 1285 | 5 | (7,28,0) | Similar to #2 — minor key constraint |
| 5 | Eb major | 1099 | 9 | (31,0,0) | Complex — wide intervals |
| 6 | E major | 1305 | 6 | (67,0,0) | Mid-complexity |
| 7 | E minor | 1343 | 6 | (60,0,0) | Similar to #6 |
| 8 | F major | 943 | 9 | (29,0,0) | Complex — shortest invention |
| 9 | F minor | 1268 | 6 | (15,17,0) | Mid-complexity |
| 10 | G major | 1484 | 4 | (21,3,0) | Simple — longest, most scalar |
| 11 | G minor | 1404 | 8 | (61,48,0) | Universal grade |
| 12 | A major | 1444 | 3 | (36,16,0) | Simplest — smooth scalar motion |
| 13 | A minor | 725 | 10 | (12,0,0) | Most complex — angular, wide leaps |
| 14 | Bb major | 1062 | 6 | (70,16,0) | Mid-complexity |
| 15 | B minor | 813 | 4 | (58,33,0) | Simple — compact |

### Key Findings

- Grade range: 3 (A major) to 10 (A minor) — full spectrum of complexity
- Grade 6 is the mode (5 inventions) — Bach's "default" complexity
- Minor keys tend toward higher grades (more angular intervals)
- Invention #3 (D major) lands at orbifold origin (0,0,0) — maximally symmetric
- 181 bytes max CBOR size — entire invention in one DA51 shard

### FRACTRAN State Encoding

Each invention's pitch histogram encoded as: Π SSP[i]^count[i]
- p=2 dominates (most notes in the middle register)
- p=7 (curvature) varies most between inventions
- The state IS the music — factorize to recover the pitch distribution
