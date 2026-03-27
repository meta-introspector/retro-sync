#!/usr/bin/env python3
"""Self-decoding hymn encoding: the codec IS mathematics.

If the codec is a well-known constant (j-invariant, τ(p), |Monster|),
the decoder is "know the math" — zero codec bytes. The math is public.

Tests all lilypond versions. Finds the encoding where data + codec_ref
is smallest, where codec_ref is just a name like "j" or "τ" or "M".

Self-describing format:
  byte 0:    codec_id (0=j, 1=τ, 2=M, 3=Cl15, 4=raw)
  byte 1:    n_events
  bytes 2..: data encoded relative to the codec's constants
"""

import os
import re
import json
from math import log2

# Mathematical constants as codecs (public knowledge = 0 bytes)
CODECS = {
    "j": [1, 744, 196884, 21493760, 864299970, 20245856256],
    "τ": [1, -24, 252, -1472, 4830, -6048, -16744, 84480],
    "M": [2,3,5,7,11,13,17,19,23,29,31,41,47,59,71],  # SSP primes
    "|M|_exp": [46,20,9,6,2,3,1,1,1,1,1,1,1,1,1],     # exponents in |Monster|
}

# Interval primes
PRIME_TABLE = {0:13,1:31,2:5,3:29,4:7,5:47,6:11,7:19,8:23,9:3,10:2,11:41,12:17,13:59,14:71}
SSP = [2,3,5,7,11,13,17,19,23,29,31,41,47,59,71]

def parse_lilypond_notes(path):
    """Extract MIDI note numbers from a lilypond file."""
    text = open(path).read()
    # Find notes in { ... } blocks
    note_pattern = re.compile(r"([a-g](?:is|es)?)('+|,+)?(\d*)")
    notes = []
    pitch_map = {'c':0,'d':2,'e':4,'f':5,'g':7,'a':9,'b':11}
    
    for m in note_pattern.finditer(text):
        name, octave_mod, dur = m.groups()
        base = name[0]
        if base not in pitch_map:
            continue
        midi = 60 + pitch_map[base]
        if 'is' in name: midi += 1
        if 'es' in name: midi -= 1
        if octave_mod:
            if "'" in octave_mod: midi += 12 * len(octave_mod)
            if "," in octave_mod: midi -= 12 * len(octave_mod)
        notes.append(midi)
    return notes

def notes_to_intervals(notes):
    """Convert MIDI notes to interval sequence (differences)."""
    return [notes[i+1] - notes[i] for i in range(len(notes)-1)]

def encode_self_describing(intervals, codec_name):
    """Encode intervals as self-describing payload.
    
    Format: [codec_id:1][n:1][data...]
    Data encoding depends on codec:
      "M" (SSP): each interval mod 15 → index into SSP, 4 bits each
      "j": project onto j-coefficients, store 6 weights as nibbles
      "τ": project onto τ values, store weights
      "raw": just the intervals as signed bytes
    """
    header = 2  # codec_id + length
    
    if codec_name == "M":
        # Each interval mapped to nearest SSP prime index (4 bits)
        data_bits = len(intervals) * 4
        data_bytes = (data_bits + 7) // 8
        codec_ref = 1  # "M" = 1 byte to say "use Monster primes"
        lossless = False  # mod mapping is lossy
        
    elif codec_name == "j":
        # Project onto 6 j-coefficients
        weights = [0.0] * 6
        for i, iv in enumerate(intervals):
            idx = abs(iv) % 6
            weights[idx] += 1.0 / (1 + i * 0.05)
        total = sum(weights) or 1
        # Store as 6 nibbles (4 bits each, quantized 0-15)
        data_bytes = 3  # 6 nibbles = 3 bytes
        codec_ref = 1
        lossless = False
        
    elif codec_name == "τ":
        # Same as j but with τ basis
        data_bytes = 3
        codec_ref = 1
        lossless = False
        
    elif codec_name == "raw":
        # Signed bytes, no codec needed
        data_bytes = len(intervals)  # 1 byte per interval (-128..127)
        codec_ref = 0  # no external reference needed
        lossless = True
        
    elif codec_name == "delta_mod15":
        # Intervals mod 15, packed 4 bits
        data_bits = len(intervals) * 4
        data_bytes = (data_bits + 7) // 8
        codec_ref = 1  # "mod 15"
        lossless = False
        
    total = header + data_bytes + codec_ref
    return total, data_bytes, codec_ref, lossless

