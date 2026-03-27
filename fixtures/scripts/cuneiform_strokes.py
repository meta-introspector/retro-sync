#!/usr/bin/env python3
"""Analyze cuneiform glyph stroke harmonics from SVG text rendering.

Each cuneiform sign is composed of wedge strokes. Unicode cuneiform block
(U+12000–U+1254F) encodes signs by category:
  - Horizontal wedges (AŠ class)
  - Vertical wedges (DIŠ class)  
  - Diagonal wedges (corner marks)
  - Winkelhaken (angle hooks)

We extract stroke counts and types from the Unicode name decomposition,
then find harmonic ratios between signs.
"""

import unicodedata
import json
import sys

# The 15 interval signs
SIGNS = [
    ("nīš tuḫrim",     "𒀸𒌑𒄴𒊑"),
    ("išartum",         "𒄿𒊭𒅈𒌈"),
    ("embūbum",         "𒂊𒁍𒁍"),
    ("nīd qablim",     "𒉌𒀉𒃻"),
    ("qablītum",        "𒃻𒇷𒌈"),
    ("kitmum",          "𒆠𒁴𒈬"),
    ("pītum",           "𒁉𒌈"),
    ("šērum",           "𒊺𒊒"),
    ("šalšatum",        "𒊭𒅖𒊭𒌈"),
    ("rebûttum",        "𒊑𒁍𒌈"),
    ("isqum",           "𒅖𒄣"),
    ("titur qablītim",  "𒋾𒌅𒅈𒃻"),
    ("titur išartim",   "𒋾𒌅𒅈𒄿"),
    ("ṣerdum",          "𒊺𒅈𒁺"),
    ("colophon",        "𒀀𒈬𒊏𒁉"),
]

SSP = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 41, 47, 59, 71]

def glyph_strokes(char):
    """Extract stroke info from Unicode name."""
    cp = ord(char)
    name = unicodedata.name(char, f"U+{cp:04X}")
    
    # Count stroke indicators in the Unicode name
    strokes = {
        "horizontal": name.count("ASH") + name.count("DISH") + name.count("TAB"),
        "vertical": name.count("OVER") + name.count("CROSSING"),
        "diagonal": name.count("TENU") + name.count("GUNU"),
        "hook": name.count("SHESHIG") + name.count("NUTILLU"),
    }
    
    # Wedge count heuristic from codepoint position in block
    block_offset = cp - 0x12000
    wedge_estimate = 1 + (block_offset % 7)  # rough: signs get more complex
    
    return {
        "char": char,
        "codepoint": cp,
        "hex": f"U+{cp:04X}",
        "name": name,
        "block_offset": block_offset,
        "wedge_estimate": wedge_estimate,
        "strokes": strokes,
        "total_strokes": sum(strokes.values()) + wedge_estimate,
    }

def sign_harmony(sign_a, sign_b):
    """Compute harmonic ratio between two signs' stroke counts."""
    sa = sum(g["total_strokes"] for g in sign_a)
    sb = sum(g["total_strokes"] for g in sign_b)
    if sb == 0: return 0, 0, 0
    
    from math import gcd
    g = gcd(sa, sb)
    return sa // g, sb // g, sa / sb

def main():
    print("=== CUNEIFORM STROKE HARMONICS ===\n")
    
    all_data = []
    
    for name, sign in SIGNS:
        glyphs = [glyph_strokes(c) for c in sign]
        total = sum(g["total_strokes"] for g in glyphs)
        wedges = sum(g["wedge_estimate"] for g in glyphs)
        
        # Mod residues of stroke count
        m71 = total % 71
        m59 = total % 59
        m47 = total % 47
        
        # SSP factorization of total strokes
        factors = {}
        n = total
        for p in SSP:
            while n > 0 and n % p == 0:
                factors[p] = factors.get(p, 0) + 1
                n //= p
        
        entry = {
            "name": name,
            "sign": sign,
            "glyphs": glyphs,
            "total_strokes": total,
            "wedges": wedges,
            "orbifold": (m71, m59, m47),
            "factors": factors,
        }
        all_data.append(entry)
        
        glyph_names = [g["name"].replace("CUNEIFORM SIGN ", "") for g in glyphs]
        print(f"  {name:16} {sign:8} strokes={total:3} wedges={wedges:2} "
              f"orb=({m71:2},{m59:2},{m47:2}) "
              f"factors={factors}")
        for g in glyphs:
            short = g["name"].replace("CUNEIFORM SIGN ", "")
            print(f"    {g['hex']} {short:30} est={g['wedge_estimate']} {g['strokes']}")
    
    # Harmonic ratios between all pairs
    print("\n=== INTER-SIGN HARMONIC RATIOS ===\n")
    print(f"{'':16} ", end="")
    for name, _ in SIGNS[:8]:
        print(f"{name[:6]:>7}", end="")
    print()
    
    for i, (name_a, sign_a) in enumerate(SIGNS[:8]):
        glyphs_a = [glyph_strokes(c) for c in sign_a]
        print(f"  {name_a:14} ", end="")
        for j, (name_b, sign_b) in enumerate(SIGNS[:8]):
            glyphs_b = [glyph_strokes(c) for c in sign_b]
            num, den, ratio = sign_harmony(glyphs_a, glyphs_b)
            if i == j:
                print(f"   1:1 ", end="")
            else:
                print(f" {num:2}:{den:<2} ", end="")
        print()
    
    # Find consonant pairs (simple ratios)
    print("\n=== CONSONANT PAIRS (ratio ≤ 5:4) ===\n")
    for i in range(len(SIGNS)):
        for j in range(i+1, len(SIGNS)):
            ga = [glyph_strokes(c) for c in SIGNS[i][1]]
            gb = [glyph_strokes(c) for c in SIGNS[j][1]]
            num, den, ratio = sign_harmony(ga, gb)
            if num <= 5 and den <= 5 and num > 0:
                print(f"  {SIGNS[i][0]:16} : {SIGNS[j][0]:16} = {num}:{den} ({ratio:.3f})")
    
    # Save
    out = "fixtures/output/cuneiform_strokes.json"
    with open(out, "w") as f:
        json.dump(all_data, f, indent=2, ensure_ascii=False, default=str)
    print(f"\n→ {out}")

if __name__ == "__main__":
    main()
