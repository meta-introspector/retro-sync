#!/usr/bin/env python3
"""Compress Hurrian Hymn h.6 to a single FRACTRAN integer.

The hymn is a sequence of (interval_index, count) pairs.
Each interval maps to an SSP prime. The hymn becomes:

  F(hymn) = Π p_i^count_i

This single integer encodes the entire song. Factorization recovers it.
Also tests: how few numbers do we actually need?
"""

import json
from math import log2, gcd
from functools import reduce

# Optimal interval→prime mapping (from resonance solver)
INTERVAL_PRIMES = {
    0: 13, 1: 31, 2: 5, 3: 29, 4: 7, 5: 47, 6: 11,
    7: 19, 8: 23, 9: 3, 10: 2, 11: 41, 12: 17, 13: 59, 14: 71,
}

INTERVAL_NAMES = [
    "nīš tuḫrim", "išartum", "embūbum", "nīd qablim", "qablītum",
    "kitmum", "pītum", "šērum", "šalšatum", "rebûttum",
    "isqum", "titur qablītim", "titur išartim", "ṣerdum", "colophon",
]

# Hymn from tablet RS 15.30
HYMN = [
    (4, 3), (3, 1), (4, 3), (8, 1), (1, 10), (12, 2), (13, 1),
    (8, 2), (8, 2), (3, 2), (14, 1),
]

# Ramanujan tau values (best discriminator from basis_compare)
TAU = [1, -24, 252, -1472, 4830, -6048, -16744, 84480, -113643]

def encode_fractran(sequence):
    """Encode hymn as single FRACTRAN integer: Π p_i^count_i."""
    # Merge counts per prime
    prime_counts = {}
    for idx, count in sequence:
        p = INTERVAL_PRIMES[idx]
        prime_counts[p] = prime_counts.get(p, 0) + count
    
    product = 1
    for p, e in sorted(prime_counts.items()):
        product *= p ** e
    return product, prime_counts

def decode_fractran(n):
    """Recover interval sequence from FRACTRAN integer."""
    prime_to_idx = {v: k for k, v in INTERVAL_PRIMES.items()}
    recovered = {}
    for p in sorted(INTERVAL_PRIMES.values(), reverse=True):
        e = 0
        while n % p == 0:
            n //= p
            e += 1
        if e > 0 and p in prime_to_idx:
            recovered[prime_to_idx[p]] = e
    return recovered, n  # n should be 1 if fully decoded

def encode_runlength(sequence):
    """Encode as run-length pairs: (symbol, count) with minimal bits."""
    n_symbols = len(set(idx for idx, _ in sequence))
    bits_per_symbol = int(log2(n_symbols)) + 1
    max_count = max(c for _, c in sequence)
    bits_per_count = int(log2(max_count)) + 1
    total_bits = len(sequence) * (bits_per_symbol + bits_per_count)
    return total_bits, bits_per_symbol, bits_per_count

def encode_tau_projection(sequence):
    """Project onto Ramanujan tau basis (best discriminator)."""
    # Compute orbifold path
    state = 1
    path = []
    for idx, count in sequence:
        p = INTERVAL_PRIMES[idx]
        for _ in range(count):
            state = (state * p) % (71 * 59 * 47)
            path.append(state)
    
    # Activate tau coefficients
    n_tau = min(len(TAU), 6)
    weights = [0.0] * n_tau
    for i, s in enumerate(path):
        idx = (s % 71 + s % 59 + s % 47) % n_tau
        weights[idx] += 1.0 / (1 + i * 0.1)
    total = sum(weights) or 1.0
    weights = [w / total for w in weights]
    
    f_tau = sum(w * t for w, t in zip(weights, TAU[:n_tau]))
    return f_tau, weights

