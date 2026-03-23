#!/usr/bin/env bash
# Strip gamma/chromaticity from stego PNGs so browsers don't apply color management.
# This preserves exact pixel values for LSB steganography.
set -euo pipefail

SRC="${1:-/var/www/solana.solfunmeme.com/retro-sync/tiles}"
echo "Stripping PNG metadata from $SRC/*.png"

for f in "$SRC"/*.png; do
    convert "$f" -strip PNG32:"$f"
    echo "  $(basename "$f")"
done

echo "Done. Verifying tile 01..."
python3 -c "
from PIL import Image; import numpy as np
img = Image.open('$SRC/01.png')
print('Info keys:', list(img.info.keys()))
px = np.array(img)
print('Shape:', px.shape)
flat = px.reshape(-1, px.shape[-1])
PLANES = 6
out = []
for i in range(4):
    byte = 0
    for b in range(8):
        bit_idx = i*8+b; pxi = bit_idx//PLANES; plane = bit_idx%PLANES
        ch = plane%3; bit_pos = plane//3
        byte |= ((int(flat[pxi][ch]) >> bit_pos) & 1) << b
    out.append(byte)
print('Magic:', bytes(out))
"
