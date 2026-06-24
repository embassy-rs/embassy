#![no_std]
#![no_main]
#![allow(static_mut_refs)]

//! OxivGL (C LVGL v9.5) multi-widget demo on the gen4-RP2350-70CT.
//!
//! Real LVGL compiled via [`oxivgl-sys`] (`conf/lv_conf.h`), rendered into the
//! single persistent PSRAM scan-out framebuffer driven by the PIO + DMA RGB
//! engine (`pio_rgb`). This is the OxivGL counterpart to `rlvgl_widget_demo`,
//! reusing the same board bring-up (PSRAM, backlight, FT5446 touch, scan-out)
//! but driving the UI with the C LVGL stack instead of the pure-Rust `rlvgl`.
//!
//! Capacitive touch runs in a dedicated interrupt-driven Embassy task
//! ([`touch_feed::run_touch_int_task`]); the UI task owns LVGL.
//!
//! ```bash
//! cargo run --release --bin oxivgl_widget_demo --features oxivgl-demo
//! ```

extern crate alloc;

use core::mem::MaybeUninit;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_gen4_rp2350_70ct_examples::board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_gen4_rp2350_70ct_examples::firmware_id::FIRMWARE_ID;
use embassy_gen4_rp2350_70ct_examples::oxivgl::platform::{self, LVGL_BUF_BYTES};
use embassy_gen4_rp2350_70ct_examples::pio_rgb::{self, ScanOutIrqs};
use embassy_gen4_rp2350_70ct_examples::touch_feed;
use embassy_time::Timer;
use embedded_alloc::LlffHeap as Heap;
use oxivgl::display::LvglBuffers;
use {defmt_rtt as _, panic_probe as _};

// LVGL has its own builtin pool (`LV_MEM_SIZE`); the Rust global allocator only
// backs the small widget Vecs in the view, so a modest heap is plenty and keeps
// SRAM free for the LVGL pool and the PIO scan-out bounce buffers.
const HEAP_SIZE: usize = 32 * 1024;

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
        "gen4-RP2350-70CT OxivGL demo ({}x{}) firmware={:a}",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        FIRMWARE_ID.as_bytes()
    );

    let p = board::init();
    Timer::after_millis(100).await;
    board::log_board_info();

    let _psram = board::init_psram(p.QMI_CS1, p.PIN_0).expect("PSRAM required for display");
    // Single persistent RGB565 framebuffer in PSRAM. LVGL renders in PARTIAL
    // mode and flushes dirty regions straight into this live framebuffer, so the
    // background and static widgets stay resident and only the changed pixels
    // are rewritten each frame (no full-frame rewrite, no double-buffer swap).
    let fb = _psram.base_address().cast::<u16>();
    pio_rgb::fill_framebuffer(fb, 0x001F);

    let mut backlight = board::init_backlight(p.PWM_SLICE0, p.PIN_17);
    backlight.set_level(15);

    let mut i2c = board::init_i2c(p.I2C1, p.PIN_39, p.PIN_46);
    let mut touch_pins = board::init_touch_pins(p.PIN_47, p.PIN_38);
    board::init_ft5446(&mut i2c, &mut touch_pins).await;
    let touch_int = touch_pins.int;
    spawner.spawn(unwrap!(touch_feed::run_touch_int_task(i2c, touch_int)));

    pio_rgb::init_scanout(
        p.PIO1,
        p.PIO2,
        p.DMA_CH0,
        p.DMA_CH1,
        p.DMA_CH2,
        ScanOutIrqs,
        p.PIN_18,
        p.PIN_19,
        p.PIN_20,
        p.PIN_21,
        p.PIN_22,
        p.PIN_23,
        p.PIN_24,
        p.PIN_25,
        p.PIN_26,
        p.PIN_27,
        p.PIN_28,
        p.PIN_29,
        p.PIN_30,
        p.PIN_31,
        p.PIN_32,
        p.PIN_33,
        p.PIN_34,
        p.PIN_35,
        p.PIN_36,
        p.PIN_37,
        // scan-out displays the single persistent framebuffer
        fb,
    );

    // SAFETY: static LVGL stripe buffers are only used from the UI task.
    let bufs = unsafe { &mut LVGL_BUFS };
    spawner.spawn(unwrap!(ui_task(fb, bufs)));

    loop {
        Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn ui_task(fb: *mut u16, bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>) -> ! {
    platform::run_widget_demo(fb, bufs).await
}
