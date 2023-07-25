#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::dac::{DacCh1, DacChannel, Value};
use embassy_stm32::dma::NoDma;
use embassy_stm32::pac::timer::vals::{Mms, Opm};
use embassy_stm32::peripherals::{TIM6, TIM7};
use embassy_stm32::rcc::AdcClockSource;
use embassy_stm32::rcc::low_level::RccPeripheral;
use embassy_stm32::time::{mhz, Hertz};
use embassy_stm32::timer::low_level::Basic16bitInstance;
use embassy_stm32::Config;
use embassy_time::{Delay, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

pub type Dac1Type =
    embassy_stm32::dac::DacCh1<'static, embassy_stm32::peripherals::DAC1, embassy_stm32::peripherals::DMA1_CH3>;

pub type Dac2Type =
    embassy_stm32::dac::DacCh2<'static, embassy_stm32::peripherals::DAC1, embassy_stm32::peripherals::DMA1_CH4>;

pub type Adc1Type =
    embassy_stm32::adc::Adc<'static, embassy_stm32::peripherals::ADC1>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(mhz(100));
    config.rcc.hclk = Some(mhz(100));
    config.rcc.pll1.q_ck = Some(mhz(100));
    config.rcc.adc_clock_source = AdcClockSource::PerCk;

    // Initialize the board and obtain a Peripherals instance
    let mut p: embassy_stm32::Peripherals = embassy_stm32::init(config);

    let mut dac = DacCh1::new(p.DAC1, NoDma, p.PA4);
    unwrap!(dac.set_trigger_enable(false));

    let mut adc = Adc::new(p.ADC1, &mut Delay);

    adc.set_sample_time(SampleTime::Cycles32_5);

    let mut vrefint_channel = adc.enable_vrefint();

    loop {
        for v in 0..=255 {
            unwrap!(dac.set(Value::Bit8(to_sine_wave(v))));
            Timer::after(Duration::from_millis(50)).await;
            let vrefint = adc.read_internal(&mut vrefint_channel);
            let measured = adc.read(&mut unsafe {embassy_stm32::Peripherals::steal()}.PA4);
            info!("value / measured: {} / {}", to_sine_wave(v), measured.saturating_sub(vrefint)/255);
            Timer::after(Duration::from_millis(50)).await;
        }
    }
}



use micromath::F32Ext;

fn to_sine_wave(v: u8) -> u8 {
    if v >= 128 {
        // top half
        let r = 3.14 * ((v - 128) as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    } else {
        // bottom half
        let r = 3.14 + 3.14 * (v as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    }
}
