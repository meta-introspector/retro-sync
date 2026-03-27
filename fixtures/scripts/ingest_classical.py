#!/usr/bin/env python3
"""Ingest MIDI classical dataset → interval histograms + Cl(15) blades + orbifold positions.

Reads all .mid files, extracts notes via JSON sidecar, computes:
- Interval histogram (15 bins)
- Cl(15,0,0) blade grade (fingerprint)
- Orbifold position (mod 71, 59, 47)
- Composer stats

Output: fixtures/output/classical_ingest.json
"""

import os, json, sys

DATA_DIR = "datasets/midi-classical-data"
OUT = "fixtures/output/classical_ingest.json"

def load_json_notes(path):
    """Load notes from processed JSON sidecar. Format: [{track_name, notes: [{note_number, start_time_ms, ...}]}]"""
    try:
        with open(path) as f:
            data = json.load(f)
        notes = []
        if isinstance(data, list):
            for track in data:
                if isinstance(track, dict) and "notes" in track:
                    for n in track["notes"]:
                        if isinstance(n, dict) and "note_number" in n:
                            notes.append((n.get("start_time_ms", len(notes)), int(n["note_number"])))
        # Sort by time, return just pitches
        notes.sort(key=lambda x: x[0])
        return [p for _, p in notes]
    except:
        return []

def load_json_notes_from_value(val):
    if isinstance(val, list):
        notes = []
        for item in val:
            if isinstance(item, (int, float)):
                notes.append(int(item))
            elif isinstance(item, dict):
                p = item.get("pitch", item.get("note", item.get("midi", None)))
                if p is not None: notes.append(int(p))
            elif isinstance(item, list):
                # Nested tracks
                notes.extend(load_json_notes_from_value(item))
        return notes
    return []

def intervals(notes):
    return [notes[i+1] - notes[i] for i in range(len(notes)-1)]

def histogram(ivs):
    h = [0] * 15
    for iv in ivs:
        h[abs(iv) % 15] += 1
    return h

def cl15_grade(ivs):
    mv = {0: 1}
    for iv in ivs[:500]:  # cap at 500 for speed
        idx = abs(iv) % 15
        blade = 1 << idx
        new = {}
        for mask, coeff in mv.items():
            result = mask ^ blade
            sign = 1
            for bit in range(idx):
                if mask & (1 << bit): sign *= -1
            new[result] = new.get(result, 0) + coeff * sign
        mv = {k: v for k, v in new.items() if v != 0}
    if not mv: return 0, 0
    top_mask = max(mv.keys(), key=lambda k: abs(mv[k]))
    return bin(top_mask).count('1'), len(mv)

def orbifold(ivs):
    state = 1
    SSP = [2,3,5,7,11,13,17,19,23,29,31,41,47,59,71]
    for iv in ivs[:200]:
        p = SSP[abs(iv) % 15]
        state = (state * p) % (71 * 59 * 47)
    return (state % 71, state % 59, state % 47)

def main():
    if not os.path.isdir(DATA_DIR):
        print(f"ERROR: {DATA_DIR} not found"); return

    jsons = sorted(f for f in os.listdir(DATA_DIR) if f.endswith('_processed.json'))
    print(f"=== INGEST: {len(jsons)} JSON files from midi-classical ===\n")

    results = []
    composers = {}
    grades = {}
    total = 0
    skipped = 0

    for i, jf in enumerate(jsons):
        path = os.path.join(DATA_DIR, jf)
        notes = load_json_notes(path)
        if len(notes) < 10:
            skipped += 1
            continue

        name = jf.replace('_processed.json', '')
        composer = name.split('-')[0] if '-' in name else name
        ivs = intervals(notes)
        hist = histogram(ivs)
        grade, n_terms = cl15_grade(ivs)
        orb = orbifold(ivs)

        entry = {
            "name": name,
            "composer": composer,
            "notes": len(notes),
            "intervals": len(ivs),
            "blade_grade": grade,
            "blade_terms": n_terms,
            "orbifold": orb,
            "histogram": hist,
        }
        results.append(entry)

        composers[composer] = composers.get(composer, 0) + 1
        grades[grade] = grades.get(grade, 0) + 1
        total += 1

        if (i + 1) % 500 == 0:
            print(f"  processed {i+1}/{len(jsons)}...")

    print(f"\n  Total: {total} pieces, {skipped} skipped\n")

    # Composer stats
    top_composers = sorted(composers.items(), key=lambda x: -x[1])[:15]
    print("Top composers:")
    for c, n in top_composers:
        print(f"  {c:<20} {n:>4} pieces")

    # Grade distribution
    print(f"\nCl(15) blade grade distribution:")
    for g in sorted(grades.keys()):
        bar = "#" * (grades[g] // 5)
        print(f"  grade {g:>2}: {grades[g]:>4} pieces {bar}")

    # Orbifold coverage
    positions = set(r["orbifold"] for r in results)
    print(f"\nOrbifold coverage: {len(positions)} distinct positions in Z/71×Z/59×Z/47")
    max_possible = 71 * 59 * 47
    print(f"  ({len(positions)}/{max_possible} = {len(positions)/max_possible*100:.2f}%)")

    # Save
    with open(OUT, "w") as f:
        json.dump({
            "total": total,
            "skipped": skipped,
            "composers": dict(top_composers),
            "grade_distribution": grades,
            "orbifold_coverage": len(positions),
            "pieces": results[:100],  # first 100 for size
        }, f, indent=2)
    print(f"\n→ {OUT}")

if __name__ == "__main__":
    main()
