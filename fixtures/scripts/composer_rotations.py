#!/usr/bin/env python3
"""Per-composer optimal prime rotation + style distance matrix.

For each composer: find the permutation of 15 SSP primes that maximizes
Cl(15) blade concentration across their corpus. Then compute Kendall tau
distances between composers' optimal rotations.
"""

import os, json, random
from collections import defaultdict
from itertools import combinations

DATA_DIR = "datasets/midi-classical-data"
SSP = [2,3,5,7,11,13,17,19,23,29,31,41,47,59,71]

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

def cl15_grade_with_perm(ivs, perm):
    """Compute blade grade using a specific prime permutation."""
    mv = {0: 1}
    for iv in ivs[:300]:
        idx = perm[abs(iv) % 15]
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

def concentration(grades):
    """Inverse variance — higher = more concentrated."""
    if not grades: return 0
    mean = sum(grades) / len(grades)
    var = sum((g - mean)**2 for g in grades) / len(grades)
    return 1.0 / (var + 0.1)

def kendall_tau(perm_a, perm_b):
    """Kendall tau distance between two permutations (normalized 0-1)."""
    n = len(perm_a)
    discordant = 0
    total = 0
    for i, j in combinations(range(n), 2):
        ai, aj = perm_a[i], perm_a[j]
        bi, bj = perm_b[i], perm_b[j]
        if (ai - aj) * (bi - bj) < 0:
            discordant += 1
        total += 1
    return discordant / total if total > 0 else 0

def optimize_rotation(all_ivs, n_trials=200):
    """Find best permutation via random search + local improvement."""
    identity = list(range(15))
    best_perm = identity[:]
    best_score = concentration([cl15_grade_with_perm(iv, identity) for iv in all_ivs[:50]])

    random.seed(42)
    for _ in range(n_trials):
        # Random swap from best
        perm = best_perm[:]
        i, j = random.sample(range(15), 2)
        perm[i], perm[j] = perm[j], perm[i]
        grades = [cl15_grade_with_perm(iv, perm) for iv in all_ivs[:50]]
        score = concentration(grades)
        if score > best_score:
            best_score = score
            best_perm = perm
    return best_perm, best_score

def main():
    jsons = sorted(f for f in os.listdir(DATA_DIR) if f.endswith('_processed.json'))
    
    # Load per-composer
    composer_ivs = defaultdict(list)
    for jf in jsons:
        notes = load_notes(os.path.join(DATA_DIR, jf))
        if len(notes) < 20: continue
        composer = jf.split('-')[0]
        composer_ivs[composer].append(intervals(notes))

    # Filter to composers with ≥50 pieces
    top = {c: ivs for c, ivs in composer_ivs.items() if len(ivs) >= 50}
    print(f"=== COMPOSER ROTATION SEARCH ({len(top)} composers, ≥50 pieces) ===\n")

    results = {}
    for composer, ivs in sorted(top.items(), key=lambda x: -len(x[1])):
        perm, score = optimize_rotation(ivs)
        # Compute grade stats with optimal perm
        grades = [cl15_grade_with_perm(iv, perm) for iv in ivs[:100]]
        mean_g = sum(grades) / len(grades) if grades else 0
        mode_g = max(set(grades), key=grades.count) if grades else 0
        
        # Also compute with identity for comparison
        id_grades = [cl15_grade_with_perm(iv, list(range(15))) for iv in ivs[:100]]
        id_score = concentration(id_grades)
        
        improvement = (score / id_score - 1) * 100 if id_score > 0 else 0
        
        primes_order = [SSP[p] for p in perm]
        results[composer] = {"perm": perm, "primes": primes_order, "score": score, "mean_grade": mean_g, "mode": mode_g, "n": len(ivs)}
        
        print(f"  {composer:<16} n={len(ivs):>4}  conc={score:.3f} (+{improvement:.0f}%)  mean={mean_g:.1f}  mode={mode_g}  primes={primes_order[:5]}...")

    # Style distance matrix
    composers = sorted(results.keys(), key=lambda c: -results[c]["n"])[:8]
    print(f"\n=== STYLE DISTANCE MATRIX (Kendall tau) ===\n")
    print(f"{'':>14}", end="")
    for c in composers:
        print(f"{c[:8]:>9}", end="")
    print()

    matrix = {}
    for a in composers:
        print(f"  {a:<12}", end="")
        for b in composers:
            if a == b:
                d = 0.0
            else:
                d = kendall_tau(results[a]["perm"], results[b]["perm"])
            matrix[(a,b)] = d
            print(f"{d:>9.2f}", end="")
        print()

    # Find closest/farthest pairs
    print(f"\n=== CLOSEST PAIRS ===")
    pairs = [(a, b, matrix[(a,b)]) for a in composers for b in composers if a < b]
    pairs.sort(key=lambda x: x[2])
    for a, b, d in pairs[:5]:
        print(f"  {a:<12} ↔ {b:<12} τ={d:.3f}")
    print(f"\n=== FARTHEST PAIRS ===")
    for a, b, d in pairs[-5:]:
        print(f"  {a:<12} ↔ {b:<12} τ={d:.3f}")

    # Save
    out = "fixtures/output/composer_rotations.json"
    with open(out, "w") as f:
        json.dump({"composers": results, "distances": {f"{a}:{b}": d for (a,b), d in matrix.items()}}, f, indent=2)
    print(f"\n→ {out}")

if __name__ == "__main__":
    main()
