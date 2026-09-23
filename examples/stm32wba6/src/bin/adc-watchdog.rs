//! ADC4 analog watchdog: wait for a pin voltage to leave a programmed window.
//!
//! Connect a voltage source or potentiometer to **PA0** (ADC4 channel 9). The example waits for the
//! sample to go above ~0.6 V, then waits for it to fall below ~0.2 V, and repeats.
//!
//! Hardware oversampling (8×) is enabled. When oversampling is active the AWD hardware compares
//! `ADC_DR[15:4]` against the threshold registers (RM: "most significant 12 bits of the 16-bit
//! oversampled result"). With an averaging shift that yields a 12-bit result in DR[11:0], only
//! the upper 8 bits (`DR[11:4]`) are compared, so thresholds must be right-shifted by 4.
//! `enable_watchdog` does this automatically — pass thresholds in the same 12-bit space as the
//! sample values and the driver scales them correctly.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{
    Adc, Config as AdcConfig, OversamplingRatio, Resolution, SampleTime, WatchdogChannels, WatchdogIndex,
};
use embassy_stm32::{Config, adc, bind_interrupts, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    ADC4 => adc::InterruptHandler<peripherals::ADC4>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    info!("ADC4 analog watchdog example (PA0)");

    let mut adc_config = AdcConfig::default();
    adc_config.resolution = Some(Resolution::Bits12);
    // 8× oversampling with matching right-shift → same 12-bit range, lower noise.
    // enable_watchdog will automatically scale AWD thresholds by >> 4 to match
    // the hardware's DR[15:4] comparison window.
    adc_config.averaging = Some(OversamplingRatio::X8);
    let mut adc = Adc::new(p.ADC4, Irqs, adc_config);
    let mut pin = p.PA0;

    let max = adc.resolution().max_count();

    loop {
        {
            // Wait for PA0 to exceed ~0.6 V (raw > 0x07F at 12-bit / 3.3 V).
            let mut wd = adc.enable_watchdog(WatchdogIndex::Awd1, WatchdogChannels::from_channel(&pin), 0, 0x07F);
            let raw = wd.monitor(&mut adc, &mut pin, SampleTime::Cycles125).await;
            let v = 3.3 * raw as f32 / max as f32;
            info!("Above high threshold, raw={} ~{} V", raw, v);
        }

        {
            // Wait for PA0 to drop below ~0.2 V (raw < 0x01F at 12-bit / 3.3 V).
            let mut wd = adc.enable_watchdog(WatchdogIndex::Awd1, WatchdogChannels::from_channel(&pin), 0x01F, 0x0FFF);
            let raw = wd.monitor(&mut adc, &mut pin, SampleTime::Cycles125).await;
            let v = 3.3 * raw as f32 / max as f32;
            info!("Below low threshold, raw={} ~{} V", raw, v);
        }
    }
}
