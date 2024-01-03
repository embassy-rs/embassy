#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::f32::consts::E;

use cortex_m::singleton;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcPin, Instance, InterruptHandler, RxDma, SamplerState};
use embassy_stm32::gpio::low_level::Pin;
use embassy_stm32::{bind_interrupts, Peripheral};
use embassy_time::{Delay, Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");
    let p = embassy_stm32::init(Default::default());
    let adc_data = singleton!(ADCDAT : [u16; 60] = [0u16; 60]).unwrap();
    let mut adc = Adc::new(p.ADC1, p.DMA2_CH0, adc_data, &mut Delay);

    let mut pin = p.PA0._pin() as usize;
    let mut block = p.PA0.block();
    {
        let crlh = if pin < 8 { 0 } else { 1 };
        block
            .moder()
            .modify(|w| w.set_moder(pin, embassy_stm32::pac::gpio::vals::Moder::ANALOG));
    }

    let mut pin = p.PA1._pin() as usize;
    let mut block = p.PA1.block();
    {
        let crlh = if pin < 8 { 0 } else { 1 };
        block
            .moder()
            .modify(|w| w.set_moder(pin, embassy_stm32::pac::gpio::vals::Moder::ANALOG));
    }

    adc.start_read_continuous();

    let mut vref = adc.enable_vref();
    let cal: embassy_stm32::adc::Calibration = vref.calibrate(&mut adc).await;
    info!("Calibration: {:?}", cal.vdda_uv());

    let convert_to_millivolts = |sample: u16| {
        // From http://www.st.com/resource/en/datasheet/DM00071990.pdf
        // 6.3.24 Reference voltage
        const VREFINT_MV: u32 = 3300; // mV
        (u32::from(sample) * VREFINT_MV / u32::from(cal.vdda_uv() / 1000)) as u16
    };

    // Switching alternatively between ADC continuous mode with DMA and ADC single reading
    let mut buf = [0u16; 60];
    loop {
        let tic = Instant::now();
        adc.get_dma_buf(&mut buf);
        let toc = Instant::elapsed(&tic);
        info!("Elapsed: {} ", toc.as_micros());
        info!("{:?} ", buf);
        info!(
            "{:?}mV, {:?}mV ",
            convert_to_millivolts(buf[0]),
            convert_to_millivolts(buf[1])
        );
        Timer::after_millis(1000).await;
    }

    // single_read(&mut adc, &mut pin).await;

    // continous_read(&mut adc, &mut pin).await;
}
