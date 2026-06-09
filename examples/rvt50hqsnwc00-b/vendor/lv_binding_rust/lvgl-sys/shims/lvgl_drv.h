#ifdef __cplusplus
extern "C" {
#endif

#ifndef LVGL_DRV_H
#define LVGL_DRV_H

#include "lv_drv_conf.h"

/* Displays */

#if USE_SDL
#include "lv_drivers/sdl/sdl.h"
#endif

#if USE_MONITOR
#include "lv_drivers/display/monitor.h"
#endif

#if USE_WINDOWS
#include "lv_drivers/win_drv.h"
#endif

#if USE_GTK
#include "lv_drivers/gtkdrv/gtkdrv.h"
#endif

#if USE_SSD1963
#include "lv_drivers/display/SSD1963.h"
#endif

#if USE_R61581
#include "lv_drivers/display/R61581.h"
#endif

#if USE_ST7565
#include "lv_drivers/display/ST7565.h"
#endif

#if USE_GC9A01
#include "lv_drivers/display/GC9A01.h"
#endif

#if USE_UC1610
#include "lv_drivers/display/UC1610.h"
#endif

#if USE_SHARP_MIP
#include "lv_drivers/display/SHARP_MIP.h"
#endif

#if USE_ILI9341
#include "lv_drivers/display/ILI9341.h"
#endif

#if USE_FBDEV || USE_BSD_FBDEV
#include "lv_drivers/display/fbdev.h"
#endif

#if USE_DRM
#include "lv_drivers/display/drm.h"
#endif

/* Input devices */

#if USE_XPT2046
#include "lv_drivers/indev/XPT2046.h"
#endif

#if USE_FT5406EE8
#include "lv_drivers/indev/FT5406EE8.h"
#endif

#if USE_AD_TOUCH
#include "lv_drivers/indev/AD_touch.h"
#endif

#if USE_MOUSE
#include "lv_drivers/indev/mouse.h"
#endif

#if USE_MOUSEWHEEL
#include "lv_drivers/indev/mousewheel.h"
#endif

#if USE_LIBINPUT
#include "lv_drivers/indev/libinput_drv.h"
#endif

#if USE_EVDEV || USE_BSD_EVDEV
#include "lv_drivers/indev/evdev.h"
#endif

#if USE_KEYBOARD
#include "lv_drivers/indev/keyboard.h"
#endif

#endif /* LVGL_DRV_H */

#ifdef __cplusplus
} /* extern "C" */
#endif
