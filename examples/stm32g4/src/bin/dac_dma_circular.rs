//! DAC ring-buffered DMA: a 10 kHz sine wave on PA4 (DAC1_CH1).
//!
//! A pre-computed 64-sample table is pre-filled into a DMA ring buffer, clocked by TIM8_TRGO
//! at ~639 kHz, giving an output frequency of ~10 kHz with no CPU involvement after setup.
//!
//! This demonstrates [`DacChannel::into_ring_buffered_12right`]: the channel is converted into
//! a `RingBufferedDacChannel`, the buffer is filled with `write_immediate`, and DMA is started.
//! For a dynamic waveform (e.g. audio streaming), call `write_exact` in a loop to supply new
//! samples while DMA is consuming the old ones.
//!
//! Wiring: observe PA4 with an oscilloscope; it produces a 10 kHz sine centred at ~1.65 V
//! (DAC mid-scale).

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::dac::DacChannel;
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::low_level::{MasterMode, RoundTo, Timer};
use embassy_stm32::triggers::TIM8_TRGO;
use embassy_stm32::{Config, bind_interrupts, dma, peripherals};
use embassy_time::Timer as EmbassyTimer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// 64-sample 12-bit right-aligned sine table, centred at 2048, amplitude 2000 counts.
// TIM8 at 170 MHz / 266 ≈ 639 kHz; 64 samples per period → output ≈ 10 kHz.
// u32 elements are required because the DAC DMA uses 32-bit peripheral transfers;
// only bits [11:0] of each word are used by the DHR12R register.
const N: usize = 64;
const SINE: [u32; N] = [
    2048, 2244, 2438, 2629, 2813, 2991, 3159, 3317, 3462, 3594, 3711, 3812, 3896, 3962, 4010, 4038, 4048, 4038, 4010,
    3962, 3896, 3812, 3711, 3594, 3462, 3317, 3159, 2991, 2813, 2629, 2438, 2244, 2048, 1852, 1658, 1467, 1283, 1105,
    937, 779, 634, 502, 385, 284, 200, 134, 86, 58, 48, 58, 86, 134, 200, 284, 385, 502, 634, 779, 937, 1105, 1283,
    1467, 1658, 1852,
];

static DMA_BUF: StaticCell<[u32; N]> = StaticCell::new();

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul85,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::Div2), // 170 MHz system clock
        });
        config.rcc.sys = Sysclk::Pll1R;
    }
    let p = embassy_stm32::init(config);

    // TIM8: PSC=0, ARR=265 → 170 MHz / 266 ≈ 639 kHz.
    // MasterMode::ComparePulse: TRGO fires on CC1 match every period.
    // CCR1=0: CC1 fires on every counter reset → one DMA advance per period.
    let tim8 = Timer::new(p.TIM8);
    tim8.set_period_clocks(266, RoundTo::Slower);
    tim8.set_compare_value(Channel::Ch1, 0u16);
    tim8.set_master_mode(MasterMode::ComparePulse);

    // DAC1 CH1: triggered by TIM8_TRGO, output on PA4.
    let mut dac_ch1 = DacChannel::new_triggered(p.DAC1, p.DMA1_CH5, TIM8_TRGO, Irqs, p.PA4);
    dac_ch1.set_triggering(true);

    // Pre-populate the DMA buffer with the sine table before handing it to the ring buffer.
    // No write_immediate needed — DMA reads the pre-filled data from the first trigger.
    let mut ring = dac_ch1.into_ring_buffered_12right(DMA_BUF.init(SINE));

    ring.start();

    tim8.start();

    info!("DAC ring-buffered DMA running, ~10 kHz sine on PA4");

    // Keep ring and tim8 alive; DMA loops indefinitely without CPU intervention.
    let _ring = ring;
    let _tim8 = tim8;

    loop {
        EmbassyTimer::after_secs(1).await;
        info!("running");
    }
}
