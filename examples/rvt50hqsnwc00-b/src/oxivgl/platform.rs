//! STM32U5 + Embassy LTDC platform glue for OxivGL.

extern crate alloc;

use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig};
use embassy_stm32::peripherals;
use embassy_time::{Duration, Timer};
use oxivgl::display::{LvglBuffers, DISPLAY_READY};
use oxivgl::driver::LvglDriver;
use oxivgl::view::{register_view_events, View};
use oxivgl_sys::{lv_obj_t, lv_screen_active, LV_DEF_REFR_PERIOD};

use crate::oxivgl::display::{lvgl_disp_init_ltdc, present_framebuffer};
use crate::oxivgl::widget_view::WidgetView;
use crate::rvt50_board::DISPLAY_WIDTH;

const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;

/// OxivGL stripe buffer size (see `oxivgl::display::COLOR_BUF_LINES`).
pub const COLOR_BUF_LINES: usize = 40;
pub const LVGL_BUF_BYTES: usize = crate::rvt50_board::DISPLAY_WIDTH * COLOR_BUF_LINES * 2;

async fn init_ltdc_layer(ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>) {
    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::Rgb565,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: DISPLAY_WIDTH as _,
        window_y0: 0,
        window_y1: crate::rvt50_board::DISPLAY_HEIGHT as _,
    };
    ltdc.init_layer(&layer_config, None);
}

/// Run the OxivGL widget demo (touch optional via `touch` feature).
pub async fn run_widget_demo(
    mut ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>,
    #[cfg(feature = "touch")] mut i2c: Option<
        embassy_stm32::i2c::I2c<
            'static,
            embassy_stm32::mode::Blocking,
            embassy_stm32::i2c::Master,
        >,
    >,
) -> ! {
    init_ltdc_layer(&mut ltdc).await;

    let driver = LvglDriver::init(DISPLAY_WIDTH as i32, crate::rvt50_board::DISPLAY_HEIGHT as i32);
    // SAFETY: lv_init() completed in `LvglDriver::init`.
    unsafe {
        lvgl_disp_init_ltdc(
            DISPLAY_WIDTH as i32,
            crate::rvt50_board::DISPLAY_HEIGHT as i32,
            bufs,
        )
    };

    DISPLAY_READY.wait().await;

    #[cfg(feature = "touch")]
    {
        // SAFETY: lv_init() completed.
        let _indev = unsafe { crate::oxivgl::indev::register_pointer_indev() };
    }

    let mut view = WidgetView::default();
    let screen = unsafe { lv_screen_active() };
    assert!(!screen.is_null());
    let container = oxivgl::widgets::Obj::from_raw_non_owning(screen as *mut lv_obj_t);
    if view.create(&container).is_err() {
        defmt::warn!("oxivgl widget create failed");
        loop {
            Timer::after(Duration::from_secs(60)).await;
        }
    }
    register_view_events(&mut view);

    loop {
        #[cfg(feature = "touch")]
        if let Some(i2c) = i2c.as_mut() {
            let touch = crate::rvt50_board::read_touch(i2c);
            crate::oxivgl::indev::publish_touch(crate::oxivgl::indev::TouchSample {
                x: touch.x as i32,
                y: touch.y as i32,
                pressed: touch.pressed,
            });
        }

        let _ = view.update();

        for _ in 0..4 {
            driver.timer_handler();
            Timer::after(Duration::from_millis(LVGL_TICK_MS)).await;
        }

        let fb_ptr = present_framebuffer();
        ltdc.init_buffer(LtdcLayer::Layer1, fb_ptr as *const _);
        let _ = ltdc.reload().await;
    }
}
