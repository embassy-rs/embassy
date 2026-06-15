// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef LVGL_API_H
#define LVGL_API_H

#ifdef __cplusplus
extern "C"
{
#endif

#include "lvgl.h"

#if LV_USE_SDL
#include "src/drivers/sdl/lv_sdl_window.h"
#include "src/drivers/sdl/lv_sdl_mouse.h"
#include "src/drivers/sdl/lv_sdl_keyboard.h"
#endif

    lv_color_t _LV_COLOR_MAKE(uint8_t r, uint8_t g, uint8_t b);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /*LVGL_API*/
