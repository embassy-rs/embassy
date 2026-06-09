#include "port.h"

#include "lvgl.h"

static uint16_t *g_framebuffer;
static uint16_t g_fb_width;
static uint16_t g_fb_height;

static lv_disp_draw_buf_t g_draw_buf;
static lv_color_t g_draw_buf_pixels[800 * 10];
static lv_disp_drv_t g_disp_drv;

static lv_indev_drv_t g_indev_drv;
static lv_indev_data_t g_touch_data;

static void disp_flush_cb(lv_disp_drv_t *drv, const lv_area_t *area, lv_color_t *color_p)
{
    if (g_framebuffer != NULL) {
        int32_t x;
        int32_t y;
        for (y = area->y1; y <= area->y2; y++) {
            for (x = area->x1; x <= area->x2; x++) {
                if (x >= 0 && y >= 0 && (uint32_t)x < g_fb_width && (uint32_t)y < g_fb_height) {
                    g_framebuffer[(uint32_t)y * g_fb_width + (uint32_t)x] = color_p->full;
                }
                color_p++;
            }
        }
    }

    lv_disp_flush_ready(drv);
}

static void touch_read_cb(lv_indev_drv_t *drv, lv_indev_data_t *data)
{
    (void)drv;
    *data = g_touch_data;
}

void rvt50_lvgl_init(uint16_t *framebuffer, uint16_t width, uint16_t height)
{
    g_framebuffer = framebuffer;
    g_fb_width = width;
    g_fb_height = height;

    lv_init();

    lv_disp_draw_buf_init(&g_draw_buf, g_draw_buf_pixels, NULL, width * 10);
    lv_disp_drv_init(&g_disp_drv);
    g_disp_drv.hor_res = width;
    g_disp_drv.ver_res = height;
    g_disp_drv.flush_cb = disp_flush_cb;
    g_disp_drv.draw_buf = &g_draw_buf;
    lv_disp_drv_register(&g_disp_drv);

    lv_indev_drv_init(&g_indev_drv);
    g_indev_drv.type = LV_INDEV_TYPE_POINTER;
    g_indev_drv.read_cb = touch_read_cb;
    lv_indev_drv_register(&g_indev_drv);

    g_touch_data.state = LV_INDEV_STATE_RELEASED;
    g_touch_data.point.x = 0;
    g_touch_data.point.y = 0;

    lv_obj_t *title = lv_label_create(lv_scr_act());
    lv_label_set_text(title, "RVT50HQSNWC00-B");
    lv_obj_align(title, LV_ALIGN_CENTER, 0, -60);

    lv_obj_t *subtitle = lv_label_create(lv_scr_act());
    lv_label_set_text(subtitle, "LVGL on Embassy");
    lv_obj_align(subtitle, LV_ALIGN_CENTER, 0, -20);

    lv_obj_t *hint = lv_label_create(lv_scr_act());
    lv_label_set_text(hint, "Touch the screen");
    lv_obj_align(hint, LV_ALIGN_CENTER, 0, 40);
}

void rvt50_lvgl_set_touch(uint16_t x, uint16_t y, bool pressed)
{
    g_touch_data.point.x = x;
    g_touch_data.point.y = y;
    g_touch_data.state = pressed ? LV_INDEV_STATE_PRESSED : LV_INDEV_STATE_RELEASED;
}

void rvt50_lvgl_tick(uint32_t ms)
{
    lv_tick_inc(ms);
}

void rvt50_lvgl_handler(void)
{
    lv_timer_handler();
}
