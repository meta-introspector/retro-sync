#!/usr/bin/env python3
"""export-cwr.py — Generate CWR 2.2 from catalog/works.json.

Produces a CWR file compatible with CISAC collection societies.
Uses the same format as retro-sync's royalty_reporting.rs generate_cwr().

Usage: python3 scripts/export-cwr.py [catalog/works.json] [output.cwr]
"""

import json, sys, os
from datetime import datetime

INPUT = sys.argv[1] if len(sys.argv) > 1 else "catalog/works.json"
OUTPUT = sys.argv[2] if len(sys.argv) > 2 else "catalog/retro-sync.cwr"

def pad(s, n):
    return str(s)[:n].ljust(n)

def cwr_header(sender_id, n_works):
    ts = datetime.utcnow()
    return (
        f"HDRPB{pad(sender_id, 9)}{pad('RETRO-SYNC', 45)}"
        f"01.10{pad('', 15)}{ts.strftime('%Y%m%d')}{ts.strftime('%H%M%S')}"
        f"{ts.strftime('%Y%m%d')}               \n"
    )

def cwr_trailer(n_groups, n_transactions, n_records):
    return f"TRL{n_groups:08d}{n_transactions:08d}{n_records:08d}\n"

def cwr_work(work, seq):
    """Generate CWR NWR (New Work Registration) record."""
    title = pad(work.get("title", ""), 60)
    iswc = pad(work.get("iswc", "") or "", 11)
    lang = pad("EN", 2)
    
    # NWR record
    nwr = f"NWR{seq:08d}00000000{title}{lang}  ORI   MTX                    ORI  N  U N\n"
    
    records = [nwr]
    
    # SPU (Publisher) records — we have no publisher for PD works
    # SWR (Writer) records
    for i, writer in enumerate(work.get("writers", [])):
        name = writer.get("name", "")
        parts = name.split(" ", 1)
        last = pad(parts[-1] if parts else "", 45)
        first = pad(parts[0] if len(parts) > 1 else "", 30)
        ipi = pad(writer.get("ipi", "") or "", 11)
        role = pad(writer.get("role", "C"), 2)
        share = f"{int(writer.get('share', 100) * 100):05d}"
        
        swr = f"SWR{seq:08d}{i:02d}{last}{first}{role}{ipi}  {share}{share}00000I0008\n"
        records.append(swr)
    
    # REC (Recording) record
    rec = work.get("recording", {})
    if rec:
        rec_title = pad(rec.get("title", work.get("title", "")), 60)
        isrc = pad(rec.get("isrc", "") or "", 12)
        records.append(f"REC{seq:08d}{rec_title}{isrc}                              \n")
    
    return records

def main():
    catalog = json.load(open(INPUT))
    works = catalog.get("works", [])
    sender_id = catalog.get("sender_id", "RETROSYNC")[:9]
    
    print(f"=== CWR 2.2 EXPORT ===")
    print(f"  Works: {len(works)}")
    print(f"  Sender: {sender_id}")
    
    lines = []
    lines.append(cwr_header(sender_id, len(works)))
    
    # Group header
    lines.append(f"GRHNWR{1:05d}0001{len(works):08d}\n")
    
    n_records = 2  # HDR + GRH
    for i, work in enumerate(works):
        recs = cwr_work(work, i + 1)
        lines.extend(recs)
        n_records += len(recs)
    
    # Group trailer
    lines.append(f"GRT{1:05d}{len(works):08d}{n_records:08d}\n")
    n_records += 1
    
    # File trailer
    lines.append(cwr_trailer(1, len(works), n_records + 1))
    
    os.makedirs(os.path.dirname(OUTPUT), exist_ok=True)
    with open(OUTPUT, 'w') as f:
        f.writelines(lines)
    
    print(f"  Output: {OUTPUT} ({os.path.getsize(OUTPUT)} bytes)")
    print(f"  Records: {n_records}")
    print(f"\n  Preview:")
    for line in lines[:5]:
        print(f"    {line.rstrip()}")
    print(f"    ...")

if __name__ == "__main__":
    main()
