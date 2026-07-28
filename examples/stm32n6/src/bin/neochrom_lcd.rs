#![no_std]
#![no_main]

//! NeoChrom (GPU2D) + LTDC display example for the STM32N6570-DK.
//!
//! Drives the on-board 5" RK050HR18C-B01 panel (800x480 parallel RGB) via LTDC,
//! using the NeoChrom (GPU2D) hardware accelerator to render directly into
//! double-buffered RGB565 AXISRAM framebuffers.
//!
//! Features integrated:
//! - NeoChrom / NemaGFX GPU2D driver with real peripheral init and interrupt handling.
//! - GPU-accelerated clears, fills, circles, lines, and blits into LTDC scan-out buffers.
//! - RIF (Resource Isolation Framework) master attribute promotion for GPU2D & LTDC.
//! - Run-mode clocking enabled across all AXISRAM banks.
//! - RK050HR18C panel power sequencing & LTDC 24-bit RGB888 pin muxing.
//! - Double buffering in AXISRAM with VBlank reload sync (`ltdc.set_buffer().await`).
//!
//! Inspired by ST's
//! [`x-cube-image-processing`](https://github.com/STMicroelectronics/x-cube-image-processing)
//! DrawPolygon and Resize_GPU examples on the STM32N6570-DK.
//!
//! For real GPU2D on hardware:
//! ```text
//! cargo run --release --bin neochrom_lcd --no-default-features
//! ```

#[path = "../rk050hr18c.rs"]
mod rk050hr18c;

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig, PixelFormat};
use embassy_stm32::rcc::mux::Ltdcsel;
use embassy_stm32::rcc::{CpuClk, IcConfig, Icint, Icsel, Pll, Plldivm, Pllpdiv, Pllsel, SupplyConfig, SysClk};
use embassy_stm32::rif::{RifMaster, RifMasterAttributes, RifPeripheral, RifPeripheralAttributes};
use embassy_stm32::{Config, bind_interrupts, gpu2d, pac, peripherals};
use embassy_stm32_neochrom::{BlendMode, ExternalFrameBuffer, FrameBuffer, GpuSurface, NeoChrom, Rgba8888};
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

use crate::rk050hr18c::{HEIGHT, LTDC_CONFIG, Rk050Hr18c, WIDTH};

bind_interrupts!(struct Irqs {
    LTDC_LO => ltdc::InterruptHandler<peripherals::LTDC>;
    GPU2D_ER => gpu2d::InterruptHandler<peripherals::GPU2D>;
});

const FB0_BASE: usize = 0x3410_0000;
const FB1_BASE: usize = 0x3420_0000;

