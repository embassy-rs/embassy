#include "lvgl_sys.h"

lv_color_t _LV_COLOR_MAKE(uint8_t r, uint8_t g, uint8_t b)
{
    return lv_color_make(r, g, b);
}

uint16_t _LV_COLOR_GET_R(lv_color_t color)
{
    return LV_COLOR_GET_R(color);
}

uint16_t _LV_COLOR_GET_G(lv_color_t color)
{
    return LV_COLOR_GET_G(color);
}

uint16_t _LV_COLOR_GET_B(lv_color_t color)
{
    return LV_COLOR_GET_B(color);
}

uint16_t _LV_COLOR_GET_A(lv_color_t color)
{
    return LV_COLOR_GET_A(color);
}
