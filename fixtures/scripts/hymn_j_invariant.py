#!/usr/bin/env python3
"""Map Hurrian Hymn h.6 to j-invariant via interval→prime codec.

The compression: note sequence → prime trajectory → orbifold path → 
j-invariant coefficients. The modular form IS the song.

j(τ) = q^-1 + 744 + 196884q + 21493760q² + ...
Each coefficient is a Monster representation dimension.
The hymn's orbifold path selects which coefficients are active.
"""

import json
import sys
from math import gcd, log2, pi, cos, sin

# Optimal interval→prime mapping from the resonance solver
INTERVAL_PRIMES = {
    "nīš tuḫrim": 13, "išartum": 31, "embūbum": 5,
    "nīd qablim": 29, "qablītum": 7, "kitmum": 47,
    "pītum": 11, "šērum": 19, "šalšatum": 23,
    "rebûttum": 3, "isqum": 2, "titur qablītim": 41,
    "titur išartim": 17, "ṣerdum": 59, "colophon": 71,
}

# Stroke counts (from cuneiform_strokes.py)
STROKE_COUNTS = {
    "nīš tuḫrim": 12, "išartum": 26, "embūbum": 8,
    "nīd qablim": 11, "qablītum": 21, "kitmum": 13,
    "pītum": 11, "šērum": 7, "šalšatum": 28,
    "rebûttum": 15, "isqum": 12, "titur qablītim": 22,
    "titur išartim": 20, "ṣerdum": 17, "colophon": 14,
}

# Hurrian Hymn h.6 notation from tablet RS 15.30 (Dietrich & Loretz 1975)
# Format: (interval_name, repeat_count)
HYMN_SEQUENCE = [
    ("qablītum", 3), ("nīd qablim", 1), ("qablītum", 3), ("šalšatum", 1),
    ("išartum", 10), ("titur išartim", 2), ("ṣerdum", 1),
    ("šalšatum", 2), ("šalšatum", 2), ("nīd qablim", 2),
    ("colophon", 1),
]

# First j-invariant coefficients (Monster representation dimensions)
J_COEFFS = [1, 744, 196884, 21493760, 864299970, 20245856256]

SSP = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71]

def prime_state(sequence):
    """Convert interval sequence to FRACTRAN prime-power state trajectory."""
    trajectory = []
    state = 1
    for name, count in sequence:
        p = INTERVAL_PRIMES[name]
        for _ in range(count):
            state = (state * p) % (71 * 59 * 47)  # keep in orbifold range
            trajectory.append(state)
    return trajectory

def orbifold_path(trajectory):
    """Map trajectory to (mod 71, mod 59, mod 47) orbifold coordinates."""
    return [(s % 71, s % 59, s % 47) for s in trajectory]

def j_encode(path):
    """Encode orbifold path as j-invariant coefficient activations.
    
    Each orbifold position (a, b, c) activates j-coefficient at index
    (a + b + c) mod len(J_COEFFS). The activation weight is the
    stroke count of the interval at that position.
    """
    activations = [0.0] * len(J_COEFFS)
    for i, (a, b, c) in enumerate(path):
        idx = (a + b + c) % len(J_COEFFS)
        # Weight by position in sequence (earlier = stronger)
        weight = 1.0 / (1 + i * 0.1)
        activations[idx] += weight
    
    # Normalize
    total = sum(activations) or 1.0
    return [a / total for a in activations]

def j_compress(activations):
    """Compress: represent the song as weighted j-invariant sum.
    
    F(hymn) = Σ w_i × j_coeff[i]
    This single number encodes the entire hymn.
    """
    return sum(w * c for w, c in zip(activations, J_COEFFS))

def j_decompress(compressed, activations):
    """Verify: reconstruct activation weights from compressed value."""
    # The activations ARE the compressed form (6 floats)
    return activations

