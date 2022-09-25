#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcPin, Instance, RxDma, SamplerState};
use embassy_stm32::Peripheral;
use embassy_time::{Delay, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

async fn single_read<T: Instance + Peripheral<P = T>, RXDMA>(adc: &mut Adc<'_, T, RXDMA>, pin: &mut impl AdcPin<T>) {
    info!("Single read");

    for _ in 0..20 {
        let v = adc.read(pin);
        info!("--> {} - {} mV", v, adc.to_millivolts(v));
        Timer::after(Duration::from_millis(300)).await;
    }
}

async fn continous_read<T: Instance + Peripheral<P = T>, RXDMA: RxDma<T> + Peripheral<P = RXDMA>>(
    adc: &mut Adc<'_, T, RXDMA>,
    pin: &mut impl AdcPin<T>,
) {
    info!("Continuous read");

    let mut data = [[0; 500]; 2];
    let mut cnt: u8 = 0;
    adc.read_continuous(pin, &mut data, &mut |buf| {
        let state;
        if cnt <= 20 {
            info!("--> 0 {} ", buf[0]);
            info!("--> 99 {} ", buf[99]);
            info!("--> 199 {} ", buf[199]);
            info!("--> 299 {} ", buf[299]);
            info!("--> 399 {} ", buf[399]);
            info!("--> 499 {} ", buf[499]);
            state = SamplerState::Sampled
        } else {
            state = SamplerState::Stopped
        }
        cnt = cnt + 1;
        state
    })
    .await;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");
    let p = embassy_stm32::init(Default::default());
    let mut adc = Adc::new(p.ADC1, p.DMA1_CH1, &mut Delay);
    let mut pin = p.PB1;

    let mut vref = adc.enable_vref(&mut Delay);
    adc.calibrate(&mut vref);
    adc.set_sample_time(embassy_stm32::adc::SampleTime::Cycles239_5);

    // Switching alternatively between ADC continuous mode with DMA and ADC single reading
    continous_read(&mut adc, &mut pin).await;

    single_read(&mut adc, &mut pin).await;

    continous_read(&mut adc, &mut pin).await;
}
