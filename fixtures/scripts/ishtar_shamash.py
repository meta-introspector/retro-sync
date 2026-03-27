#!/usr/bin/env python3
"""Reconstruct Star of Ishtar and Shamash from first principles.

Layers (inside out):
1. Shamash solar disc (circle)
2. Shamash cross (4-pointed, cardinal directions)
3. Shamash wavy rays (4 wavy lines between cross arms)
4. Ishtar star (8-pointed, evenly spaced)

Each layer has its own symmetry group:
- Shamash disc: O(2) continuous rotation
- Shamash cross: C4 (4-fold rotational)
- Shamash rays: C4 × Z2 (4-fold + reflection in each quadrant)
- Ishtar star: C8 (8-fold rotational)

Data encoding: each symmetry-breaking parameter carries payload bytes.
"""

import math

SZ = 1024
CX, CY = SZ/2, SZ/2
SCRATCH = "/var/www/solana.solfunmeme.com/retro-sync/scratch"

PAYLOAD = (b"Hurrian Hymn h.6 Nikkal Teshub Shamash Ishtar Ugarit 1400BCE "
           + bytes(range(256)))

def eat_bytes(data, offset, n):
    chunk = data[offset[0]:offset[0]+n]
    offset[0] += n
    return chunk

def shamash_disc(cx, cy, r, data, off):
    """Layer 1: Solar disc — circle with data in radius precision."""
    b = eat_bytes(data, off, 2)
    r_mod = r + (b[0] - 128) * 0.01  # sub-pixel radius modulation
    fill_r = max(80, 200 + (b[1] % 40) - 20)
    return f'<circle cx="{cx}" cy="{cy}" r="{r_mod:.3f}" fill="rgb({fill_r},180,60)" opacity="0.9"/>\n'

def shamash_cross(cx, cy, r_inner, r_outer, data, off):
    """Layer 2: 4-pointed cross — C4 symmetry, data in arm widths."""
    svg = ""
    for i in range(4):
        b = eat_bytes(data, off, 2)
        angle = i * math.pi / 2
        w = 8 + b[0] % 12  # arm width: 8-19
        shade = max(80, 160 + b[1] % 60 - 30)
        
        x1 = cx + math.cos(angle) * r_inner
        y1 = cy + math.sin(angle) * r_inner
        x2 = cx + math.cos(angle) * r_outer
        y2 = cy + math.sin(angle) * r_outer
        
        svg += (f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" '
                f'stroke="rgb({shade},{shade-20},{shade-60})" stroke-width="{w}" '
                f'stroke-linecap="round"/>\n')
    return svg

def shamash_wavy_rays(cx, cy, r_inner, r_outer, data, off):
    """Layer 3: Wavy rays — ONE wave shape, rotated 8× around center (C8).
    3 copies per direction (spread), all identical wave form.
    Data in: wave amplitude, frequency, phase (shared across all rays)."""
    svg = ""
    # One set of wave params for ALL rays (identical waves)
    b = eat_bytes(data, off, 4)
    amplitude = 12 + b[0] % 25
    freq = 3 + b[1] % 5
    phase = b[2] * 0.02
    shade_r = max(80, 200 + b[3] % 40 - 20)
    
    for i in range(8):  # 8-fold rotation
        for copy in range(3):  # 3 copies per direction
            spread = (copy - 1) * 0.08  # tight spread
            base_angle = i * math.pi / 4 + math.pi / 8 + spread  # offset by half-step to sit between star points
            
            pts = []
            n_pts = 30
            for j in range(n_pts):
                t = j / (n_pts - 1)
                r = r_inner + (r_outer - r_inner) * t
                wave = amplitude * math.sin(freq * t * math.pi * 2 + phase) * (1 - t * 0.3)
                
                perp = base_angle + math.pi / 2
                x = cx + math.cos(base_angle) * r + math.cos(perp) * wave
                y = cy + math.sin(base_angle) * r + math.sin(perp) * wave
                pts.append(f"{x:.1f} {y:.1f}")
            
            d = "M" + " L".join(pts)
            w = 3 - copy * 0.5
            svg += (f'<path d="{d}" fill="none" stroke="rgb({shade_r},170,50)" '
                    f'stroke-width="{w:.1f}" stroke-linecap="round" opacity="{0.7 - copy*0.1:.1f}"/>\n')
    return svg
    return svg

