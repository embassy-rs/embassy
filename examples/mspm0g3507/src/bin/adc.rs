#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::adc::{self, Adc, AdcChannel, Conversion};
use embassy_mspm0::{Config, bind_interrupts, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

bind_interrupts!(struct Irqs {
    ADC0 => adc::InterruptHandler<peripherals::ADC0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");
    let mut p = embassy_mspm0::init(Config::default());
    let mut pa27 = p.PA27.reborrow_adc();

    // Configure adc with sequence 0 to 1
    let mut adc = Adc::new_async(p.ADC0, Irqs, Default::default());
    let mut readings = [0u16; 2];

    loop {
        let r = adc.blocking_read(&mut pa27, Conversion::default());
        info!("Raw adc PA27: {}", r);

        // With a voltage range of 0-3.3V and a resolution of 12 bits, the raw value can be
        // approximated to voltage (~0.0008 per step).
        let mut x = r as u32;
        x = x * 8;
        info!("Adc voltage PA27: {},{:#04}", x / 10_000, x % 10_000);
        // Read a sequence of channels
        adc.irq_read_sequence(
            [
                (p.PA22.reborrow_adc(), Conversion::default()),
                (p.PB20.reborrow_adc(), Conversion::default()),
            ]
            .into_iter(),
            &mut readings,
        )
        .await;
        info!("Raw adc sequence: {}", readings);

        Timer::after_millis(400).await;
    }
}