const SPRITE_SIZE: u32 = 64;
const SPRITE_PIXELS: usize = (SPRITE_SIZE * SPRITE_SIZE) as usize;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.supply_config = SupplyConfig::External;

    // PLL1: 800 MHz CPU, 200 MHz system bus.
    config.rcc.pll1 = Some(Pll::Oscillator {
        source: Pllsel::Hsi,
        divm: Plldivm::Div4,
        fractional: 0,
        divn: 50,
        divp1: Pllpdiv::Div1,
        divp2: Pllpdiv::Div1,
    });
    config.rcc.ic1 = Some(IcConfig {
        source: Icsel::Pll1,
        divider: Icint::Div1,
    });
    let sys_ic = IcConfig {
        source: Icsel::Pll1,
        divider: Icint::Div4,
    };
    config.rcc.ic2 = Some(sys_ic);
    config.rcc.ic6 = Some(sys_ic);
    config.rcc.ic11 = Some(sys_ic);
    config.rcc.cpu = CpuClk::Ic1;
    config.rcc.sys = SysClk::Ic2;

    // PLL4: 32 MHz pixel clock for LTDC.
    config.rcc.pll4 = Some(Pll::Bypass { source: Pllsel::Hsi });
    config.rcc.ic16 = Some(IcConfig {
        source: Icsel::Pll4,
        divider: Icint::Div2,
    });
    config.rcc.mux.ltdcsel = Ltdcsel::Ic16;

    let p = embassy_stm32::init(config);
    info!("stm32n6 neochrom_lcd demo starting");

    enable_all_sram();
    promote_display_and_gpu_masters();

    let mut panel = Rk050Hr18c::new(p.PE1, p.PQ3, p.PQ6);
    panel.power_on().await;

    let mut ltdc = Ltdc::<_, ltdc::Rgb888>::new_with_pins(
        p.LTDC, Irqs, p.PB13, p.PB14, p.PE11, p.PG13, p.PG15, p.PA7, p.PB2, p.PG6, p.PH3, p.PH6, p.PA8, p.PA2, p.PG12,
        p.PG1, p.PA1, p.PA0, p.PB15, p.PB12, p.PB11, p.PG8, p.PG0, p.PD9, p.PD15, p.PB4, p.PH4, p.PA15, p.PG11, p.PD8,
    );
    ltdc.init(&LTDC_CONFIG);

    #[cfg(feature = "stub-gpu2d")]
    let mut gpu = NeoChrom::new().expect("NeoChrom init failed");

    #[cfg(not(feature = "stub-gpu2d"))]
    let mut gpu = NeoChrom::new(p.GPU2D, Irqs).expect("NeoChrom init failed");

    let layer_config = LtdcLayerConfig {
        pixel_format: PixelFormat::RGB565,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: WIDTH,
        window_y0: 0,
        window_y1: HEIGHT,
    };

    let fb0 = ExternalFrameBuffer::rgb565(FB0_BASE, WIDTH as u32, HEIGHT as u32);
    let fb1 = ExternalFrameBuffer::rgb565(FB1_BASE, WIDTH as u32, HEIGHT as u32);
    let mut sprite = FrameBuffer::<SPRITE_SIZE, SPRITE_SIZE, SPRITE_PIXELS>::new();
    init_sprite(&mut sprite);

    ltdc.init_layer(&layer_config, None);
    ltdc.init_buffer(LtdcLayer::Layer1, FB0_BASE as *const ());
    pac::LTDC.srcr().write(|w| w.set_imr(pac::ltdc::vals::Imr::Reload));

    let scanout = [fb0, fb1];
    let mut back_idx = 1usize;
    let mut frame = 0u32;
    let mut fps_start = Instant::now();
    let mut frame_count = 0u32;

    loop {
        let frame_start = Instant::now();
        let back = scanout[back_idx];
        let hue = frame % 360;
        let bg = hsl_to_rgba(hue, 35, 10);
        let accent = hsl_to_rgba((hue + 140) % 360, 85, 55);

        gpu.begin_frame(&back).expect("begin_frame failed");
        gpu.clear_in_frame(bg).expect("GPU clear failed");
        gpu.fill_rect_in_frame(0, 0, WIDTH as i32, 48, Rgba8888::new(24, 6, 10, 0xFF))
            .expect("title bar fill failed");

        let cx = (WIDTH / 2) as i32;
        let cy = (HEIGHT / 2) as i32;
        let radius = 80 + ((frame / 3) % 60) as i32;
        gpu.fill_circle_in_frame(cx, cy, radius, accent)
            .expect("fill_circle failed");

        gpu.draw_line_in_frame(0, 48, WIDTH as i32 - 1, HEIGHT as i32 - 1, Rgba8888::WHITE)
            .expect("draw_line failed");
        gpu.draw_line_in_frame(WIDTH as i32 - 1, 48, 0, HEIGHT as i32 - 1, Rgba8888::WHITE)
            .expect("draw_line failed");

        let tri_x = 40 + ((frame / 2) % 120) as i32;
        gpu.fill_triangle_in_frame(
            tri_x,
            HEIGHT as i32 - 40,
            tri_x + 80,
            HEIGHT as i32 - 40,
            tri_x + 40,
            HEIGHT as i32 - 120,
            Rgba8888::new(255, 220, 0, 0xFF),
        )
        .expect("fill_triangle failed");

        let quad_shift = ((frame / 3) % 40) as i32;
        gpu.fill_quad_in_frame(
            520 + quad_shift,
            80,
            620 + quad_shift,
            90,
            600 + quad_shift,
            180,
            500 + quad_shift,
            170,
            Rgba8888::new(0, 200, 80, 0xFF),
        )
        .expect("fill_quad failed");

        gpu.draw_stroke_triangle_aa_in_frame(
            tri_x as f32,
            (HEIGHT - 40) as f32,
            (tri_x + 80) as f32,
            (HEIGHT - 40) as f32,
            (tri_x + 40) as f32,
            (HEIGHT - 120) as f32,
            2.0,
            Rgba8888::WHITE,
        )
        .expect("stroke triangle failed");

        gpu.set_blend_blit(BlendMode::Src);
        let sprite_x = (WIDTH as i32 - SPRITE_SIZE as i32 - 24) - ((frame / 2) % 200) as i32;
        gpu.blit_in_frame(&sprite, sprite_x, 64).expect("blit failed");

        let preview_w = 160 + ((frame / 4) % 80) as i32;
        let preview_h = 96 + ((frame / 4) % 48) as i32;
        gpu.blit_rect_fit_in_frame(&sprite, 24, 64, preview_w, preview_h)
            .expect("blit_rect_fit failed");

        let angle = (frame * 3) % 360;
        gpu.blit_rotate_in_frame(&sprite, WIDTH as i32 - 120, HEIGHT as i32 - 120, angle)
            .expect("blit_rotate failed");

        gpu.end_frame_async().await.expect("GPU frame failed");

        let t_flip = Instant::now();
        ltdc.set_buffer(LtdcLayer::Layer1, back.phys_addr() as *const ())
            .await
            .unwrap();
        let flip_us = t_flip.elapsed().as_micros();

        back_idx = 1 - back_idx;
        frame += 1;
        frame_count += 1;

        if fps_start.elapsed().as_millis() >= 1000 {
            info!("fps={} flip_us={} hue={}", frame_count, flip_us, hue);
            fps_start = Instant::now();
            frame_count = 0;
        }

        let elapsed = frame_start.elapsed().as_millis();
        if elapsed < 16 {
            Timer::after_millis(16 - elapsed).await;
        }
    }
}

