#![no_std]
#![no_main]
#![allow(static_mut_refs)]

//! Multi-widget LVGL-style demo on the Riverdi RVT50.
//!
//! Demonstrates label, button, slider, progress bar, switch, and checkbox
//! widgets using [rlvgl](https://github.com/SoftOboros/rlvgl) on Embassy LTDC.
//! This is the Rust/Embassy counterpart to widget examples in Riverdi's
//! `riverdi-50-stm32u5-lvgl` Cube project (`Middlewares/Third_Party/LVGL`).
//!
//! ```bash
//! cargo run --bin widget_demo --features rlvgl
//! cargo run --bin widget_demo --features rlvgl,touch
//! ```

extern crate alloc;

use core::mem::MaybeUninit;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::rlvgl::runtime::{self, Heap};
use embassy_rvt50hqsnwc00_b_examples::rlvgl::widget_demo;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_stm32::ltdc::{self, Ltdc};
use embassy_stm32::peripherals;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[global_allocator]
static HEAP: Heap = Heap::empty();

static mut HEAP_MEM: MaybeUninit<[u8; runtime::HEAP_SIZE]> = MaybeUninit::uninit();
static mut FB1: [u16; runtime::FB_PIXELS] = [0; runtime::FB_PIXELS];
static mut FB2: [u16; runtime::FB_PIXELS] = [0; runtime::FB_PIXELS];

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    // SAFETY: static mut framebuffers are only accessed from the UI task.
    let (fb1, fb2) = unsafe { (&mut FB1, &mut FB2) };
    // SAFETY: heap memory initialized once before any allocation.
    runtime::init_heap(&HEAP, unsafe { &mut HEAP_MEM });

    info!(
        "RVT50 widget demo ({}x{})",
        DISPLAY_WIDTH, DISPLAY_HEIGHT
    );

    let p = rvt50_board::init_clocks();
    rvt50_board::enable_icache();

    let root = widget_demo::build_widget_demo(DISPLAY_WIDTH as i32, DISPLAY_HEIGHT as i32);

    spawner.spawn(unwrap!(heartbeat_info_task()));

    #[cfg(feature = "touch")]
    {
        let rvt50_board::DisplayResources { ltdc, i2c, touch_int: _ } =
            rvt50_board::init_display(p).await;
        spawner.spawn(unwrap!(ui_touch_task(ltdc, i2c, root, fb1, fb2)));
    }

    #[cfg(not(feature = "touch"))]
    {
        let rvt50_board::DisplayResources { ltdc } = rvt50_board::init_display(p).await;
        spawner.spawn(unwrap!(ui_task(ltdc, root, fb1, fb2)));
    }

    loop {
        Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn heartbeat_info_task() -> ! {
    loop {
        info!("widget demo heartbeat");
        Timer::after_secs(5).await;
    }
}

#[cfg(not(feature = "touch"))]
#[embassy_executor::task]
async fn ui_task(
    ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    root: alloc::rc::Rc<core::cell::RefCell<rlvgl::core::WidgetNode>>,
    fb1: &'static mut [u16; runtime::FB_PIXELS],
    fb2: &'static mut [u16; runtime::FB_PIXELS],
) -> ! {
    runtime::render_loop(ltdc, root, fb1, fb2).await
}

#[cfg(feature = "touch")]
#[embassy_executor::task]
async fn ui_touch_task(
    ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    i2c: embassy_stm32::i2c::I2c<
        'static,
        embassy_stm32::mode::Blocking,
        embassy_stm32::i2c::Master,
    >,
    root: alloc::rc::Rc<core::cell::RefCell<rlvgl::core::WidgetNode>>,
    fb1: &'static mut [u16; runtime::FB_PIXELS],
    fb2: &'static mut [u16; runtime::FB_PIXELS],
) -> ! {
    runtime::render_loop_touch(ltdc, i2c, root, fb1, fb2).await
}
