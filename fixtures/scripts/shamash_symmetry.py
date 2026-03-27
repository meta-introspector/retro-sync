#!/usr/bin/env python3
"""Deep symmetry analysis of Shamash SVG → FRACTRAN state → orbifold balance.

1. Extract all bezier control points from the 5 paths
2. Find rotational + reflective symmetries by angle clustering
3. Map each symmetry group to a FRACTRAN prime register
4. Compute orbifold position and find optimal balance
"""

import re, math, json
from collections import Counter

SHAMASH = "/var/www/solana.solfunmeme.com/retro-sync/scratch/shamash_star.svg"
SCRATCH = "/var/www/solana.solfunmeme.com/retro-sync/scratch"
SSP = [2,3,5,7,11,13,17,19,23,29,31,41,47,59,71]

def parse_all_coords(svg):
    """Extract all coordinate pairs from all paths."""
    all_pts = []
    for m in re.finditer(r'\bd="(M[^"]+)"', svg):
        d = m.group(1)
        nums = [float(x) for x in re.findall(r'[-]?\d+\.?\d*', d)]
        for i in range(0, len(nums)-1, 2):
            all_pts.append((nums[i], nums[i+1]))
    return all_pts

def find_center_of_mass(pts):
    if not pts: return 0, 0
    return sum(x for x,y in pts)/len(pts), sum(y for x,y in pts)/len(pts)

def to_polar(pts, cx, cy):
    """Convert to polar (r, θ) relative to center."""
    polar = []
    for x, y in pts:
        dx, dy = x - cx, y - cy
        r = math.sqrt(dx*dx + dy*dy)
        theta = math.atan2(dy, dx)
        polar.append((r, theta))
    return polar

def detect_n_fold(polar, max_n=24):
    """Detect N-fold rotational symmetry by angle histogram."""
    results = {}
    for n in range(2, max_n+1):
        # Bin angles into n sectors
        bins = [0] * n
        for r, theta in polar:
            if r < 5: continue  # skip center
            sector = int(((theta + math.pi) / (2 * math.pi)) * n) % n
            bins[sector] += 1
        # Symmetry score: how uniform are the bins?
        mean = sum(bins) / n
        if mean == 0: continue
        variance = sum((b - mean)**2 for b in bins) / n
        cv = math.sqrt(variance) / mean  # coefficient of variation
        uniformity = 1.0 / (cv + 0.01)
        results[n] = {"bins": bins, "uniformity": round(uniformity, 2), "cv": round(cv, 4)}
    return results

def detect_reflection(polar):
    """Detect reflection symmetry axes."""
    axes = []
    for angle in [0, math.pi/4, math.pi/2, 3*math.pi/4]:
        # Reflect all points across this axis and check overlap
        reflected = []
        for r, theta in polar:
            new_theta = 2 * angle - theta
            reflected.append((r, new_theta))
        # Score: how many reflected points are close to original points
        matches = 0
        for r1, t1 in reflected:
            for r2, t2 in polar:
                if abs(r1-r2) < 10 and abs(t1-t2) < 0.1:
                    matches += 1
                    break
        axes.append({"angle_deg": round(math.degrees(angle)), "matches": matches})
    return axes

def radial_histogram(polar, n_bins=15):
    """Bin points by radius → maps to SSP primes."""
    if not polar: return [0]*n_bins
    max_r = max(r for r, _ in polar)
    bins = [0] * n_bins
    for r, _ in polar:
        idx = min(n_bins-1, int(r / max(max_r, 1) * n_bins))
        bins[idx] += 1
    return bins

def to_fractran_state(radial_bins):
    """Map radial histogram to FRACTRAN state: Π p_i^(bin_i)."""
    state = 1
    parts = []
    for i, count in enumerate(radial_bins):
        if count > 0 and i < len(SSP):
            exp = min(count, 8)  # cap exponent
            state *= SSP[i] ** exp
            parts.append(f"{SSP[i]}^{exp}")
    return state, parts

def orbifold(state):
    return (state % 71, state % 59, state % 47)

