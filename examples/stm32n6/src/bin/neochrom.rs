#![no_std]
#![no_main]

//! NeoChrom (NemaGFX) smoke test for the STM32N6570-DK.
//!
//! Initializes the GPU via [`embassy_stm32_neochrom::NeoChrom`] and exercises the
//! hardware-accelerated fill, line, circle, and triangle APIs on a small RGBA8888
//! framebuffer.
//!
//! With the default `stub-gpu2d` feature the HAL is a link-time stub — the
//! framebuffer may stay zero on hardware until the stub is disabled. The example
//! still validates NemaGFX init, linking, and the GPU command-list path from an
//! Embassy application.
//!
//! For real GPU2D on hardware:
//! ```text
//! cargo run --release --bin neochrom --no-default-features
//! ```
//!
//! Inspired by ST's
//! [`x-cube-image-processing`](https://github.com/STMicroelectronics/x-cube-image-processing)
//! reference on the STM32N6570-DK.
//!
//! Generate `stm32-bindings` first:
//! ```text
//! cd ../../../stm32-bindings
//! cargo run --release --bin stm32-bindings-gen -- --module nema_gfx
//! ```

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::rcc::SupplyConfig;
use embassy_stm32::{Config, pac};
#[cfg(not(feature = "stub-gpu2d"))]
use embassy_stm32::{bind_interrupts, peripherals};
#[cfg(not(feature = "stub-gpu2d"))]
use embassy_stm32_neochrom::InterruptHandler as Gpu2dInterruptHandler;
use embassy_stm32_neochrom::{FrameBuffer, NeoChrom, Rgba8888};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const FB_WIDTH: u32 = 64;
const FB_HEIGHT: u32 = 64;
const FB_PIXELS: usize = (FB_WIDTH * FB_HEIGHT) as usize;

#[cfg(not(feature = "stub-gpu2d"))]
bind_interrupts!(struct Irqs {
    GPU2D_ER => Gpu2dInterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    // DK uses external SMPS (UM3300 Tab.6); embassy default = internal SMPS hangs init() at VOSRDY.
    config.rcc.supply_config = SupplyConfig::External;
    let p = embassy_stm32::init(config);
    #[cfg(feature = "stub-gpu2d")]
    let _ = p;

    enable_all_sram();
    promote_gpu2d_master_attributes();

    info!("stm32n6 neochrom example starting");

    #[cfg(feature = "stub-gpu2d")]
    let mut gpu = NeoChrom::new().expect("NeoChrom init failed");

    #[cfg(not(feature = "stub-gpu2d"))]
    let mut gpu = NeoChrom::new(p.GPU2D, Irqs).expect("NeoChrom init failed");

    let fb = FrameBuffer::<FB_WIDTH, FB_HEIGHT, FB_PIXELS>::new();
    let mut frame = 0u32;

    loop {
        let hue = frame % 360;
        let bg = hsl_to_rgba(hue, 40, 12);
        let accent = hsl_to_rgba((hue + 120) % 360, 80, 55);
        let highlight = hsl_to_rgba((hue + 240) % 360, 90, 70);

        gpu.clear(&fb, bg).expect("NeoChrom clear failed");
        gpu.fill_rect(&fb, 0, 0, FB_WIDTH as i32, 8, Rgba8888::new(24, 6, 10, 0xFF))
            .expect("fill_rect failed");

        let cx = (FB_WIDTH / 2) as i32;
        let cy = (FB_HEIGHT / 2) as i32;
        let radius = 12 + ((frame / 4) % 16) as i32;
        gpu.fill_circle(&fb, cx, cy, radius, accent)
            .expect("fill_circle failed");

        gpu.draw_line(&fb, 0, 0, FB_WIDTH as i32 - 1, FB_HEIGHT as i32 - 1, highlight)
            .expect("draw_line failed");
        gpu.draw_line(&fb, FB_WIDTH as i32 - 1, 0, 0, FB_HEIGHT as i32 - 1, highlight)
            .expect("draw_line failed");

        let tri_offset = ((frame / 2) % 20) as i32;
        gpu.fill_triangle(
            &fb,
            4 + tri_offset,
            FB_HEIGHT as i32 - 6,
            20 + tri_offset,
            FB_HEIGHT as i32 - 6,
            12 + tri_offset,
            FB_HEIGHT as i32 - 22,
            Rgba8888::WHITE,
        )
        .expect("fill_triangle failed");

        info!("gpu frame={} hue={}", frame, hue);
        frame += 1;
        Timer::after_millis(250).await;
    }
}

fn hsl_to_rgba(h: u32, s: u8, l: u8) -> Rgba8888 {
    let s = s as f32 / 100.0;
    let l = l as f32 / 100.0;
    let h = (h % 360) as f32 / 60.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Rgba8888::new(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
        0xFF,
    )
}

/// Enable run-mode clocks for every AXISRAM bank (same as the LTDC example).
fn enable_all_sram() {
    pac::RCC.memenr().modify(|w| {
        w.set_axisram1en(true);
        w.set_axisram2en(true);
        w.set_axisram3en(true);
        w.set_axisram4en(true);
        w.set_axisram5en(true);
        w.set_axisram6en(true);
        w.set_ahbsram1en(true);
        w.set_ahbsram2en(true);
        w.set_bkpsramen(true);
    });
}

/// Promote GPU2D AXI master attributes so the RISAF default region accepts hardware DMA reads and writes.
fn promote_gpu2d_master_attributes() {
    use embassy_stm32::rif::{RifMaster, RifMasterAttributes, RifPeripheral, RifPeripheralAttributes};
    RifMaster::Gpu2d.set_attributes(&RifMasterAttributes::new(1, true, true));
    RifPeripheral::Gpu2d.set_attributes(&RifPeripheralAttributes::new(true, true));
}
