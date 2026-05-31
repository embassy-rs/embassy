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
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, SampleTime, adc4};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::{Config, bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC4 => adc4::InterruptHandler<peripherals::ADC4>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::Hsi,
        prediv: PllPreDiv::Div1,
        mul: PllMul::Mul30,
        divr: Some(PllDiv::Div5),
        divq: None,
        divp: Some(PllDiv::Div30),
        frac: Some(0),
    });

    config.rcc.ahb_pre = AHBPrescaler::Div1;
    config.rcc.apb1_pre = APBPrescaler::Div1;
    config.rcc.apb2_pre = APBPrescaler::Div1;
    config.rcc.apb7_pre = APBPrescaler::Div1;
    config.rcc.ahb5_pre = AHB5Prescaler::Div4;
    config.rcc.voltage_scale = VoltageScale::Range1;
    config.rcc.sys = Sysclk::Pll1R;

    let p = embassy_stm32::init(config);

    info!("ADC4 analog watchdog example (PA0)");

    let mut adc = Adc::new_adc4(p.ADC4);
    let mut pin = p.PA0;
    adc.set_resolution_adc4(adc4::Resolution::Bits12);
    // 8× oversampling with matching right-shift → same 12-bit range, lower noise.
    // enable_watchdog will automatically scale AWD thresholds by >> 4 to match
    // the hardware's DR[15:4] comparison window.
    adc.set_averaging_adc4(adc4::Averaging::Samples8);

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
