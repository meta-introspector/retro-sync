#!/usr/bin/env python3
"""Analyze cuneiform sign Unicode codepoints for Monster resonance mapping.

Each cuneiform sign has Unicode codepoints with numerical invariants:
- Sum of codepoints mod 71/59/47 → orbifold position
- Prime factorization of codepoint values
- Hamming weight, digit sum, etc.

Output: resonance scores for mapping 71 cuneiform tiles to 71 Monster conjugacy classes.
"""

import sys
import json
from math import gcd, log2

# The 15 Hurrian interval names in cuneiform
INTERVAL_SIGNS = [
    "𒀸𒌑𒄴𒊑",  # nīš tuḫrim
    "𒄿𒊭𒅈𒌈",  # išartum
    "𒂊𒁍𒁍",    # embūbum
    "𒉌𒀉𒃻",    # nīd qablim
    "𒃻𒇷𒌈",    # qablītum
    "𒆠𒁴𒈬",    # kitmum
    "𒁉𒌈",      # pītum
    "𒊺𒊒",      # šērum
    "𒊭𒅖𒊭𒌈",  # šalšatum
    "𒊑𒁍𒌈",    # rebûttum
    "𒅖𒄣",      # isqum
    "𒋾𒌅𒅈𒃻",  # titur qablītim
    "𒋾𒌅𒅈𒄿",  # titur išartim
    "𒊺𒅈𒁺",    # ṣerdum
    "𒀀𒈬𒊏𒁉",  # colophon
]

INTERVAL_NAMES = [
    "nīš tuḫrim", "išartum", "embūbum", "nīd qablim", "qablītum",
    "kitmum", "pītum", "šērum", "šalšatum", "rebûttum",
    "isqum", "titur qablītim", "titur išartim", "ṣerdum", "colophon",
]

SSP = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71]

def sign_invariants(sign):
    """Compute numerical invariants of a cuneiform sign string."""
    cps = [ord(c) for c in sign]
    total = sum(cps)
    prod = 1
    for c in cps:
        prod *= c
    
    # Prime factorization of sum
    factors = {}
    n = total
    for p in SSP:
        while n % p == 0:
            factors[p] = factors.get(p, 0) + 1
            n //= p
    
    return {
        "sign": sign,
        "codepoints": [hex(c) for c in cps],
        "n_chars": len(cps),
        "sum": total,
        "product": prod,
        "mod71": total % 71,
        "mod59": total % 59,
        "mod47": total % 47,
        "orbifold": (total % 71, total % 59, total % 47),
        "sum_factors_ssp": factors,
        "hamming": bin(total).count("1"),
        "digit_sum": sum(int(d) for d in str(total)),
    }

def resonance_score(inv, target_mod71):
    """Score how well a sign resonates with a specific mod-71 position."""
    score = 0.0
    
    # Direct mod 71 match
    dist = abs(inv["mod71"] - target_mod71)
    dist = min(dist, 71 - dist)
    score += 50.0 / (dist + 1)
    
    # SSP factor bonus
    for p, e in inv["sum_factors_ssp"].items():
        if p == SSP[target_mod71 % 15]:
            score += 20.0 * e
    
    # Character count resonance
    if inv["n_chars"] == target_mod71 % 7 + 1:
        score += 10.0
    
    # Hamming weight match
    if inv["hamming"] == target_mod71 % 8:
        score += 5.0
    
    return score

def main():
    print("=== CUNEIFORM SIGN NUMERICAL INVARIANTS ===\n")
    
    results = []
    for i, (sign, name) in enumerate(zip(INTERVAL_SIGNS, INTERVAL_NAMES)):
        inv = sign_invariants(sign)
        inv["name"] = name
        inv["index"] = i
        results.append(inv)
        
        print(f"  {i:2}: {sign:12} {name:16} "
              f"sum={inv['sum']:6} "
              f"orbifold=({inv['mod71']:2},{inv['mod59']:2},{inv['mod47']:2}) "
              f"chars={inv['n_chars']} "
              f"hamming={inv['hamming']} "
              f"factors={inv['sum_factors_ssp']}")
    
    # Show which mod-71 positions are naturally occupied
    print("\n=== NATURAL ORBIFOLD POSITIONS (mod 71) ===\n")
    occupied = {}
    for r in results:
        pos = r["mod71"]
        if pos not in occupied:
            occupied[pos] = []
        occupied[pos].append(r["name"])
    
    for pos in sorted(occupied.keys()):
        print(f"  position {pos:2}/71: {', '.join(occupied[pos])}")
    
    print(f"\n  {len(occupied)} of 71 positions occupied by 15 intervals")
    print(f"  {71 - len(occupied)} positions available for additional cuneiform signs")
    
    # Output as JSON for downstream use
    out_path = "fixtures/output/cuneiform_invariants.json"
    with open(out_path, "w") as f:
        json.dump(results, f, indent=2, ensure_ascii=False)
    print(f"\n→ {out_path}")

if __name__ == "__main__":
    main()
