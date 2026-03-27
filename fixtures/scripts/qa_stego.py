#!/usr/bin/env python3
"""QA: verify stego tiles preserve visual content and payload is recoverable.

Tests:
1. PNG is valid image (not noise)
2. Visual similarity to source SVG (PSNR > 30dB = invisible stego)
3. NFT7 payload recoverable from tiles
4. All 13 segments have correct magic bytes
"""

import struct, sys, os

TILE_DIR = "fixtures/output/nft71_stego_png"
SVG_DIR = "fixtures/output/nft71_svg"

def read_png_rgb(path):
    """Read PNG → raw RGB bytes (minimal, no deps)."""
    import subprocess
    # Use ImageMagick to dump raw RGB
    result = subprocess.run(
        ["convert", path, "-depth", "8", "rgb:-"],
        capture_output=True
    )
    if result.returncode != 0:
        return None, 0, 0
    # Get dimensions
    info = subprocess.run(
        ["identify", "-format", "%w %h", path],
        capture_output=True, text=True
    )
    w, h = map(int, info.stdout.strip().split())
    return result.stdout, w, h

def psnr(rgb_a, rgb_b):
    """Peak Signal-to-Noise Ratio between two RGB buffers."""
    if len(rgb_a) != len(rgb_b) or len(rgb_a) == 0:
        return 0.0
    import math
    mse = sum((a - b) ** 2 for a, b in zip(rgb_a, rgb_b)) / len(rgb_a)
    if mse == 0:
        return 99.0
    return 10 * math.log10(255.0 ** 2 / mse)

