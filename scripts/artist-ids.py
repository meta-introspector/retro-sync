#!/usr/bin/env python3
"""artist-ids.py — Generate unique artist/publisher IDs from Wikidata QIDs.

Format: RS-{QID}-{DA51_SHORT}
  RS = retro-sync prefix
  QID = Wikidata entity ID (globally unique)
  DA51_SHORT = first 8 hex of SHA256, gives orbifold position

Also looks up real IPI/ISNI from Wikidata SPARQL if available.

Usage: python3 scripts/artist-ids.py [catalog/works.json]
"""

import json, hashlib, sys, os
from urllib.request import Request, urlopen
from urllib.parse import quote as urlquote

def da51_addr(qid):
    """Compute DA51 address from QID."""
    h = hashlib.sha256(qid.encode()).digest()
    short = h[:4].hex()
    orb71 = int.from_bytes(h[12:13], 'big') % 71
    orb59 = int.from_bytes(h[13:14], 'big') % 59
    orb47 = int.from_bytes(h[14:15], 'big') % 47
    return short, (orb71, orb59, orb47)

def rs_id(qid):
    """Generate retro-sync artist ID: RS-Q1339-a7b3c2d1"""
    short, orb = da51_addr(qid)
    return f"RS-{qid}-{short}", orb

def lookup_wikidata(qid):
    """Fetch IPI/ISNI from Wikidata SPARQL."""
    query = f"""SELECT ?ipi ?isni ?viaf WHERE {{
      OPTIONAL {{ wd:{qid} wdt:P3453 ?ipi . }}
      OPTIONAL {{ wd:{qid} wdt:P213 ?isni . }}
      OPTIONAL {{ wd:{qid} wdt:P214 ?viaf . }}
    }} LIMIT 1"""
    url = f"https://query.wikidata.org/sparql?query={urlquote(query)}&format=json"
    try:
        req = Request(url, headers={"User-Agent": "retro-sync/0.1"})
        resp = urlopen(req, timeout=10)
        data = json.loads(resp.read())
        bindings = data.get("results", {}).get("bindings", [{}])[0]
        return {
            "ipi": bindings.get("ipi", {}).get("value"),
            "isni": bindings.get("isni", {}).get("value"),
            "viaf": bindings.get("viaf", {}).get("value"),
        }
    except:
        return {}

def main():
    catalog_path = sys.argv[1] if len(sys.argv) > 1 else "catalog/works.json"
    output_path = sys.argv[2] if len(sys.argv) > 2 else "catalog/artists.json"
    
    catalog = json.load(open(catalog_path))
    
    # Collect unique writers
    seen = {}
    for work in catalog["works"]:
        for writer in work.get("writers", []):
            qid = writer.get("qid")
            name = writer.get("name", "Unknown")
            if qid and qid not in seen:
                seen[qid] = name
    
    print(f"=== ARTIST ID GENERATOR ===")
    print(f"  {len(seen)} unique artists from {len(catalog['works'])} works\n")
    
    artists = []
    for qid, name in sorted(seen.items(), key=lambda x: x[1]):
        artist_id, orb = rs_id(qid)
        
        # Lookup real identifiers from Wikidata
        wd = lookup_wikidata(qid)
        
        artist = {
            "id": artist_id,
            "name": name,
            "qid": qid,
            "ipi": wd.get("ipi"),
            "isni": wd.get("isni"),
            "viaf": wd.get("viaf"),
            "orbifold": list(orb),
            "da51": f"0xda51{hashlib.sha256(qid.encode()).hexdigest()[:8]}",
        }
        artists.append(artist)
        
        ids = []
        if wd.get("ipi"): ids.append(f"IPI:{wd['ipi']}")
        if wd.get("isni"): ids.append(f"ISNI:{wd['isni']}")
        if wd.get("viaf"): ids.append(f"VIAF:{wd['viaf']}")
        id_str = " ".join(ids) if ids else "(no external IDs)"
        
        print(f"  {artist_id:<24} {name:<30} orb={orb} {id_str}")
    
    # Update catalog works with artist IDs
    for work in catalog["works"]:
        for writer in work.get("writers", []):
            qid = writer.get("qid")
            if qid:
                writer["rs_id"], _ = rs_id(qid)
    
    # Save
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, 'w') as f:
        json.dump({"artists": artists, "count": len(artists)}, f, indent=2)
    
    with open(catalog_path, 'w') as f:
        json.dump(catalog, f, indent=2)
    
    print(f"\n  {len(artists)} artists → {output_path}")
    print(f"  catalog updated with rs_id fields")

if __name__ == "__main__":
    main()
