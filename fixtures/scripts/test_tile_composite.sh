#!/bin/bash
# test_tile_composite.sh — Debug tile compositing pipeline
# Outputs each stage to scratch/ for visual inspection

set -e
S=/var/www/solana.solfunmeme.com/retro-sync/scratch
cd /mnt/data1/time-2026/03-march/20/retro-sync

echo "=== Stage 1: SVG source ==="
cp fixtures/output/nft71_svg/01.svg $S/debug_01_svg.svg
echo "  $S/debug_01_svg.svg"

echo "=== Stage 2: Photo background ==="
cp fixtures/output/nft71_bg/01.png $S/debug_02_bg.png
echo "  $S/debug_02_bg.png"

echo "=== Stage 3: SVG rasterized by resvg ==="
python3 -c "
import ctypes, os
# Use imagemagick as fallback to rasterize SVG
os.system('convert fixtures/output/nft71_svg/01.svg -resize 512x512! $S/debug_03_svg_raster.png 2>/dev/null')
"
echo "  $S/debug_03_svg_raster.png"

echo "=== Stage 4: Final composited stego tile ==="
cp fixtures/output/nft71_stego_png/01.png $S/debug_04_final.png
echo "  $S/debug_04_final.png"

echo "=== Stage 5: Pixel stats ==="
python3 -c "
import struct
# Read final tile
f = open('$S/debug_04_final.png', 'rb')
data = f.read()
f.close()
print(f'  Final PNG: {len(data)} bytes')

# Read bg tile
f = open('$S/debug_02_bg.png', 'rb')
bg = f.read()
f.close()
print(f'  BG PNG: {len(bg)} bytes')

# Check if they're identical (meaning SVG overlay had no effect)
if data == bg:
    print('  ❌ IDENTICAL — SVG overlay not applied!')
else:
    # Count differing bytes
    diff = sum(1 for a,b in zip(data, bg) if a != b)
    print(f'  Diff bytes: {diff} ({diff*100//len(data)}%)')
    if diff < len(data) * 0.01:
        print('  ⚠ Very few differences — overlay barely visible')
    else:
        print('  ✅ Significant differences — overlay applied')
"

echo ""
echo "View all stages:"
echo "  https://solana.solfunmeme.com/retro-sync/scratch/debug_01_svg.svg"
echo "  https://solana.solfunmeme.com/retro-sync/scratch/debug_02_bg.png"
echo "  https://solana.solfunmeme.com/retro-sync/scratch/debug_03_svg_raster.png"
echo "  https://solana.solfunmeme.com/retro-sync/scratch/debug_04_final.png"
