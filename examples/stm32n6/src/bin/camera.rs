#![no_std]
#![no_main]

//! Camera capture → LCD example for the STM32N6570-DK + MB1854 daughterboard.
//!
//! Pipeline: IMX335 (5 Mpx RAW10 RGGB) → CSI-2 2-lane → DCMIPP Pipe1
//! (Bayer demosaic + crop + downsize) → 800×480 RGB565 → LTDC layer 1 →
//! 5" RK050HR18C-B01 panel.
//!
//! The pipeline runs entirely on hardware: there is no CPU involvement in
//! the per-frame path. DCMIPP ping-pongs two framebuffers in AXISRAM, and
//! at each frame-complete IRQ we hand the just-filled buffer to LTDC via
//! `set_buffer().await` for vblank-synchronised swap.

#[path = "../imx335.rs"]
mod imx335;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::csi::{self, Csi, LaneCount};
use embassy_futures::select::{Either, select};
use embassy_stm32::dcmipp::{
    self, BayerPattern, ChannelGains, Dcmipp, DownsizeConfig, InputSource, PixelFormat as DcmippPixelFormat,
    Pipe1Config,
};
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::ltdc::{self, Ltdc, LtdcLayer, LtdcLayerConfig, PixelFormat};
use embassy_stm32::pac;
use embassy_stm32::rcc::mux::{Dcmippsel, Ltdcsel};
use embassy_stm32::rcc::{CpuClk, IcConfig, Icint, Icsel, Pll, Plldivm, Pllpdiv, Pllsel, SysClk};
use embassy_stm32::{Config, bind_interrupts, interrupt, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[path = "../rk050hr18c.rs"]
mod rk050hr18c;
use rk050hr18c::{HEIGHT, LTDC_CONFIG, Rk050Hr18c, WIDTH};

use crate::imx335::Imx335;

bind_interrupts!(struct Irqs {
    LTDC_LO => ltdc::InterruptHandler<peripherals::LTDC>;
    CSI => csi::InterruptHandler<peripherals::CSI>;
    DCMIPP => dcmipp::InterruptHandler<peripherals::DCMIPP>;
    EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
});

/// Static white-balance gains applied at boot. Tuned for the IMX335 in the
/// MB1854 module under typical indoor light without ISP correction —
/// edit if the cast looks different on your bench.
const STATIC_WB: ChannelGains = ChannelGains {
    r: 1.6,
    g: 1.0,
    b: 1.4,
};
/// Auto-WB EMA smoothing factor. Higher = faster but noisier.
const AUTO_WB_ALPHA: f32 = 0.15;

// Two framebuffers in AXISRAM (same regions the lcd.rs example uses for its
// LTDC layer). 800×480 × 2 bytes RGB565 = 768 000 bytes; each fits in the
// 1 MB bank between FB0_BASE and FB1_BASE.
const FB0_BASE: usize = 0x3410_0000;
const FB1_BASE: usize = 0x3420_0000;
const FB_PITCH_BYTES: u16 = WIDTH as u16 * 2;

// Sensor active area = 2592×1944.
const SENSOR_W: u16 = 2592;
const SENSOR_H: u16 = 1944;

// IMX335 in 2-lane RAW10 mode programs its internal PLL for ~1600 Mbps per
// lane (per ST BSP `DCMIPP_CSI_PHY_BT_1600`). We pass that to the CSI HAL so
// the D-PHY band table picks the matching `hs_freq_range`/`osc_freq_target`.
const CSI_RATE_MBPS: u32 = 1600;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(rcc_config());
    info!("stm32n6 camera example starting");

    enable_all_sram();
    promote_axi_masters_to_secure();

    // Sensor.
    let i2c = I2c::new_blocking(p.I2C1, p.PH9, p.PC1, Default::default());
    let pwr_en = Output::new(p.PC8, Level::Low, Speed::Low);
    let nrst = Output::new(p.PD2, Level::Low, Speed::Low);
    let mut cam = Imx335::new(i2c, pwr_en, nrst);
    unwrap!(cam.power_on().await);
    unwrap!(cam.init().await);
    // Crank gain (~30 dB) and open the shutter wide. The IMX335 has no
    // on-chip AEC; without an external auto-exposure loop the default
    // exposure is short and the image looks dim. These static values
    // light it up enough to be useful for a demo.
    unwrap!(cam.set_gain_db_x10(150));
    unwrap!(cam.set_shutter_lines(2000));
    info!("imx335: probe + init OK");

    // CSI: 2 lanes, ~891 Mbps/lane, VC0 only.
    let mut csi = Csi::new(p.CSI, Irqs, csi::Config::new(LaneCount::Two, CSI_RATE_MBPS));

    // DCMIPP Pipe1: RGGB demosaic → downsize 2592×1944 to 800×480 → RGB565.
    // No crop — the BSP runs the same setup with zero-overhead crop.
    let dcmipp = Dcmipp::new(p.DCMIPP, Irqs);
    let (_pipe0, mut pipe1, _pipe2) = dcmipp.split();
    let mut p1cfg = Pipe1Config::new(InputSource::Csi, DcmippPixelFormat::Rgb565, FB_PITCH_BYTES);
    p1cfg.demosaic = Some(BayerPattern::Rggb);
    p1cfg.downsize = Some(DownsizeConfig {
        input: (SENSOR_W, SENSOR_H),
        output: (WIDTH as u16, HEIGHT as u16),
    });
    pipe1.configure(&p1cfg);
    pipe1.set_color_gains(STATIC_WB);
    pipe1.enable_rgb_stats(WIDTH as u16, HEIGHT as u16);

    // LTDC + panel — same wiring as bin/lcd.rs.
    let mut panel = Rk050Hr18c::new(p.PE1, p.PQ3, p.PQ6);
    panel.power_on().await;
    let mut ltdc = Ltdc::<_, ltdc::Rgb888>::new_with_pins(
        p.LTDC,
        Irqs,
        p.PB13, // CLK
        p.PB14, // HSYNC
        p.PE11, // VSYNC
        p.PG13, // DE
        p.PG15, p.PA7, p.PB2, p.PG6, p.PH3, p.PH6, p.PA8, p.PA2, // B0..B7
        p.PG12, p.PG1, p.PA1, p.PA0, p.PB15, p.PB12, p.PB11, p.PG8, // G0..G7
        p.PG0, p.PD9, p.PD15, p.PB4, p.PH4, p.PA15, p.PG11, p.PD8, // R0..R7
    );
    ltdc.init(&LTDC_CONFIG);

    let layer_config = LtdcLayerConfig {
        pixel_format: PixelFormat::RGB565,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: WIDTH,
        window_y0: 0,
        window_y1: HEIGHT,
    };
    ltdc.init_layer(&layer_config, None);
    ltdc.init_buffer(LtdcLayer::Layer1, FB0_BASE as *const ());
    pac::LTDC.srcr().write(|w| w.set_imr(pac::ltdc::vals::Imr::Reload));

    // Start the pipeline. DCMIPP holds the buffer ownership while running
    // — `start_continuous` is `unsafe` because the MMIO master writes
    // asynchronously to FB0/FB1 until `pipe1.stop()` is called below.
    unsafe {
        pipe1.start_continuous(FB0_BASE as *mut u8, FB1_BASE as *mut u8);
    }
    csi.start();
    unwrap!(cam.start_streaming().await);
    info!("streaming");

    // Diagnostics: snapshot CSI + DCMIPP status registers a moment after
    // we kick streaming so we can see whether the D-PHY locked and whether
    // the pipe is moving. If `wait_frame` never resolves, this is the
    // first place to look.
    Timer::after_millis(500).await;
    let csi_r = pac::CSI;
    let dcmipp_r = pac::DCMIPP;
    info!(
        "after start: csi sr0=0x{:08x} sr1=0x{:08x} pcr=0x{:08x} pfcr=0x{:08x} prcr=0x{:08x} cr=0x{:08x}; \
         dcmipp cmsr2=0x{:08x} p1sr=0x{:08x} p1fscr=0x{:08x}",
        csi_r.sr0().read().0,
        csi_r.sr1().read().0,
        csi_r.pcr().read().0,
        csi_r.pfcr().read().0,
        csi_r.prcr().read().0,
        csi_r.cr().read().0,
        dcmipp_r.cmsr2().read().0,
        dcmipp_r.p1sr().read().0,
        dcmipp_r.p1fscr().read().0,
    );

    // Press the User1 button (B2 / PC13, active high → Pull::Down) to
    // toggle auto white balance. When auto is on we read the post-demosaic
    // R/G/B per-channel means latched at end-of-frame and EMA-update the
    // CCM diagonal toward neutral grey.
    let mut button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Down, Irqs);
    let mut auto_wb = false;
    let mut wb = STATIC_WB;
    info!("auto WB = {} (press B2 to toggle)", auto_wb);

    let mut frames: u32 = 0;
    loop {
        match select(pipe1.wait_frame(), button.wait_for_rising_edge()).await {
            Either::First(Ok(idx)) => {
                let ptr = if idx == 0 { FB0_BASE } else { FB1_BASE } as *const ();
                unwrap!(ltdc.set_buffer(LtdcLayer::Layer1, ptr).await);
                frames = frames.wrapping_add(1);

                if auto_wb {
                    let (mr, mg, mb) = pipe1.read_rgb_means();
                    if mr.min(mg).min(mb) > 0 {
                        let r = mr as f32;
                        let g = mg as f32;
                        let b = mb as f32;
                        let target = (r + g + b) / 3.0;
                        let new_r = clamp_gain(target / r);
                        let new_g = clamp_gain(target / g);
                        let new_b = clamp_gain(target / b);
                        wb.r = wb.r * (1.0 - AUTO_WB_ALPHA) + new_r * AUTO_WB_ALPHA;
                        wb.g = wb.g * (1.0 - AUTO_WB_ALPHA) + new_g * AUTO_WB_ALPHA;
                        wb.b = wb.b * (1.0 - AUTO_WB_ALPHA) + new_b * AUTO_WB_ALPHA;
                        pipe1.set_color_gains(wb);
                    }
                }

                if frames % 30 == 0 {
                    info!("frame {} idx {} wb=({},{},{})", frames, idx, wb.r, wb.g, wb.b);
                }
            }
            Either::First(Err(e)) => {
                info!("dcmipp error: {:?}", defmt::Debug2Format(&e));
            }
            Either::Second(()) => {
                auto_wb = !auto_wb;
                if !auto_wb {
                    wb = STATIC_WB;
                    pipe1.set_color_gains(wb);
                }
                info!("auto WB = {}", auto_wb);
            }
        }
    }
}

