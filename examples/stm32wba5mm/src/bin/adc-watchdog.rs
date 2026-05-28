//! ADC4 analog watchdog: wait for a pin voltage to leave a programmed window.
//!
//! Connect a voltage source or potentiometer to **PA0** (ADC4 channel 9). The example waits for the
//! sample to go above ~0.6 V, then waits for it to fall below ~0.2 V, and repeats.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, SampleTime, adc4};
use embassy_stm32::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC4 => adc4::InterruptHandler<peripherals::ADC4>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("ADC4 analog watchdog example (PA0)");

    let mut adc = Adc::new_adc4(p.ADC4);
    let mut pin = p.PA0;
    adc.set_resolution_adc4(adc4::Resolution::Bits12);
    adc.set_averaging_adc4(adc4::Averaging::Disabled);

    let pin_ch = pin.degrade_adc().get_hw_channel();

    let max = adc4::resolution_to_max_count(adc4::Resolution::Bits12);

    loop {
        {
            // Wait for PA0 to exceed ~0.6 V (raw > 0x07F at 12-bit / 3.3 V).
            let mut wd = adc.enable_watchdog(
                adc4::WatchdogIndex::Awd1,
                adc4::WatchdogChannels::Single(pin_ch),
                0,
                0x07F,
            );
            let raw = wd.monitor(&mut adc, &mut pin, SampleTime::Cycles125).await;
            let v = 3.3 * raw as f32 / max as f32;
            info!("Above high threshold, raw={} ~{} V", raw, v);
        }

        {
            // Wait for PA0 to drop below ~0.2 V (raw < 0x01F at 12-bit / 3.3 V).
            let mut wd = adc.enable_watchdog(
                adc4::WatchdogIndex::Awd1,
                adc4::WatchdogChannels::Single(pin_ch),
                0x01F,
                0x0FFF,
            );
            let raw = wd.monitor(&mut adc, &mut pin, SampleTime::Cycles125).await;
            let v = 3.3 * raw as f32 / max as f32;
            info!("Below low threshold, raw={} ~{} V", raw, v);
        }
    }
}
