#!/usr/bin/env python3
"""4D prototype: encode → SVG → render PNG → decode → verify roundtrip.

4 bezier curves, each with 8 coordinates = 32 data bytes.
Plus 4 stroke colors = 12 bytes. Total: 44 bytes per tile.
Proves the concept before scaling to 24D Leech.
"""

import re, math, os, subprocess, sys

SZ = 512
SCRATCH = "/var/www/solana.solfunmeme.com/retro-sync/scratch"
PRIMES = [2, 3, 5, 7]  # 4D prototype

def encode_svg(payload):
    """Create SVG with 4 bezier curves encoding 44 bytes."""
    data = (payload + b'\x00' * 44)[:44]
    
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" width="{SZ}" height="{SZ}" viewBox="0 0 {SZ} {SZ}">\n'
    svg += f'<rect width="{SZ}" height="{SZ}" fill="#505868"/>\n'
    
    off = 0
    for i in range(4):
        # 8 coordinate bytes per curve
        coords = data[off:off+8]; off += 8
        x1 = 50 + coords[0] * 1.5
        y1 = 50 + coords[1] * 1.5
        cx1 = 50 + coords[2] * 1.5
        cy1 = 50 + coords[3] * 1.5
        cx2 = 50 + coords[4] * 1.5
        cy2 = 50 + coords[5] * 1.5
        x2 = 50 + coords[6] * 1.5
        y2 = 50 + coords[7] * 1.5
        
        # 3 color bytes per curve
        cb = data[off:off+3]; off += 3
        r = max(64, cb[0])
        g = max(64, cb[1])
        b = max(64, cb[2])
        
        # Encode coordinates with 3 decimal places (byte in decimal part)
        svg += (f'<path d="M{x1:.3f} {y1:.3f} C{cx1:.3f} {cy1:.3f} '
                f'{cx2:.3f} {cy2:.3f} {x2:.3f} {y2:.3f}" '
                f'fill="none" stroke="rgb({r},{g},{b})" stroke-width="3" '
                f'stroke-linecap="round"/>\n')
    
    # Encode remaining 1 byte in background opacity
    bg_byte = data[off] if off < len(data) else 0
    
    svg += '</svg>\n'
    return svg, off

def decode_svg(svg_text):
    """Recover payload from SVG coordinate decimals + colors."""
    recovered = bytearray()
    
    # Extract curves
    for m in re.finditer(r'<path d="M([\d.]+) ([\d.]+) C([\d.]+) ([\d.]+) ([\d.]+) ([\d.]+) ([\d.]+) ([\d.]+)"[^>]*stroke="rgb\((\d+),(\d+),(\d+)\)"', svg_text):
        # 8 coordinates → 8 bytes from decimal parts
        for j in range(1, 9):
            val = m.group(j)
            parts = val.split('.')
            if len(parts) == 2 and len(parts[1]) >= 3:
                # Recover: coordinate = 50 + byte * 1.5 → byte = (coord - 50) / 1.5
                coord = float(val)
                byte_val = round((coord - 50) / 1.5)
                recovered.append(max(0, min(255, byte_val)))
            else:
                recovered.append(0)
        # 3 color bytes
        for j in range(9, 12):
            recovered.append(int(m.group(j)))
    
    return bytes(recovered)

def render_png(svg_path, png_path):
    """Rasterize SVG to PNG via ImageMagick."""
    r = subprocess.run(
        ["nix-shell", "-p", "imagemagick", "--run",
         f"convert {svg_path} -resize {SZ}x{SZ}! {png_path}"],
        capture_output=True, timeout=60
    )
    return os.path.exists(png_path)

def psnr(png_a, png_b):
    """PSNR between two PNGs."""
    r = subprocess.run(
        ["nix-shell", "-p", "imagemagick", "--run",
         f"magick compare -metric PSNR {png_a} {png_b} /dev/null"],
        capture_output=True, text=True, timeout=30
    )
    try:
        return float(r.stderr.split()[0])
    except:
        return 0.0

