#!/usr/bin/env bash
set -euo pipefail
# stego-build.sh — Build stego tiles for a project directory
# Usage: stego-build.sh <project_dir>
# Reads project.toml, bundles MIDIs, calls cargo stego encoder, verifies.

PROJECT_DIR="${1:?Usage: stego-build.sh <project_dir>}"
PROJECT_DIR=$(realpath "$PROJECT_DIR")

SVG_DIR="$PROJECT_DIR/output/svg"
STEGO_DIR="$PROJECT_DIR/output/stego"
MIDI_DIR="$PROJECT_DIR/midi"
PAYLOAD="$PROJECT_DIR/output/payload.bin"

[ -f "$PROJECT_DIR/project.toml" ] || { echo "❌ No project.toml in $PROJECT_DIR"; exit 1; }
[ -d "$SVG_DIR" ] || { echo "❌ No SVGs in $SVG_DIR — run midi2svg.sh first"; exit 1; }

mkdir -p "$STEGO_DIR"

echo "=== STEGO BUILD: $(basename "$PROJECT_DIR") ==="
echo

# 1. Bundle MIDIs as payload
echo "1. Bundling MIDIs..."
cat "$MIDI_DIR"/*.mid > "$PAYLOAD" 2>/dev/null || true
echo "   payload: $(stat -c%s "$PAYLOAD" 2>/dev/null || stat -f%z "$PAYLOAD") bytes"

# 2. Build NFT7 container with MIDI bundle + project.toml as segments
echo "2. Building NFT7 payload..."
python3 -c "
import struct, os, sys

project_dir = '$PROJECT_DIR'
payload_path = '$PAYLOAD'
toml_path = os.path.join(project_dir, 'project.toml')

segments = []

# MIDI bundle
midi_data = open(payload_path, 'rb').read() if os.path.exists(payload_path) else b''
if midi_data:
    segments.append(('midi_bundle', midi_data))

# project.toml as metadata
toml_data = open(toml_path, 'rb').read() if os.path.exists(toml_path) else b''
if toml_data:
    segments.append(('metadata', toml_data))

# Build NFT7: magic + count + [name_len + name + data_len + data]...
out = bytearray()
out.extend(b'NFT7')
out.extend(struct.pack('<I', len(segments)))
for name, data in segments:
    nb = name.encode()
    out.extend(struct.pack('<I', len(nb)))
    out.extend(nb)
    out.extend(struct.pack('<I', len(data)))
    out.extend(data)

nft7_path = os.path.join(project_dir, 'output', 'nft7_payload.bin')
open(nft7_path, 'wb').write(out)
print(f'   NFT7: {len(out)} bytes, {len(segments)} segments')
for name, data in segments:
    print(f'     {name}: {len(data)} bytes')
"

# 3. Rasterize SVGs + embed stego
echo "3. Rasterizing + embedding stego..."
NFT7="$PROJECT_DIR/output/nft7_payload.bin"

python3 -c "
import struct, os, sys

svg_dir = '$SVG_DIR'
stego_dir = '$STEGO_DIR'
nft7_path = '$NFT7'

# Load NFT7 payload
payload = open(nft7_path, 'rb').read()

# Split across 71 tiles (196608 bytes each)
TILE_CAP = 512 * 512 * 6 // 8  # 196608
chunks = []
for i in range(71):
    start = i * TILE_CAP
    chunk = bytearray(TILE_CAP)
    if start < len(payload):
        end = min(start + TILE_CAP, len(payload))
        chunk[:end-start] = payload[start:end]
    chunks.append(bytes(chunk))

print(f'   payload: {len(payload)} bytes across 71 tiles ({TILE_CAP} B/tile)')

# For each SVG: rasterize to RGB, embed stego, write PNG
try:
    import subprocess
    ok = 0
    for idx in range(1, 72):
        svg_path = os.path.join(svg_dir, f'{idx:02d}.svg')
        png_path = os.path.join(stego_dir, f'{idx:02d}.png')
        
        if not os.path.exists(svg_path):
            continue
        
        # Rasterize SVG → PNG via convert (ImageMagick)
        tmp_png = f'/tmp/retro_tile_{idx:02d}.png'
        r = subprocess.run(['convert', svg_path, '-resize', '512x512!', '-depth', '8', tmp_png],
                          capture_output=True)
        if r.returncode != 0 or not os.path.exists(tmp_png):
            continue
        
        # Read PNG → RGB (handle RGBA and greyscale)
        import png as pypng
        reader = pypng.Reader(filename=tmp_png)
        w, h, rows, info = reader.asRGBA8()
        rgb = bytearray()
        for row in rows:
            for x in range(w):
                rgb.append(row[x * 4])
                rgb.append(row[x * 4 + 1])
                rgb.append(row[x * 4 + 2])
        
        # Embed stego (6-layer bit-plane)
        chunk = chunks[idx - 1]
        for i in range(min(len(chunk), TILE_CAP)):
            byte = chunk[i]
            for b in range(8):
                bit_idx = i * 8 + b
                px = bit_idx // 6
                plane = bit_idx % 6
                if px >= 512 * 512:
                    break
                ch = plane % 3
                bit_pos = plane // 3
                pos = px * 3 + ch
                val = (byte >> b) & 1
                rgb[pos] = (rgb[pos] & ~(1 << bit_pos)) | (val << bit_pos)
        
        # Write PNG
        writer = pypng.Writer(width=512, height=512, greyscale=False, bitdepth=8)
        rows_out = [rgb[y*512*3:(y+1)*512*3] for y in range(512)]
        with open(png_path, 'wb') as f:
            writer.write(f, rows_out)
        ok += 1
    
    print(f'   ✅ {ok}/71 stego PNGs')
except ImportError:
    print('   ⚠ pypng not available — falling back to cargo encoder')
    sys.exit(1)
"

# 4. Verify
echo "4. Verifying..."
python3 -c "
import struct, os

stego_dir = '$STEGO_DIR'
TILE_CAP = 512 * 512 * 6 // 8

try:
    import png as pypng
except ImportError:
    print('   ⚠ pypng not available'); exit(0)

chunks = []
for idx in range(1, 72):
    path = os.path.join(stego_dir, f'{idx:02d}.png')
    if not os.path.exists(path):
        continue
    reader = pypng.Reader(filename=path)
    w, h, rows, info = reader.read()
    rgb = bytearray()
    planes = info.get('planes', 3)
    for row in rows:
        for x in range(w):
            rgb.append(row[x * planes])
            rgb.append(row[x * planes + 1])
            rgb.append(row[x * planes + 2])
    
    chunk = bytearray(TILE_CAP)
    for i in range(TILE_CAP):
        byte = 0
        for b in range(8):
            bit_idx = i * 8 + b
            px = bit_idx // 6
            plane = bit_idx % 6
            if px >= 512*512: break
            ch = plane % 3
            bit_pos = plane // 3
            pos = px * 3 + ch
            byte |= ((rgb[pos] >> bit_pos) & 1) << b
        chunk[i] = byte
    chunks.append(bytes(chunk))

payload = b''.join(chunks)
if payload[:4] == b'NFT7':
    count = struct.unpack('<I', payload[4:8])[0]
    off = 8
    print(f'   ✅ NFT7 valid: {count} segments')
    for _ in range(count):
        if off+4 > len(payload): break
        nl = struct.unpack('<I', payload[off:off+4])[0]; off += 4
        name = payload[off:off+nl].decode('utf-8','replace'); off += nl
        dl = struct.unpack('<I', payload[off:off+4])[0]; off += 4
        magic = payload[off:off+4].hex() if off+4 <= len(payload) else ''
        print(f'     {name:15} {dl:>10} B  {magic}')
        off += dl
else:
    print(f'   ❌ NFT7 decode failed: {payload[:4].hex()}')
"

echo
echo "=== BUILD COMPLETE ==="
echo "  Tiles: $STEGO_DIR/"
echo "  Payload: $NFT7"