/// Clamp a single-channel gain to a sensible range so dark / saturated
/// frames can't drive the CCM to extremes. The hardware allows up to ~3.99
/// at the 256 = 1.0 scale; we cap below that.
fn clamp_gain(g: f32) -> f32 {
    if g < 0.25 {
        0.25
    } else if g > 3.5 {
        3.5
    } else {
        g
    }
}

fn rcc_config() -> Config {
    let mut config = Config::default();

    // PLL1 800 MHz VCO, identical to the lcd.rs configuration.
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

    // PLL4 → IC16 = 32 MHz LTDC pixel clock (lcd.rs setup).
    config.rcc.pll4 = Some(Pll::Bypass { source: Pllsel::Hsi });
    config.rcc.ic16 = Some(IcConfig {
        source: Icsel::Pll4,
        divider: Icint::Div2,
    });
    config.rcc.mux.ltdcsel = Ltdcsel::Ic16;

    // PLL1 / IC17 = 800/3 ≈ 266 MHz DCMIPP pixel pipeline kernel clock.
    // The BSP targets 300 MHz from a 1.2 GHz VCO; we get as close as the
    // 800 MHz VCO allows without exceeding the 333 MHz max (RM0486 §39).
    // 200 MHz (Div4) wasn't enough — the pipe overran on every frame.
    config.rcc.ic17 = Some(IcConfig {
        source: Icsel::Pll1,
        divider: Icint::Div3,
    });
    config.rcc.mux.dcmippsel = Dcmippsel::Ic17;

    // IC18 — CSI D-PHY config-clock divider. The BSP picks PLL1/60 (= 20 MHz
    // off a 1.2 GHz VCO; 13.3 MHz off our 800 MHz VCO). Even though there's
    // no csisel mux in the RCC metapac, the CSI test/control interface
    // depends on this branch being active.
    config.rcc.ic18 = Some(IcConfig {
        source: Icsel::Pll1,
        divider: Icint::Div60,
    });

    config
}

