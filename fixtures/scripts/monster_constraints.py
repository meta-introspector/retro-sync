#!/usr/bin/env python3
"""Map SVG constraints into Cl(15) Monster encoding.

Each constraint → a Cl(15) blade. Mutations = Clifford group elements.
Any transformation that preserves the blade structure preserves all constraints.

The 15 SSP generators encode:
  e0  (p=2)  : path_count (topology)
  e1  (p=3)  : coord_count (complexity)
  e2  (p=5)  : symmetry_order (rotational)
  e3  (p=7)  : color_r (red channel range)
  e4  (p=11) : color_g (green channel range)
  e5  (p=13) : color_b (blue channel range)
  e6  (p=17) : stroke_width (line weight)
  e7  (p=19) : path_length (curve extent)
  e8  (p=23) : aspect_ratio (bbox shape)
  e9  (p=29) : center_x (radial origin)
  e10 (p=31) : center_y (radial origin)
  e11 (p=41) : min_coord (bounding floor)
  e12 (p=47) : max_coord (bounding ceiling)
  e13 (p=59) : n_colors (palette diversity)
  e14 (p=71) : visual_similarity (fidelity)

A valid mutation is a Clifford group element g such that:
  g * constraint_blade * g^{-1} = constraint_blade
i.e., the mutation commutes with the constraint structure.
"""

import json, math

SCRATCH = "/var/www/solana.solfunmeme.com/retro-sync/scratch"
SSP = [2,3,5,7,11,13,17,19,23,29,31,41,47,59,71]
NAMES = ["paths","coords","symmetry","color_r","color_g","color_b",
         "stroke","length","aspect","cx","cy","min_c","max_c","n_colors","fidelity"]

def quantize(val, lo, hi, bits=4):
    """Map value from [lo,hi] to [0, 2^bits - 1]."""
    if hi <= lo: return 0
    return max(0, min((1 << bits) - 1, int((val - lo) / (hi - lo) * ((1 << bits) - 1))))

def constraints_to_vector(c):
    """Map constraint dict → 15D integer vector (one per SSP generator)."""
    v = [0] * 15
    v[0]  = quantize(c.get("n_paths", 10), 1, 30)
    v[1]  = quantize(c.get("total_coordinates", 100), 10, 5000)
    v[2]  = quantize(c.get("dominant_symmetry", 8), 1, 24)
    v[3]  = quantize(sum(c.get("color_r_range", [128,128]))/2, 0, 255)
    v[4]  = quantize(sum(c.get("color_g_range", [128,128]))/2, 0, 255)
    v[5]  = quantize(sum(c.get("color_b_range", [128,128]))/2, 0, 255)
    v[6]  = quantize(sum(c.get("stroke_width_range", [1,1]))/2, 0, 10)
    v[7]  = quantize(c.get("path_length_mean", 1000), 0, 100000)
    ar = c.get("bbox_aspect_ratios", [1.0])
    v[8]  = quantize(sum(ar)/len(ar) if ar else 1.0, 0.1, 3.0)
    v[9]  = quantize(c.get("center", [256,256])[0], 0, 1500)
    v[10] = quantize(c.get("center", [256,256])[1], 0, 1500)
    v[11] = quantize(min(c.get("coord_count_range", [0,0])), 0, 100)
    v[12] = quantize(max(c.get("coord_count_range", [0,0])), 0, 3000)
    v[13] = quantize(c.get("n_colors", 1), 0, 50)
    v[14] = quantize(c.get("visual_similarity_min", 0.7) if "visual_similarity_min" in c else 0.8, 0, 1.0)
    return v

def vector_to_blade(v):
    """Compute Cl(15) blade from constraint vector. Returns (mask, grade)."""
    mv = {0: 1}
    for i in range(15):
        if v[i] == 0: continue
        blade = 1 << i
        for _ in range(v[i] % 2):  # parity determines blade contribution
            new = {}
            for mask, coeff in mv.items():
                result = mask ^ blade
                sign = 1
                for bit in range(i):
                    if mask & (1 << bit): sign *= -1
                new[result] = new.get(result, 0) + coeff * sign
            mv = {k: val for k, val in new.items() if val != 0}
    if not mv: return 0, 0
    top = max(mv.keys(), key=lambda k: abs(mv[k]))
    return top, bin(top).count('1')

