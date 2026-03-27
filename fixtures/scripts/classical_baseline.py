#!/usr/bin/env python3
"""Random baseline + composer clustering for Cl(15) blade grades.

1. Random baseline: generate random interval sequences, compare grade distribution
2. Composer clustering: grade histogram per composer
"""

import os, json, random
from collections import defaultdict

DATA_DIR = "datasets/midi-classical-data"

def load_json_notes(path):
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
        notes.sort(key=lambda x: x[0])
        return [p for _, p in notes]
    except:
        return []

def intervals(notes):
    return [notes[i+1] - notes[i] for i in range(len(notes)-1)]

def cl15_grade(ivs):
    mv = {0: 1}
    for iv in ivs[:500]:
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
    if not mv: return 0
    return bin(max(mv.keys(), key=lambda k: abs(mv[k]))).count('1')

def main():
    jsons = sorted(f for f in os.listdir(DATA_DIR) if f.endswith('_processed.json'))

    # ── Collect real data ──
    print("=== LOADING REAL DATA ===\n")
    composer_grades = defaultdict(list)
    all_grades = []
    all_lengths = []

    for i, jf in enumerate(jsons):
        notes = load_json_notes(os.path.join(DATA_DIR, jf))
        if len(notes) < 10: continue
        ivs = intervals(notes)
        g = cl15_grade(ivs)
        composer = jf.split('-')[0] if '-' in jf else "unknown"
        composer_grades[composer].append(g)
        all_grades.append(g)
        all_lengths.append(len(ivs))
        if (i+1) % 1000 == 0: print(f"  {i+1}/{len(jsons)}...")

    print(f"  {len(all_grades)} pieces loaded\n")

    # ── 1. RANDOM BASELINE ──
    print("=== 1. RANDOM BASELINE ===\n")
    random.seed(42)
    rand_grades = []
    for length in all_lengths:
        # Random intervals: uniform from -12 to +12 (one octave)
        rivs = [random.randint(-12, 12) for _ in range(min(length, 500))]
        rand_grades.append(cl15_grade(rivs))

    # Compare distributions
    real_dist = defaultdict(int)
    rand_dist = defaultdict(int)
    for g in all_grades: real_dist[g] += 1
    for g in rand_grades: rand_dist[g] += 1

    print(f"  {'Grade':>6} {'Real':>6} {'Real%':>6} {'Random':>6} {'Rand%':>6} {'Δ':>6}")
    print(f"  {'-'*40}")
    for g in range(0, 16):
        r = real_dist.get(g, 0)
        d = rand_dist.get(g, 0)
        rp = r / len(all_grades) * 100 if all_grades else 0
        dp = d / len(rand_grades) * 100 if rand_grades else 0
        delta = rp - dp
        bar_r = "#" * int(rp)
        bar_d = "·" * int(dp)
        if r > 0 or d > 0:
            print(f"  {g:>6} {r:>6} {rp:>5.1f}% {d:>6} {dp:>5.1f}% {delta:>+5.1f}  {bar_r}{bar_d}")

    # KL divergence approximation
    real_mean = sum(all_grades) / len(all_grades) if all_grades else 0
    rand_mean = sum(rand_grades) / len(rand_grades) if rand_grades else 0
    real_var = sum((g - real_mean)**2 for g in all_grades) / len(all_grades) if all_grades else 0
    rand_var = sum((g - rand_mean)**2 for g in rand_grades) / len(rand_grades) if rand_grades else 0
    print(f"\n  Real: mean={real_mean:.2f} var={real_var:.2f}")
    print(f"  Rand: mean={rand_mean:.2f} var={rand_var:.2f}")
    print(f"  Δmean = {abs(real_mean - rand_mean):.2f}")
    if real_mean != rand_mean:
        print(f"  ✅ DIFFERENT — music is NOT random in Cl(15)")
    else:
        print(f"  ⚠ Similar — no clear signal")

    # ── 2. COMPOSER CLUSTERING ──
    print(f"\n=== 2. COMPOSER CLUSTERING ===\n")
    top = sorted(composer_grades.items(), key=lambda x: -len(x[1]))[:12]

    print(f"  {'Composer':<16} {'N':>5} {'Mean':>5} {'Mode':>5} {'Var':>5}  Grade distribution (4-12)")
    print(f"  {'-'*75}")
    for composer, grades in top:
        n = len(grades)
        mean = sum(grades) / n
        var = sum((g - mean)**2 for g in grades) / n
        mode = max(set(grades), key=grades.count)
        # Mini histogram for grades 4-12
        hist = defaultdict(int)
        for g in grades: hist[g] += 1
        bar = ""
        for g in range(4, 13):
            c = hist.get(g, 0)
            pct = c / n
            if pct > 0.3: bar += "█"
            elif pct > 0.15: bar += "▓"
            elif pct > 0.05: bar += "▒"
            elif pct > 0: bar += "░"
            else: bar += " "
        print(f"  {composer:<16} {n:>5} {mean:>5.1f} {mode:>5} {var:>5.1f}  |{bar}| 4..12")

    # ── 3. Composer separation test ──
    print(f"\n=== 3. COMPOSER SEPARATION ===\n")
    # Can we distinguish Bach from Mozart from Beethoven by grade alone?
    for a_name, a_grades in top[:5]:
        for b_name, b_grades in top[:5]:
            if a_name >= b_name: continue
            a_mean = sum(a_grades) / len(a_grades)
            b_mean = sum(b_grades) / len(b_grades)
            diff = abs(a_mean - b_mean)
            pooled_std = ((sum((g-a_mean)**2 for g in a_grades) + sum((g-b_mean)**2 for g in b_grades)) / (len(a_grades)+len(b_grades)))**0.5
            effect = diff / pooled_std if pooled_std > 0 else 0
            sig = "✅ separable" if effect > 0.3 else "⚠ overlap" if effect > 0.1 else "❌ same"
            print(f"  {a_name:<12} vs {b_name:<12}: Δmean={diff:.2f} effect={effect:.2f} {sig}")

    # Save
    out = "fixtures/output/classical_baseline.json"
    with open(out, "w") as f:
        json.dump({
            "real_mean": real_mean, "real_var": real_var,
            "rand_mean": rand_mean, "rand_var": rand_var,
            "composer_means": {c: sum(g)/len(g) for c, g in top},
        }, f, indent=2)
    print(f"\n→ {out}")

if __name__ == "__main__":
    main()
