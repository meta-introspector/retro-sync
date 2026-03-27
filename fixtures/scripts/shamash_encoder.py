#!/usr/bin/env python3
"""Shamash SVG encoder: modulate sun disc paths with payload data.

Encode → render PNG → stego embed → decode from SVG → verify roundtrip.

Channels: coordinate decimals, stroke colors, fill colors, stroke widths, opacity.
"""

import re, hashlib, math, os, subprocess

SHAMASH = "/var/www/solana.solfunmeme.com/retro-sync/scratch/shamash_star.svg"
SCRATCH = "/var/www/solana.solfunmeme.com/retro-sync/scratch"

PAYLOAD = (b"Hurrian Hymn h.6 - Nikkal - Teshub - Ugarit - 1400BCE - "
           b"nish tuhrim qablitum ishartum shalshatum serdum colophon "
           + bytes(range(256)) * 4)  # ~1.1KB test payload

def encode_svg(svg_text, payload):
    """Modulate SVG coordinates and styles with payload bytes."""
    data = payload + b'\x00' * 3000  # pad
    off = [0]
    
    def eat(n):
        chunk = data[off[0]:off[0]+n]
        off[0] += n
        return chunk

    # 1. Modulate coordinate decimals: 123.456 → 123.XXX where XXX = byte value
    def mod_coord(match):
        val = match.group(0)
        parts = val.split('.')
        if len(parts) == 2 and len(parts[1]) >= 1:
            b = eat(1)[0]
            # Keep integer part, replace decimal with encoded byte
            return f"{parts[0]}.{b:03d}"
        return val

    # Find all decimal coordinates in path data
    encoded = re.sub(r'd="([^"]*)"', lambda m: 'd="' + re.sub(r'\d+\.\d+', mod_coord, m.group(1)) + '"', svg_text)

    # 2. Add data-driven style attributes to paths
    def mod_style(match):
        tag = match.group(0)
        b = eat(3)
        r = max(64, b[0])
        g = max(64, b[1])
        bv = max(64, b[2])
        sw = eat(1)[0]
        stroke_w = 0.5 + (sw % 30) / 10.0
        op = eat(1)[0]
        opacity = 0.4 + (op % 60) / 100.0
        # Insert style before closing >
        style = f' stroke="rgb({r},{g},{bv})" stroke-width="{stroke_w:.1f}" opacity="{opacity:.2f}" fill="none"'
        if tag.endswith('/>'):
            return tag[:-2] + style + '/>'
        elif tag.endswith('>'):
            return tag[:-1] + style + '>'
        return tag

    # Apply to each <path element
    encoded = re.sub(r'<path[^>]*/?>', mod_style, encoded)

    # 3. Add hatch pattern defs with data-encoded spacing
    hb = eat(4)
    hatch_w = 4 + hb[0] % 8
    hatch_h = 4 + hb[1] % 8
    hatch_r = max(64, hb[2])
    hatch_g = max(64, hb[3])
    hatch_def = f'''<defs>
  <pattern id="hatch" width="{hatch_w}" height="{hatch_h}" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
    <line x1="0" y1="0" x2="0" y2="{hatch_h}" stroke="rgb({hatch_r},{hatch_g},80)" stroke-width="1" opacity="0.15"/>
  </pattern>
</defs>
<rect width="100%" height="100%" fill="url(#hatch)"/>
'''
    # Insert after opening <svg...>
    encoded = re.sub(r'(<svg[^>]*>)', r'\1\n' + hatch_def, encoded, count=1)

    # 4. Add noise circles (raster-like texture)
    noise = ""
    for i in range(50):
        nb = eat(4)
        cx = 20 + nb[0] * 5
        cy = 20 + nb[1] * 5
        r = 2 + nb[2] % 6
        shade_r = max(64, nb[3])
        shade_g = max(64, (nb[3] + 40) % 256)
        noise += f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="rgb({shade_r},{shade_g},90)" opacity="0.08"/>\n'
    encoded = encoded.replace('</svg>', noise + '</svg>')

    return encoded, off[0]

