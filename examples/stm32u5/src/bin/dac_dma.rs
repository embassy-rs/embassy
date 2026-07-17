//! DAC Example tested on the stm32u545re board, DAC pin connected to an external LED

#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::dac::DacChannel;
use embassy_stm32::peripherals::GPDMA1_CH0;
use embassy_stm32::rcc::{LsConfig, mux};
use embassy_stm32::timer::low_level::RoundTo::Faster;
use embassy_stm32::triggers::TIM6_TRGO;
use embassy_stm32::{Config, bind_interrupts, dma, pac};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL0 => dma::InterruptHandler<GPDMA1_CH0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Device has started");
    let mut config = Config::default();

    // turns on internal LSI(needed for DAC sync)
    config.rcc.ls = LsConfig::default_lsi();
    // changing the mux to point to our clock(LSI)
    config.rcc.mux.dac1sel = mux::Dacsel::Lsi;
    let p = embassy_stm32::init(config);

    info!("Board connected!");

    // Any timer that supports the hardware TRGO can be selected here
    let mut dac = DacChannel::new_triggered(p.DAC1, p.GPDMA1_CH0, TIM6_TRGO, Irqs, p.PA4);

    embassy_stm32::rcc::enable_and_reset::<embassy_stm32::peripherals::TIM6>();

    // The timer is needed such that the DMA knows when to "fire"
    let timer = embassy_stm32::timer::low_level::Timer::new(p.TIM6);
    timer.set_frequency(embassy_stm32::time::Hertz(10000), Faster);

    // DAC listens to MMS
    pac::TIM6
        .cr2()
        .modify(|w| w.set_mms(embassy_stm32::pac::timer::vals::Mms::Update));

    timer.start();

    let mut i = 0;

    loop {
        dac.write(&[i]).await;
        i = i.wrapping_add(1);
    }
}
