#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{self, Adc, Config, SampleTime, WatchdogChannels, WatchdogIndex};
use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals::ADC1;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    ADC1_COMP => adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("ADC watchdog example");

    let mut adc = Adc::new(p.ADC1, Irqs, Config::default());
    let mut pin = p.PC1;

    loop {
        // Wait for pin to go high
        {
            let mut wd = adc.enable_watchdog(WatchdogIndex::Awd1, WatchdogChannels::from_channel(&pin), 0, 0x07F);
            let v_high = wd.monitor(&mut adc, &mut pin, SampleTime::Cycles135).await;
            info!("ADC sample is high {}", v_high);
        }

        // Wait for pin to go low
        {
            let mut wd = adc.enable_watchdog(WatchdogIndex::Awd1, WatchdogChannels::from_channel(&pin), 0x01f, 0xFFF);
            let v_low = wd.monitor(&mut adc, &mut pin, SampleTime::Cycles135).await;
            info!("ADC sample is low {}", v_low);
        }
    }
}
