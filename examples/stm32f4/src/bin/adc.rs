#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayUs;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, InterruptHandler, Resolution, Temperature, Vref};
use embassy_stm32::interrupt::{InterruptExt, Priority};
use embassy_stm32::peripherals::{PA0, PC0};
use embassy_stm32::{bind_interrupts, interrupt};
use embassy_time::{Delay, Instant, Timer};
use embedded_hal::adc::Channel;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC => InterruptHandler<embassy_stm32::peripherals::ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");
    let p = embassy_stm32::init(Default::default());
    interrupt::ADC.set_priority(Priority::P1);
    let mut delay = Delay;
    let mut adc = Adc::new(p.ADC1);
    let mut pin = p.PC1;

    let mut vrefint = adc.enable_vrefint();
    let mut temp = adc.enable_temperature();

    // Startup delay can be combined to the maximum of either
    delay.delay_us(Temperature::start_time_us().max(VrefInt::start_time_us()));

    let vrefint_sample = adc.read(&mut vrefint);

    // let convert_to_millivolts = |sample| {
    //     // From http://www.st.com/resource/en/datasheet/DM00071990.pdf
    //     // 6.3.24 Reference voltage
    //     const VREFINT_MV: u32 = 1500; // mV
    //     (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    // };

    // let convert_to_celcius = |sample| {
    //     // From http://www.st.com/resource/en/datasheet/DM00071990.pdf
    //     // 6.3.22 Temperature sensor characteristics
    //     const V25: i32 = 760; // mV
    //     const AVG_SLOPE: f32 = 2.5; // mV/C

    //     let sample_mv = convert_to_millivolts(sample) as i32;

    //     (sample_mv - V25) as f32 / AVG_SLOPE + 25.0
    // };
    // adc.stop_adc().await;
    // info!("VrefInt: {}", vrefint_sample);
    const MAX_ADC_SAMPLE: u16 = (1 << 12) - 1;
    // info!("VCCA: {} mV", convert_to_millivolts(MAX_ADC_SAMPLE));
    // adc.start_adc().await;

    // info!("Sample rate: {} Hz", adc.);
    //adc.set_channel_sample_sequence(&[0, 10]).await;
    loop {
        // Read pin
        let tic = Instant::now();
        let data = adc.read_sample_sequence(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).await;

        let toc = Instant::elapsed(&tic);
        info!("Data: {:?} | {} us", data, toc.as_micros());
        // info!("Channel 0: {} ({} mV)", data[0], convert_to_millivolts(data[0]));
        // info!("Channel 10: {} ({} mV)", data[1], convert_to_millivolts(data[1]));
        // info!("Took {} us", toc.as_micros());

        // let tic = Instant::now();
        // let v1 = adc.read(&mut pin2).await;
        // let toc = Instant::elapsed(&tic);
        // info!("PC0: {} | {} us", v1, toc.as_micros());

        // let vref = adc.read(&mut vrefint).await;
        // info!("VrefInt: {} ({} mV)", vref, convert_to_millivolts(vref));
        // let t = adc.read(&mut temp).await;
        // info!("bits: {}, T: {} C", t, convert_to_celcius(t));

        Timer::after_millis(100).await;
    }
}
