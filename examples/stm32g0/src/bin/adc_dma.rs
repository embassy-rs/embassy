#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel as _, Clock, Presc, SampleTime};
use embassy_stm32::{bind_interrupts, dma, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

static mut DMA_BUF: [u16; 2] = [0; 2];

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut read_buffer = unsafe { &mut DMA_BUF[..] };

    let p = embassy_stm32::init(Default::default());

    info!("Hello World!");

    let mut adc = Adc::new_with_clock(p.ADC1, Clock::Async { div: Presc::DIV1 });

    let mut dma = p.DMA1_CH1;
    let mut vrefint_channel = adc.enable_vrefint().degrade_adc();
    let mut pa0 = p.PA0.degrade_adc();

    loop {
        adc.read(
            dma.reborrow(),
            Irqs,
            [
                (&mut vrefint_channel, SampleTime::CYCLES160_5),
                (&mut pa0, SampleTime::CYCLES160_5),
            ]
            .into_iter(),
            &mut read_buffer,
        )
        .await;

        let vrefint = read_buffer[0];
        let measured = read_buffer[1];
        info!("vrefint: {}", vrefint);
        info!("measured: {}", measured);
        Timer::after_millis(500).await;
    }
}
