#!/usr/bin/env python3
"""Fit logarithmic spiral parameters to the actual Shamash SVG paths.

1. Extract all path coordinates from the original Shamash
2. For each path, fit r = a*e^(b*θ) by least squares
3. Extract taper, symmetry order, color
4. These fitted params ARE the constraints — the origin
5. Rank how well each path fits the spiral model
"""

import re, math, json

SCRATCH = "/var/www/solana.solfunmeme.com/retro-sync/scratch"
SHAMASH = f"{SCRATCH}/shamash_star.svg"

def extract_path_coords(svg):
    """Extract coordinate pairs from each path d= attribute (starting with M)."""
    paths = []
    for m in re.finditer(r'\bd="(M[^"]*)"', svg):
        d = m.group(1)
        nums = [float(x) for x in re.findall(r'[-]?\d+\.?\d*', d)]
        if len(nums) >= 4:
            pts = [(nums[i], nums[i+1]) for i in range(0, len(nums)-1, 2)]
            paths.append(pts)
    return paths

def find_center(all_paths):
    """Estimate center from all coordinates."""
    xs, ys = [], []
    for pts in all_paths:
        for x, y in pts:
            xs.append(x)
            ys.append(y)
    return sum(xs)/len(xs), sum(ys)/len(ys)

def fit_spiral(pts, cx, cy):
    """Fit r = a*e^(b*θ) to a set of points relative to center (cx,cy).
    
    Returns: a, b, residual, theta_range, r_range
    """
    if len(pts) < 3:
        return 0, 0, float('inf'), 0, 0
    
    # Convert to polar
    polar = []
    for x, y in pts:
        dx, dy = x - cx, y - cy
        r = math.sqrt(dx*dx + dy*dy)
        theta = math.atan2(dy, dx)
        if r > 1:  # skip center points
            polar.append((theta, r))
    
    if len(polar) < 3:
        return 0, 0, float('inf'), 0, 0
    
    # Unwrap theta (handle discontinuity at ±π)
    polar.sort(key=lambda p: p[0])
    thetas = [p[0] for p in polar]
    rs = [p[1] for p in polar]
    
    # Fit log(r) = log(a) + b*θ via linear regression
    log_rs = [math.log(max(r, 0.1)) for r in rs]
    n = len(thetas)
    sum_t = sum(thetas)
    sum_lr = sum(log_rs)
    sum_t2 = sum(t*t for t in thetas)
    sum_tlr = sum(t*lr for t, lr in zip(thetas, log_rs))
    
    denom = n * sum_t2 - sum_t * sum_t
    if abs(denom) < 1e-10:
        return 0, 0, float('inf'), 0, 0
    
    b = (n * sum_tlr - sum_t * sum_lr) / denom
    log_a = (sum_lr - b * sum_t) / n
    a = math.exp(log_a)
    
    # Residual
    residual = 0
    for theta, r in zip(thetas, rs):
        r_pred = a * math.exp(b * theta)
        residual += (r - r_pred) ** 2
    residual = math.sqrt(residual / n)
    
    theta_range = max(thetas) - min(thetas)
    r_range = max(rs) - min(rs)
    
    return a, b, residual, theta_range, r_range

