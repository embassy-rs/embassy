#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{self, Adc, AdcChannel, RxDma, SampleTime};
use embassy_stm32::peripherals::{ADC1, ADC2, GPDMA1_CH0, GPDMA1_CH1, PA0, PA1, PA2, PA3};
use embassy_stm32::{Config, Peri};
use embassy_time::{Duration, Instant, Ticker};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL25,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV4), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL25,
            divp: None,
            divq: None,
            divr: Some(PllDiv::DIV4), // 100mhz
        });
        config.rcc.sys = Sysclk::PLL1_P; // 200 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV1; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.adcdacsel = mux::Adcdacsel::PLL2_R;
    }
    let p = embassy_stm32::init(config);

    spawner.spawn(unwrap!(adc1_task(p.ADC1, p.GPDMA1_CH0, p.PA0, p.PA2)));
    spawner.spawn(unwrap!(adc2_task(p.ADC2, p.GPDMA1_CH1, p.PA1, p.PA3)));
}

#[embassy_executor::task]
async fn adc1_task(
    adc: Peri<'static, ADC1>,
    dma: Peri<'static, GPDMA1_CH0>,
    pin1: Peri<'static, PA0>,
    pin2: Peri<'static, PA2>,
) {
    adc_task(adc, dma, pin1, pin2).await;
}

#[embassy_executor::task]
async fn adc2_task(
    adc: Peri<'static, ADC2>,
    dma: Peri<'static, GPDMA1_CH1>,
    pin1: Peri<'static, PA1>,
    pin2: Peri<'static, PA3>,
) {
    adc_task(adc, dma, pin1, pin2).await;
}

async fn adc_task<'a, T: adc::Instance>(
    adc: Peri<'a, T>,
    mut dma: Peri<'a, impl RxDma<T>>,
    pin1: impl AdcChannel<T>,
    pin2: impl AdcChannel<T>,
) {
    let mut adc = Adc::new(adc);
    let mut pin1 = pin1.degrade_adc();
    let mut pin2 = pin2.degrade_adc();

    info!("adc init");

    let mut ticker = Ticker::every(Duration::from_millis(500));
    let mut tic = Instant::now();
    let mut buffer = [0u16; 512];
    loop {
        // This is not a true continuous read as there is downtime between each
        // call to `Adc::read` where the ADC is sitting idle.
        adc.read(
            dma.reborrow(),
            [(&mut pin1, SampleTime::CYCLES2_5), (&mut pin2, SampleTime::CYCLES2_5)].into_iter(),
            &mut buffer[0..2],
        )
        .await;
        let toc = Instant::now();
        info!("\n adc1: {} dt = {}", buffer[0..16], (toc - tic).as_micros());
        tic = toc;

        ticker.next().await;
    }
}
