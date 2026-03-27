#!/usr/bin/env python3
"""Group pieces by era (from filename heuristics) and note count, not composer.

Composers change styles. The real axes are:
1. Historical era (approximate from composer birth dates)
2. Complexity (note count as proxy)
3. Blade grade + concentration
"""

import os, json
from collections import defaultdict

DATA_DIR = "datasets/midi-classical-data"

# Approximate composer birth years for era grouping
COMPOSER_YEARS = {
    "bach": 1685, "haendel": 1685, "vivaldi": 1678, "scarlatti": 1685,
    "haydn": 1732, "clementi": 1752, "mozart": 1756,
    "beethoven": 1770, "schubert": 1797, "mendelsonn": 1809,
    "chopin": 1810, "schumann": 1810, "liszt": 1811, "brahms": 1833,
    "tchaikovsky": 1840, "grieg": 1843, "debussy": 1862, "satie": 1866,
    "ravel": 1875, "bartok": 1881, "stravinsky": 1882,
    "burgmuller": 1806, "albeniz": 1860, "granados": 1867,
    "rachmaninoff": 1873, "scriabin": 1872, "prokofiev": 1891,
    "buxehude": 1637, "couperin": 1668, "rameau": 1683,
}

ERAS = [
    ("Baroque", 1600, 1750),
    ("Classical", 1750, 1820),
    ("Early Romantic", 1820, 1860),
    ("Late Romantic", 1860, 1910),
    ("Modern", 1910, 2000),
]

NOTE_BINS = [
    ("tiny", 0, 50),
    ("short", 50, 200),
    ("medium", 200, 500),
    ("long", 500, 1500),
    ("epic", 1500, 99999),
]

def load_notes(path):
    try:
        with open(path) as f:
            data = json.load(f)
        notes = []
        for track in data if isinstance(data, list) else []:
            if isinstance(track, dict) and "notes" in track:
                for n in track["notes"]:
                    if isinstance(n, dict) and "note_number" in n:
                        notes.append((n.get("start_time_ms", len(notes)), int(n["note_number"])))
        notes.sort()
        return [p for _, p in notes]
    except:
        return []

def intervals(notes):
    return [notes[i+1] - notes[i] for i in range(len(notes)-1)]

def cl15_grade(ivs):
    mv = {0: 1}
    for iv in ivs[:300]:
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
    top = max(mv.keys(), key=lambda k: abs(mv[k]))
    return bin(top).count('1'), top

def get_era(composer):
    year = COMPOSER_YEARS.get(composer, 1800)
    for name, start, end in ERAS:
        if start <= year < end:
            return name
    return "Unknown"

def get_note_bin(n):
    for name, lo, hi in NOTE_BINS:
        if lo <= n < hi:
            return name
    return "epic"

def concentration(grades):
    if not grades: return 0
    mean = sum(grades) / len(grades)
    var = sum((g - mean)**2 for g in grades) / len(grades)
    return 1.0 / (var + 0.1)

def main():
    jsons = sorted(f for f in os.listdir(DATA_DIR) if f.endswith('_processed.json'))
    
    era_data = defaultdict(list)      # era → [(grade, mask, n_notes)]
    bin_data = defaultdict(list)      # note_bin → [(grade, mask)]
    era_bin_data = defaultdict(list)  # (era, bin) → [grade]
    
    print("=== GROUPING BY ERA + NOTE COUNT ===\n")
    
    for i, jf in enumerate(jsons):
        notes = load_notes(os.path.join(DATA_DIR, jf))
        if len(notes) < 10: continue
        composer = jf.split('-')[0]
        era = get_era(composer)
        nbin = get_note_bin(len(notes))
        ivs = intervals(notes)
        grade, mask = cl15_grade(ivs)
        
        era_data[era].append((grade, mask, len(notes)))
        bin_data[nbin].append((grade, mask))
        era_bin_data[(era, nbin)].append(grade)
        
        if (i+1) % 1000 == 0: print(f"  {i+1}/{len(jsons)}...")

    # 1. By era
    print(f"\n=== BY ERA ===\n")
    print(f"  {'Era':<18} {'N':>5} {'Mean':>5} {'Mode':>5} {'Conc':>6} {'Masks':>6}")
    for era_name, _, _ in ERAS:
        pieces = era_data.get(era_name, [])
        if not pieces: continue
        grades = [g for g, _, _ in pieces]
        masks = set(m for _, m, _ in pieces)
        mean = sum(grades) / len(grades)
        mode = max(set(grades), key=grades.count)
        conc = concentration(grades)
        print(f"  {era_name:<18} {len(pieces):>5} {mean:>5.1f} {mode:>5} {conc:>6.3f} {len(masks):>6}")

    # 2. By note count
    print(f"\n=== BY NOTE COUNT ===\n")
    print(f"  {'Bin':<10} {'N':>5} {'Mean':>5} {'Mode':>5} {'Conc':>6}")
    for bin_name, _, _ in NOTE_BINS:
        pieces = bin_data.get(bin_name, [])
        if not pieces: continue
        grades = [g for g, _ in pieces]
        mean = sum(grades) / len(grades)
        mode = max(set(grades), key=grades.count) if grades else 0
        conc = concentration(grades)
        print(f"  {bin_name:<10} {len(pieces):>5} {mean:>5.1f} {mode:>5} {conc:>6.3f}")

    # 3. Era × note count heatmap
    print(f"\n=== ERA × NOTE COUNT (mean grade) ===\n")
    print(f"  {'':>18}", end="")
    for bn, _, _ in NOTE_BINS:
        print(f"{bn:>8}", end="")
    print()
    for era_name, _, _ in ERAS:
        print(f"  {era_name:<18}", end="")
        for bn, _, _ in NOTE_BINS:
            grades = era_bin_data.get((era_name, bn), [])
            if grades:
                mean = sum(grades) / len(grades)
                print(f"{mean:>7.1f}n={len(grades):<3}", end="") if len(grades) >= 5 else print(f"{'':>8}", end="")
            else:
                print(f"{'':>8}", end="")
        print()

    # 4. Blade mask diversity per era
    print(f"\n=== BLADE MASK DIVERSITY ===\n")
    for era_name, _, _ in ERAS:
        pieces = era_data.get(era_name, [])
        if not pieces: continue
        masks = [m for _, m, _ in pieces]
        unique = len(set(masks))
        total = len(masks)
        print(f"  {era_name:<18} {unique:>4} unique masks / {total:>4} pieces = {unique/total:.2f} diversity")

    out = "fixtures/output/era_analysis.json"
    with open(out, "w") as f:
        json.dump({
            "eras": {e: {"n": len(d), "mean_grade": sum(g for g,_,_ in d)/len(d)} for e, d in era_data.items() if d},
            "bins": {b: {"n": len(d), "mean_grade": sum(g for g,_ in d)/len(d)} for b, d in bin_data.items() if d},
        }, f, indent=2)
    print(f"\n→ {out}")

if __name__ == "__main__":
    main()
