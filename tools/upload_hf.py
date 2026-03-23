#!/usr/bin/env python3
"""Upload stego PNG tiles + WASM pkg to HuggingFace.

Usage:
  python3 tools/upload_hf.py          # upload to both space and dataset
  python3 tools/upload_hf.py space    # space only (tiles + wasm)
  python3 tools/upload_hf.py dataset  # dataset only (tiles)
"""
import sys
from huggingface_hub import HfApi

api = HfApi()
repo = "introspector/retro-sync"
tiles = "fixtures/output/nft71_stego_png"
pkg = "docs/pkg"

targets = sys.argv[1:] or ["space", "dataset"]

for t in targets:
    print(f"Uploading tiles to {t}...")
    api.upload_folder(folder_path=tiles, path_in_repo="tiles",
                      repo_id=repo, repo_type=t)

if "space" in targets:
    print("Uploading WASM pkg to space...")
    api.upload_folder(folder_path=pkg, path_in_repo="pkg",
                      repo_id=repo, repo_type="space")

print("Done.")
