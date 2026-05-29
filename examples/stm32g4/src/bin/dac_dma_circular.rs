//! DAC circular DMA: two complementary sine waves on PA4 (DAC1_CH1) and PA5 (DAC1_CH2).
//!
//! A pre-computed 64-sample table is played back continuously via DMA, clocked byTIM8_TRGO
//! at ~639 kHz, giving an output frequency of ~10 kHz with no CPU involvement after setup.
//!
//! This demonstrates `DacChannel::start_circular_dma`: a non-blocking alternative to the
//! async `write(..., circular: true)` that is usable in synchronous contexts. The DMA runs
//! indefinitely and is never stopped by a `Drop` guard.
//!
//! Wiring: observe PA4 and PA5 with an oscilloscope; both channels produce a 10 kHz sine
//! centred at ~1.65 V (DAC mid-scale), with PA5 phase-inverted relative to PA4.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::dac::{Dac, ValueArray};
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::low_level::{MasterMode, RoundTo, Timer};
use embassy_stm32::triggers::TIM8_TRGO;
use embassy_stm32::{Config, bind_interrupts, dma, peripherals};
use embassy_time::Timer as EmbassyTimer;
use {defmt_rtt as _, panic_probe as _};

// 64-sample 12-bit right-aligned sine tables, centred at 2048, amplitude 2000 counts.
// CH1 = positive sine, CH2 = inverted sine.
// TIM8 at 170 MHz / 266 ≈ 639 kHz; 64 samples per period → output ≈ 10 kHz.
const N: usize = 64;
static SINE_CH1: [u16; N] = [
    2048, 2244, 2438, 2629, 2813, 2991, 3159, 3317, 3462, 3594, 3711, 3812, 3896, 3962, 4010,
    4038, 4048, 4038, 4010, 3962, 3896, 3812, 3711, 3594, 3462, 3317, 3159, 2991, 2813, 2629,
    2438, 2244, 2048, 1852, 1658, 1467, 1283, 1105, 937, 779, 634, 502, 385, 284, 200, 134, 86,
    58, 48, 58, 86, 134, 200, 284, 385, 502, 634, 779, 937, 1105, 1283, 1467, 1658, 1852,
];
static SINE_CH2: [u16; N] = [
    2048, 1852, 1658, 1467, 1283, 1105, 937, 779, 634, 502, 385, 284, 200, 134, 86, 58, 48, 58,
    86, 134, 200, 284, 385, 502, 634, 779, 937, 1105, 1283, 1467, 1658, 1852, 2048, 2244, 2438,
    2629, 2813, 2991, 3159, 3317, 3462, 3594, 3711, 3812, 3896, 3962, 4010, 4038, 4048, 4038,
    4010, 3962, 3896, 3812, 3711, 3594, 3462, 3317, 3159, 2991, 2813, 2629, 2438, 2244,
];

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
    DMA1_CHANNEL6 => dma::InterruptHandler<peripherals::DMA1_CH6>;
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

    // DAC1: both channels triggered by TIM8_TRGO, output on PA4 and PA5.
    let (mut dac_ch1, mut dac_ch2) = Dac::new_triggered(
        p.DAC1,
        p.DMA1_CH5,
        p.DMA1_CH6,
        TIM8_TRGO,
        TIM8_TRGO,
        Irqs,
        p.PA4,
        p.PA5,
    )
    .split();

    dac_ch1.set_triggering(true);
    dac_ch2.set_triggering(true);
    dac_ch1.start_circular_dma(ValueArray::Bit12Right(&SINE_CH1));
    dac_ch2.start_circular_dma(ValueArray::Bit12Right(&SINE_CH2));

    tim8.start();

    info!("DAC circular DMA running, ~10 kHz complementary sine on PA4/PA5");

    // Keep dac_ch1, dac_ch2, and tim8 alive so Drop does not stop the peripherals.
    let _dac_ch1 = dac_ch1;
    let _dac_ch2 = dac_ch2;
    let _tim8 = tim8;

    loop {
        EmbassyTimer::after_secs(1).await;
        info!("running");
    }
}
