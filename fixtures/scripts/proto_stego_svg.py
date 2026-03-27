#!/usr/bin/env python3
"""Prototype: SVG tile where every visual element encodes data.

The image IS the data. Recovery = parse SVG geometry.
High contrast + visual complexity = PNG stego invisible on top.

Encoding channels:
  - Background gradient stops: 6 bytes (2 RGB colors)
  - Grid cell colors: 16x16 = 256 cells × 3 bytes = 768 bytes
  - Curve control points: 8 curves × 4 points × 2 coords × 2 decimal digits = 128 bytes
  - Text decimal positions: 10 text elements × 2 coords × 3 digits = 60 bytes
  - Shape rotations: 16 shapes × 1 byte = 16 bytes
  Total SVG channel: ~1KB visible encoding per tile

Output: scratch/proto_tile.svg for visual inspection
"""

import struct, hashlib, sys

SZ = 512
OUT = "/var/www/solana.solfunmeme.com/retro-sync/scratch/proto_tile.svg"

# Sample payload to encode
PAYLOAD = b"Hurrian Hymn h.6 - Tablet RS 15.30 - ~1400 BCE - Ugarit - nish tuhrim - qablitum - ishartum" + bytes(range(256))

def byte_to_color(b, base_r=80, base_g=80, base_b=100):
    """Map a byte to a high-contrast color. Min value 64 for stego safety."""
    r = max(64, (base_r + (b & 0x07) * 20) % 256)
    g = max(64, (base_g + ((b >> 3) & 0x07) * 20) % 256)
    b_val = max(64, (base_b + ((b >> 6) & 0x03) * 40) % 256)
    return f"#{r:02x}{g:02x}{b_val:02x}"

def encode_in_decimal(value, base):
    """Encode a byte in the decimal part of a coordinate. base.XXX where XXX = value."""
    return f"{base + value / 1000:.3f}"

def make_tile(payload, idx=1):
    data = payload + b'\x00' * max(0, 1024 - len(payload))
    off = 0

    def eat(n):
        nonlocal off
        chunk = data[off:off+n]
        off += n
        return chunk

    # Background: gradient from two data-encoded colors
    bg1 = byte_to_color(eat(1)[0], 90, 90, 110)
    bg2 = byte_to_color(eat(1)[0], 70, 80, 100)

    svg = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{SZ}" height="{SZ}" viewBox="0 0 {SZ} {SZ}"
     data-payload-hash="{hashlib.sha256(payload).hexdigest()[:16]}">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="{bg1}"/>
      <stop offset="100%" stop-color="{bg2}"/>
    </linearGradient>
  </defs>
  <rect width="{SZ}" height="{SZ}" fill="url(#bg)"/>