def fit_taper(pts, cx, cy):
    """Estimate taper: how does distance-from-center relate to "width"?
    
    Approximate width by distance between consecutive points perpendicular to radial.
    """
    if len(pts) < 4:
        return 0, 0
    
    # Compute distances from center
    dists = [math.sqrt((x-cx)**2 + (y-cy)**2) for x, y in pts]
    
    # Compute local "spread" (distance between consecutive points)
    spreads = []
    for i in range(len(pts)-1):
        dx = pts[i+1][0] - pts[i][0]
        dy = pts[i+1][1] - pts[i][1]
        spreads.append(math.sqrt(dx*dx + dy*dy))
    
    if not spreads:
        return 0, 0
    
    R0 = max(spreads[:len(spreads)//4]) if spreads else 0  # base width
    Rtip = min(spreads[-len(spreads)//4:]) if spreads else 0  # tip width
    
    return R0, Rtip

def main():
    svg = open(SHAMASH).read()
    paths = extract_path_coords(svg)
    
    print(f"=== FIT SPIRAL PARAMS TO ORIGINAL SHAMASH ===\n")
    print(f"Paths found: {len(paths)}")
    
    cx, cy = find_center(paths)
    print(f"Estimated center: ({cx:.1f}, {cy:.1f})\n")
    
    results = []
    
    print(f"{'Path':>5} {'Pts':>5} {'a':>8} {'b':>8} {'Resid':>8} {'θ range':>8} {'r range':>8} {'R0':>6} {'Rtip':>6} {'Fit':>6}")
    print("-" * 80)
    
    for i, pts in enumerate(paths):
        a, b, resid, theta_range, r_range = fit_spiral(pts, cx, cy)
        R0, Rtip = fit_taper(pts, cx, cy)
        
        # Quality: lower residual relative to r_range = better fit
        fit_quality = 1.0 / (resid / max(r_range, 1) + 0.01) if r_range > 0 else 0
        
        entry = {
            "path_idx": i,
            "n_points": len(pts),
            "a": round(a, 2),
            "b": round(b, 4),
            "residual": round(resid, 2),
            "theta_range": round(theta_range, 3),
            "r_range": round(r_range, 1),
            "R0": round(R0, 1),
            "Rtip": round(Rtip, 1),
            "fit_quality": round(fit_quality, 2),
        }
        results.append(entry)
        
        quality = "★★★" if fit_quality > 5 else "★★" if fit_quality > 1 else "★" if fit_quality > 0.1 else "·"
        print(f"{i:>5} {len(pts):>5} {a:>8.2f} {b:>8.4f} {resid:>8.1f} {theta_range:>8.3f} {r_range:>8.1f} {R0:>6.1f} {Rtip:>6.1f} {quality:>6}")
    
    # Summary: the fitted parameters ARE the constraints
    good_fits = [r for r in results if r["fit_quality"] > 1]
    
    print(f"\n=== ORIGIN CONSTRAINTS (from {len(good_fits)} well-fitted paths) ===\n")
    if good_fits:
        a_range = [min(r["a"] for r in good_fits), max(r["a"] for r in good_fits)]
        b_range = [min(r["b"] for r in good_fits), max(r["b"] for r in good_fits)]
        R0_range = [min(r["R0"] for r in good_fits), max(r["R0"] for r in good_fits)]
        theta_range = [min(r["theta_range"] for r in good_fits), max(r["theta_range"] for r in good_fits)]
        
        constraints = {
            "center": [round(cx, 1), round(cy, 1)],
            "a_range": [round(a_range[0], 2), round(a_range[1], 2)],
            "b_range": [round(b_range[0], 4), round(b_range[1], 4)],
            "R0_range": [round(R0_range[0], 1), round(R0_range[1], 1)],
            "theta_range": [round(theta_range[0], 3), round(theta_range[1], 3)],
            "n_rays": len(good_fits),
            "symmetry_order": len(good_fits),
        }
        
        for k, v in constraints.items():
            print(f"  {k}: {v}")
        
        # Save
        out = f"{SCRATCH}/shamash_fitted.json"
        with open(out, 'w') as f:
            json.dump({"constraints": constraints, "paths": results}, f, indent=2)
        print(f"\n→ {out}")
    else:
        print("  ⚠ No paths fit the spiral model well")
        print("  The Shamash may use circular arcs, not logarithmic spirals")
        # Still save raw data
        out = f"{SCRATCH}/shamash_fitted.json"
        with open(out, 'w') as f:
            json.dump({"constraints": {"center": [round(cx,1), round(cy,1)]}, "paths": results}, f, indent=2)
        print(f"\n→ {out}")

if __name__ == "__main__":
    main()
