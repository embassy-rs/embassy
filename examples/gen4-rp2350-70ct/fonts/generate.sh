#!/usr/bin/env bash
# Regenerate Montserrat LVGL fonts with Latin-1 glyphs for German umlauts.
set -euo pipefail
cd "$(dirname "$0")"

FONT="${FONT:-/tmp/Montserrat-Regular.ttf}"
if [[ ! -f "$FONT" ]]; then
  curl -fsSL -L -o "$FONT" \
    "https://github.com/JulietaUla/Montserrat/raw/master/fonts/ttf/Montserrat-Regular.ttf"
fi

for size in 14 16; do
  out="lv_font_montserrat_${size}_latin.c"
  npx --yes lv_font_conv \
    --font "$FONT" -r 0x20-0x7F,0xA0-0xFF \
    --size "$size" --bpp 4 --format lvgl --no-compress \
    --lv-font-name "lv_font_montserrat_${size}_latin" \
    -o "$out"
  # oxivgl-sys builds with LV_CONF_INCLUDE_SIMPLE (lvgl.h), not lvgl/lvgl.h.
  sed -i 's|#include "lvgl/lvgl.h"|#include "lvgl.h"|' "$out"
done
