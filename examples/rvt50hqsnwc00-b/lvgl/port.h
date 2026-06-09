#pragma once

#include <stdbool.h>
#include <stdint.h>

void rvt50_lvgl_init(uint16_t *framebuffer, uint16_t width, uint16_t height);
void rvt50_lvgl_set_touch(uint16_t x, uint16_t y, bool pressed);
void rvt50_lvgl_tick(uint32_t ms);
void rvt50_lvgl_handler(void);
