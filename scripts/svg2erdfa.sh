#!/usr/bin/env python3
"""Extract note geometry from lilypond SVG → DA51 CBOR + FRACTRAN state.

Reads translate(x, y) positions from SVG, maps:
  x → time (FRACTRAN step)
  y → pitch (SSP prime index)
  
Outputs: DA51 CBOR shards + FRACTRAN integer state.
"""

import re, json, struct, hashlib, sys, os

SSP = [2,3,5,7,11,13,17,19,23,29,31,41,47,59,71]

def extract_positions(svg_path):
    """Extract all translate(x, y) positions from SVG."""
    svg = open(svg_path).read()
    positions = []
    for m in re.finditer(r'translate\(([\d.]+),\s*([\d.]+)\)', svg):
        x, y = float(m.group(1)), float(m.group(2))
        positions.append((x, y))
    return positions

def positions_to_notes(positions):
    """Convert SVG positions to (time, pitch) pairs.
    
    x = time position (sort by x for temporal order)
    y = staff position → pitch (lower y = higher pitch in SVG coords)
    """
    if not positions:
        return []
    
    # Sort by x (time), then y (pitch)
    sorted_pos = sorted(positions, key=lambda p: (p[0], p[1]))
    
    # Quantize y to staff lines (each staff line ≈ 1.0 unit apart)
    # Map to MIDI-like pitch: y=8 → high, y=22 → low
    notes = []
    for x, y in sorted_pos:
        pitch = max(0, min(14, int((25 - y) * 0.7)))  # map to 0-14 (15 SSP slots)
        time_q = round(x * 10)  # quantized time
        notes.append((time_q, pitch))
    
    return notes

def notes_to_fractran(notes):
    """Encode note sequence as FRACTRAN integer: Π SSP[pitch]^count."""
    counts = [0] * 15
    for _, pitch in notes:
        counts[pitch] += 1
    
    state = 1
    parts = []
    for i in range(15):
        if counts[i] > 0:
            state *= SSP[i] ** min(counts[i], 8)  # cap exponent to avoid overflow
            parts.append(f"{SSP[i]}^{counts[i]}")
    
    return state, counts, parts

def notes_to_intervals(notes):
    """Extract interval sequence from note pairs."""
    intervals = []
    for i in range(len(notes) - 1):
        if notes[i+1][0] != notes[i][0]:  # different time = melodic interval
            intervals.append(notes[i+1][1] - notes[i][1])
    return intervals

def cl15_blade(intervals):
    """Compute Cl(15) blade grade from intervals."""
    mv = {0: 1}
    for iv in intervals[:500]:
        idx = abs(iv) % 15
        blade = 1 << idx
        new = {}
        for mask, coeff in mv.items():
            result = mask ^ blade
            sign = 1
            for bit in range(idx):
                if mask & (1 << bit): sign *= -1
            new[result] = new.get(result, 0) + coeff * sign
        mv = {k: v for k, v in new.items() if v != 0}
    if not mv: return 0, 0
    top = max(mv.keys(), key=lambda k: abs(mv[k]))
    return bin(top).count('1'), top

def encode_da51_cbor(name, notes, state, counts, grade, blade_mask):
    """Encode as DA51-tagged CBOR shard."""
    # Minimal CBOR: DA51 tag + map
    buf = bytearray()
    buf.extend(b'\xd9\xda\x51')  # DA51 tag
    
    # CBOR map with 6 entries
    buf.append(0xa6)  # map(6)
    
    def cbor_str(s):
        b = s.encode()
        if len(b) < 24:
            return bytes([0x60 + len(b)]) + b
        return bytes([0x78, len(b)]) + b
    
    def cbor_uint(n):
        if n < 24: return bytes([n])
        if n < 256: return bytes([0x18, n])
        if n < 65536: return bytes([0x19]) + n.to_bytes(2, 'big')
        return bytes([0x1a]) + n.to_bytes(4, 'big')
    
    # id
    buf.extend(cbor_str("id")); buf.extend(cbor_str(name))
    # notes
    buf.extend(cbor_str("notes")); buf.extend(cbor_uint(len(notes)))
    # state
    buf.extend(cbor_str("state")); buf.extend(cbor_str(str(state)))
    # grade
    buf.extend(cbor_str("grade")); buf.extend(cbor_uint(grade))
    # blade
    buf.extend(cbor_str("blade")); buf.extend(cbor_uint(blade_mask))
    # counts
    buf.extend(cbor_str("counts"))
    buf.append(0x8f)  # array(15)
    for c in counts:
        buf.extend(cbor_uint(c))
    
    return bytes(buf)

def main():
    svg_dir = sys.argv[1] if len(sys.argv) > 1 else "projects/bach-invention/output/svg"
    out_dir = sys.argv[2] if len(sys.argv) > 2 else "projects/bach-invention/output/erdfa"
    os.makedirs(out_dir, exist_ok=True)
    
    svgs = sorted(f for f in os.listdir(svg_dir) if f.endswith('.svg'))
    
    print(f"=== SVG → DA51 CBOR + FRACTRAN ===")
    print(f"  Input:  {svg_dir} ({len(svgs)} SVGs)")
    print(f"  Output: {out_dir}")
    print()
    
    all_shards = []
    
    for svg_file in svgs:
        path = os.path.join(svg_dir, svg_file)
        name = svg_file.replace('.svg', '')
        
        # Extract
        positions = extract_positions(path)
        notes = positions_to_notes(positions)
        intervals = notes_to_intervals(notes)
        state, counts, parts = notes_to_fractran(notes)
        grade, blade_mask = cl15_blade(intervals)
        
        # Orbifold
        orb = (state % 71, state % 59, state % 47) if state > 0 else (0,0,0)
        
        # DA51 CBOR
        cbor = encode_da51_cbor(name, notes, state, counts, grade, blade_mask)
        cbor_path = os.path.join(out_dir, f"{name}.cbor")
        open(cbor_path, 'wb').write(cbor)
        
        print(f"  {name}: {len(positions)} pos → {len(notes)} notes → "
              f"grade {grade} orb={orb} state={'×'.join(parts[:4])}{'...' if len(parts)>4 else ''} "
              f"→ {len(cbor)}B cbor")
        
        all_shards.append({
            "name": name,
            "positions": len(positions),
            "notes": len(notes),
            "intervals": len(intervals),
            "grade": grade,
            "blade_mask": blade_mask,
            "orbifold": orb,
            "cbor_size": len(cbor),
        })
    
    # Summary
    print(f"\n  {len(all_shards)} shards → {out_dir}/")
    
    # Save manifest
    manifest = os.path.join(out_dir, "manifest.json")
    with open(manifest, 'w') as f:
        json.dump(all_shards, f, indent=2)
    print(f"  manifest → {manifest}")

if __name__ == "__main__":
    main()
