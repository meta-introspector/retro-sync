#!/usr/bin/env python3
"""Merge 120 bred Shamash variants into one animated SVG.

Each frame is a different bred variant. CSS keyframes cycle through them.
The sun disc morphs as data changes — the encoding IS the animation.
"""

import re, math, os, random

SCRATCH = "/var/www/solana.solfunmeme.com/retro-sync/scratch"
SHAMASH = f"{SCRATCH}/shamash_star.svg"
OUT = f"{SCRATCH}/shamash_animated.svg"
N_FRAMES = 120

SSP = [2,3,5,7,11,13,17,19,23,29,31,41,47,59,71]

def load_shamash():
    return open(SHAMASH).read()

def breed_frame(base_svg, frame_idx, payload):
    """Generate one frame variant."""
    random.seed(frame_idx * 7919)
    genome = [random.randint(0, 255) for _ in range(20)]
    data = (payload + bytes([(frame_idx * 13 + i) % 256 for i in range(3000)]))[:3000]
    off = [0]
    def eat(n):
        c = data[off[0]:off[0]+n]; off[0] += n; return c

    # Extract extra rays for this frame
    n_rays = 6 + frame_idx % 8
    rays = []
    cx, cy = 681, 681
    for j in range(n_rays):
        rb = eat(6)
        angle = (j / n_rays) * 2 * math.pi + frame_idx * 0.05
        r1 = 200 + rb[0]
        r2 = 400 + rb[1]
        wave = (rb[2] - 128) * 0.4
        perp = angle + math.pi / 2
        x1 = cx + math.cos(angle) * r1
        y1 = cy + math.sin(angle) * r1
        x2 = cx + math.cos(angle) * r2
        y2 = cy + math.sin(angle) * r2
        mx = (x1+x2)/2 + math.cos(perp) * wave
        my = (y1+y2)/2 + math.sin(perp) * wave
        cr = max(64, rb[3])
        cg = max(64, rb[4])
        cb = max(64, rb[5])
        sw = 1.5 + rb[2] % 3
        rays.append((x1,y1,mx,my,x2,y2,cr,cg,cb,sw))
    
    return rays, n_rays

def main():
    print(f"=== ANIMATED SHAMASH: {N_FRAMES} FRAMES ===\n")
    
    base = load_shamash()
    payload = b"Hurrian Hymn h.6 Nikkal Teshub Ugarit" + bytes(range(256)) * 8
    
    # Generate all frames' ray data
    all_frames = []
    for f in range(N_FRAMES):
        rays, n = breed_frame(base, f, payload)
        all_frames.append(rays)
    
    # Build animated SVG
    # Use the base Shamash as background, animate the extra rays
    dur = 12.0  # seconds for full cycle
    
    # Extract viewBox from base
    vb = re.search(r'viewBox="([^"]*)"', base)
    vb_str = vb.group(1) if vb else "0 0 1362 1362"
    w = re.search(r'width="(\d+)"', base)
    h = re.search(r'height="(\d+)"', base)
    width = w.group(1) if w else "1362"
    height = h.group(1) if h else "1362"
    
    svg = f'''<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="{vb_str}">
'''
    
    # Embed base shamash paths (static)
    for m in re.finditer(r'(<path[^>]*/>)', base):
        svg += f'  {m.group(1)}\n'
    
    # For each ray slot, animate across frames
    max_rays = max(len(f) for f in all_frames)
    
    for ray_idx in range(max_rays):
        # Collect this ray's position across all frames
        n_keys = min(N_FRAMES, 24)  # sample 24 keyframes
        step = max(1, N_FRAMES // n_keys)
        
        x1_vals, y1_vals = [], []
        mx_vals, my_vals = [], []
        x2_vals, y2_vals = [], []
        cr_vals, cg_vals, cb_vals = [], [], []
        sw_vals = []
        op_vals = []
        
        for k in range(n_keys):
            f = (k * step) % N_FRAMES
            rays = all_frames[f]
            if ray_idx < len(rays):
                x1,y1,mx,my,x2,y2,cr,cg,cb,sw = rays[ray_idx]
                x1_vals.append(f"{x1:.0f}")
                y1_vals.append(f"{y1:.0f}")
                mx_vals.append(f"{mx:.0f}")
                my_vals.append(f"{my:.0f}")
                x2_vals.append(f"{x2:.0f}")
                y2_vals.append(f"{y2:.0f}")
                cr_vals.append(str(cr))
                cg_vals.append(str(cg))
                cb_vals.append(str(cb))
                sw_vals.append(f"{sw:.1f}")
                op_vals.append("0.6")
            else:
                # Ray doesn't exist in this frame — fade out
                x1_vals.append(x1_vals[-1] if x1_vals else "681")
                y1_vals.append(y1_vals[-1] if y1_vals else "681")
                mx_vals.append(mx_vals[-1] if mx_vals else "681")
                my_vals.append(my_vals[-1] if my_vals else "681")
                x2_vals.append(x2_vals[-1] if x2_vals else "681")
                y2_vals.append(y2_vals[-1] if y2_vals else "681")
                cr_vals.append("100")
                cg_vals.append("100")
                cb_vals.append("100")
                sw_vals.append("0.5")
                op_vals.append("0")
        
        # Build animated path using SMIL
        # Initial values from first frame
        if not x1_vals: continue
        
        d_vals = ";".join(
            f"M{x1_vals[i]} {y1_vals[i]} Q{mx_vals[i]} {my_vals[i]} {x2_vals[i]} {y2_vals[i]}"
            for i in range(len(x1_vals))
        )
        
        color_vals = ";".join(
            f"rgb({cr_vals[i]},{cg_vals[i]},{cb_vals[i]})"
            for i in range(len(cr_vals))
        )
        
        opacity_vals = ";".join(op_vals)
        width_vals = ";".join(sw_vals)
        
        delay = ray_idx * 0.1
        
        svg += f'''  <path d="M{x1_vals[0]} {y1_vals[0]} Q{mx_vals[0]} {my_vals[0]} {x2_vals[0]} {y2_vals[0]}"
    fill="none" stroke="rgb({cr_vals[0]},{cg_vals[0]},{cb_vals[0]})" 
    stroke-width="{sw_vals[0]}" stroke-linecap="round" opacity="0.6">
    <animate attributeName="d" values="{d_vals}" dur="{dur}s" repeatCount="indefinite"/>
    <animate attributeName="stroke" values="{color_vals}" dur="{dur}s" repeatCount="indefinite"/>
    <animate attributeName="opacity" values="{opacity_vals}" dur="{dur}s" repeatCount="indefinite"/>
    <animate attributeName="stroke-width" values="{width_vals}" dur="{dur}s" repeatCount="indefinite"/>
  </path>
'''
    
    svg += '</svg>\n'
    
    with open(OUT, 'w') as f:
        f.write(svg)
    
    sz = os.path.getsize(OUT)
    print(f"  {N_FRAMES} frames, {max_rays} ray slots, {dur}s cycle")
    print(f"  {sz//1024}KB → {OUT}")
    print(f"  View: https://solana.solfunmeme.com/retro-sync/scratch/shamash_animated.svg")

if __name__ == "__main__":
    main()
