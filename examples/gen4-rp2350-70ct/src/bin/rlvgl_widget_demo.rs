#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate alloc;

use core::mem::MaybeUninit;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_gen4_rp2350_70ct_examples::board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_gen4_rp2350_70ct_examples::firmware_id::FIRMWARE_ID;
use embassy_gen4_rp2350_70ct_examples::pio_rgb::{self, ScanOutIrqs};
use embassy_gen4_rp2350_70ct_examples::rlvgl::{render_tree, DemoUi};
use embassy_gen4_rp2350_70ct_examples::touch_feed;
use embassy_time::{Duration, Timer};
use embedded_alloc::LlffHeap as Heap;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const HEAP_SIZE: usize = 256 * 1024;

#[global_allocator]
static HEAP: Heap = Heap::empty();

static mut HEAP_MEM: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
static UI: StaticCell<DemoUi> = StaticCell::new();

fn init_heap() {
    unsafe {
        HEAP.init(HEAP_MEM.as_mut_ptr() as usize, HEAP_SIZE);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    init_heap();

    info!(
        "gen4-RP2350-70CT rlvgl demo ({}x{}) firmware={:a}",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        FIRMWARE_ID.as_bytes()
    );

    let p = board::init();
    Timer::after_millis(100).await;
    board::log_board_info();

    let _psram = board::init_psram(p.QMI_CS1, p.PIN_0).expect("PSRAM required for display");
    // Single persistent RGB565 framebuffer in PSRAM (matches the C reference's
    // single-framebuffer + partial-flush strategy). The background and static
    // widgets stay resident, so only the few changing widgets are redrawn each
    // frame — no full-frame 768 KiB rewrite and no double-buffer swap competing
    // with scan-out on the QMI bus.
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

    let ui = UI.init(DemoUi::new());
    // Paint the initial full frame into the framebuffer that is being scanned out.
    render_tree(fb, ui.root());

    spawner.spawn(unwrap!(ui_task(fb, ui)));

    loop {
        Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn ui_task(fb: *mut u16, ui: &'static mut DemoUi) -> ! {
    let touch_rx = touch_feed::receiver();
    let mut anim_tick: u32 = 0;

    loop {
        let mut touched = false;
        let mut animated = false;

        while let Ok(sample) = touch_rx.try_receive() {
            ui.handle_touch(sample.x, sample.y, sample.pressed);
            touched = true;
        }

        anim_tick = anim_tick.wrapping_add(1);
        if anim_tick % 40 == 0 {
            ui.tick_bar();
            animated = true;
        }

        if touched {
            // A touch can change static widgets (button press state, status
            // label text), so repaint the whole tree into the single live
            // framebuffer. Touches are rare, so the occasional full write is
            // cheap relative to doing it every animation tick.
            render_tree(fb, ui.root());
        } else if animated {
            // Frequent animation: redraw only the changing widgets (bar + LED)
            // directly into the persistent framebuffer. This writes a few KiB
            // instead of the full 768 KiB frame, keeping the QMI bus free for
            // the scan-out refill DMA.
            ui.render_dynamic(fb);
        }

        Timer::after(Duration::from_millis(16)).await;
    }
}
