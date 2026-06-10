//! STM32U5 + Embassy LTDC platform glue for OxivGL.

extern crate alloc;

use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig};
use embassy_stm32::peripherals;
use embassy_time::{Duration, Timer};
use oxivgl::display::{LvglBuffers, DISPLAY_READY};
use oxivgl::driver::LvglDriver;
use oxivgl::view::{register_view_events, View};
use oxivgl_sys::{lv_obj_t, lv_screen_active, LV_DEF_REFR_PERIOD};
use static_cell::StaticCell;

use crate::oxivgl::display::{
    front_framebuffer, lvgl_disp_init_ltdc, lvgl_display, prefill_background,
    present_framebuffer, sync_back_from_front,
};
use crate::oxivgl::widget_view::WidgetView;
use crate::rvt50_board::DISPLAY_WIDTH;

const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;
/// LVGL timer ticks per LTDC frame (~32 ms refresh cadence).
const LVGL_TICKS_PER_FRAME: usize = 4;

/// OxivGL stripe buffer height (lines × width × 2 bytes per stripe buffer).
pub const COLOR_BUF_LINES: usize = 20;
pub const LVGL_BUF_BYTES: usize = crate::rvt50_board::DISPLAY_WIDTH * COLOR_BUF_LINES * 2;

static VIEW: StaticCell<WidgetView> = StaticCell::new();

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

#[cfg(feature = "touch")]
use defmt::info;

#[cfg(feature = "touch")]
static mut TOUCH_WAS_PRESSED: bool = false;

#[cfg(feature = "touch")]
fn publish_board_touch(
    i2c: &mut embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Blocking,
        embassy_stm32::i2c::Master,
    >,
) {
    let touch = crate::rvt50_board::read_touch(i2c);
    // SAFETY: UI task only.
    let was_pressed = unsafe { TOUCH_WAS_PRESSED };
    if touch.pressed && !was_pressed {
        info!(
            "oxivgl touch down x={} y={} raw=0x{:02x}",
            touch.x,
            touch.y,
            touch.raw_status
        );
    } else if !touch.pressed && was_pressed {
        info!("oxivgl touch up");
    }
    // SAFETY: UI task only.
    unsafe {
        TOUCH_WAS_PRESSED = touch.pressed;
    }
    crate::oxivgl::indev::publish_touch(crate::oxivgl::indev::TouchSample {
        x: touch.x as i32,
        y: touch.y as i32,
        pressed: touch.pressed,
    });
}

async fn lvgl_ticks<const N: usize>(
    driver: &LvglDriver,
    #[cfg(feature = "touch")] i2c: &mut Option<
        embassy_stm32::i2c::I2c<
            'static,
            embassy_stm32::mode::Blocking,
            embassy_stm32::i2c::Master,
        >,
    >,
) {
    for _ in 0..N {
        #[cfg(feature = "touch")]
        if let Some(i2c) = i2c.as_mut() {
            publish_board_touch(i2c);
        }
        driver.timer_handler();
        Timer::after(Duration::from_millis(LVGL_TICK_MS)).await;
    }
}

async fn present_to_ltdc(ltdc: &mut Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>) {
    let fb_ptr = present_framebuffer();
    ltdc.init_buffer(LtdcLayer::Layer1, fb_ptr as *const _);
    let _ = ltdc.reload().await;
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
        );
    };

    DISPLAY_READY.wait().await;
    prefill_background();

    // Show the prefilled background immediately.
    ltdc.init_buffer(LtdcLayer::Layer1, front_framebuffer() as *const _);
    let _ = ltdc.reload().await;

    #[cfg(feature = "touch")]
    {
        let disp = lvgl_display();
        // SAFETY: display was created above; lv_init() completed.
        let _indev = unsafe { crate::oxivgl::indev::register_pointer_indev(disp) };
    }

    let view = VIEW.init(WidgetView::default());
    let screen = unsafe { lv_screen_active() };
    assert!(!screen.is_null());
    let container = oxivgl::widgets::Obj::from_raw_non_owning(screen as *mut lv_obj_t);
    if view.create(&container).is_err() {
        defmt::warn!("oxivgl widget create failed");
        loop {
            Timer::after(Duration::from_secs(60)).await;
        }
    }
    register_view_events(view);

    // First full render: extra ticks so partial LVGL stripes cover the panel.
    sync_back_from_front();
    #[cfg(feature = "touch")]
    lvgl_ticks::<8>(&driver, &mut i2c).await;
    #[cfg(not(feature = "touch"))]
    lvgl_ticks::<8>(&driver).await;
    let _ = view.update();
    present_to_ltdc(&mut ltdc).await;

    loop {
        sync_back_from_front();

        #[cfg(feature = "touch")]
        lvgl_ticks::<LVGL_TICKS_PER_FRAME>(&driver, &mut i2c).await;
        #[cfg(not(feature = "touch"))]
        lvgl_ticks::<LVGL_TICKS_PER_FRAME>(&driver).await;

        let _ = view.update();
        present_to_ltdc(&mut ltdc).await;
    }
}
