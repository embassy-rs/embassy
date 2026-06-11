#![no_std]
#![no_main]
#![allow(static_mut_refs)]

//! OxivGL (C LVGL v9.5) multi-widget demo on the Riverdi RVT50.
//!
//! Uses real LVGL compiled via [`oxivgl-sys`], with `conf/lv_conf.h` tuned for
//! the STM32U5 LTDC RGB565 panel. This is the Rust/OxivGL counterpart to widget
//! demos in Riverdi's `riverdi-50-stm32u5-lvgl` Cube project.
//!
//! **Requires nightly Rust** (see `rust-toolchain.toml` in this crate).
//!
//! With `touch`, I2C sampling runs in a dedicated Embassy task
//! ([`touch_feed::run_touch_poll_task`]); the UI task owns LVGL and LTDC.
//!
//! ```bash
//! cargo run --bin oxivgl_widget_demo --features oxivgl
//! cargo run --bin oxivgl_widget_demo --features oxivgl,touch
//! ```

extern crate alloc;

use core::mem::MaybeUninit;

use defmt::{info, unwrap};
use embedded_alloc::LlffHeap as Heap;
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::oxivgl::platform::{self, LVGL_BUF_BYTES};
#[cfg(feature = "touch")]
use embassy_rvt50hqsnwc00_b_examples::oxivgl::touch_feed;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_stm32::ltdc::{self, Ltdc};
use embassy_stm32::peripherals;
use embassy_time::Timer;
use oxivgl::display::LvglBuffers;
use {defmt_rtt as _, panic_probe as _};

/// Match rlvgl demo heap sizing; LVGL also uses its built-in allocator in `lv_conf.h`.
const HEAP_SIZE: usize = 256 * 1024;

#[global_allocator]
static HEAP: Heap = Heap::empty();

static mut HEAP_MEM: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
static mut LVGL_BUFS: LvglBuffers<{ LVGL_BUF_BYTES }> = LvglBuffers::new();

fn init_heap() {
    // SAFETY: called once before any allocation.
    unsafe {
        HEAP.init(HEAP_MEM.as_mut_ptr() as usize, HEAP_SIZE);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    init_heap();

    info!(
        "RVT50 OxivGL widget demo ({}x{})",
        DISPLAY_WIDTH, DISPLAY_HEIGHT
    );

    let p = rvt50_board::init_clocks();
    rvt50_board::enable_icache();

    // SAFETY: static LVGL stripe buffers are only used from the UI task.
    let bufs = unsafe { &mut LVGL_BUFS };

    spawner.spawn(unwrap!(heartbeat_info_task()));

    #[cfg(feature = "touch")]
    spawner.spawn(unwrap!(touch_info_task()));

    #[cfg(feature = "touch")]
    {
        let rvt50_board::DisplayResources { ltdc, i2c, touch_int: _ } =
            rvt50_board::init_display(p).await;
        spawner.spawn(unwrap!(touch_feed::run_touch_poll_task(i2c)));
        spawner.spawn(unwrap!(ui_touch_task(ltdc, bufs)));
    }

    #[cfg(not(feature = "touch"))]
    {
        let rvt50_board::DisplayResources { ltdc } = rvt50_board::init_display(p).await;
        spawner.spawn(unwrap!(ui_task(ltdc, bufs)));
    }

    loop {
        Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn heartbeat_info_task() -> ! {
    loop {
        info!("oxivgl widget demo heartbeat");
        Timer::after_secs(5).await;
    }
}

#[cfg(feature = "touch")]
#[embassy_executor::task]
async fn touch_info_task() -> ! {
    use core::sync::atomic::Ordering;

    use embassy_rvt50hqsnwc00_b_examples::oxivgl::touch_dbg;

    loop {
        let pressed = touch_dbg::PRESSED.load(Ordering::Relaxed);
        let i2c_ok = touch_dbg::I2C_OK.load(Ordering::Relaxed);
        let raw = touch_dbg::RAW_STATUS.load(Ordering::Relaxed);
        let x = touch_dbg::X.load(Ordering::Relaxed);
        let y = touch_dbg::Y.load(Ordering::Relaxed);
        let active = touch_dbg::ACTIVE_OBJ.load(Ordering::Relaxed);
        let hit_btn = touch_dbg::HIT_BTN.load(Ordering::Relaxed);
        let events = touch_dbg::EVENT_COUNT.load(Ordering::Relaxed);

        info!(
            "oxivgl touch dbg i2c_ok={} raw=0x{:02x} pressed={} x={} y={} active_obj={:08x} layout_hit={} lvgl_events={}",
            i2c_ok,
            raw,
            pressed,
            x,
            y,
            active,
            hit_btn,
            events
        );
        Timer::after_secs(2).await;
    }
}

#[cfg(not(feature = "touch"))]
#[embassy_executor::task]
async fn ui_task(
    ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>,
) -> ! {
    platform::run_widget_demo(ltdc, bufs).await
}

#[cfg(feature = "touch")]
#[embassy_executor::task]
async fn ui_touch_task(
    ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>,
) -> ! {
    platform::run_widget_demo(ltdc, bufs).await
}
