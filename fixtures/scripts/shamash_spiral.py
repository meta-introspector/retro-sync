#!/usr/bin/env python3
"""Shamash rays as logarithmic spirals with Frenet-Serret taper.

Each ray: r = a*e^(b*θ), R(s) = R0*(1-s/L), cyclic C_n symmetry.
6 bytes per ray encode: a, b, R0, L, torsion, color_hue.
Animated: FRACTRAN controller modulates parameters over 120 frames.
"""

import math, os

SCRATCH = "/var/www/solana.solfunmeme.com/retro-sync/scratch"
SZ = 1024
CX, CY = SZ/2, SZ/2
N_FRAMES = 120
DUR = 12.0

PAYLOAD = (b"Hurrian Hymn h.6 Nikkal Teshub Shamash Ugarit 1400BCE "
           b"nish tuhrim qablitum ishartum shalshatum serdum colophon "
           + bytes(range(256)) * 4)

def spiral_points(a, b, theta_start, theta_end, n_pts=40):
    """Generate points along logarithmic spiral r = a*e^(b*θ)."""
    pts = []
    for i in range(n_pts):
        t = theta_start + (theta_end - theta_start) * i / (n_pts - 1)
        r = a * math.exp(b * t)
        x = CX + r * math.cos(t)
        y = CY + r * math.sin(t)
        pts.append((x, y))
    return pts

def taper_width(s, R0, L):
    """Tapering radius: R(s) = R0 * (1 - s/L)."""
    return max(0.3, R0 * (1 - s / max(L, 1)))

def ray_svg(pts, R0, L, hue, opacity=0.7):
    """Render one ray as a tapered path with stroke-width animation."""
    if len(pts) < 2: return ""
    # Build path
    d = f"M{pts[0][0]:.1f} {pts[0][1]:.1f}"
    for i in range(1, len(pts)):
        d += f" L{pts[i][0]:.1f} {pts[i][1]:.1f}"
    
    # Average width for this ray
    avg_w = taper_width(0.5, R0, L)
    
    r = max(64, int(180 + 60 * math.cos(hue)))
    g = max(64, int(160 + 60 * math.cos(hue + 2.1)))
    b = max(64, int(140 + 60 * math.cos(hue + 4.2)))
    
    return (f'<path d="{d}" fill="none" stroke="rgb({r},{g},{b})" '
            f'stroke-width="{avg_w:.1f}" stroke-linecap="round" opacity="{opacity:.2f}"/>\n')

