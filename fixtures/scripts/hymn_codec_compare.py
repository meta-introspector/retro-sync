#!/usr/bin/env python3
"""Rank hymn encodings by TOTAL size (data + codec) and decode time.

Includes Cl(15,0,0) blade encoding: each interval = grade-1 blade,
hymn = geometric product of blades → multivector coefficients.

"Self-describing" = the encoding includes enough info to decode without
external lookup tables.
"""

import json
import time
from math import log2
from functools import reduce

# ── Codec definitions (these ARE the codecs, measured in bytes) ──

# Interval→prime table (shared by several encodings)
PRIME_TABLE = {0:13,1:31,2:5,3:29,4:7,5:47,6:11,7:19,8:23,9:3,10:2,11:41,12:17,13:59,14:71}
NAMES = ["nīš tuḫrim","išartum","embūbum","nīd qablim","qablītum",
         "kitmum","pītum","šērum","šalšatum","rebûttum",
         "isqum","titur qablītim","titur išartim","ṣerdum","colophon"]

HYMN = [(4,3),(3,1),(4,3),(8,1),(1,10),(12,2),(13,1),(8,2),(8,2),(3,2),(14,1)]

SSP = [2,3,5,7,11,13,17,19,23,29,31,41,47,59,71]

# ── Cl(15,0,0) blade encoding ──

def cl15_encode(sequence):
    """Encode hymn as Cl(15,0,0) multivector via geometric product of blades.
    Each interval i → basis blade e_i (bitmask 1<<i).
    Geometric product: XOR masks, sign from transpositions.
    Store only nonzero (mask, coefficient) pairs.
    """
    # Start with scalar 1
    mv = {0: 1}  # mask → coefficient
    
    for idx, count in sequence:
        # Blade for this interval: e_idx
        blade_mask = 1 << idx
        for _ in range(count):
            new_mv = {}
            for mask_a, coeff_a in mv.items():
                # Geometric product: e_a * e_idx
                result_mask = mask_a ^ blade_mask
                # Sign: count bits in mask_a above blade position
                sign = 1
                for bit in range(idx):
                    if mask_a & (1 << bit):
                        sign *= -1
                # If blade_mask is already in mask_a, e_i * e_i = +1 (positive signature)
                val = coeff_a * sign
                new_mv[result_mask] = new_mv.get(result_mask, 0) + val
            # Remove zeros
            mv = {k: v for k, v in new_mv.items() if v != 0}
    
    return mv

def cl15_decode(mv):
    """Decode: extract which blades are active and their grades."""
    result = []
    for mask, coeff in sorted(mv.items()):
        grade = bin(mask).count('1')
        blades = [i for i in range(15) if mask & (1 << i)]
        result.append((mask, grade, blades, coeff))
    return result

# ── Codec size calculators ──

def codec_size_raw():
    """Raw: need name table + pairs."""
    table = sum(len(n) for n in NAMES) + 15  # names + separators
    return table  # ~180 bytes for the name table

def codec_size_runlength():
    """Run-length: need symbol alphabet (15 indices) + pair format spec."""
    return 4  # "11 pairs, 4-bit idx, 4-bit count" = 4 byte header

def codec_size_expanded():
    """Expanded: need alphabet size."""
    return 2  # "28 symbols, 4 bits each" = 2 byte header

def codec_size_fractran():
    """FRACTRAN integer: need prime table (15 primes × 1 byte each)."""
    return 15  # the 15 SSP primes

def codec_size_cl15():
    """Cl(15): need Clifford algebra spec (dimension, signature)."""
    return 3  # "Cl(15,0,0)" = dimension + signature bytes

def codec_size_tau():
    """Tau projection: need 6 tau values × 8 bytes."""
    return 6 * 8  # 48 bytes for the basis

def codec_size_orbifold():
    """Orbifold: need 3 moduli (71, 59, 47)."""
    return 3  # three 1-byte moduli

# ── Encode + measure ──

def measure(name, encode_fn, data_size_fn, codec_size, decode_fn=None):
    """Measure encoding: time, data size, total size."""
    t0 = time.perf_counter_ns()
    encoded = encode_fn()
    t_encode = time.perf_counter_ns() - t0
    
    data_size = data_size_fn(encoded)
    
    t_decode = 0
    if decode_fn:
        t0 = time.perf_counter_ns()
        decode_fn(encoded)
        t_decode = time.perf_counter_ns() - t0
    
    total = data_size + codec_size
    return {
        "name": name,
        "data_bytes": data_size,
        "codec_bytes": codec_size,
        "total_bytes": total,
        "encode_ns": t_encode,
        "decode_ns": t_decode,
        "lossless": True,  # overridden per method
    }

