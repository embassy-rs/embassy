#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// required-features: dac-adc-pin

#[path = "../common.rs"]
mod common;
use common::*;
use defmt::assert;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::dac::{DacCh1, DacChannel, Value};
use embassy_stm32::dma::NoDma;
use embassy_time::{Delay, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the board and obtain a Peripherals instance
    let p: embassy_stm32::Peripherals = embassy_stm32::init(config());

    let mut dac = DacCh1::new(p.DAC1, NoDma, p.PA4);
    unwrap!(dac.set_trigger_enable(false));

    let mut adc = Adc::new(p.ADC1, &mut Delay);

    adc.set_sample_time(SampleTime::Cycles32_5);

    let mut vrefint_channel = adc.enable_vrefint();

    for v in 0..=255 {
        // First set the DAC output value
        let dac_output_val = to_sine_wave(v);
        unwrap!(dac.set(Value::Bit8(dac_output_val)));

        // Now wait a little to obtain a stable value
        Timer::after(Duration::from_millis(20)).await;

        // Read reference voltage
        let vrefint = adc.read_internal(&mut vrefint_channel);
        // Need to steal the peripherals here because PA4 is obviously in use already
        let measured = adc.read(&mut unsafe { embassy_stm32::Peripherals::steal() }.PA4);
        // Calibrate and normalize the measurement to get close to the dac_output_val
        let measured_normalized = ((measured as i32 - vrefint as i32) / 255) as i16;

        info!("value / measured: {} / {}", dac_output_val, measured_normalized);

        // The deviations are quite enormous but that does not matter since this is only a quick test
        assert!((dac_output_val as i16 - measured_normalized).abs() < 15);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

use core::f32::consts::PI;

use micromath::F32Ext;

fn to_sine_wave(v: u8) -> u8 {
    if v >= 128 {
        // top half
        let r = PI * ((v - 128) as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    } else {
        // bottom half
        let r = PI + PI * (v as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    }
}