def cl15_blade(radial_bins):
    """Compute Cl(15) blade from radial histogram."""
    mv = {0: 1}
    for i, count in enumerate(radial_bins):
        if count == 0 or i >= 15: continue
        blade = 1 << i
        for _ in range(count % 2):
            new = {}
            for mask, coeff in mv.items():
                result = mask ^ blade
                sign = 1
                for bit in range(i):
                    if mask & (1 << bit): sign *= -1
                new[result] = new.get(result, 0) + coeff * sign
            mv = {k: v for k, v in new.items() if v != 0}
    if not mv: return 0, 0
    top = max(mv.keys(), key=lambda k: abs(mv[k]))
    return top, bin(top).count('1')

def main():
    svg = open(SHAMASH).read()
    pts = parse_all_coords(svg)
    cx, cy = find_center_of_mass(pts)
    polar = to_polar(pts, cx, cy)
    
    print(f"=== SHAMASH DEEP SYMMETRY ANALYSIS ===\n")
    print(f"Points: {len(pts)}, Center: ({cx:.1f}, {cy:.1f})\n")
    
    # 1. N-fold rotational symmetry
    print("1. ROTATIONAL SYMMETRY\n")
    sym = detect_n_fold(polar)
    ranked = sorted(sym.items(), key=lambda x: -x[1]["uniformity"])
    print(f"  {'N':>3} {'Uniformity':>10} {'CV':>8}  Bin distribution")
    for n, info in ranked[:10]:
        bar = "".join("█" if b > sum(info["bins"])/n*1.2 else "▒" if b > sum(info["bins"])/n*0.8 else "░" for b in info["bins"][:min(n,20)])
        print(f"  {n:>3} {info['uniformity']:>10.1f} {info['cv']:>8.4f}  {bar}")
    
    best_n = ranked[0][0]
    print(f"\n  Best symmetry: {best_n}-fold (uniformity={ranked[0][1]['uniformity']:.1f})")
    
    # 2. Reflection symmetry
    print(f"\n2. REFLECTION SYMMETRY\n")
    refl = detect_reflection(polar)
    for ax in refl:
        bar = "#" * (ax["matches"] // 10)
        print(f"  axis {ax['angle_deg']:>3}°: {ax['matches']:>4} matches  {bar}")
    
    # 3. Radial histogram → SSP primes
    print(f"\n3. RADIAL HISTOGRAM → SSP PRIMES\n")
    radial = radial_histogram(polar)
    for i, count in enumerate(radial):
        bar = "█" * (count // 20)
        print(f"  r-bin {i:>2} (p={SSP[i]:>2}): {count:>4} pts  {bar}")
    
    # 4. FRACTRAN state
    state, parts = to_fractran_state(radial)
    orb = orbifold(state)
    blade_mask, blade_grade = cl15_blade(radial)
    
    print(f"\n4. FRACTRAN STATE\n")
    print(f"  State = {' × '.join(parts)}")
    print(f"  Orbifold: ({orb[0]}, {orb[1]}, {orb[2]}) mod (71, 59, 47)")
    print(f"  Cl(15) blade: 0x{blade_mask:04x} grade={blade_grade}")
    
    # 5. Optimal balance: which symmetry order minimizes orbifold distance to (0,0,0)?
    print(f"\n5. OPTIMAL BALANCE (orbifold distance to origin)\n")
    for n, info in ranked[:8]:
        # Recompute with n-fold binning
        n_bins = min(n, 15)
        rbins = radial_histogram(polar, n_bins)
        rbins_padded = rbins + [0] * (15 - len(rbins))
        s, _ = to_fractran_state(rbins_padded)
        o = orbifold(s)
        dist = o[0] + o[1] + o[2]  # L1 distance to origin
        _, grade = cl15_blade(rbins_padded)
        print(f"  {n:>2}-fold: orb=({o[0]:>2},{o[1]:>2},{o[2]:>2}) dist={dist:>3} grade={grade:>2} uniformity={info['uniformity']:.1f}")
    
    # Save
    result = {
        "center": [round(cx,1), round(cy,1)],
        "n_points": len(pts),
        "best_symmetry": best_n,
        "radial_histogram": radial,
        "fractran_parts": parts,
        "orbifold": orb,
        "blade_mask": blade_mask,
        "blade_grade": blade_grade,
        "symmetry_scores": {str(n): info["uniformity"] for n, info in ranked[:10]},
    }
    out = f"{SCRATCH}/shamash_symmetry.json"
    with open(out, 'w') as f:
        json.dump(result, f, indent=2)
    print(f"\n→ {out}")

if __name__ == "__main__":
    main()