def ray_animated_svg(ray_idx, n_rays, frames_data):
    """Animated ray: path morphs across keyframes."""
    n_keys = min(len(frames_data), 24)
    step = max(1, len(frames_data) // n_keys)
    
    d_vals = []
    color_vals = []
    width_vals = []
    
    for k in range(n_keys):
        f = (k * step) % len(frames_data)
        params = frames_data[f]
        if ray_idx >= len(params): 
            d_vals.append(d_vals[-1] if d_vals else f"M{CX} {CY}")
            color_vals.append(color_vals[-1] if color_vals else "rgb(128,128,100)")
            width_vals.append("0.5")
            continue
        
        a, b_param, R0, L, torsion, hue = params[ray_idx]
        base_angle = (ray_idx / n_rays) * 2 * math.pi + torsion * 0.01
        pts = spiral_points(a, b_param, base_angle, base_angle + 1.5 + L * 0.003, 30)
        
        d = f"M{pts[0][0]:.0f} {pts[0][1]:.0f}"
        for p in pts[1:]:
            d += f" L{p[0]:.0f} {p[1]:.0f}"
        d_vals.append(d)
        
        r = max(64, int(180 + 60 * math.cos(hue)))
        g = max(64, int(160 + 60 * math.cos(hue + 2.1)))
        bv = max(64, int(140 + 60 * math.cos(hue + 4.2)))
        color_vals.append(f"rgb({r},{g},{bv})")
        width_vals.append(f"{taper_width(0.3, R0, L):.1f}")
    
    return f'''<path d="{d_vals[0]}" fill="none" stroke="{color_vals[0]}" 
    stroke-width="{width_vals[0]}" stroke-linecap="round" opacity="0.65">
    <animate attributeName="d" values="{';'.join(d_vals)}" dur="{DUR}s" repeatCount="indefinite"/>
    <animate attributeName="stroke" values="{';'.join(color_vals)}" dur="{DUR}s" repeatCount="indefinite"/>
    <animate attributeName="stroke-width" values="{';'.join(width_vals)}" dur="{DUR}s" repeatCount="indefinite"/>
  </path>
'''

def generate_frame_params(frame_idx, payload, n_rays):
    """Extract ray parameters from payload for one frame."""
    off = (frame_idx * n_rays * 6) % max(1, len(payload) - n_rays * 6)
    rays = []
    for i in range(n_rays):
        chunk = payload[off:off+6]
        off += 6
        if len(chunk) < 6: chunk = chunk + b'\x80' * (6 - len(chunk))
        
        a = 30 + chunk[0] * 0.3          # spiral start radius: 30-106
        b_param = 0.15 + chunk[1] * 0.003 # tightness: 0.15-0.92
        R0 = 2 + chunk[2] / 30           # base width: 2-10.5
        L = 150 + chunk[3] * 1.5         # length: 150-532
        torsion = chunk[4]                # twist: 0-255
        hue = chunk[5] * 0.025           # color angle: 0-6.3
        
        # Frame-specific modulation (the "animation")
        phase = frame_idx * 0.05 + i * 0.3
        a += 10 * math.sin(phase)
        b_param += 0.05 * math.sin(phase * 1.3)
        torsion += frame_idx * 2
        
        rays.append((a, b_param, R0, L, torsion, hue))
    return rays

def main():
    n_rays = 13  # like Shamash
    
    print(f"=== SPIRAL SHAMASH: {n_rays} logarithmic spiral rays ===\n")
    
    # Generate all frame parameters
    all_frames = [generate_frame_params(f, PAYLOAD, n_rays) for f in range(N_FRAMES)]
    
    # Static SVG (frame 0)
    svg_static = f'<svg xmlns="http://www.w3.org/2000/svg" width="{SZ}" height="{SZ}" viewBox="0 0 {SZ} {SZ}">\n'
    svg_static += f'<rect width="{SZ}" height="{SZ}" fill="#3a3028"/>\n'
    # Sun disc center
    svg_static += f'<circle cx="{CX}" cy="{CY}" r="60" fill="#E8C840" opacity="0.8"/>\n'
    svg_static += f'<circle cx="{CX+15}" cy="{CY-5}" r="50" fill="#3a3028" opacity="0.7"/>\n'  # crescent
    
    for i in range(n_rays):
        params = all_frames[0][i]
        a, b_param, R0, L, torsion, hue = params
        base_angle = (i / n_rays) * 2 * math.pi + torsion * 0.01
        pts = spiral_points(a, b_param, base_angle, base_angle + 1.5 + L * 0.003, 30)
        svg_static += ray_svg(pts, R0, L, hue)
    
    svg_static += '</svg>\n'
    
    static_path = f"{SCRATCH}/shamash_spiral.svg"
    with open(static_path, 'w') as f:
        f.write(svg_static)
    print(f"  Static: {static_path}")
    
    # Animated SVG
    svg_anim = f'<svg xmlns="http://www.w3.org/2000/svg" width="{SZ}" height="{SZ}" viewBox="0 0 {SZ} {SZ}">\n'
    svg_anim += f'<rect width="{SZ}" height="{SZ}" fill="#3a3028"/>\n'
    svg_anim += f'<circle cx="{CX}" cy="{CY}" r="60" fill="#E8C840" opacity="0.8"/>\n'
    svg_anim += f'<circle cx="{CX+15}" cy="{CY-5}" r="50" fill="#3a3028" opacity="0.7"/>\n'
    
    for i in range(n_rays):
        svg_anim += ray_animated_svg(i, n_rays, all_frames)
    
    svg_anim += '</svg>\n'
    
    anim_path = f"{SCRATCH}/shamash_spiral_anim.svg"
    with open(anim_path, 'w') as f:
        f.write(svg_anim)
    
    # Stats
    bytes_encoded = n_rays * 6  # per frame
    print(f"  Animated: {anim_path} ({os.path.getsize(anim_path)//1024}KB)")
    print(f"  Rays: {n_rays}, frames: {N_FRAMES}, cycle: {DUR}s")
    print(f"  Bytes/frame: {bytes_encoded} (6 per ray: a, b, R0, L, τ, hue)")
    print(f"  Total encoded: {bytes_encoded * N_FRAMES} bytes across all frames")
    print(f"\n  View static:   https://solana.solfunmeme.com/retro-sync/scratch/shamash_spiral.svg")
    print(f"  View animated: https://solana.solfunmeme.com/retro-sync/scratch/shamash_spiral_anim.svg")

if __name__ == "__main__":
    main()
