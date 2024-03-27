#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m::singleton;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{config, Adc, SampleTime};
use embassy_stm32::usb_otg::In;
use embassy_time::{Delay, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    const ADC_BUF_SIZE: usize = 132;
    let mut p = embassy_stm32::init(Default::default());
    info!("uwu");
    let adc_data = singleton!(ADCDAT : [u16; ADC_BUF_SIZE] = [0u16; ADC_BUF_SIZE]).unwrap();
    let mut adc = Adc::new(p.ADC1, &mut Delay);

    // let mut vref = adc.enable_vref();

    // let cal: embassy_stm32::adc::Calibration = vref.calibrate(&mut adc).await;
    // info!("Calibration: {:?}", cal.vdda_uv());
    // info!(
    //     "Vref if 3000mV: {:?}",
    //     cal.cal_uv(4095, embassy_stm32::adc::Resolution::TwelveBit)
    // );

    adc.set_sample_sequence(config::Sequence::One, &mut p.PA0, SampleTime::Cycles112)
        .await;

    adc.set_sample_sequence(config::Sequence::Two, &mut p.PA1, SampleTime::Cycles112)
        .await;

    adc.set_sample_sequence(config::Sequence::Three, &mut p.PC0, SampleTime::Cycles112)
        .await;

    let mut adc_dma = adc.start_read_continuous(p.DMA2_CH0, adc_data);
    Timer::after_secs(1).await;

    let mut tic = Instant::now();
    loop {
        let data = adc.get_dma_buf::<ADC_BUF_SIZE>(adc_data, &mut adc_dma);
        let toc = Instant::now();
        info!(
            "\n dt = {}",
            // data,
            (toc - tic).as_micros() //     // cal.cal_uv(data[0], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
                                    //     // cal.cal_uv(data[1], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
                                    //     // cal.cal_uv(data[2], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
                                    //     // cal.cal_uv(buf[3], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
                                    //     // cal.cal_uv(buf[4], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
                                    //     // cal.cal_uv(buf[5], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
        );
        tic = toc;
        Timer::after_millis(1).await;
    }
    // adc.stop_continuous_conversion().await;

    // adc.start_read_continuous();
    // adc.set_sample_sequence(config::Sequence::One, &mut p.PA0, SampleTime::Cycles112)
    //     .await;

    // adc.set_sample_sequence(config::Sequence::Two, &mut p.PA1, SampleTime::Cycles112)
    //     .await;

    // adc.set_sample_sequence(config::Sequence::Three, &mut p.PC0, SampleTime::Cycles112)
    //     .await;

    // let mut buf1 = [0u16; 6];
    // loop {
    //     adc.get_dma_buf(&mut buf1);
    //     info!(
    //         "\n [{:?}mV, {:?}mV, {:?}mV] \n [{:?}mV, {:?}mV, {:?}mV] ",
    //         cal.cal_uv(buf1[0], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
    //         cal.cal_uv(buf1[1], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
    //         cal.cal_uv(buf1[2], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
    //         cal.cal_uv(buf1[3], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
    //         cal.cal_uv(buf1[4], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
    //         cal.cal_uv(buf1[5], embassy_stm32::adc::Resolution::TwelveBit) / 1000,
    //     );
    //     Timer::after_millis(100).await;
    // }
    // continous_read(&mut adc, &mut pin).await;
}
