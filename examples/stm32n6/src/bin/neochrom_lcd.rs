#![no_std]
#![no_main]

//! NeoChrom (GPU2D) + LTDC display example for the STM32N6570-DK.
//!
//! Drives the on-board 5" RK050HR18C-B01 panel (800x480 parallel RGB) via LTDC,
//! using the NeoChrom (GPU2D) hardware accelerator to perform GPU-driven clears and
//! rendering into double-buffered AXISRAM framebuffers.
//!
//! Features integrated:
//! - NeoChrom / NemaGFX GPU2D driver setup with interrupt handling.
//! - RIF (Resource Isolation Framework) master attribute promotion for GPU2D & LTDC.
//! - Run-mode clocking enabled across all AXISRAM banks.
//! - RK050HR18C panel power sequencing & LTDC 24-bit RGB888 pin muxing.
//! - Double buffering in AXISRAM with VBlank reload sync (`ltdc.set_buffer().await`).

#[path = "../framebuffer.rs"]
mod framebuffer;
#[path = "../rk050hr18c.rs"]
mod rk050hr18c;

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig, PixelFormat};
use embassy_stm32::rcc::mux::Ltdcsel;
use embassy_stm32::rcc::{CpuClk, IcConfig, Icint, Icsel, Pll, Plldivm, Pllpdiv, Pllsel, SupplyConfig, SysClk};
use embassy_stm32::rif::{RifMaster, RifMasterAttributes, RifPeripheral, RifPeripheralAttributes};
use embassy_stm32::{Config, bind_interrupts, gpu2d, pac, peripherals};
use embassy_stm32_neochrom::{FrameBuffer, NeoChrom, Rgba8888};
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

use crate::framebuffer::Framebuffer;
use crate::rk050hr18c::{HEIGHT, LTDC_CONFIG, Rk050Hr18c, WIDTH};

bind_interrupts!(struct Irqs {
    LTDC_LO => ltdc::InterruptHandler<peripherals::LTDC>;
    GPU2D_ER => gpu2d::InterruptHandler<peripherals::GPU2D>;
});

const FB0_BASE: usize = 0x3410_0000;
const FB1_BASE: usize = 0x3420_0000;
const FB_PIXELS: usize = WIDTH as usize * HEIGHT as usize;

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

    let fb0_slice: &'static mut [u16] = unsafe { core::slice::from_raw_parts_mut(FB0_BASE as *mut u16, FB_PIXELS) };
    let fb1_slice: &'static mut [u16] = unsafe { core::slice::from_raw_parts_mut(FB1_BASE as *mut u16, FB_PIXELS) };
    let fb0 = Framebuffer::new(fb0_slice, WIDTH, HEIGHT);
    let fb1 = Framebuffer::new(fb1_slice, WIDTH, HEIGHT);

    let small_fb = FrameBuffer::<64, 64, 4096>::new();

    ltdc.init_layer(&layer_config, None);
    ltdc.init_buffer(LtdcLayer::Layer1, fb0.as_ptr());
    pac::LTDC.srcr().write(|w| w.set_imr(pac::ltdc::vals::Imr::Reload));

    let fbs = [fb0.as_ptr(), fb1.as_ptr()];
    let mut back_idx = 1;

    let colors = [Rgba8888::RED, Rgba8888::GREEN, Rgba8888::BLUE, Rgba8888::WHITE];
    let mut color_idx = 0usize;
    let mut fps_start = Instant::now();
    let mut frame_count = 0u32;

    loop {
        let frame_start = Instant::now();
        let color = colors[color_idx];

        gpu.clear(&small_fb, color).expect("GPU clear failed");

        let t_flip = Instant::now();
        ltdc.set_buffer(LtdcLayer::Layer1, fbs[back_idx]).await.unwrap();
        let flip_us = t_flip.elapsed().as_micros();

        back_idx = 1 - back_idx;
        color_idx = (color_idx + 1) % colors.len();
        frame_count += 1;

        if fps_start.elapsed().as_millis() >= 1000 {
            info!("fps={} flip_us={}", frame_count, flip_us);
            fps_start = Instant::now();
            frame_count = 0;
        }

        let elapsed = frame_start.elapsed().as_millis();
        if elapsed < 16 {
            Timer::after_millis(16 - elapsed).await;
        }
    }
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
