#!/usr/bin/env python3
"""SVD rank test + Clifford blade stability for Hurrian hymn versions.

1. SVD: build interval feature matrix across all versions, check singular value collapse
2. Stability: perturb each version (transpose, permute, noise) and check if blade grade holds
3. Basis invariance: reconstruction error across bases
"""

import os, re, json, random
from math import log2, sqrt

LY_DIR = "fixtures/lilypond"
SSP = [2,3,5,7,11,13,17,19,23,29,31,41,47,59,71]

def parse_notes(path):
    text = open(path).read()
    pat = re.compile(r"([a-g](?:is|es)?)('+|,+)?(\d*)")
    pitch = {'c':0,'d':2,'e':4,'f':5,'g':7,'a':9,'b':11}
    notes = []
    for m in pat.finditer(text):
        name, omod, _ = m.groups()
        b = name[0]
        if b not in pitch: continue
        midi = 60 + pitch[b]
        if 'is' in name: midi += 1
        if 'es' in name: midi -= 1
        if omod:
            if "'" in omod: midi += 12 * len(omod)
            if "," in omod: midi -= 12 * len(omod)
        notes.append(midi)
    return notes

def intervals(notes):
    return [notes[i+1] - notes[i] for i in range(len(notes)-1)]

def interval_histogram(ivs):
    """15-bin histogram: count of each interval mod 15."""
    h = [0] * 15
    for iv in ivs:
        h[abs(iv) % 15] += 1
    return h

def cl15_blade(ivs):
    mv = {0: 1}
    for iv in ivs:
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
    return mv

def blade_grade(mv):
    if not mv: return 0
    mask = max(mv.keys(), key=lambda k: abs(mv[k]))
    return bin(mask).count('1')

def svd_manual(matrix):
    """Simple SVD via eigendecomposition of M^T M (no numpy needed)."""
    n = len(matrix)
    m = len(matrix[0]) if n > 0 else 0
    # Compute M^T M
    mtm = [[0.0]*m for _ in range(m)]
    for i in range(m):
        for j in range(m):
            for k in range(n):
                mtm[i][j] += matrix[k][i] * matrix[k][j]
    # Power iteration for top singular values
    sigmas = []
    for _ in range(min(m, 6)):
        v = [1.0/sqrt(m)] * m
        for _ in range(100):
            w = [sum(mtm[i][j]*v[j] for j in range(m)) for i in range(m)]
            norm = sqrt(sum(x*x for x in w))
            if norm < 1e-10: break
            v = [x/norm for x in w]
        sigma = sqrt(sum(mtm[i][j]*v[i]*v[j] for i in range(m) for j in range(m)))
        sigmas.append(sigma)
        # Deflate
        for i in range(m):
            for j in range(m):
                mtm[i][j] -= sigma*sigma * v[i] * v[j]
    return sigmas

