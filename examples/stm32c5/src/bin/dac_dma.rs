#![no_std]
#![no_main]

// Tested on STM32C562RE. Observe DAC1 channel 1 on PA4 relative to GND
// with an oscilloscope.
// TIM6 triggers 12,800 samples/s; the 128-sample DMA ring produces a 100 Hz
// triangle wave spanning approximately 0..VREF+. No external wiring is needed
// other than the scope probe.

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dac::{DacChannel, u12r};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{MasterMode, RoundTo, Timer};
use embassy_stm32::triggers::TIM6_TRGO;
use embassy_stm32::{Config, bind_interrupts, dma, peripherals};
use embassy_time::Timer as EmbassyTimer;
use panic_probe as _;
use static_cell::StaticCell;

const SAMPLE_COUNT: usize = 128;
static DMA_BUF: StaticCell<[u12r; SAMPLE_COUNT]> = StaticCell::new();

bind_interrupts!(struct Irqs {
    LPDMA1_CH0 => dma::InterruptHandler<peripherals::LPDMA1_CH0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    config.rcc.mux.adcdacsel = embassy_stm32::rcc::mux::Adcdacsel::Hclk1;
    config.rcc.adcdac_pre = embassy_stm32::pac::rcc::vals::Adcdacpre::Div2;

    let p = embassy_stm32::init(config);

    let timer = Timer::new(p.TIM6);
    timer.set_frequency(Hertz(12_800), RoundTo::Slower);
    timer.set_master_mode(MasterMode::Update);

    // Use the external buffered output. new_internal() does not drive PA4.
    let mut dac = DacChannel::new_triggered(p.DAC1, p.LPDMA1_CH0, TIM6_TRGO, p.PA4, Irqs);
    dac.set(u12r(0));

    let samples = DMA_BUF.init([u12r(0); SAMPLE_COUNT]);
    for (i, sample) in samples.iter_mut().enumerate() {
        let step = if i < SAMPLE_COUNT / 2 { i } else { SAMPLE_COUNT - 1 - i };
        *sample = u12r((step * 4095 / (SAMPLE_COUNT / 2 - 1)) as u16);
    }

    // Arm DMA before starting the timer so the first trigger can be serviced.
    let mut dac = dac.into_ring_buffered(samples);
    dac.start();
    timer.start();

    info!("DAC DMA running: PA4, triangle wave ~100 Hz, 0..VREF+");

    // Keep both drivers alive while DMA repeatedly reads the same sample table.
    loop {
        EmbassyTimer::after_secs(1).await;
    }
}
