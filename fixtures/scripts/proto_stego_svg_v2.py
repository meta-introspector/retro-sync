#!/usr/bin/env python3
"""Stego SVG tile v2: Hurrian Bronze Age art style.

Visual motifs from cylinder seals, Khabur ware, deity symbols.
Data encoded in: grid colors, curve control points, shape positions.
Text on dark panels for readability. Stego-safe (min pixel value 64).
"""

import math, hashlib

SZ = 512
OUT = "/var/www/solana.solfunmeme.com/retro-sync/scratch/proto_tile_v2.svg"

PAYLOAD = b"Hurrian Hymn h.6 - Tablet RS 15.30 - Nikkal - Teshub - Ugarit" + bytes(range(256))

def byte_color(b, base_r=80, base_g=70, base_b=60):
    r = max(64, (base_r + (b & 0x07) * 18) % 220)
    g = max(64, (base_g + ((b >> 3) & 0x07) * 18) % 200)
    bv = max(64, (base_b + ((b >> 6) & 0x03) * 35) % 200)
    return f"#{r:02x}{g:02x}{bv:02x}"

def make_tile(payload, idx=1):
    data = payload + b'\x00' * max(0, 1024 - len(payload))
    off = 0
    def eat(n):
        nonlocal off
        c = data[off:off+n]; off += n; return c

    # Warm bronze/terracotta palette
    bg1 = byte_color(eat(1)[0], 100, 80, 60)
    bg2 = byte_color(eat(1)[0], 70, 55, 45)

    svg = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{SZ}" height="{SZ}" viewBox="0 0 {SZ} {SZ}">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{bg1}"/>
      <stop offset="100%" stop-color="{bg2}"/>
    </linearGradient>
  </defs>
  <rect width="{SZ}" height="{SZ}" fill="url(#bg)"/>
