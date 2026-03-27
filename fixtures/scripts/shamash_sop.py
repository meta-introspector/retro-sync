#!/usr/bin/env python3
"""Decompose Shamash SVG into human-readable drawing instructions (SOP).

Analyze the original SVG structure:
1. What commands are used (M, L, C, A, Z, etc.)
2. What symmetries exist (rotational, reflective)
3. Group into logical drawing steps
4. Output: a step-by-step procedure a person could follow to draw it

This is the "first principles" reconstruction.
"""

import re, math, json, sys

SHAMASH = "/var/www/solana.solfunmeme.com/retro-sync/scratch/shamash_star.svg"
OUT = "/var/www/solana.solfunmeme.com/retro-sync/scratch/shamash_sop.json"

def parse_path(d):
    """Parse SVG path d= into structured commands."""
    # Tokenize: split into (command, args) pairs
    tokens = re.findall(r'([MmLlHhVvCcSsQqTtAaZz])([^MmLlHhVvCcSsQqTtAaZz]*)', d)
    commands = []
    for cmd, args_str in tokens:
        nums = [float(x) for x in re.findall(r'[-]?\d+\.?\d*', args_str)]
        commands.append({"cmd": cmd, "args": nums})
    return commands

def classify_path(commands):
    """Classify a path: circle, arc, wavy line, straight, etc."""
    cmd_types = set(c["cmd"] for c in commands)
    n_cmds = len(commands)
    
    if 'a' in cmd_types or 'A' in cmd_types:
        return "arc/circle"
    elif ('c' in cmd_types or 'C' in cmd_types) and n_cmds > 20:
        return "wavy_ray"
    elif ('c' in cmd_types or 'C' in cmd_types):
        return "curve"
    elif 'l' in cmd_types or 'L' in cmd_types:
        return "line_segments"
    elif 'Z' in cmd_types or 'z' in cmd_types:
        return "closed_shape"
    else:
        return "other"

def find_center(commands):
    """Find approximate center from first M command."""
    for c in commands:
        if c["cmd"] in "Mm" and len(c["args"]) >= 2:
            return c["args"][0], c["args"][1]
    return 0, 0

def detect_rotational_symmetry(all_paths):
    """Detect rotational symmetry by comparing path structures."""
    # Group paths by similar command sequences
    signatures = []
    for path in all_paths:
        sig = tuple(c["cmd"] for c in path["commands"])
        n_args = sum(len(c["args"]) for c in path["commands"])
        signatures.append((sig, n_args))
    
    # Count identical signatures
    from collections import Counter
    sig_counts = Counter(signatures)
    
    symmetries = []
    for sig, count in sig_counts.most_common():
        if count > 1:
            symmetries.append({"pattern": str(sig[0][:5]), "args": sig[1], "copies": count})
    
    return symmetries

def path_to_instruction(path_info, idx):
    """Convert a parsed path into a human-readable drawing instruction."""
    ptype = path_info["type"]
    n_cmds = len(path_info["commands"])
    n_coords = sum(len(c["args"]) for c in path_info["commands"])
    start = find_center(path_info["commands"])
    
    cmd_summary = {}
    for c in path_info["commands"]:
        cmd_summary[c["cmd"]] = cmd_summary.get(c["cmd"], 0) + 1
    
    if ptype == "arc/circle":
        # Extract arc params
        for c in path_info["commands"]:
            if c["cmd"] in "aA" and len(c["args"]) >= 7:
                rx, ry = c["args"][0], c["args"][1]
                return {
                    "step": idx + 1,
                    "action": "draw_circle",
                    "description": f"Draw a circle/arc with radius ~{rx:.0f}×{ry:.0f}",
                    "start": list(start),
                    "params": {"rx": rx, "ry": ry},
                    "complexity": n_coords,
                }
        return {"step": idx+1, "action": "draw_arc", "description": "Draw an arc", "complexity": n_coords}
    
    elif ptype == "wavy_ray":
        return {
            "step": idx + 1,
            "action": "draw_wavy_rays",
            "description": f"Draw wavy sun rays using {cmd_summary.get('c', cmd_summary.get('C', 0))} cubic bezier curves",
            "start": list(start),
            "n_curves": cmd_summary.get('c', 0) + cmd_summary.get('C', 0),
            "n_lines": cmd_summary.get('l', 0) + cmd_summary.get('L', 0),
            "complexity": n_coords,
        }
    
    elif ptype == "line_segments":
        return {
            "step": idx + 1,
            "action": "draw_radial_lines",
            "description": f"Draw {n_cmds} straight line segments (radial spokes)",
            "start": list(start),
            "n_segments": n_cmds,
            "complexity": n_coords,
        }
    
    else:
        return {
            "step": idx + 1,
            "action": f"draw_{ptype}",
            "description": f"Draw {ptype} with {n_cmds} commands",
            "start": list(start),
            "complexity": n_coords,
        }

