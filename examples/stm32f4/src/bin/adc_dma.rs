#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m::singleton;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{config, Adc, SampleTime};
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    let mut p = embassy_stm32::init(Default::default());
    let adc_data = singleton!(ADCDAT : [u16; 6] = [0u16; 6]).unwrap();
    let mut adc = Adc::new(p.ADC1, p.DMA2_CH0, adc_data, &mut Delay);

    let mut vref = adc.enable_vref();

    let cal: embassy_stm32::adc::Calibration = vref.calibrate(&mut adc).await;
    info!("Calibration: {:?}", cal.vdda_uv());
    info!(
        "Vref if 3000mV: {:?}",
        cal.cal_uv(4095, embassy_stm32::adc::Resolution::TwelveBit)
    );

    adc.set_sample_sequence(config::Sequence::One, &mut p.PA0, SampleTime::Cycles112)
        .await;

    adc.set_sample_sequence(config::Sequence::Two, &mut p.PA1, SampleTime::Cycles112)
        .await;

    adc.set_sample_sequence(config::Sequence::Three, &mut p.PC0, SampleTime::Cycles112)
        .await;

    adc.start_read_continuous();

    let mut buf = [0u16; 6];
    let mut i = 0;
    while i < 3 {
        adc.get_dma_buf(&mut buf);
        info!(
            "\n [{:?}mV, {:?}mV, {:?}mV] \n [{:?}mV, {:?}mV, {:?}mV] ",
            cal.cal_uv(buf[0], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
            cal.cal_uv(buf[1], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
            cal.cal_uv(buf[2], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
            cal.cal_uv(buf[3], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
            cal.cal_uv(buf[4], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
            cal.cal_uv(buf[5], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
        );
        Timer::after_millis(100).await;
        i += 1;
    }

    adc.start_read_continuous();
    adc.set_sample_sequence(config::Sequence::One, &mut p.PA0, SampleTime::Cycles112)
        .await;

    adc.set_sample_sequence(config::Sequence::Two, &mut p.PA1, SampleTime::Cycles112)
        .await;

    adc.set_sample_sequence(config::Sequence::Three, &mut p.PC0, SampleTime::Cycles112)
        .await;

    let mut buf1 = [0u16; 6];
    loop {
        adc.get_dma_buf(&mut buf1);
        info!(
            "\n [{:?}mV, {:?}mV, {:?}mV] \n [{:?}mV, {:?}mV, {:?}mV] ",
            cal.cal_uv(buf1[0], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
            cal.cal_uv(buf1[1], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
            cal.cal_uv(buf1[2], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
            cal.cal_uv(buf1[3], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
            cal.cal_uv(buf1[4], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
            cal.cal_uv(buf1[5], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
        );
        Timer::after_millis(100).await;
    }
    // continous_read(&mut adc, &mut pin).await;
}