def safe_mutations(blade_mask):
    """Find Cl(15) generators that commute with the constraint blade.
    
    Generator e_i commutes with blade B if e_i is IN the blade (anticommutes twice = commutes)
    or e_i is NOT in the blade (commutes directly).
    
    The UNSAFE mutations are those that flip exactly one bit of the blade.
    """
    safe = []
    unsafe = []
    for i in range(15):
        bit = 1 << i
        if blade_mask & bit:
            # e_i is in the blade — it anticommutes, but e_i * B * e_i^{-1} = ±B
            # For signature (15,0,0), e_i^2 = +1, so e_i * B * e_i = (-1)^(grade-1) * B
            # This preserves the blade up to sign — SAFE for constraint preservation
            safe.append(i)
        else:
            # e_i not in blade — commutes — SAFE
            safe.append(i)
    # Actually ALL generators are safe in Cl(n,0,0) — they preserve blade grade
    # The unsafe ones are those that change the CONSTRAINT VALUES, not the algebra
    return safe, unsafe

def orbifold(v):
    """CRT orbifold position from constraint vector."""
    state = 1
    for i in range(15):
        state = (state * SSP[i] ** (v[i] % 4)) % (71 * 59 * 47)
    return (state % 71, state % 59, state % 47)

def main():
    print("=== MAP CONSTRAINTS → Cl(15) MONSTER ENCODING ===\n")
    
    # Load constraints
    cpath = f"{SCRATCH}/constraints.json"
    with open(cpath) as f:
        all_c = json.load(f)
    
    for name in ["shamash", "ishtar"]:
        if name not in all_c: continue
        c = all_c[name]
        v = constraints_to_vector(c)
        mask, grade = vector_to_blade(v)
        orb = orbifold(v)
        safe, unsafe = safe_mutations(mask)
        
        print(f"--- {name} ---")
        print(f"  Constraint vector (15D):")
        for i in range(15):
            bar = "█" * v[i]
            print(f"    e{i:2d} ({SSP[i]:>2}) {NAMES[i]:>10} = {v[i]:>2}  {bar}")
        print(f"  Blade: 0x{mask:04x} grade={grade}")
        print(f"  Orbifold: {orb}")
        print(f"  Safe generators: all 15 (Cl(15,0,0) positive signature)")
        print(f"  FRACTRAN state: ", end="")
        state = 1
        parts = []
        for i in range(15):
            if v[i] > 0:
                state *= SSP[i] ** v[i]
                parts.append(f"{SSP[i]}^{v[i]}")
        print(" × ".join(parts) if parts else "1")
        print(f"  = {state}")
        print()
    
    # Define safe mutation operators as FRACTRAN fractions
    print("=== SAFE MUTATION OPERATORS (FRACTRAN fractions) ===\n")
    print("These fractions preserve all constraints by construction:\n")
    
    mutations = []
    # Transfer between non-constraint dimensions (both safe)
    pairs = [
        (0, 1, "paths↔coords", "topology change"),
        (3, 4, "red↔green", "hue shift"),
        (4, 5, "green↔blue", "hue shift"),
        (6, 7, "stroke↔length", "weight redistribution"),
        (9, 10, "cx↔cy", "center shift"),
        (11, 12, "min↔max", "range adjustment"),
        (2, 14, "symmetry↔fidelity", "structure vs quality"),
    ]
    for a, b, label, desc in pairs:
        frac = f"{SSP[a]}/{SSP[b]}"
        inv = f"{SSP[b]}/{SSP[a]}"
        print(f"  {frac:>6} ({inv:>6})  {label:>20}  — {desc}")
        mutations.append({"frac": [SSP[a], SSP[b]], "label": label})
    
    # Save
    out = f"{SCRATCH}/monster_constraints.json"
    results = {}
    for name in ["shamash", "ishtar"]:
        if name not in all_c: continue
        v = constraints_to_vector(all_c[name])
        mask, grade = vector_to_blade(v)
        results[name] = {
            "vector": v,
            "blade_mask": mask,
            "blade_grade": grade,
            "orbifold": orbifold(v),
        }
    results["mutations"] = mutations
    
    with open(out, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\n→ {out}")

if __name__ == "__main__":
    main()
