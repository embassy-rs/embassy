#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::adc::{self, Adc, AdcChannel, AdcConfig, Resolution, Vrsel};
use embassy_mspm0::{bind_interrupts, peripherals, Config};
use embassy_time::Timer;
use {defmt_rtt as _, panic_halt as _};

bind_interrupts!(struct Irqs {
    ADC0 => adc::InterruptHandler<peripherals::ADC0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");
    let p = embassy_mspm0::init(Config::default());

    let adc_config = AdcConfig {
        resolution: Resolution::BIT12,
        vr_select: Vrsel::VddaVssa,
        sample_time: 50,
    };

    // Configure adc with sequence 0 to 1
    let mut adc = Adc::new_async(p.ADC0, adc_config, Irqs);
    let pin1 = p.PA22.degrade_adc();
    let pin2 = p.PB20.degrade_adc();
    let sequence = [(&pin1, Vrsel::VddaVssa), (&pin2, Vrsel::VddaVssa)];
    let mut readings = [0u16; 2];
    let mut pin3 = p.PA27;

    loop {
        let r = adc.read_channel(&mut pin3).await;
        info!("Raw adc PA27: {}", r);
        // With a voltage range of 0-3.3V and a resolution of 12 bits, the raw value can be
        // approximated to voltage (~0.0008 per step).
        let mut x = r as u32;
        x = x * 8;
        info!("Adc voltage PA27: {},{:#04}", x / 10_000, x % 10_000);
        // Read a sequence of channels
        adc.read_sequence(sequence.into_iter(), &mut readings).await;
        info!("Raw adc sequence: {}", readings);

        Timer::after_millis(400).await;
    }
}
