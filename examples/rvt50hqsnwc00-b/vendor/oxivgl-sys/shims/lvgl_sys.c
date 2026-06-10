// SPDX-License-Identifier: MIT OR Apache-2.0

#include "lvgl_sys.h"

lv_color_t _LV_COLOR_MAKE(uint8_t r, uint8_t g, uint8_t b)
{
    return lv_color_make(r, g, b);
}