def main():
    files = sorted(f for f in os.listdir(LY_DIR) if f.endswith('.ly'))
    
    print("=== SVD RANK TEST + CLIFFORD STABILITY ===\n")
    
    # ── 1. Build feature matrix ──
    print("1. INTERVAL FEATURE MATRIX (15-bin histograms)\n")
    matrix = []
    names = []
    all_ivs = {}
    
    for f in files:
        notes = parse_notes(os.path.join(LY_DIR, f))
        if len(notes) < 3: continue
        ivs = intervals(notes)
        name = f.replace('.ly', '')
        hist = interval_histogram(ivs)
        # Normalize
        total = sum(hist) or 1
        row = [h / total for h in hist]
        matrix.append(row)
        names.append(name)
        all_ivs[name] = ivs
        print(f"  {name:14} {len(notes):5} notes  hist={[round(x,2) for x in row[:8]]}...")
    
    # ── 2. SVD ──
    print(f"\n2. SVD OF {len(matrix)}×15 FEATURE MATRIX\n")
    sigmas = svd_manual(matrix)
    total_var = sum(s*s for s in sigmas)
    cumulative = 0
    print(f"  {'σ':>4}  {'value':>10}  {'% var':>7}  {'cumul':>7}  bar")
    for i, s in enumerate(sigmas):
        var = s*s / total_var * 100 if total_var > 0 else 0
        cumulative += var
        bar = "#" * int(var)
        print(f"  σ{i+1}  {s:>10.4f}  {var:>6.1f}%  {cumulative:>6.1f}%  {bar}")
    
    # Effective rank
    threshold = 0.99
    eff_rank = 0
    cum = 0
    for s in sigmas:
        cum += s*s / total_var if total_var > 0 else 0
        eff_rank += 1
        if cum >= threshold: break
    print(f"\n  Effective rank (99% variance): {eff_rank} of {len(sigmas)}")
    
    # ── 3. Clifford blade stability ──
    print(f"\n3. CLIFFORD BLADE STABILITY\n")
    print(f"  {'Version':<14} {'Original':>8} {'Transp':>8} {'Permute':>8} {'Noise':>8} {'Stable':>7}")
    
    stable_count = 0
    for name, ivs in all_ivs.items():
        g_orig = blade_grade(cl15_blade(ivs))
        
        # Transpose: shift all intervals by +2
        g_trans = blade_grade(cl15_blade([iv + 2 for iv in ivs]))
        
        # Permute: shuffle order
        random.seed(42)
        shuffled = ivs[:]
        random.shuffle(shuffled)
        g_perm = blade_grade(cl15_blade(shuffled))
        
        # Noise: flip sign of 10% of intervals
        random.seed(42)
        noisy = [iv if random.random() > 0.1 else -iv for iv in ivs]
        g_noise = blade_grade(cl15_blade(noisy))
        
        stable = g_orig == g_trans == g_perm == g_noise
        stable_count += stable
        mark = "✅" if stable else "⚠"
        print(f"  {name:<14} {g_orig:>8} {g_trans:>8} {g_perm:>8} {g_noise:>8} {mark:>7}")
    
    print(f"\n  Stable: {stable_count}/{len(all_ivs)} versions")
    
    # ── 4. Blade uniqueness ──
    print(f"\n4. BLADE UNIQUENESS (do different versions produce different blades?)\n")
    blades = {}
    for name, ivs in all_ivs.items():
        mv = cl15_blade(ivs)
        key = tuple(sorted(mv.items()))
        if key not in blades:
            blades[key] = []
        blades[key].append(name)
    
    print(f"  {len(blades)} distinct blades from {len(all_ivs)} versions:")
    for key, versions in blades.items():
        mv = dict(key)
        g = blade_grade(mv)
        masks = [f"0x{m:04x}" for m in mv.keys()]
        print(f"    grade {g:2}, masks={masks}: {', '.join(versions)}")
    
    # ── 5. Basis invariance ──
    print(f"\n5. BASIS INVARIANCE (reconstruction error)\n")
    bases = {
        "j": [1, 744, 196884, 21493760, 864299970, 20245856256],
        "τ": [1, 24, 252, 1472, 4830, 6048],
        "E8": [1, 240, 2160, 6720, 17520, 30240],
        "factorial": [1, 2, 6, 24, 120, 720],
    }
    
    for name, ivs in list(all_ivs.items())[:3]:
        print(f"  {name}:")
        for bname, bvals in bases.items():
            # Project
            weights = [0.0] * 6
            for i, iv in enumerate(ivs):
                idx = abs(iv) % 6
                weights[idx] += 1.0 / (1 + i * 0.05)
            total = sum(weights) or 1
            weights = [w / total for w in weights]
            f_val = sum(w * b for w, b in zip(weights, bvals))
            # "Error" = how much info is in the residual (weights entropy)
            ent = -sum(w * log2(w) if w > 0 else 0 for w in weights)
            print(f"    {bname:<10} F={f_val:>14.1f}  entropy={ent:.4f}")
        print()
    
    # Save
    out = "fixtures/output/hymn_stability.json"
    with open(out, "w") as f:
        json.dump({
            "singular_values": sigmas,
            "effective_rank": eff_rank,
            "blade_grades": {n: blade_grade(cl15_blade(iv)) for n, iv in all_ivs.items()},
            "n_distinct_blades": len(blades),
            "stable_count": stable_count,
        }, f, indent=2)
    print(f"→ {out}")

if __name__ == "__main__":
    main()
