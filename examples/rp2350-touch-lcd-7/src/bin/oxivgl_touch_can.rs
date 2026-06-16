#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate alloc;

use core::mem::MaybeUninit;
use core::slice;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp2350_touch_lcd_7_examples::board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_time::Timer;
use embassy_rp2350_touch_lcd_7_examples::can_driver;
use embassy_rp2350_touch_lcd_7_examples::oxivgl::display;
use embassy_rp2350_touch_lcd_7_examples::oxivgl::hall_platform;
use embassy_rp2350_touch_lcd_7_examples::oxivgl::platform::LVGL_BUF_BYTES;
use embassy_rp2350_touch_lcd_7_examples::oxivgl::touch_feed;
use embassy_rp2350_touch_lcd_7_examples::pio_rgb;
use embassy_rp2350_touch_lcd_7_examples::touch_can;
use embassy_rp2350_touch_lcd_7_examples::usb_monitor;
use embassy_rp2350_touch_lcd_7_examples::xl2515::CanSpi;
use embedded_alloc::LlffHeap as Heap;
use oxivgl::display::LvglBuffers;
use touch_hall_common::{CAN_BAUD, CAN_ENABLED, HALL_NAME};
use {panic_probe as _};

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
        "RP2350 OxivGL hall CAN ({}x{}) — {}",
        DISPLAY_WIDTH, DISPLAY_HEIGHT, HALL_NAME
    );

    let p = board::init();
    usb_monitor::spawn(&spawner, p.USB);
    // Let the USB defmt task enumerate before early boot logs.
    Timer::after_millis(200).await;
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

    let can = CanSpi::new(p.SPI0, p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5);
    can_driver::install(can, CAN_BAUD).await;

    if CAN_ENABLED {
        spawner.spawn(unwrap!(touch_can::tx_task()));
        spawner.spawn(unwrap!(touch_can::rx_task()));
        info!("XL2515 CAN enabled at {} bit/s", CAN_BAUD);
    } else {
        info!("CAN disabled in config — UI only");
    }

    let bufs = unsafe { &mut LVGL_BUFS };
    spawner.spawn(unwrap!(ui_task(bufs)));

    loop {
        embassy_time::Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn ui_task(bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>) -> ! {
    hall_platform::run_hall_demo(bufs).await
}
