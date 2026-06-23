#![no_std]
#![no_main]

//! Clean OxivGL widget demo for the 4D Systems gen4-RP2350-70CT.
//!
//! Brings up PSRAM double framebuffers, the PIO RGB scan-out for the 800x480
//! panel, and the FT5446 touch controller, then runs an LVGL widget view.

extern crate alloc;

use core::mem::MaybeUninit;
use core::slice;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_gen4_rp2350_70ct_examples::board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_gen4_rp2350_70ct_examples::firmware_id::FIRMWARE_ID;
use embassy_gen4_rp2350_70ct_examples::oxivgl::display::{self, PanelMemory};
use embassy_gen4_rp2350_70ct_examples::oxivgl::{platform, touch_feed};
use embassy_gen4_rp2350_70ct_examples::pio_rgb;
use embedded_alloc::LlffHeap as Heap;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const HEAP_SIZE: usize = 248 * 1024;

/// Diagnostic flag: when `true`, bypass LVGL/touch and drive the PIO RGB
/// scan-out with a cycling solid color (red → green → blue → white → black).
/// Use this to verify that pixel data is latched by the panel and that the
/// RGB565 data-line bit order is correct, then set back to `false`.
const SOLID_COLOR_TEST: bool = true;

#[global_allocator]
static HEAP: Heap = Heap::empty();

static mut HEAP_MEM: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
static PANEL_MEM: StaticCell<PanelMemory> = StaticCell::new();

fn init_heap() {
    unsafe {
        HEAP.init(core::ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    init_heap();

    info!(
        "gen4-RP2350-70CT OxivGL widget demo ({}x{}) [{}]",
        DISPLAY_WIDTH, DISPLAY_HEIGHT, FIRMWARE_ID
    );

    let p = board::init();
    board::log_board_info();

    // PSRAM holds the two full-screen framebuffers plus the DMA staging buffers.
    let psram = board::init_psram(p.QMI_CS1, p.PIN_0).expect("PSRAM required for framebuffers");
    let base = psram.base_address();
    let size = psram.size() as usize;
    let psram_slice = unsafe { slice::from_raw_parts_mut(base, size) };
    let panel_mem =
        display::init_psram_memory(psram_slice.as_mut_ptr(), size).expect("PSRAM too small for framebuffers");
    let panel_mem: &'static PanelMemory = PANEL_MEM.init(panel_mem);

    let mut lcd = board::init_lcd_pins(p.PIN_17);
    lcd.set_backlight(true);

    pio_rgb::init_scanout(
        p.PIO1,
        p.PIO2,
        p.DMA_CH0,
        pio_rgb::ScanOutIrqs, // DE / VSYNC / HSYNC / PCLK
        p.PIN_18,
        p.PIN_19,
        p.PIN_20,
        p.PIN_21, // RGB565 data lines DATA0..DATA15 (GPIO22..=37)
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
    );

    if SOLID_COLOR_TEST {
        // Diagnostic path: drive the PIO RGB scan-out with solid colors only.
        // RGB565 test values: red=0xF800, green=0x07E0, blue=0x001F.
        const COLORS: [(&str, u16); 5] = [
            ("RED 0xF800", 0xF800),
            ("GREEN 0x07E0", 0x07E0),
            ("BLUE 0x001F", 0x001F),
            ("WHITE 0xFFFF", 0xFFFF),
            ("BLACK 0x0000", 0x0000),
        ];
        let mut idx = 0usize;
        display::fill_all(COLORS[0].1);
        loop {
            let (name, px) = COLORS[idx % COLORS.len()];
            display::fill_draw(px);
            pio_rgb::present_swap();
            info!("solid-color test: {} ({=u16:#x})", name, px);
            idx += 1;
            embassy_time::Timer::after_secs(2).await;
        }
    }

    let mut i2c = board::init_i2c(p.I2C1, p.PIN_39, p.PIN_46);
    let mut touch_pins = board::init_touch_pins(p.PIN_47, p.PIN_38);
    board::init_ft5446(&mut i2c, &mut touch_pins).await;
    let touch_int = touch_pins.int;
    spawner.spawn(unwrap!(touch_feed::run_touch_int_task(i2c, touch_int)));

    spawner.spawn(unwrap!(ui_task(panel_mem)));

    loop {
        embassy_time::Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn ui_task(panel_mem: &'static PanelMemory) -> ! {
    platform::run_widget_demo(panel_mem).await
}
