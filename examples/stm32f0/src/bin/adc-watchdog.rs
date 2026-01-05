#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{self, Adc, SampleTime, WatchdogChannels};
use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals::ADC1;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1_COMP => adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("ADC watchdog example");

    let mut adc = Adc::new(p.ADC1, Irqs);
    let pin = p.PC1;

    loop {
        // Wait for pin to go high
        adc.init_watchdog(WatchdogChannels::from_channel(&pin), 0, 0x07F);
        let v_high = adc.monitor_watchdog(SampleTime::CYCLES13_5).await;
        info!("ADC sample is high {}", v_high);

        // Wait for pin to go low
        adc.init_watchdog(WatchdogChannels::from_channel(&pin), 0x01f, 0xFFF);
        let v_low = adc.monitor_watchdog(SampleTime::CYCLES13_5).await;
        info!("ADC sample is low {}", v_low);
    }
}