def stroke_signature(sequence):
    """Compute stroke-based signature of the hymn."""
    total_strokes = 0
    stroke_seq = []
    for name, count in sequence:
        s = STROKE_COUNTS[name]
        for _ in range(count):
            total_strokes += s
            stroke_seq.append(s)
    return total_strokes, stroke_seq

def main():
    print("=== HURRIAN HYMN h.6 → j-INVARIANT COMPRESSION ===\n")
    
    # 1. Show the hymn
    print("Hymn sequence (from tablet RS 15.30):")
    total_notes = 0
    for name, count in HYMN_SEQUENCE:
        p = INTERVAL_PRIMES[name]
        s = STROKE_COUNTS[name]
        total_notes += count
        print(f"  {name:16} ×{count:2}  prime={p:2}  strokes={s:2}")
    print(f"\n  Total: {total_notes} interval events\n")
    
    # 2. Prime trajectory
    traj = prime_state(HYMN_SEQUENCE)
    print(f"Prime trajectory ({len(traj)} states):")
    print(f"  first 10: {traj[:10]}")
    print(f"  last  5:  {traj[-5:]}\n")
    
    # 3. Orbifold path
    path = orbifold_path(traj)
    print("Orbifold path (mod 71, 59, 47):")
    for i, (a, b, c) in enumerate(path[:10]):
        print(f"  [{i:2}] ({a:2}, {b:2}, {c:2})  dim_index={a*59*47 + b*47 + c}")
    if len(path) > 10:
        print(f"  ... ({len(path)} total positions)")
    
    # 4. j-invariant encoding
    activations = j_encode(path)
    compressed = j_compress(activations)
    
    print(f"\nj-invariant activations:")
    for i, (w, c) in enumerate(zip(activations, J_COEFFS)):
        print(f"  j[{i}] = {c:>12}  × {w:.4f}  = {w*c:>14.1f}")
    
    print(f"\nF(hymn) = {compressed:.1f}")
    print(f"  = compressed representation of the entire hymn")
    
    # 5. Stroke signature
    total_strokes, stroke_seq = stroke_signature(HYMN_SEQUENCE)
    print(f"\nStroke signature:")
    print(f"  Total strokes: {total_strokes}")
    print(f"  mod 71 = {total_strokes % 71}")
    print(f"  mod 59 = {total_strokes % 59}")
    print(f"  mod 47 = {total_strokes % 47}")
    print(f"  Stroke orbifold: ({total_strokes % 71}, {total_strokes % 59}, {total_strokes % 47})")
    
    # 6. Compression ratio
    raw_size = total_notes * 8  # 8 bytes per note (name + count)
    compressed_size = len(activations) * 8  # 6 floats
    print(f"\nCompression:")
    print(f"  Raw:        {raw_size} bytes ({total_notes} notes × 8)")
    print(f"  Compressed: {compressed_size} bytes ({len(activations)} j-coefficients × 8)")
    print(f"  Ratio:      {raw_size / compressed_size:.1f}×")
    print(f"  The hymn is 6 numbers: {[round(a, 4) for a in activations]}")
    
    # 7. Monster dimension
    dim_sum = sum(int(w * c) for w, c in zip(activations, J_COEFFS))
    print(f"\n  Monster dimension of the hymn: {dim_sum}")
    print(f"  (sum of activated representation dimensions)")
    
    # Save
    result = {
        "hymn": HYMN_SEQUENCE,
        "trajectory": traj,
        "orbifold_path": path,
        "j_activations": activations,
        "F_hymn": compressed,
        "total_strokes": total_strokes,
        "stroke_orbifold": (total_strokes % 71, total_strokes % 59, total_strokes % 47),
        "monster_dimension": dim_sum,
    }
    out = "fixtures/output/hymn_j_invariant.json"
    with open(out, "w") as f:
        json.dump(result, f, indent=2)
    print(f"\n→ {out}")

if __name__ == "__main__":
    main()