'''

    # Khabur ware geometric border (data in spacing)
    for i in range(32):
        b = eat(1)[0]
        x = 8 + i * 16 + (b & 3)
        w = 4 + (b >> 6)
        shade = max(100, 140 + (b >> 2) % 60)
        svg += f'  <rect x="{x}" y="4" width="{w}" height="6" fill="rgb({shade},{shade-30},{shade-50})"/>\n'
        svg += f'  <rect x="{x}" y="502" width="{w}" height="6" fill="rgb({shade},{shade-30},{shade-50})"/>\n'

    # Side borders — cylinder seal pattern (vertical zigzag)
    for i in range(24):
        b = eat(1)[0]
        y = 14 + i * 20 + (b & 7)
        svg += f'  <rect x="4" y="{y}" width="6" height="{6+(b>>5)}" fill="rgb({130+(b>>3)%40},{100+(b>>1)%30},{80})"/>\n'
        svg += f'  <rect x="502" y="{y}" width="6" height="{6+(b>>5)}" fill="rgb({130+(b>>3)%40},{100+(b>>1)%30},{80})"/>\n'

    # 12×12 color grid (144 bytes) — Khabur ware mosaic
    cell = 28
    ox, oy = 76, 190
    for row in range(6):
        for col in range(12):
            b = eat(1)[0]
            color = byte_color(b, 90 + row * 8, 75 + col * 5, 60)
            x = ox + col * cell
            y = oy + row * cell
            svg += f'  <rect x="{x}" y="{y}" width="{cell-1}" height="{cell-1}" fill="{color}" rx="2"/>\n'

    # Sun-and-crescent (Shimegi + Kushuh) — top center
    sb = eat(4)
    sun_r = 20 + sb[0] // 15
    svg += f'  <circle cx="256" cy="42" r="{sun_r}" fill="#E8C840" opacity="0.8"/>\n'
    svg += f'  <circle cx="{256+sun_r-8}" cy="38" r="{sun_r-4}" fill="{bg1}"/>\n'  # crescent cutout
    # Sun rays (data in angles)
    for i in range(12):
        a = math.pi * 2 * i / 12 + sb[1] * 0.01
        x1 = 256 + math.cos(a) * (sun_r + 4)
        y1 = 42 + math.sin(a) * (sun_r + 4)
        x2 = 256 + math.cos(a) * (sun_r + 10 + sb[2] // 30)
        y2 = 42 + math.sin(a) * (sun_r + 10 + sb[2] // 30)
        svg += f'  <line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" stroke="#E8C840" stroke-width="1.5" opacity="0.6"/>\n'

    # Winged figure silhouettes (left and right, data in wing span)
    for side in [-1, 1]:
        wb = eat(3)
        cx = 256 + side * 180
        # Body
        svg += f'  <ellipse cx="{cx}" cy="100" rx="8" ry="18" fill="#C0A070" opacity="0.5"/>\n'
        # Wings (bezier, data in control points)
        wing_span = 25 + wb[0] // 8
        wy = 85 + wb[1] // 20
        svg += f'  <path d="M{cx} 90 Q{cx+side*wing_span} {wy} {cx+side*wing_span*1.5:.0f} 95" fill="none" stroke="#C0A070" stroke-width="1.5" opacity="0.5"/>\n'
        svg += f'  <path d="M{cx} 95 Q{cx+side*wing_span} {wy+8} {cx+side*wing_span*1.3:.0f} 105" fill="none" stroke="#C0A070" stroke-width="1" opacity="0.4"/>\n'

    # Ziggurat (wide base at bottom, narrow top)
    zig = eat(5)
    for s in range(5):
        w = 280 - s * 45
        x = 256 - w // 2
        y = 430 - s * 18
        shade = max(90, 110 + zig[s] // 3)
        svg += f'  <rect x="{x}" y="{y}" width="{w}" height="17" fill="rgb({shade},{shade-15},{shade-35})" rx="1"/>\n'

    # Cuneiform on dark panel
    svg += f'  <rect x="100" y="68" width="312" height="52" fill="#1a1510" rx="5" opacity="0.85"/>\n'
    svg += f'  <text x="256" y="103" text-anchor="middle" fill="#FFD700" font-size="42" font-weight="bold">𒀸𒌑𒄴𒊑</text>\n'

    # Interval name on dark panel
    svg += f'  <rect x="120" y="128" width="272" height="28" fill="#1a1510" rx="4" opacity="0.8"/>\n'
    svg += f'  <text x="256" y="149" text-anchor="middle" fill="#E8D8B0" font-family="monospace" font-size="18" font-weight="bold">nīš tuḫrim</text>\n'

    # Notation
    svg += f'  <text x="256" y="175" text-anchor="middle" fill="#C0B890" font-family="monospace" font-size="9">qáb-li-te 3 · ir-bu-te 1 · qáb-li-te 3 · ša-aḫ-ri 1 · i-šar-te 10</text>\n'

    # Orbifold on dark panel
    svg += f'  <rect x="130" y="365" width="252" height="22" fill="#1a1510" rx="3" opacity="0.75"/>\n'
    svg += f'  <text x="256" y="381" text-anchor="middle" fill="#B0B8C0" font-family="monospace" font-size="10">orbifold ({idx%71},{idx%59},{idx%47}) · shard {idx}/71</text>\n'

    # Bottom info on dark panel
    svg += f'  <rect x="50" y="455" width="412" height="40" fill="#1a1510" rx="4" opacity="0.8"/>\n'
    svg += f'  <text x="256" y="472" text-anchor="middle" fill="#D0C8B0" font-family="monospace" font-size="12" font-weight="bold">Hurrian Hymn h.6 · Tablet RS 15.30 · ~1400 BC</text>\n'
    svg += f'  <text x="256" y="488" text-anchor="middle" fill="#A09880" font-family="monospace" font-size="9">DA51 · Cl(15,0,0) · Nikkal · Ugarit · {off}B in SVG</text>\n'

    svg += '</svg>\n'
    return svg, off

svg, encoded = make_tile(PAYLOAD, idx=1)
with open(OUT, 'w') as f:
    f.write(svg)

print(f"→ {OUT}")
print(f"  {encoded} bytes encoded in SVG geometry")
print(f"  View: https://solana.solfunmeme.com/retro-sync/scratch/proto_tile_v2.svg")