def main():
    print("=== HYMN COMPRESSION: HOW FEW NUMBERS? ===\n")
    
    # 1. Single FRACTRAN integer
    product, prime_counts = encode_fractran(HYMN)
    bits = product.bit_length()
    print(f"1. FRACTRAN integer (Π p^e):")
    print(f"   F(hymn) = {product}")
    print(f"   bits = {bits}, bytes = {(bits + 7) // 8}")
    print(f"   primes: {dict(sorted(prime_counts.items()))}")
    
    # Verify roundtrip
    recovered, remainder = decode_fractran(product)
    print(f"   roundtrip: {'✅ lossless' if remainder == 1 else '⚠ lossy (remainder=' + str(remainder) + ')'}")
    
    # NOTE: this loses ordering info (which qablītum×3 came first vs second)
    # The product merges: qablītum×3 + qablītum×3 = qablītum×6
    total_from_product = sum(prime_counts.values())
    total_from_hymn = sum(c for _, c in HYMN)
    print(f"   events: {total_from_product} (merged) vs {total_from_hymn} (original)")
    print(f"   ⚠ ordering lost — this is a BAG encoding, not a SEQUENCE\n")
    
    # 2. Run-length encoding
    total_bits, bps, bpc = encode_runlength(HYMN)
    print(f"2. Run-length encoding:")
    print(f"   {len(HYMN)} pairs × ({bps} + {bpc}) bits = {total_bits} bits = {(total_bits + 7) // 8} bytes")
    print(f"   ✅ lossless, preserves ordering\n")
    
    # 3. Sequence of prime indices only (counts inline)
    # Expand: [4,4,4,3,4,4,4,8,1,1,1,1,1,1,1,1,1,1,12,12,13,8,8,8,8,3,3,14]
    expanded = []
    for idx, count in HYMN:
        expanded.extend([idx] * count)
    n_distinct = len(set(expanded))
    bits_expanded = len(expanded) * (int(log2(14)) + 1)  # 4 bits per symbol
    print(f"3. Expanded sequence:")
    print(f"   {len(expanded)} symbols from alphabet of {n_distinct}")
    print(f"   {bits_expanded} bits = {(bits_expanded + 7) // 8} bytes (4 bits/symbol)")
    print(f"   ✅ lossless\n")
    
    # 4. Ramanujan tau projection (best discriminator)
    f_tau, weights = encode_tau_projection(HYMN)
    print(f"4. Ramanujan τ projection:")
    print(f"   F_τ(hymn) = {f_tau:.4f}")
    print(f"   weights = {[round(w, 4) for w in weights]}")
    print(f"   {len(weights)} numbers × 8 bytes = {len(weights) * 8} bytes")
    print(f"   ⚠ lossy — projection, not invertible\n")
    
    # 5. Orbifold triple (ultimate compression)
    from functools import reduce
    state = reduce(lambda s, p: (s * p) % (71*59*47), 
                   [INTERVAL_PRIMES[i] for i, c in HYMN for _ in range(c)], 1)
    orb = (state % 71, state % 59, state % 47)
    print(f"5. Orbifold triple (final state):")
    print(f"   ({orb[0]}, {orb[1]}, {orb[2]})")
    print(f"   3 numbers × 1 byte = 3 bytes")
    print(f"   ⚠ extremely lossy — only final position\n")
    
    # Summary
    print("=== COMPRESSION SUMMARY ===\n")
    print(f"  {'Method':<30} {'Bytes':>6} {'Lossless':>10} {'Preserves order':>16}")
    print(f"  {'-'*65}")
    print(f"  {'Raw (name+count pairs)':<30} {total_from_hymn * 8:>6} {'yes':>10} {'yes':>16}")
    print(f"  {'Expanded 4-bit symbols':<30} {(bits_expanded+7)//8:>6} {'yes':>10} {'yes':>16}")
    print(f"  {'Run-length pairs':<30} {(total_bits+7)//8:>6} {'yes':>10} {'yes':>16}")
    print(f"  {'FRACTRAN integer':<30} {(bits+7)//8:>6} {'bag only':>10} {'no':>16}")
    print(f"  {'Ramanujan τ projection':<30} {len(weights)*8:>6} {'no':>10} {'no':>16}")
    print(f"  {'Orbifold triple':<30} {3:>6} {'no':>10} {'no':>16}")
    
    # Save
    result = {
        "fractran_integer": str(product),
        "fractran_bits": bits,
        "prime_counts": {str(k): v for k, v in prime_counts.items()},
        "runlength_bits": total_bits,
        "expanded_length": len(expanded),
        "tau_projection": f_tau,
        "tau_weights": weights,
        "orbifold_final": orb,
    }
    out = "fixtures/output/hymn_compression.json"
    with open(out, "w") as f:
        json.dump(result, f, indent=2)
    print(f"\n→ {out}")

if __name__ == "__main__":
    main()