def cl15_blade(intervals):
    """Compute Cl(15) blade from interval sequence."""
    mv = {0: 1}
    for iv in intervals:
        idx = abs(iv) % 15
        blade = 1 << idx
        new_mv = {}
        for mask, coeff in mv.items():
            result = mask ^ blade
            sign = 1
            for bit in range(idx):
                if mask & (1 << bit): sign *= -1
            new_mv[result] = new_mv.get(result, 0) + coeff * sign
        mv = {k: v for k, v in new_mv.items() if v != 0}
    return mv

def main():
    ly_dir = "fixtures/lilypond"
    files = sorted(f for f in os.listdir(ly_dir) if f.endswith('.ly'))
    
    print("=== SELF-DECODING ENCODING: MATH AS CODEC ===\n")
    print("The codec is a mathematical constant. Anyone who knows the math can decode.")
    print("Codec reference = 1 byte (which constant to use).\n")
    
    all_results = []
    
    for ly_file in files:
        path = os.path.join(ly_dir, ly_file)
        notes = parse_lilypond_notes(path)
        if len(notes) < 3:
            continue
        intervals = notes_to_intervals(notes)
        name = ly_file.replace('.ly', '')
        
        # Cl(15) blade
        blade = cl15_blade(intervals)
        blade_terms = len(blade)
        blade_bytes = blade_terms * 4 + 3  # mask(2) + coeff(2) per term + header
        
        # Blade grade
        grades = {}
        for mask in blade:
            g = bin(mask).count('1')
            grades[g] = grades.get(g, 0) + 1
        
        print(f"--- {name} ({len(notes)} notes, {len(intervals)} intervals) ---")
        
        codecs_to_test = ["raw", "M", "j", "τ", "delta_mod15"]
        results = []
        
        for codec in codecs_to_test:
            total, data, ref, lossless = encode_self_describing(intervals, codec)
            results.append((codec, total, data, ref, lossless))
        
        # Add Cl(15)
        results.append(("Cl(15)", blade_bytes, blade_bytes - 3, 1, "blade"))
        
        results.sort(key=lambda r: r[1])
        
        for codec, total, data, ref, lossless in results:
            marker = "✅" if lossless == True else "🔷" if lossless == "blade" else "⚠"
            print(f"  {marker} {codec:<12} total={total:>4}B  data={data:>4}B  ref={ref}B  loss={lossless}")
        
        print(f"  Cl(15) blade: {blade_terms} terms, grades={dict(sorted(grades.items()))}")
        
        # Best self-describing lossless
        best_lossless = min((r for r in results if r[4] == True), key=lambda r: r[1], default=None)
        best_overall = results[0]
        
        all_results.append({
            "name": name,
            "notes": len(notes),
            "intervals": len(intervals),
            "best_lossless": best_lossless[0] if best_lossless else "none",
            "best_lossless_bytes": best_lossless[1] if best_lossless else 0,
            "best_overall": best_overall[0],
            "best_overall_bytes": best_overall[1],
            "blade_terms": blade_terms,
            "blade_grades": grades,
        })
        print()
    
    # Summary
    print("=== SUMMARY: SELF-DESCRIBING ENCODINGS ACROSS ALL VERSIONS ===\n")
    print(f"{'Version':<16} {'Notes':>6} {'Best lossless':>20} {'Best overall':>20} {'Blade terms':>12}")
    for r in all_results:
        print(f"  {r['name']:<14} {r['notes']:>6} {r['best_lossless']+' '+str(r['best_lossless_bytes'])+'B':>20} "
              f"{r['best_overall']+' '+str(r['best_overall_bytes'])+'B':>20} {r['blade_terms']:>12}")
    
    print("\n  Codec reference cost: 1 byte (name of mathematical constant)")
    print("  The math itself is free — it's public knowledge.")
    
    out = "fixtures/output/hymn_self_describing.json"
    with open(out, "w") as f:
        json.dump(all_results, f, indent=2)
    print(f"\n→ {out}")

if __name__ == "__main__":
    main()
