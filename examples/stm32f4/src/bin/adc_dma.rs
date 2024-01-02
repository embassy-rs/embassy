#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcPin, Instance, RxDma, SamplerState};
use embassy_stm32::Peripheral;
use embassy_time::{Delay, Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

// async fn single_read<T: Instance + Peripheral<P = T>, RXDMA>(adc: &mut Adc<'_, T, RXDMA>, pin: &mut impl AdcPin<T>) {
//     info!("Single read");

//     for _ in 0..20 {
//         let v = adc.read(pin);
//         info!("--> {}", v);
//         Timer::after(Duration::from_millis(300)).await;
//     }
// }

async fn continous_read<T: Instance + Peripheral<P = T>, RXDMA: RxDma<T> + Peripheral<P = RXDMA>>(
    adc: &mut Adc<'_, T, RXDMA>,
    pin: &mut impl AdcPin<T>,
) {
    // info!("Continuous read");
    let mut cnt: u8 = 0;
    let mut data: [u16; 36] = [0u16; 36];
    adc.read_continuous(pin, &mut data, &mut |buf| {
        let state;
        if cnt < 7 as u8 {
            // info!("sampler: {} ", buf);
            //     info!("--> 32 {} ", buf[32]);
            //     info!("--> 64 {} ", buf[64]);
            //     info!("--> 96 {} ", buf[96]);
            //     info!("--> 128 {} ", buf[128]);
            //     info!("--> 255 {} ", buf[255]);
            state = SamplerState::Sampled
        } else {
            // info!("Stopped");
            state = SamplerState::Stopped
        }
        cnt = cnt + 1;
        // info!("Cnt: {}", cnt);
        state
    })
    .await;
    // info!("{} ", data);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");
    let p = embassy_stm32::init(Default::default());
    let mut adc = Adc::new(p.ADC1, p.DMA1_CH0, &mut Delay);
    let mut pin = p.PA0;

    // let mut vref = adc.enable_vref();
    // adc.read(&mut vref);
    // adc.set_sample_time(&mut pin, embassy_stm32::adc::SampleTime::Cycles112);

    // Switching alternatively between ADC continuous mode with DMA and ADC single reading
    loop {
        let tic = Instant::now();
        continous_read(&mut adc, &mut pin).await;
        let toc = Instant::elapsed(&tic);
        info!("Elapsed: {} ", toc.as_micros());
        Timer::after_millis(1000).await;
    }

    // single_read(&mut adc, &mut pin).await;

    // continous_read(&mut adc, &mut pin).await;
}