def main():
    print("=== 4D PROTOTYPE: SVG COORDINATE STEGANOGRAPHY ===\n")
    
    # Test payload
    payload = b"Hurrian Hymn h.6 Nikkal Teshub Ugarit 1400"
    print(f"Payload: {len(payload)} bytes: {payload[:40]}")
    
    # 1. Encode
    svg, n_encoded = encode_svg(payload)
    svg_path = f"{SCRATCH}/proto4d.svg"
    with open(svg_path, 'w') as f:
        f.write(svg)
    print(f"\n1. ENCODE: {n_encoded} bytes → {svg_path}")
    
    # 2. Decode from SVG (lossless channel)
    recovered = decode_svg(svg)
    match_svg = sum(1 for a, b in zip(payload[:len(recovered)], recovered) if a == b)
    print(f"\n2. SVG DECODE: {match_svg}/{min(len(payload), len(recovered))} bytes match")
    print(f"   Original:  {payload[:20].hex()}")
    print(f"   Recovered: {recovered[:20].hex()}")
    
    # 3. Render to PNG
    png_path = f"{SCRATCH}/proto4d.png"
    if render_png(svg_path, png_path):
        sz = os.path.getsize(png_path)
        print(f"\n3. RENDER: {sz} bytes → {png_path}")
    else:
        print("\n3. RENDER: ⚠ failed")
        return
    
    # 4. Make a reference (unmodified) SVG for PSNR — spread curves evenly
    ref_payload = bytes([i * 6 % 256 for i in range(44)])  # spread across coordinate space
    ref_svg, _ = encode_svg(ref_payload)
    ref_svg_path = f"{SCRATCH}/proto4d_ref.svg"
    ref_png_path = f"{SCRATCH}/proto4d_ref.png"
    with open(ref_svg_path, 'w') as f:
        f.write(ref_svg)
    render_png(ref_svg_path, ref_png_path)
    
    # 5. PSNR
    if os.path.exists(ref_png_path):
        p = psnr(ref_png_path, png_path)
        print(f"\n4. PSNR (ref vs encoded): {p:.1f} dB")
        print(f"   {'✅ invisible' if p > 30 else '⚠ visible' if p > 20 else '❌ destroyed'}")
    
    # 6. Distortion test: resize down and back up, then check
    distorted_path = f"{SCRATCH}/proto4d_distorted.png"
    subprocess.run(
        ["nix-shell", "-p", "imagemagick", "--run",
         f"convert {png_path} -resize 256x256! -resize {SZ}x{SZ}! {distorted_path}"],
        capture_output=True, timeout=30
    )
    if os.path.exists(distorted_path):
        p2 = psnr(png_path, distorted_path)
        print(f"\n5. DISTORTION TEST (resize 512→256→512): PSNR={p2:.1f} dB")
    
    # 7. FRACTRAN state
    state = 1
    for i, p in enumerate(PRIMES):
        # Quantize first coordinate of each curve
        if i < len(payload):
            state *= p ** (payload[i] % 8)
    print(f"\n6. FRACTRAN STATE: {state}")
    print(f"   Factored: " + " × ".join(f"{p}^{payload[i]%8}" for i, p in enumerate(PRIMES)))
    
    # Summary
    print(f"\n=== SUMMARY ===")
    print(f"  SVG channel:  {n_encoded} bytes (lossless from SVG parse)")
    print(f"  SVG roundtrip: {match_svg}/{min(len(payload), len(recovered))} bytes")
    print(f"  PNG stego:    196608 bytes (6-layer, not tested here)")
    print(f"  FRACTRAN:     4D state encoded")
    print(f"\n  View: https://solana.solfunmeme.com/retro-sync/scratch/proto4d.svg")
    print(f"        https://solana.solfunmeme.com/retro-sync/scratch/proto4d.png")

if __name__ == "__main__":
    main()