def ishtar_star(cx, cy, r_inner, r_outer, data, off):
    """Layer 4: 8-pointed star — C8 symmetry.
    Data in: point sharpness, inner/outer ratio per point."""
    b = eat_bytes(data, off, 8)
    
    pts = []
    for i in range(16):
        angle = i * math.pi / 8
        if i % 2 == 0:
            # Outer point
            r = r_outer + (b[i // 2] % 20) - 10  # data modulates tip
        else:
            # Inner notch
            r = r_inner + (b[i // 2] % 10) - 5
        x = cx + math.cos(angle) * r
        y = cy + math.sin(angle) * r
        pts.append(f"{x:.1f},{y:.1f}")
    
    return (f'<polygon points="{" ".join(pts)}" '
            f'fill="none" stroke="rgb(200,180,80)" stroke-width="3" '
            f'stroke-linejoin="miter" opacity="0.8"/>\n')

def annular_ring(cx, cy, r, width, data, off):
    """Annular ring between layers — data in width modulation."""
    b = eat_bytes(data, off, 1)
    w = width + (b[0] % 4) - 2
    shade = max(80, 150 + b[0] % 40)
    return (f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="none" '
            f'stroke="rgb({shade},{shade-10},{shade-40})" stroke-width="{w}" opacity="0.6"/>\n')

def main():
    data = PAYLOAD + b'\x80' * 500
    off = [0]
    
    svg = f'''<svg xmlns="http://www.w3.org/2000/svg" width="{SZ}" height="{SZ}" viewBox="0 0 {SZ} {SZ}">
<rect width="{SZ}" height="{SZ}" fill="#3a3020"/>
'''
    
    # Build inside-out
    # Layer 4: Ishtar 8-pointed star (outermost)
    svg += '<!-- Ishtar 8-pointed star (C8 symmetry) -->\n'
    svg += ishtar_star(CX, CY, 200, 400, data, off)
    
    # Outer annular ring
    svg += annular_ring(CX, CY, 300, 4, data, off)
    
    # Layer 3: Shamash wavy rays (between cross arms)
    svg += '<!-- Shamash wavy rays (C4 × Z2) -->\n'
    svg += shamash_wavy_rays(CX, CY, 80, 480, data, off)
    
    # Inner annular ring
    svg += annular_ring(CX, CY, 180, 3, data, off)
    
    # Layer 2: No crossbars in original — skip
    # svg += shamash_cross(CX, CY, 60, 280, data, off)
    
    # Layer 1: Shamash solar disc (center)
    svg += '<!-- Shamash solar disc (O(2) symmetry) -->\n'
    svg += shamash_disc(CX, CY, 70, data, off)
    
    # Center eye
    svg += f'<circle cx="{CX}" cy="{CY}" r="15" fill="#3a3020" opacity="0.8"/>\n'
    
    svg += '</svg>\n'
    
    out_path = f"{SCRATCH}/ishtar_shamash.svg"
    with open(out_path, 'w') as f:
        f.write(svg)
    
    print(f"=== STAR OF ISHTAR AND SHAMASH ===\n")
    print(f"Layers (inside out):")
    print(f"  1. Shamash disc    — O(2)  — circle, data in radius")
    print(f"  2. Shamash cross   — C4    — 4 arms, data in widths")
    print(f"  3. Shamash rays    — C4×Z2 — 4 wavy lines, data in amplitude/freq/phase")
    print(f"  4. Ishtar star     — C8    — 8 points, data in tip positions")
    print(f"  + 2 annular rings  — O(2)  — data in width")
    print(f"\n  Bytes encoded: {off[0]}")
    print(f"  Symmetry groups: O(2) ⊂ C4 ⊂ C4×Z2 ⊂ C8")
    print(f"  Nested: Justice (C4) inside Desire (C8)")
    print(f"\n  → {out_path}")
    print(f"  View: https://solana.solfunmeme.com/retro-sync/scratch/ishtar_shamash.svg")

if __name__ == "__main__":
    main()