def main():
    svg = open(SHAMASH).read()
    
    # Extract dimensions
    w = re.search(r'width="(\d+)"', svg)
    h = re.search(r'height="(\d+)"', svg)
    width = int(w.group(1)) if w else 0
    height = int(h.group(1)) if h else 0
    
    # Extract all paths
    raw_paths = re.findall(r'\bd="(M[^"]+)"', svg)
    
    print(f"=== SHAMASH SVG DECOMPOSITION (SOP) ===\n")
    print(f"Canvas: {width}×{height}")
    print(f"Paths: {len(raw_paths)}\n")
    
    all_paths = []
    for i, d in enumerate(raw_paths):
        commands = parse_path(d)
        ptype = classify_path(commands)
        path_info = {
            "index": i,
            "type": ptype,
            "commands": commands,
            "n_commands": len(commands),
            "n_coords": sum(len(c["args"]) for c in commands),
        }
        all_paths.append(path_info)
        
        cmd_counts = {}
        for c in commands:
            cmd_counts[c["cmd"]] = cmd_counts.get(c["cmd"], 0) + 1
        
        print(f"  Path {i}: {ptype:15} {len(commands):4} cmds, {path_info['n_coords']:5} coords  {cmd_counts}")
    
    # Detect symmetries
    symmetries = detect_rotational_symmetry(all_paths)
    print(f"\nSymmetries detected:")
    for s in symmetries:
        print(f"  {s['copies']}× pattern (first cmds: {s['pattern']}, {s['args']} args)")
    
    # Generate SOP (Standard Operating Procedure)
    print(f"\n=== DRAWING INSTRUCTIONS (SOP) ===\n")
    
    sop = {
        "title": "How to Draw the Shamash Sun Disc",
        "canvas": {"width": width, "height": height, "center": [width//2, height//2]},
        "symmetries": symmetries,
        "steps": [],
    }
    
    # Sort paths by type for logical ordering
    order = {"arc/circle": 0, "closed_shape": 1, "line_segments": 2, "wavy_ray": 3, "curve": 4, "other": 5}
    sorted_paths = sorted(all_paths, key=lambda p: order.get(p["type"], 5))
    
    for idx, path_info in enumerate(sorted_paths):
        instruction = path_to_instruction(path_info, idx)
        sop["steps"].append(instruction)
        print(f"  Step {instruction['step']}: {instruction['description']}")
    
    # Summary
    total_coords = sum(p["n_coords"] for p in all_paths)
    print(f"\n=== SUMMARY ===")
    print(f"  Total drawing commands: {sum(p['n_commands'] for p in all_paths)}")
    print(f"  Total coordinates: {total_coords}")
    print(f"  Data capacity (3 decimal digits per coord): {total_coords} bytes")
    print(f"  Symmetry groups: {len(symmetries)}")
    
    # Save
    with open(OUT, 'w') as f:
        json.dump(sop, f, indent=2, default=str)
    print(f"\n→ {OUT}")

if __name__ == "__main__":
    main()
