#!/usr/bin/env python3
"""catalog-gen.py — Generate WorkRegistration catalog from all onboarded projects.

Reads each project's project.toml + MIDI files + erdfa shards.
Outputs catalog/works.json in the retro-sync API bulk upload format.

Usage: python3 scripts/catalog-gen.py [projects_dir] [output]
"""

import os, json, sys

try:
    import toml
except ImportError:
    toml = None

PROJECTS_DIR = sys.argv[1] if len(sys.argv) > 1 else "projects"
OUTPUT = sys.argv[2] if len(sys.argv) > 2 else "catalog/works.json"

# Composer metadata (Wikidata QIDs for enrichment)
COMPOSERS = {
    "bach": {"name": "Johann Sebastian Bach", "qid": "Q1339", "born": 1685, "died": 1750},
    "bartok": {"name": "Béla Bartók", "qid": "Q83326", "born": 1881, "died": 1945},
    "beethoven": {"name": "Ludwig van Beethoven", "qid": "Q255", "born": 1770, "died": 1827},
    "chopin": {"name": "Frédéric Chopin", "qid": "Q1268", "born": 1810, "died": 1849},
    "debussy": {"name": "Claude Debussy", "qid": "Q151606", "born": 1862, "died": 1918},
    "gershwin": {"name": "George Gershwin", "qid": "Q123829", "born": 1898, "died": 1937},
    "grieg": {"name": "Edvard Grieg", "qid": "Q80621", "born": 1843, "died": 1907},
    "joplin": {"name": "Scott Joplin", "qid": "Q191499", "born": 1868, "died": 1917},
    "mozart": {"name": "Wolfgang Amadeus Mozart", "qid": "Q254", "born": 1756, "died": 1791},
    "pachelbel": {"name": "Johann Pachelbel", "qid": "Q76512", "born": 1653, "died": 1706},
    "ravel": {"name": "Maurice Ravel", "qid": "Q1178", "born": 1875, "died": 1937},
    "satie": {"name": "Erik Satie", "qid": "Q150600", "born": 1866, "died": 1925},
    "scarlatti": {"name": "Domenico Scarlatti", "qid": "Q185465", "born": 1685, "died": 1757},
    "stravinsky": {"name": "Igor Stravinsky", "qid": "Q7314", "born": 1882, "died": 1971},
    "tchaikovsky": {"name": "Pyotr Tchaikovsky", "qid": "Q7315", "born": 1840, "died": 1893},
    "vivaldi": {"name": "Antonio Vivaldi", "qid": "Q1340", "born": 1678, "died": 1741},
}

def load_erdfa(project_dir):
    """Load erdfa manifest if available."""
    manifest = os.path.join(project_dir, "output", "erdfa", "manifest.json")
    if os.path.exists(manifest):
        return json.load(open(manifest))
    return []

def main():
    os.makedirs(os.path.dirname(OUTPUT), exist_ok=True)
    
    works = []
    projects = sorted(d for d in os.listdir(PROJECTS_DIR) 
                      if os.path.isdir(os.path.join(PROJECTS_DIR, d)))
    
    print(f"=== CATALOG GENERATOR ===")
    print(f"  Projects: {PROJECTS_DIR} ({len(projects)} found)")
    
    for proj in projects:
        proj_dir = os.path.join(PROJECTS_DIR, proj)
        midi_dir = os.path.join(proj_dir, "midi")
        
        if not os.path.isdir(midi_dir):
            continue
        
        # Detect composer from project name
        composer_key = proj.split('-')[0]
        composer = COMPOSERS.get(composer_key, {"name": composer_key.title(), "qid": None})
        
        # Load erdfa data
        erdfa = load_erdfa(proj_dir)
        erdfa_by_idx = {s.get("name", ""): s for s in erdfa}
        
        midis = sorted(f for f in os.listdir(midi_dir) if f.endswith('.mid'))
        
        for i, midi_file in enumerate(midis):
            # Clean title from filename
            title = midi_file.replace('.mid', '')
            # Remove leading number prefix
            title = title.lstrip('0123456789_')
            # Remove composer prefix
            for prefix in [composer_key + '-', composer_key + '_']:
                if title.startswith(prefix):
                    title = title[len(prefix):]
            title = title.replace('_', ' ').replace('-', ' ').strip().title()
            
            # Get erdfa data for this shard
            shard_key = f"{i+1:02d}"
            shard = erdfa_by_idx.get(shard_key, {})
            
            work = {
                "title": title,
                "iswc": None,
                "writers": [{
                    "name": composer["name"],
                    "ipi": None,
                    "qid": composer.get("qid"),
                    "role": "C",
                    "share": 100,
                }],
                "publishers": [],
                "performing_artists": [],
                "recording": {
                    "isrc": None,
                    "title": title,
                    "format": "MIDI",
                    "source": "midi-classical-dataset",
                    "file": midi_file,
                },
                "territories": ["WW"],
                "license": "PD",
                "retrosync": {
                    "project": proj,
                    "shard_index": i + 1,
                    "blade_grade": shard.get("grade"),
                    "orbifold": shard.get("orbifold"),
                },
            }
            works.append(work)
        
        print(f"  {proj}: {len(midis)} works by {composer['name']}")
    
    catalog = {
        "works": works,
        "sender_id": "retro-sync-pd-catalog",
        "catalog_version": "0.1.0",
        "total_works": len(works),
        "projects": len(projects),
    }
    
    with open(OUTPUT, 'w') as f:
        json.dump(catalog, f, indent=2)
    
    print(f"\n  {len(works)} works → {OUTPUT}")

if __name__ == "__main__":
    main()
