#!/usr/bin/env bash
set -euo pipefail
# publish-catalog.sh — Publish catalog + tiles to HuggingFace dataset
# Uses upload_folder for speed instead of file-by-file

REPO_ROOT=$(realpath "$(dirname "$0")/..")
REPO="introspector/retro-sync"

echo "=== PUBLISH TO HUGGINGFACE ==="

python3 -c "
from huggingface_hub import HfApi
import os

api = HfApi()
repo = '$REPO'
root = '$REPO_ROOT'

# Upload catalog folder
print('1. Catalog...')
api.upload_folder(folder_path=os.path.join(root, 'catalog'), path_in_repo='catalog',
                  repo_id=repo, repo_type='dataset')
print('   ✅ catalog/')

# Upload each project's stego tiles
for proj in sorted(os.listdir(os.path.join(root, 'projects'))):
    stego = os.path.join(root, 'projects', proj, 'output', 'stego')
    if not os.path.isdir(stego): continue
    n = len([f for f in os.listdir(stego) if f.endswith('.png')])
    if n == 0: continue
    print(f'2. {proj} ({n} tiles)...')
    api.upload_folder(folder_path=stego, path_in_repo=f'{proj}/tiles',
                      repo_id=repo, repo_type='dataset')
    toml = os.path.join(root, 'projects', proj, 'project.toml')
    if os.path.exists(toml):
        api.upload_file(path_or_fileobj=toml, path_in_repo=f'{proj}/project.toml',
                       repo_id=repo, repo_type='dataset')
    print(f'   ✅ {proj}')

# Upload docs
print('3. Docs...')
for f in ['index.html', 'menu.html', 'GETTING-STARTED.md']:
    p = os.path.join(root, 'docs', f)
    if os.path.exists(p):
        api.upload_file(path_or_fileobj=p, path_in_repo=f, repo_id=repo, repo_type='dataset')
        print(f'   ✅ {f}')

print(f'\n✅ https://huggingface.co/datasets/{repo}')
"

echo "=== DONE ==="
