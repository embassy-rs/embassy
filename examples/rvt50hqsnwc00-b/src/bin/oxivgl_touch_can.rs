#![no_std]
#![no_main]
#![allow(static_mut_refs)]

//! JSON-driven OxivGL hall lighting UI with CAN press/hold/repeat on the
//! Riverdi RVT50HQSNWC00-B.
//!
//! Configuration is generated at build time from
//! `examples/touch-projects/Demo/{hall,can}_config.json`.
//!
//! Button highlight state is driven by an optional Rhai script (`state_script` in
//! `can_config.json`). Edit `state.rhai` in the touch project to use logic
//! expressions over incoming CAN data (`can_bit`, `can_byte`, `minp_*` helpers).
//!
//! ```bash
//! cargo run --bin oxivgl_touch_can --features oxivgl
//! ```

extern crate alloc;

use core::mem::MaybeUninit;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::oxivgl::hall_platform;
use embassy_rvt50hqsnwc00_b_examples::oxivgl::platform::LVGL_BUF_BYTES;
use embassy_rvt50hqsnwc00_b_examples::oxivgl::touch_feed;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{self, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use embassy_rvt50hqsnwc00_b_examples::touch_can;
use embassy_stm32::can::{CanRx, CanTx};
use embassy_stm32::ltdc::{self, Ltdc};
use embassy_stm32::peripherals;
use embassy_time::Timer;
use embedded_alloc::LlffHeap as Heap;
use oxivgl::display::LvglBuffers;
use static_cell::StaticCell;
use touch_hall_common::{CAN_BAUD, CAN_ENABLED, HALL_NAME};
use {defmt_rtt as _, panic_probe as _};

const HEAP_SIZE: usize = 256 * 1024;

#[global_allocator]
static HEAP: Heap = Heap::empty();

static mut HEAP_MEM: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
static mut LVGL_BUFS: LvglBuffers<{ LVGL_BUF_BYTES }> = LvglBuffers::new();

static CAN_TX: StaticCell<CanTx<'static>> = StaticCell::new();
static CAN_RX: StaticCell<CanRx<'static>> = StaticCell::new();

fn init_heap() {
    unsafe {
        HEAP.init(HEAP_MEM.as_mut_ptr() as usize, HEAP_SIZE);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    init_heap();

    info!(
        "RVT50 OxivGL hall CAN demo ({}x{}) — {}",
        DISPLAY_WIDTH, DISPLAY_HEIGHT, HALL_NAME
    );

    let p = rvt50_board::init_clocks();
    rvt50_board::enable_icache();

    let bufs = unsafe { &mut LVGL_BUFS };

    let rvt50_board::OxivglCanResources {
        ltdc,
        i2c,
        touch_int,
        fdcan,
        can_rx,
        can_tx,
        can_stb,
    } = rvt50_board::init_oxivgl_can(p).await;
    spawner.spawn(unwrap!(touch_feed::run_touch_int_task(i2c, touch_int)));

    if CAN_ENABLED {
        let mut can = rvt50_board::init_can(fdcan, can_rx, can_tx, can_stb);
        can.set_bitrate(CAN_BAUD);
        let (tx, rx, _) = can.into_normal_mode().split();
        spawner.spawn(unwrap!(touch_can::tx_task(CAN_TX.init(tx))));
        spawner.spawn(unwrap!(touch_can::rx_task(CAN_RX.init(rx))));
        info!("FDCAN enabled at {} bit/s", CAN_BAUD);
    } else {
        info!("CAN disabled in config — UI only");
    }

    spawner.spawn(unwrap!(ui_task(ltdc, bufs)));

    loop {
        Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn ui_task(
    ltdc: Ltdc<'static, peripherals::LTDC, ltdc::Rgb565>,
    bufs: &'static mut LvglBuffers<{ LVGL_BUF_BYTES }>,
) -> ! {
    hall_platform::run_hall_demo(ltdc, bufs).await
}
