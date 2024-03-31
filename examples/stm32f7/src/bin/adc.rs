#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::Adc;
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1, &mut Delay);
    let mut pin = p.PA3;

    let mut vrefint = adc.enable_vrefint();
    let vrefint_sample = adc.read(&mut vrefint);
    let convert_to_millivolts = |sample| {
        // From http://www.st.com/resource/en/datasheet/DM00273119.pdf
        // 6.3.27 Reference voltage
        const VREFINT_MV: u32 = 1210; // mV

        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    };

    loop {
        let v = adc.read(&mut pin);
        info!("--> {} - {} mV", v, convert_to_millivolts(v));
        Timer::after_millis(100).await;
    }
}
