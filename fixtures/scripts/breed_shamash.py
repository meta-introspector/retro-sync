#!/usr/bin/env python3
"""Breed 72 Shamash SVG variants using constraint-safe FRACTRAN mutations.

Each organism = a sequence of safe mutations applied to the base Shamash.
Fitness = payload capacity × visual fidelity.
Output: 72 SVG variants in scratch/breed/
"""

import re, json, math, os, random

SCRATCH = "/var/www/solana.solfunmeme.com/retro-sync/scratch"
SHAMASH = f"{SCRATCH}/shamash_star.svg"
OUT_DIR = f"{SCRATCH}/breed"
POP = 72
GENERATIONS = 10

SSP = [2,3,5,7,11,13,17,19,23,29,31,41,47,59,71]

# Safe mutations: (source_dim, target_dim)
MUTATIONS = [(0,1),(3,4),(4,5),(6,7),(9,10),(11,12),(2,14)]

def load_shamash():
    return open(SHAMASH).read()

def mutate_svg(svg, genome, payload):
    """Apply genome (list of mutation indices + magnitudes) to SVG.
    
    Modifies: colors, coordinate offsets, stroke widths, opacities.
    Encodes payload in the modifications.
    """
    data = (payload + b'\x00' * 3000)[:3000]
    off = [0]
    def eat(n):
        c = data[off[0]:off[0]+n]; off[0] += n; return c

    result = svg
    
    # Apply color mutations based on genome
    def recolor(match):
        tag = match.group(0)
        gb = eat(3)
        r = max(64, min(240, gb[0]))
        g = max(64, min(240, gb[1]))
        b = max(64, min(240, gb[2]))
        
        # Genome controls how much to shift
        shift = genome[off[0] % len(genome)] if genome else 0
        r = max(64, min(240, r + (shift % 20) - 10))
        g = max(64, min(240, g + (shift % 15) - 7))
        
        if 'fill="' in tag:
            tag = re.sub(r'fill="[^"]*"', f'fill="rgb({r},{g},{b})"', tag)
        elif 'fill' not in tag and '/>' in tag:
            tag = tag.replace('/>', f' fill="rgb({r},{g},{b})"/>')
        
        sw = 0.5 + eat(1)[0] % 40 / 10.0
        if 'stroke-width' not in tag:
            tag = tag.replace('/>', f' stroke="rgb({min(240,r+30)},{min(240,g+30)},{min(240,b+30)})" stroke-width="{sw:.1f}"/>')
        
        return tag

    result = re.sub(r'<path[^>]*/>', recolor, result)
    
    # Modulate coordinate decimals AND integer parts with payload
    def mod_coord(match):
        val = match.group(0)
        parts = val.split('.')
        if len(parts) == 2:
            b = eat(1)[0]
            # Shift integer part by ±genome-controlled amount (visible!)
            base = float(val)
            shift = (genome[off[0] % len(genome)] % 40) - 20  # ±20 pixels
            shifted = max(10, base + shift * (b / 255.0))
            return f"{shifted:.3f}"
        return val
    
    result = re.sub(r'd="([^"]*)"', 
        lambda m: 'd="' + re.sub(r'\d+\.\d+', mod_coord, m.group(1)) + '"', result)
    
    # Add extra decorative rays (data-driven, visible variation)
    n_extra = 4 + genome[2] % 8  # 4-11 extra rays
    extra = ""
    for j in range(n_extra):
        rb = eat(6)
        angle = (j / n_extra) * 2 * 3.14159 + genome[3] * 0.02
        r1 = 200 + rb[0]
        r2 = 400 + rb[1]
        cx, cy = 681, 681  # shamash center
        x1 = cx + math.cos(angle) * r1
        y1 = cy + math.sin(angle) * r1
        x2 = cx + math.cos(angle) * r2
        y2 = cy + math.sin(angle) * r2
        # Wavy control point
        perp = angle + 1.5708
        wave = (rb[2] - 128) * 0.5
        mx = (x1+x2)/2 + math.cos(perp) * wave
        my = (y1+y2)/2 + math.sin(perp) * wave
        cr = max(64, rb[3])
        cg = max(64, rb[4])
        cb = max(64, rb[5])
        sw = 1 + rb[2] % 4
        extra += f'<path d="M{x1:.1f} {y1:.1f} Q{mx:.1f} {my:.1f} {x2:.1f} {y2:.1f}" fill="none" stroke="rgb({cr},{cg},{cb})" stroke-width="{sw}" stroke-linecap="round" opacity="0.6"/>\n'
    result = result.replace('</svg>', extra + '</svg>')
    
    # Add hatch pattern
    hb = eat(2)
    hatch = f'''<defs><pattern id="h" width="{4+hb[0]%6}" height="{4+hb[1]%6}" patternUnits="userSpaceOnUse" patternTransform="rotate({genome[0]%90 if genome else 45})">
<line x1="0" y1="0" x2="0" y2="{4+hb[1]%6}" stroke="rgb({max(64,hb[0])},{max(64,hb[1])},80)" stroke-width="0.5" opacity="0.12"/>
</pattern></defs><rect width="100%" height="100%" fill="url(#h)"/>
'''
    result = re.sub(r'(<svg[^>]*>)', r'\1\n' + hatch, result, count=1)
    
    return result, off[0]