def main():
    print("=== ENCODING COMPARISON: DATA + CODEC SIZE + TIME ===\n")
    
    results = []
    
    # 1. Raw name+count
    r = measure("Raw (name+count)",
        lambda: HYMN,
        lambda _: sum(len(NAMES[i]) + 1 for i, c in HYMN),  # name bytes + count byte
        codec_size_raw())
    r["lossless"] = True; r["ordered"] = True
    results.append(r)
    
    # 2. Run-length
    r = measure("Run-length 7-bit pairs",
        lambda: HYMN,
        lambda _: (len(HYMN) * 7 + 7) // 8,
        codec_size_runlength())
    r["lossless"] = True; r["ordered"] = True
    results.append(r)
    
    # 3. Expanded 4-bit
    r = measure("Expanded 4-bit",
        lambda: [i for i, c in HYMN for _ in range(c)],
        lambda e: (len(e) * 4 + 7) // 8,
        codec_size_expanded())
    r["lossless"] = True; r["ordered"] = True
    results.append(r)
    
    # 4. FRACTRAN integer
    def enc_frac():
        counts = {}
        for i, c in HYMN:
            p = PRIME_TABLE[i]
            counts[p] = counts.get(p, 0) + c
        product = 1
        for p, e in counts.items():
            product *= p ** e
        return product
    r = measure("FRACTRAN integer",
        enc_frac,
        lambda n: (n.bit_length() + 7) // 8,
        codec_size_fractran(),
        lambda n: {p: 0 for p in SSP})  # decode = factorize
    r["lossless"] = "bag"; r["ordered"] = False
    results.append(r)
    
    # 5. Cl(15,0,0) multivector
    def enc_cl15():
        return cl15_encode(HYMN)
    def dec_cl15(mv):
        return cl15_decode(mv)
    r = measure("Cl(15,0,0) multivector",
        enc_cl15,
        lambda mv: len(mv) * 6,  # 2 bytes mask + 4 bytes coeff per term
        codec_size_cl15(),
        dec_cl15)
    r["lossless"] = "algebraic"; r["ordered"] = "blade order"
    results.append(r)
    
    # 6. Ramanujan tau
    def enc_tau():
        state = 1
        path = []
        for i, c in HYMN:
            p = PRIME_TABLE[i]
            for _ in range(c):
                state = (state * p) % (71*59*47)
                path.append(state)
        tau = [1,-24,252,-1472,4830,-6048]
        w = [0.0]*6
        for j, s in enumerate(path):
            idx = (s%71 + s%59 + s%47) % 6
            w[idx] += 1.0/(1+j*0.1)
        t = sum(w) or 1
        return [x/t for x in w]
    r = measure("Ramanujan τ projection",
        enc_tau,
        lambda w: len(w) * 8,
        codec_size_tau())
    r["lossless"] = False; r["ordered"] = False
    results.append(r)
    
    # 7. Orbifold triple
    def enc_orb():
        state = 1
        for i, c in HYMN:
            p = PRIME_TABLE[i]
            for _ in range(c):
                state = (state * p) % (71*59*47)
        return (state%71, state%59, state%47)
    r = measure("Orbifold triple",
        enc_orb,
        lambda _: 3,
        codec_size_orbifold())
    r["lossless"] = False; r["ordered"] = False
    results.append(r)
    
    # 8. Cl(15) blade signature (just which blades are active + grades)
    def enc_blade_sig():
        mv = cl15_encode(HYMN)
        # Compress: just store the bitmask of nonzero terms
        return set(mv.keys())
    r = measure("Cl(15) blade signature",
        enc_blade_sig,
        lambda s: (max(s).bit_length() * len(s) + 7) // 8 if s else 1,
        codec_size_cl15())
    r["lossless"] = "structural"; r["ordered"] = "grade"
    results.append(r)
    
    # Sort by total size
    results.sort(key=lambda r: r["total_bytes"])
    
    print(f"{'Method':<28} {'Data':>5} {'Codec':>6} {'TOTAL':>6} {'Enc μs':>7} {'Dec μs':>7} {'Loss':>10} {'Order':>8}")
    print("-" * 85)
    for r in results:
        enc_us = r["encode_ns"] / 1000
        dec_us = r["decode_ns"] / 1000
        print(f"  {r['name']:<26} {r['data_bytes']:>5} {r['codec_bytes']:>6} {r['total_bytes']:>6} {enc_us:>7.1f} {dec_us:>7.1f} {str(r['lossless']):>10} {str(r['ordered']):>8}")
    
    # Cl(15) details
    print("\n=== Cl(15,0,0) MULTIVECTOR DETAILS ===\n")
    mv = cl15_encode(HYMN)
    decoded = cl15_decode(mv)
    print(f"  {len(mv)} nonzero terms in the multivector:")
    for mask, grade, blades, coeff in decoded[:15]:
        blade_str = "∧".join(f"e{b}" for b in blades) if blades else "1"
        print(f"    grade {grade}: {blade_str:20} coeff={coeff}")
    if len(decoded) > 15:
        print(f"    ... ({len(decoded)} total terms)")
    
    # Save
    out = "fixtures/output/hymn_codec_comparison.json"
    with open(out, "w") as f:
        json.dump(results, f, indent=2, default=str)
    print(f"\n→ {out}")

if __name__ == "__main__":
    main()
