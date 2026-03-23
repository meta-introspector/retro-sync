#!/usr/bin/env python3
"""Simulate what a browser Canvas does vs raw PNG read."""
from PIL import Image
import numpy as np

img = Image.open('/var/www/solana.solfunmeme.com/retro-sync/tiles/01.png')

# Check if PNG has any ICC profile or transparency
print("Mode:", img.mode)
print("Info keys:", list(img.info.keys()))

if 'icc_profile' in img.info:
    print("HAS ICC PROFILE — browser will apply color management!")
    print("ICC length:", len(img.info['icc_profile']))
else:
    print("No ICC profile")

if 'transparency' in img.info:
    print("HAS TRANSPARENCY")

if 'gamma' in img.info:
    print("HAS GAMMA:", img.info['gamma'])

# Check sRGB chunk
if 'srgb' in img.info:
    print("HAS sRGB chunk:", img.info['srgb'])