fn init_sprite(sprite: &mut FrameBuffer<SPRITE_SIZE, SPRITE_SIZE, SPRITE_PIXELS>) {
    let pixels = sprite.pixels_mut();
    for y in 0..SPRITE_SIZE {
        for x in 0..SPRITE_SIZE {
            let dx = x as i32 - SPRITE_SIZE as i32 / 2;
            let dy = y as i32 - SPRITE_SIZE as i32 / 2;
            let dist_sq = dx * dx + dy * dy;
            let radius_sq = (SPRITE_SIZE as i32 / 2 - 4).pow(2);
            let color = if dist_sq <= radius_sq {
                Rgba8888::new(0, 180, 255, 0xFF)
            } else {
                Rgba8888::new(0, 0, 0, 0)
            };
            pixels[(y * SPRITE_SIZE + x) as usize] = color.bits();
        }
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

fn promote_display_and_gpu_masters() {
    for rif_master in [RifMaster::Gpu2d, RifMaster::Dma2d, RifMaster::LtdcL1, RifMaster::LtdcL2] {
        rif_master.set_attributes(&RifMasterAttributes::new(1, true, true));
    }
    for rif_periph in [
        RifPeripheral::Gpu2d,
        RifPeripheral::Dma2d,
        RifPeripheral::LtdcL1,
        RifPeripheral::LtdcL2,
    ] {
        rif_periph.set_attributes(&RifPeripheralAttributes::new(true, true));
    }
}
