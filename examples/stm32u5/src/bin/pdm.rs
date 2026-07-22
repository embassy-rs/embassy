//! PDM microphone capture on STM32U5 using MDF filter 0.
//!
//! Tested on NUCLEO-U545RE-Q / NUCLEO-U575ZI-Q class boards (see `Cargo.toml` chip feature).
//!
//! # Hardware
//!
//! Connect a PDM microphone (e.g. MP34DT05) to:
//! - **CLK** → `PE9` (MDF1 CCK0)
//! - **DATA** → `PE10` (MDF1 SDI0)
//! - **3V3** and **GND** as usual
//!
//! The MDF kernel clock must be configured in RCC before use — same as other U5 peripherals
//! with a dedicated clock mux (DAC, FDCAN, SAI, etc.). This example routes MDF1 from HCLK1.
//!
//! # Output
//!
//! Decimated PCM samples are read via DMA. The example prints an approximate RMS level over
//! defmt once per second. With no microphone attached the level stays near zero.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::mdf::{self, Config, Flt0, Mdf, samples_from_dma_words};
use embassy_stm32::rcc::{self, mux};
use embassy_stm32::{Config as StmConfig, bind_interrupts, dma, peripherals};
use embassy_time::Timer;
use micromath::F32Ext;
use {defmt_rtt as _, panic_probe as _};

/// Target PDM bit clock. Common for many MEMS PDM mics (1–3.2 MHz).
const PDM_CCK_HZ: u32 = 2_000_000;

const RAW_BUF_LEN: usize = 256;

bind_interrupts!(struct Irqs {
    MDF1_FLT0 => mdf::FilterInterruptHandler<peripherals::MDF1, Flt0>;
    GPDMA1_CHANNEL0 => dma::InterruptHandler<peripherals::GPDMA1_CH0>;
});

/// Build MDF config for filter 0 / SITF 0 at the given kernel clock.
fn mdf_config(kernel_hz: u32) -> Config {
    let mut config = Config::for_filter(0);
    // Processing clock = kernel / (procdiv + 1); CCK0 uses that clock with cckdiv = 1.
    config.clock.procdiv = kernel_hz.div_ceil(PDM_CCK_HZ).saturating_sub(1).min(255) as u8;
    config
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut stm_config = StmConfig::default();
    // Peripherals with kernel-clock muxes are configured in application RCC setup, not in the driver.
    stm_config.rcc.mux.mdf1sel = mux::Mdfsel::Hclk1;

    let p = embassy_stm32::init(stm_config);
    info!("PDM example started");

    let kernel_hz = rcc::frequency::<peripherals::MDF1>().0;
    let config = mdf_config(kernel_hz);

    let mut dma_buf = [0u32; RAW_BUF_LEN];
    let mut raw = [0u32; RAW_BUF_LEN];
    let mut samples = [0i32; RAW_BUF_LEN];

    let mut mdf = Mdf::new(
        p.MDF1,
        Irqs,
        config,
        p.PE9,  // CCK0
        p.PE10, // SDI0
        p.GPDMA1_CH0,
        &mut dma_buf,
    );

    info!(
        "MDF kernel clock: {} Hz, procdiv: {}, target PDM clock: {} Hz",
        kernel_hz, config.clock.procdiv, PDM_CCK_HZ
    );

    mdf.start();

    loop {
        match mdf.read_raw(&mut raw).await {
            Ok(()) => {
                samples_from_dma_words(&raw, &mut samples);
                let rms = rms(&samples);
                info!("PCM RMS (approx): {}", rms as u32);
            }
            Err(e) => error!("MDF read error: {:?}", e),
        }

        Timer::after_secs(1).await;
    }
}

fn rms(samples: &[i32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| (*s as f32) * (*s as f32)).sum();
    (sum_sq / samples.len() as f32).sqrt()
}