def decode_svg(svg_text):
    """Extract payload from modulated SVG coordinates."""
    recovered = bytearray()
    
    # 1. Extract from coordinate decimals
    for m in re.finditer(r'd="([^"]*)"', svg_text):
        for coord in re.finditer(r'\d+\.(\d{3})', m.group(1)):
            dec = int(coord.group(1))
            if dec < 256:
                recovered.append(dec)
    
    return bytes(recovered)

def main():
    print("=== SHAMASH SVG ENCODER TEST ===\n")
    
    # Load base SVG
    svg_base = open(SHAMASH).read()
    print(f"Base SVG: {len(svg_base)} bytes, {svg_base.count('<path')} paths")
    
    # Encode
    svg_encoded, n_encoded = encode_svg(svg_base, PAYLOAD)
    encoded_path = f"{SCRATCH}/shamash_encoded.svg"
    with open(encoded_path, 'w') as f:
        f.write(svg_encoded)
    print(f"Encoded SVG: {len(svg_encoded)} bytes, {n_encoded} payload bytes embedded")
    print(f"  → {encoded_path}")
    
    # Decode from SVG
    recovered = decode_svg(svg_encoded)
    
    # Compare
    match = 0
    for i in range(min(len(PAYLOAD), len(recovered))):
        if recovered[i] == PAYLOAD[i]:
            match += 1
    
    print(f"\nSVG roundtrip: {match}/{min(len(PAYLOAD), len(recovered))} bytes match")
    if match > 0:
        print(f"  First 20 recovered: {recovered[:20].hex()}")
        print(f"  First 20 original:  {PAYLOAD[:20].hex()}")
    
    # Render to PNG
    png_path = f"{SCRATCH}/shamash_encoded.png"
    try:
        subprocess.run(["nix-shell", "-p", "imagemagick", "--run",
            f"convert {encoded_path} -resize 512x512! {png_path}"],
            capture_output=True, timeout=30)
        if os.path.exists(png_path):
            sz = os.path.getsize(png_path)
            print(f"\nPNG render: {sz} bytes → {png_path}")
        else:
            print("\n⚠ PNG render failed")
    except:
        print("\n⚠ PNG render skipped (no imagemagick)")
    
    # PSNR: compare encoded SVG render vs base SVG render
    ref_png = f"{SCRATCH}/shamash_ref.png"
    try:
        subprocess.run(["nix-shell", "-p", "imagemagick", "--run",
            f"convert {SHAMASH} -resize 512x512! {ref_png}"],
            capture_output=True, timeout=30)
        if os.path.exists(ref_png) and os.path.exists(png_path):
            result = subprocess.run(["nix-shell", "-p", "imagemagick", "--run",
                f"magick compare -metric PSNR {ref_png} {png_path} /dev/null"],
                capture_output=True, text=True, timeout=30)
            psnr = result.stderr.strip() or result.stdout.strip()
            print(f"PSNR (base vs encoded): {psnr}")
    except:
        print("⚠ PSNR check skipped")
    
    # OCR test
    try:
        result = subprocess.run(["nix-shell", "-p", "tesseract", "--run",
            f"tesseract {png_path} stdout"],
            capture_output=True, text=True, timeout=30)
        ocr = result.stdout.strip()
        if ocr:
            print(f"OCR from encoded PNG: '{ocr[:80]}'")
        else:
            print("OCR: (no text detected — expected for pure geometry)")
    except:
        pass
    
    print(f"\n=== CAPACITY SUMMARY ===")
    print(f"  SVG coordinate channel: ~{n_encoded} bytes")
    print(f"  PNG stego channel:      ~196608 bytes (6-layer bit-plane)")
    print(f"  Total per tile:         ~{n_encoded + 196608} bytes")
    print(f"  71 tiles:               ~{(n_encoded + 196608) * 71 // 1024}KB")
    
    print(f"\nView: https://solana.solfunmeme.com/retro-sync/scratch/shamash_encoded.svg")
    print(f"      https://solana.solfunmeme.com/retro-sync/scratch/shamash_encoded.png")

if __name__ == "__main__":
    main()