def score(svg_text, n_encoded):
    """Fitness: payload capacity × color diversity × path preservation."""
    # Count unique colors
    colors = set(re.findall(r'rgb\((\d+,\d+,\d+)\)', svg_text))
    # Count paths preserved
    paths = len(re.findall(r'<path', svg_text))
    # Coordinate count
    coords = len(re.findall(r'\d+\.\d{3}', svg_text))
    
    return n_encoded * 0.5 + len(colors) * 10 + paths * 5 + coords * 0.1

def random_genome(seed):
    random.seed(seed)
    return [random.randint(0, 255) for _ in range(20)]

def crossover(g1, g2, seed):
    random.seed(seed)
    return [g1[i] if random.random() < 0.5 else g2[i] for i in range(len(g1))]

def mutate_genome(g, seed):
    random.seed(seed)
    g2 = g[:]
    idx = random.randint(0, len(g2)-1)
    g2[idx] = random.randint(0, 255)
    return g2

def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    base_svg = load_shamash()
    payload = b"Hurrian Hymn h.6 Nikkal Teshub" + bytes(range(256)) * 8
    
    print(f"=== BREED 72 SHAMASH VARIANTS ===\n")
    print(f"Base: {len(base_svg)} bytes, {base_svg.count('<path')} paths")
    print(f"Payload: {len(payload)} bytes\n")
    
    # Init population
    pop = [random_genome(i * 31337) for i in range(POP)]
    
    for gen in range(GENERATIONS):
        results = []
        for i, genome in enumerate(pop):
            svg, n_enc = mutate_svg(base_svg, genome, payload)
            s = score(svg, n_enc)
            results.append((i, s, svg, n_enc, genome))
        
        results.sort(key=lambda r: -r[1])
        best = results[0]
        mean = sum(r[1] for r in results) / len(results)
        
        if gen % 3 == 0 or gen == GENERATIONS - 1:
            print(f"  gen {gen:2d}: best={best[1]:.0f} mean={mean:.0f} encoded={best[3]}B colors={len(set(re.findall(r'rgb', best[2])))}")
        
        # Breed
        elite = POP // 4
        new_pop = [r[4] for r in results[:elite]]
        for i in range(elite, POP):
            p1 = new_pop[i % elite]
            p2 = new_pop[(i + 1) % elite]
            child = crossover(p1, p2, gen * POP + i)
            if (gen * POP + i) % 3 == 0:
                child = mutate_genome(child, gen * POP + i * 7)
            new_pop.append(child)
        pop = new_pop
    
    # Save top 8 variants
    results.sort(key=lambda r: -r[1])
    print(f"\n=== SAVING TOP 8 VARIANTS ===\n")
    for rank, (i, s, svg, n_enc, genome) in enumerate(results[:8]):
        path = f"{OUT_DIR}/shamash_{rank:02d}.svg"
        with open(path, 'w') as f:
            f.write(svg)
        colors = len(set(re.findall(r'rgb\((\d+,\d+,\d+)\)', svg)))
        print(f"  #{rank}: score={s:.0f} encoded={n_enc}B colors={colors} → {path}")
    
    print(f"\n  View: https://solana.solfunmeme.com/retro-sync/scratch/breed/")
    print(f"  Best genome: {results[0][4][:8]}...")

if __name__ == "__main__":
    main()