/// Enable run-mode clocks for every AXISRAM / AHBSRAM bank — embassy's N6
/// init only sets the LP gates by default, leaving AXISRAM3..6 unclocked
/// for AXI bus masters (LTDC, DCMIPP).
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

/// Promote DMA2D, LTDC and DCMIPP AXI master attributes to (secure,
/// CID=1, privileged) so they can read/write the RISAF default region.
/// See RM0486 §6.3.2 master / RISUP table:
///   - DMA2D   → master 8,  RISUP 101 → SECCFGR3 bit 5
///   - DCMIPP  → master 9,  RISUP 93  → SECCFGR2 bit 29
///   - LTDC_L1 → master 10, RISUP 103 → SECCFGR3 bit 7
///   - LTDC_L2 → master 11, RISUP 104 → SECCFGR3 bit 8
fn promote_axi_masters_to_secure() {
    pac::RIFSC.risc_seccfgr(2).modify(|w| {
        w.set_cfg(29, true); // RISUP 93 — DCMIPP
    });
    pac::RIFSC.risc_privcfgr(2).modify(|w| {
        w.set_cfg(29, true);
    });
    pac::RIFSC.risc_seccfgr(3).modify(|w| {
        w.set_cfg(5, true); // RISUP 101 — DMA2D
        w.set_cfg(7, true); // RISUP 103 — LTDC_L1
        w.set_cfg(8, true); // RISUP 104 — LTDC_L2
    });
    pac::RIFSC.risc_privcfgr(3).modify(|w| {
        w.set_cfg(5, true);
        w.set_cfg(7, true);
        w.set_cfg(8, true);
    });
    for master in [8usize, 9, 10, 11] {
        pac::RIFSC.rimc_attr(master).modify(|w| {
            w.set_mcid(1);
            w.set_msec(true);
            w.set_mpriv(true);
        });
    }
}
