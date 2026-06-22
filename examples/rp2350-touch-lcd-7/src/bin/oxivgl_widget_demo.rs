#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate alloc;

use core::mem::MaybeUninit;
use core::slice;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp2350_touch_lcd_7_examples::board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_rp2350_touch_lcd_7_examples::oxivgl::display;
use embassy_rp2350_touch_lcd_7_examples::oxivgl::platform::{self, LVGL_BUF_BYTES};
use embassy_rp2350_touch_lcd_7_examples::oxivgl::touch_feed;
use embassy_rp2350_touch_lcd_7_examples::pio_rgb;
use embedded_alloc::LlffHeap as Heap;
use oxivgl::display::LvglBuffers;
use {defmt_rtt as _, panic_probe as _};

const HEAP_SIZE: usize = 256 * 1024;

#[global_allocator]
static HEAP: Heap = Heap::empty();

static mut HEAP_MEM: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
static mut LVGL_BUFS: LvglBuffers<{ LVGL_BUF_BYTES }> = LvglBuffers::new();

fn init_heap() {
    unsafe {
        HEAP.init(HEAP_MEM.as_mut_ptr() as usize, HEAP_SIZE);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    init_heap();

    info!(
        "RP2350 OxivGL widget demo ({}x{})",
        DISPLAY_WIDTH, DISPLAY_HEIGHT
    );

    let p = board::init();
    board::log_board_info();

    if let Some(psram) = board::init_psram(p.QMI_CS1, p.PIN_0) {
        let base = psram.base_address();
        let size = psram.size() as usize;
        let psram_slice = unsafe { slice::from_raw_parts_mut(base, size) };
        let _ = display::init_psram_framebuffers(psram_slice.as_mut_ptr(), size);
    }

    let mut lcd = board::init_lcd_pins(p.PIN_41, p.PIN_45, p.PIN_44);
    lcd.set_backlight(true);
    pio_rgb::init_scanout();

    let mut i2c = board::init_i2c(p.I2C1, p.PIN_7, p.PIN_6);
    let mut touch_pins = board::init_touch_pins(p.PIN_19, p.PIN_18);
    board::init_gt911(&mut i2c, &mut touch_pins).await;
    let touch_int = touch_pins.int;
    spawner.spawn(unwrap!(touch_feed::run_touch_int_task(i2c, touch_int)));

    let bufs = unsafe { &mut LVGL_BUFS };
    spawner.spawn(unwrap!(ui_task(bufs)));

    loop {
        embassy_time::Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn ui_task(bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>) -> ! {
    platform::run_widget_demo(bufs).await
}
