#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, SampleTime};
use embassy_stm32::Peripherals;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    spawner.must_spawn(adc_task(p));
}

#[embassy_executor::task]
async fn adc_task(mut p: Peripherals) {
    let mut adc = Adc::new(p.ADC1);

    loop {
        let mut measurements = [0u16; 3];
        let ok = adc
            .read_seq(
                p.DMA2_CH0.reborrow(),
                [
                    (&mut p.PA0.reborrow().degrade_adc(), SampleTime::CYCLES480),
                    (&mut p.PA1.reborrow().degrade_adc(), SampleTime::CYCLES480),
                    (&mut p.PA4.reborrow().degrade_adc(), SampleTime::CYCLES480),
                ]
                .into_iter(),
                &mut measurements,
            )
            .await
            .is_ok();

        if ok {
            info!("read data: {:?}", measurements);
        } else {
            warn!("failed to read data");
        }

        Timer::after_millis(500).await;
    }
}
