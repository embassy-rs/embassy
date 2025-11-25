#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::SharedData;
use embassy_stm32::adc::{Adc, CkModePclk, Clock, SampleTime};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init_primary(Default::default(), &SHARED_DATA);
    info!("Hello World!");

    let mut adc = Adc::new_with_clock(p.ADC1, Clock::Sync { div: CkModePclk::DIV1 });

    let mut pin = p.PB2;

    let mut vrefint = adc.enable_vrefint();
    let vrefint_sample = adc.blocking_read(&mut vrefint, SampleTime::CYCLES79_5);
    let convert_to_millivolts = |sample| {
        // From https://www.st.com/resource/en/datasheet/stm32g031g8.pdf
        // 6.3.3 Embedded internal reference voltage
        const VREFINT_MV: u32 = 1212; // mV

        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    };

    loop {
        let v = adc.blocking_read(&mut pin, SampleTime::CYCLES79_5);
        info!("--> {} - {} mV", v, convert_to_millivolts(v));
        Timer::after_millis(100).await;
    }
}
