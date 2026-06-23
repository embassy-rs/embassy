#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate alloc;

use core::mem::MaybeUninit;
use core::slice;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp2350_touch_lcd_7_examples::board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_rp2350_touch_lcd_7_examples::firmware_id::FIRMWARE_ID;
use embassy_rp2350_touch_lcd_7_examples::oxivgl::display::{self, PanelMemory};
use embassy_rp2350_touch_lcd_7_examples::oxivgl::platform;
use embassy_rp2350_touch_lcd_7_examples::oxivgl::touch_feed;
use embassy_rp2350_touch_lcd_7_examples::pio_rgb::{self, ScanOutIrqs};
use embassy_rp2350_touch_lcd_7_examples::usb_monitor;
use embassy_time::Timer;
use embedded_alloc::LlffHeap as Heap;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const HEAP_SIZE: usize = 256 * 1024;

#[global_allocator]
static HEAP: Heap = Heap::empty();

static mut HEAP_MEM: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
static PANEL_MEM: StaticCell<PanelMemory> = StaticCell::new();

fn init_heap() {
    unsafe {
        HEAP.init(HEAP_MEM.as_mut_ptr() as usize, HEAP_SIZE);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    init_heap();

    info!(
        "RP2350 OxivGL widget demo ({}x{}) firmware={:a}",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        FIRMWARE_ID.as_bytes()
    );

    let p = board::init();
    usb_monitor::spawn(&spawner, p.USB);
    Timer::after_millis(200).await;
    board::log_board_info();

    let panel_mem = if let Some(psram) = board::init_psram(p.QMI_CS1, p.PIN_0) {
        let base = psram.base_address();
        let size = psram.size() as usize;
        let psram_slice = unsafe { slice::from_raw_parts_mut(base, size) };
        display::init_psram_memory(psram_slice.as_mut_ptr(), size)
            .map(|mem| PANEL_MEM.init(mem))
    } else {
        None
    };

    let panel_mem = panel_mem.expect("PSRAM required for OxivGL display");

    let mut lcd = board::init_lcd_pins(p.PIN_41, p.PIN_45, p.PWM_SLICE10, p.PIN_44);
    lcd.set_backlight(true);
    display::prefill_background();
    usb_monitor::line("panel: prefill done, init GT911");

    let mut i2c = board::init_i2c(p.I2C1, p.PIN_7, p.PIN_6);
    let mut touch_pins = board::init_touch_pins(p.PIN_19, p.PIN_18);
    board::init_gt911(&mut i2c, &mut touch_pins).await;
    let touch_int = touch_pins.int;
    spawner.spawn(unwrap!(touch_feed::run_touch_int_task(i2c, touch_int)));

    usb_monitor::line("panel: starting PIO RGB scan-out");
    pio_rgb::init_scanout(
        p.PIO1,
        p.PIO2,
        p.DMA_CH0,
        ScanOutIrqs,
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
        p.PIN_38,
        p.PIN_39,
    );

    spawner.spawn(unwrap!(ui_task(panel_mem)));

    loop {
        embassy_time::Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn ui_task(panel_mem: &'static PanelMemory) -> ! {
    platform::run_widget_demo(panel_mem).await
}
