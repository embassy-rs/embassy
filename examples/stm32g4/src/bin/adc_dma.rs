#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel as _, SampleTime};
use embassy_stm32::{Config, bind_interrupts, dma, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

static mut DMA_BUF: [u16; 8] = [0; 8];

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let read_buffer = unsafe { &mut DMA_BUF[..] };

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div4,
            mul: PllMul::Mul85,
            divp: None,
            divq: None,
            // Main system clock at 170 MHz
            divr: Some(PllRDiv::Div2),
        });
        config.rcc.mux.adc12sel = mux::Adcsel::Sys;
        config.rcc.sys = Sysclk::Pll1R;
    }
    let mut p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1, Default::default());

    let mut dma = p.DMA1_CH1;
    let mut vrefint = adc.enable_vrefint();
    let mut vrefint_channel = vrefint.degrade_adc();
    let mut pa0 = p.PA0.degrade_adc();

    for _ in 0..5 {
        adc.read(
            dma.reborrow(),
            Irqs,
            [
                (&mut vrefint_channel, SampleTime::Cycles2475),
                (&mut pa0, SampleTime::Cycles2475),
            ]
            .into_iter(),
            None,
            read_buffer,
        )
        .await;

        for buf in read_buffer.chunks(2) {
            let vrefint = buf[0];
            let measured = buf[1];
            info!("vrefint: {}", vrefint);
            info!("measured: {}", measured);
        }

        Timer::after_millis(500).await;
    }

    cortex_m::asm::bkpt();
}
