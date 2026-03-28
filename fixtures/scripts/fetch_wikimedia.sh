#!/bin/bash
# fetch_wikimedia.sh — Download images from Wikimedia Commons by filename
# Usage: ./fetch_wikimedia.sh <output_dir> <File:Name1.jpg> [File:Name2.jpg] ...
# Uses the Wikimedia API to resolve actual URLs.

OUT="${1:-.}"
shift
mkdir -p "$OUT"

for title in "$@"; do
  echo "Fetching: $title"
  url=$(curl -s "https://commons.wikimedia.org/w/api.php?action=query&titles=$title&prop=imageinfo&iiprop=url&iiurlwidth=512&format=json" \
    | python3 -c "
import json,sys
d=json.load(sys.stdin)
for p in d['query']['pages'].values():
    if 'imageinfo' in p:
        info = p['imageinfo'][0]
        print(info.get('thumburl', info['url']))
" 2>/dev/null)

  if [ -z "$url" ]; then
    echo "  ⚠ not found"
    continue
  fi

  fname=$(echo "$title" | sed 's|^File:||;s| |_|g')
  curl -sL -o "$OUT/$fname" "$url" -H "User-Agent: Mozilla/5.0"
  
  if file "$OUT/$fname" | grep -q "image\|JPEG\|PNG"; then
    sz=$(ls -lh "$OUT/$fname" | awk '{print $5}')
    echo "  ✅ $fname ($sz)"
  else
    echo "  ❌ $fname — not an image"
    rm -f "$OUT/$fname"
  fi
done
