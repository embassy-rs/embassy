#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m_rt::entry;
use defmt::*;
use embassy_stm32::dac::{DacCh1, DacChannel, Value};
use embassy_stm32::dma::NoDma;
use embassy_stm32::time::mhz;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    info!("Hello World, dude!");

    let mut config = Config::default();
    config.rcc.sys_ck = Some(mhz(400));
    config.rcc.hclk = Some(mhz(200));
    config.rcc.pll1.q_ck = Some(mhz(100));
    let p = embassy_stm32::init(config);

    let mut dac = DacCh1::new(p.DAC1, NoDma, p.PA4);
    unwrap!(dac.set_trigger_enable(false));

    loop {
        for v in 0..=255 {
            unwrap!(dac.set(Value::Bit8(to_sine_wave(v))));
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
