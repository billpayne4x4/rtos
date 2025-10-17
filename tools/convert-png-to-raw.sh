#!/usr/bin/env bash
set -euo pipefail

IN="images/rtos-logo-transparent.png"
OUT="images/rtos-logo-transparent.raw"

echo "Converting '$IN' → '$OUT'..."

# Ensure ImageMagick is installed
if ! command -v convert >/dev/null 2>&1; then
  echo "❌ ImageMagick 'convert' not found. Please install it (apt install imagemagick)."
  exit 1
fi

# Normalize, resize to fit in 1024x1024, then pad to exact square 1024x1024
convert "$IN" \
  -colorspace sRGB \
  -alpha on \
  -background none \
  -strip \
  -resize 1024x1024 \
  -gravity center -extent 1024x1024 \
  -depth 8 RGBA:"$OUT"

# Display confirmation and dimensions
identify -format "✅ Output: %f (%wx%h)\n" "$IN" || echo "✅ Wrote $OUT (RGBA raw 1024×1024)"
echo "Done."