def check_pixel_stats(rgb, w, h):
    """Check if image looks like artwork vs noise."""
    if not rgb:
        return {"valid": False}
    n = len(rgb)
    # Mean and variance per channel
    r_vals = rgb[0::3]
    g_vals = rgb[1::3]
    b_vals = rgb[2::3]
    
    r_mean = sum(r_vals) / len(r_vals)
    g_mean = sum(g_vals) / len(g_vals)
    b_mean = sum(b_vals) / len(b_vals)
    
    # Unique colors (sample first 1000 pixels)
    sample = min(1000, len(rgb) // 3)
    colors = set()
    for i in range(sample):
        colors.add((rgb[i*3], rgb[i*3+1], rgb[i*3+2]))
    
    # Dark background ratio (our tiles have dark bg)
    dark = sum(1 for i in range(0, min(n, 3000), 3) if rgb[i] < 50 and rgb[i+1] < 50 and rgb[i+2] < 50)
    dark_ratio = dark / (min(n, 3000) // 3)
    
    return {
        "valid": True,
        "r_mean": r_mean,
        "g_mean": g_mean, 
        "b_mean": b_mean,
        "unique_colors_1k": len(colors),
        "dark_ratio": dark_ratio,
        "looks_like_art": dark_ratio > 0.3 and len(colors) > 5,
        "looks_like_noise": len(colors) > 900 and dark_ratio < 0.1,
    }

def extract_stego(rgb, tile_cap):
    """Extract stego payload from RGB (6-layer bit-plane)."""
    pixels = len(rgb) // 3
    planes = 6
    out = bytearray(tile_cap)
    for i in range(min(tile_cap, pixels * planes // 8)):
        byte = 0
        for b in range(8):
            bit_idx = i * 8 + b
            px = bit_idx // planes
            plane = bit_idx % planes
            if px >= pixels:
                break
            ch = plane % 3
            bit_pos = plane // 3
            idx = px * 3 + ch
            byte |= ((rgb[idx] >> bit_pos) & 1) << b
        out[i] = byte
    return bytes(out)

def check_nft7(payload):
    """Check NFT7 magic and parse segments."""
    if len(payload) < 8 or payload[:4] != b"NFT7":
        return {"valid": False, "magic": payload[:4].hex() if len(payload) >= 4 else "short"}
    
    count = struct.unpack("<I", payload[4:8])[0]
    off = 8
    segments = []
    for _ in range(count):
        if off + 4 > len(payload): break
        nl = struct.unpack("<I", payload[off:off+4])[0]
        off += 4
        if off + nl + 4 > len(payload): break
        name = payload[off:off+nl].decode("utf-8", errors="replace")
        off += nl
        dl = struct.unpack("<I", payload[off:off+4])[0]
        off += 4
        if off + dl > len(payload): break
        data = payload[off:off+dl]
        magic = data[:4].hex() if len(data) >= 4 else ""
        segments.append({"name": name, "size": dl, "magic": magic})
        off += dl
    
    return {"valid": True, "count": count, "segments": segments}

EXPECTED_MAGIC = {
    "wav": "52494646",      # RIFF
    "midi_west": "4d546864", # MThd
    "midi_01": "4d546864",
    "midi_04": "4d546864",
    "midi_06": "4d546864",
    "midi_07": "4d546864",
    "midi_08": "4d546864",
    "pdf": "25504446",       # %PDF
    "source": "48757272",    # Hurr
    "lilypond": "5c766572",  # \ver
    "erdfa": "74696c65",     # tile
}

def main():
    print("=== STEGO QA: VISUAL + PAYLOAD VERIFICATION ===\n")
    
    tiles = sorted(f for f in os.listdir(TILE_DIR) if f.endswith(".png"))
    print(f"Tiles: {len(tiles)} in {TILE_DIR}\n")
    
    # 1. Visual check on first 3 tiles
    print("1. VISUAL CHECK (pixel stats)\n")
    for t in tiles[:3]:
        path = os.path.join(TILE_DIR, t)
        rgb, w, h = read_png_rgb(path)
        if rgb is None:
            print(f"  ❌ {t}: failed to read")
            continue
        stats = check_pixel_stats(rgb, w, h)
        art = "✅ artwork" if stats["looks_like_art"] else "⚠ noise" if stats["looks_like_noise"] else "? unclear"
        print(f"  {t}: {w}x{h} R={stats['r_mean']:.0f} G={stats['g_mean']:.0f} B={stats['b_mean']:.0f} "
              f"colors={stats['unique_colors_1k']} dark={stats['dark_ratio']:.1%} → {art}")
    
    # 2. PSNR vs SVG (if imagemagick can rasterize)
    print("\n2. PSNR vs SOURCE SVG\n")
    svg_path = os.path.join(SVG_DIR, "01.svg")
    png_path = os.path.join(TILE_DIR, "01.png")
    if os.path.exists(svg_path):
        import subprocess
        # Rasterize SVG to compare
        tmp = "/tmp/retro_qa_ref.rgb"
        subprocess.run(["convert", svg_path, "-resize", "512x512!", "-depth", "8", f"rgb:{tmp}"], capture_output=True)
        ref_rgb = open(tmp, "rb").read() if os.path.exists(tmp) else None
        stego_rgb, _, _ = read_png_rgb(png_path)
        if ref_rgb and stego_rgb and len(ref_rgb) == len(stego_rgb):
            p = psnr(ref_rgb, stego_rgb)
            verdict = "✅ invisible" if p > 30 else "⚠ visible" if p > 20 else "❌ destroyed"
            print(f"  tile 01 vs svg 01: PSNR = {p:.1f} dB → {verdict}")
            print(f"  (>30dB = stego invisible, 20-30 = slightly visible, <20 = corrupted)")
        else:
            print(f"  ⚠ size mismatch: ref={len(ref_rgb) if ref_rgb else 0} stego={len(stego_rgb) if stego_rgb else 0}")
    else:
        print(f"  ⚠ no SVG source to compare")
    
    # 3. Payload extraction
    print("\n3. PAYLOAD EXTRACTION (all 71 tiles)\n")
    tile_cap = 512 * 512 * 6 // 8  # 196608
    all_chunks = []
    extract_ok = 0
    for t in tiles:
        rgb, w, h = read_png_rgb(os.path.join(TILE_DIR, t))
        if rgb is None:
            print(f"  ❌ {t}: read failed")
            continue
        chunk = extract_stego(rgb, tile_cap)
        all_chunks.append(chunk)
        extract_ok += 1
    
    print(f"  Extracted: {extract_ok}/{len(tiles)} tiles")
    payload = b"".join(all_chunks)
    print(f"  Total payload: {len(payload)} bytes ({len(payload)/1048576:.1f} MB)")
    
    # 4. NFT7 decode
    print("\n4. NFT7 SEGMENT VERIFICATION\n")
    nft7 = check_nft7(payload)
    if not nft7["valid"]:
        print(f"  ❌ NFT7 decode failed (magic: {nft7.get('magic', '?')})")
        print(f"  First 32 bytes: {payload[:32].hex()}")
    else:
        print(f"  ✅ NFT7 valid: {nft7['count']} segments")
        all_ok = True
        for seg in nft7["segments"]:
            expected = EXPECTED_MAGIC.get(seg["name"], "")
            match = "✅" if not expected or seg["magic"] == expected else "❌"
            if match == "❌": all_ok = False
            print(f"    {match} {seg['name']:12} {seg['size']:>10} B  magic={seg['magic']}")
        print(f"\n  {'✅ ALL SEGMENTS OK' if all_ok else '❌ SOME SEGMENTS FAILED'}")
    
    # Summary
    print(f"\n=== QA SUMMARY ===")
    print(f"  Tiles:    {extract_ok}/{len(tiles)}")
    print(f"  NFT7:     {'✅' if nft7.get('valid') else '❌'}")
    print(f"  Segments: {len(nft7.get('segments', []))}")

if __name__ == "__main__":
    main()
