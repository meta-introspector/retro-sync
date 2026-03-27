#!/usr/bin/env python3
"""Extract geometric constraints from Shamash + Ishtar SVGs.

These constraints define "what makes it look like Hurrian art":
- Radial symmetry order
- Path count, length, curvature ranges
- Bounding box ratios
- Color palette ranges
- Stroke width ranges

Output: constraints.json — the fitness function for breeding.
"""

import re, math, json

SCRATCH = "/var/www/solana.solfunmeme.com/retro-sync/scratch"

def extract_paths(svg_text):
    """Extract all path d= data."""
    paths = []
    for m in re.finditer(r'd="([^"]*)"', svg_text):
        paths.append(m.group(1))
    return paths

def extract_coords(path_d):
    """Extract all numeric coordinates from a path d string."""
    return [float(x) for x in re.findall(r'[-]?\d+\.?\d*', path_d)]

def path_bbox(coords):
    """Bounding box from coordinate list (alternating x,y)."""
    if len(coords) < 2: return (0,0,0,0)
    xs = coords[0::2]
    ys = coords[1::2]
    return (min(xs), min(ys), max(xs), max(ys))

def path_length(coords):
    """Approximate path length from coordinate pairs."""
    total = 0
    for i in range(0, len(coords)-3, 2):
        dx = coords[i+2] - coords[i]
        dy = coords[i+3] - coords[i+1]
        total += math.sqrt(dx*dx + dy*dy)
    return total

def detect_symmetry(coords, cx, cy):
    """Estimate rotational symmetry order by checking angle distribution."""
    angles = []
    for i in range(0, len(coords)-1, 2):
        a = math.atan2(coords[i+1] - cy, coords[i] - cx)
        angles.append(a)
    if len(angles) < 4: return 1
    # Check for N-fold symmetry by looking at angle gaps
    angles.sort()
    gaps = [angles[i+1] - angles[i] for i in range(len(angles)-1)]
    if not gaps: return 1
    mean_gap = sum(gaps) / len(gaps)
    if mean_gap < 0.01: return 1
    return max(1, round(2 * math.pi / mean_gap))

def extract_colors(svg_text):
    """Extract all color values."""
    colors = []
    for m in re.finditer(r'(?:fill|stroke)="(?:#([0-9a-fA-F]{6})|rgb\((\d+),(\d+),(\d+)\))"', svg_text):
        if m.group(1):
            h = m.group(1)
            colors.append((int(h[0:2],16), int(h[2:4],16), int(h[4:6],16)))
        elif m.group(2):
            colors.append((int(m.group(2)), int(m.group(3)), int(m.group(4))))
    return colors

def extract_strokes(svg_text):
    """Extract stroke widths."""
    return [float(m.group(1)) for m in re.finditer(r'stroke-width="([\d.]+)"', svg_text)]

def analyze_svg(name, path):
    """Full constraint extraction from one SVG."""
    svg = open(path).read()
    
    # Dimensions
    w_m = re.search(r'width="(\d+)"', svg)
    h_m = re.search(r'height="(\d+)"', svg)
    w = int(w_m.group(1)) if w_m else 512
    h = int(h_m.group(1)) if h_m else 512
    cx, cy = w/2, h/2
    
    paths = extract_paths(svg)
    all_coords = [extract_coords(p) for p in paths]
    colors = extract_colors(svg)
    strokes = extract_strokes(svg)
    
    # Per-path stats
    lengths = [path_length(c) for c in all_coords if len(c) >= 4]
    bboxes = [path_bbox(c) for c in all_coords if len(c) >= 4]
    coord_counts = [len(c) for c in all_coords]
    symmetries = [detect_symmetry(c, cx, cy) for c in all_coords if len(c) >= 8]
    
    # Color ranges
    r_vals = [c[0] for c in colors] if colors else [128]
    g_vals = [c[1] for c in colors] if colors else [128]
    b_vals = [c[2] for c in colors] if colors else [128]
    
    constraints = {
        "name": name,
        "dimensions": [w, h],
        "center": [cx, cy],
        "n_paths": len(paths),
        "total_coordinates": sum(coord_counts),
        "coord_count_range": [min(coord_counts) if coord_counts else 0, max(coord_counts) if coord_counts else 0],
        "path_length_range": [round(min(lengths),1) if lengths else 0, round(max(lengths),1) if lengths else 0],
        "path_length_mean": round(sum(lengths)/len(lengths),1) if lengths else 0,
        "symmetry_orders": sorted(set(symmetries)) if symmetries else [1],
        "dominant_symmetry": max(set(symmetries), key=symmetries.count) if symmetries else 1,
        "color_r_range": [min(r_vals), max(r_vals)],
        "color_g_range": [min(g_vals), max(g_vals)],
        "color_b_range": [min(b_vals), max(b_vals)],
        "n_colors": len(set(colors)),
        "stroke_width_range": [min(strokes) if strokes else 0, max(strokes) if strokes else 0],
        "bbox_aspect_ratios": [],
    }
    
    for x1,y1,x2,y2 in bboxes:
        w_bb = max(x2-x1, 0.1)
        h_bb = max(y2-y1, 0.1)
        constraints["bbox_aspect_ratios"].append(round(w_bb/h_bb, 2))
    
    return constraints

def main():
    print("=== GEOMETRIC CONSTRAINT EXTRACTION ===\n")
    
    all_constraints = {}
    
    for name, path in [
        ("shamash", f"{SCRATCH}/shamash_star.svg"),
        ("ishtar", f"{SCRATCH}/ishtar_star.svg"),
    ]:
        if not os.path.exists(path):
            print(f"  ⚠ {path} not found")
            continue
        
        c = analyze_svg(name, path)
        all_constraints[name] = c
        
        print(f"--- {name} ---")
        print(f"  Dimensions: {c['dimensions'][0]}×{c['dimensions'][1]}")
        print(f"  Paths: {c['n_paths']}, total coords: {c['total_coordinates']}")
        print(f"  Coord count/path: {c['coord_count_range']}")
        print(f"  Path lengths: {c['path_length_range']} (mean {c['path_length_mean']})")
        print(f"  Symmetry: {c['dominant_symmetry']}-fold (detected: {c['symmetry_orders']})")
        print(f"  Colors: {c['n_colors']} unique, R={c['color_r_range']} G={c['color_g_range']} B={c['color_b_range']}")
        print(f"  Strokes: {c['stroke_width_range']}")
        print(f"  Aspect ratios: {c['bbox_aspect_ratios'][:5]}...")
        print()
    
    # Combined constraints for breeding fitness
    print("=== BREEDING FITNESS CONSTRAINTS ===\n")
    fitness = {
        "min_paths": 4,
        "max_paths": 20,
        "min_coords_per_path": 8,
        "min_total_coords": 100,
        "symmetry_target": 8,  # 8-fold from Ishtar
        "color_min": 64,       # stego safety floor
        "color_max": 240,
        "stroke_width_range": [0.5, 5.0],
        "path_length_min": 50,
        "path_length_max": 5000,
        "aspect_ratio_range": [0.5, 2.0],
        "visual_similarity_min": 0.7,  # SSIM to reference
    }
    all_constraints["fitness"] = fitness
    
    for k, v in fitness.items():
        print(f"  {k}: {v}")
    
    out = f"{SCRATCH}/constraints.json"
    with open(out, 'w') as f:
        json.dump(all_constraints, f, indent=2)
    print(f"\n→ {out}")
    print(f"  View: https://solana.solfunmeme.com/retro-sync/scratch/constraints.json")

import os
if __name__ == "__main__":
    main()