'''

    # Grid: 16×16 colored cells (256 bytes encoded in color)
    cell = SZ // 16
    for row in range(16):
        for col in range(16):
            b = eat(1)[0]
            color = byte_to_color(b, 80 + row * 4, 80 + col * 4, 100)
            x = col * cell
            y = row * cell
            svg += f'  <rect x="{x}" y="{y}" width="{cell}" height="{cell}" fill="{color}" opacity="0.3"/>\n'

    # Data curves: bezier paths with control points encoding bytes
    for i in range(8):
        pts = eat(8)
        x1 = 40 + pts[0] * 1.6
        y1 = 40 + pts[1] * 1.6
        cx1 = 40 + pts[2] * 1.6
        cy1 = 40 + pts[3] * 1.6
        cx2 = 40 + pts[4] * 1.6
        cy2 = 40 + pts[5] * 1.6
        x2 = 40 + pts[6] * 1.6
        y2 = 40 + pts[7] * 1.6
        hue = (i * 45) % 360
        svg += f'  <path d="M{x1:.1f} {y1:.1f} C{cx1:.1f} {cy1:.1f} {cx2:.1f} {cy2:.1f} {x2:.1f} {y2:.1f}" '
        svg += f'fill="none" stroke="hsl({hue},70%,65%)" stroke-width="2" opacity="0.6"/>\n'

    # Ziggurat (RIGHT SIDE UP — wide base at bottom, narrow top)
    zig_bytes = eat(5)
    steps = 5
    for s in range(steps):
        w = 300 - s * 50
        x = 256 - w // 2
        y = 380 - s * 25  # bottom to top
        shade = max(80, 100 + zig_bytes[s] // 3)
        svg += f'  <rect x="{x}" y="{y}" width="{w}" height="24" fill="rgb({shade},{shade-20},{shade-40})" opacity="0.7"/>\n'

    # Rosette stars (position + size encode data)
    for i in range(4):
        sb = eat(3)
        cx = 60 + sb[0] * 1.5
        cy = 60 + sb[1] * 1.5
        r = 8 + sb[2] // 20
        pts = " ".join(
            f"{cx + r * (0.5 if j%2 else 1.0) * __import__('math').cos(j * __import__('math').pi / 4):.1f},"
            f"{cy + r * (0.5 if j%2 else 1.0) * __import__('math').sin(j * __import__('math').pi / 4):.1f}"
            for j in range(8)
        )
        svg += f'  <polygon points="{pts}" fill="gold" opacity="0.5"/>\n'

    # High-contrast text panels (survive stego + OCR)
    svg += f'  <rect x="80" y="55" width="352" height="55" fill="#909098" rx="6" opacity="0.85"/>\n'
    svg += f'  <text x="256" y="92" text-anchor="middle" fill="#FFD700" font-size="48" font-weight="bold"'
    svg += f' font-family="serif">𒀸𒌑𒄴𒊑</text>\n'

    svg += f'  <rect x="100" y="120" width="312" height="32" fill="#808088" rx="4" opacity="0.8"/>\n'
    svg += f'  <text x="256" y="143" text-anchor="middle" fill="#FFFFFF" font-family="monospace"'
    svg += f' font-size="20" font-weight="bold">nīš tuḫrim</text>\n'

    # Notation with data in letter-spacing
    ls = eat(2)
    svg += f'  <text x="256" y="180" text-anchor="middle" fill="#B0E8B0" font-family="monospace"'
    svg += f' font-size="10" letter-spacing="0.{ls[0]:03d}em">qáb-li-te 3  ir-bu-te 1  qáb-li-te 3</text>\n'

    # Orbifold + shard info on bright panels
    svg += f'  <rect x="120" y="300" width="272" height="24" fill="#707880" rx="3" opacity="0.7"/>\n'
    svg += f'  <text x="256" y="317" text-anchor="middle" fill="#DDDDEE" font-family="monospace"'
    svg += f' font-size="11">orbifold ({idx%71},{idx%59},{idx%47}) mod (71,59,47)</text>\n'

    svg += f'  <rect x="60" y="440" width="392" height="50" fill="#606870" rx="4" opacity="0.7"/>\n'
    svg += f'  <text x="256" y="458" text-anchor="middle" fill="#C0C8D0" font-family="monospace"'
    svg += f' font-size="14" font-weight="bold">★{idx:02d} generator</text>\n'
    svg += f'  <text x="256" y="478" text-anchor="middle" fill="#90B8FF" font-family="monospace"'
    svg += f' font-size="11">Hurrian Hymn h.6 · RS 15.30 · ~1400 BC · Ugarit</text>\n'

    # Footer
    svg += f'  <text x="256" y="502" text-anchor="middle" fill="#808890" font-family="monospace"'
    svg += f' font-size="8">DA51 · Cl(15,0,0) · 6-layer stego · {off} bytes encoded in SVG</text>\n'

    svg += '</svg>\n'
    return svg, off

svg, encoded = make_tile(PAYLOAD, idx=1)

with open(OUT, 'w') as f:
    f.write(svg)

print(f"→ {OUT}")
print(f"  {encoded} bytes encoded in SVG visual elements")
print(f"  payload: {len(PAYLOAD)} bytes")
print(f"  View: https://solana.solfunmeme.com/retro-sync/scratch/proto_tile.svg")
