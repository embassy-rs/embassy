#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

use defmt::*;
use embassy_stm32::dac::{Channel, Dac, Value};
use embassy_stm32::pac;

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        pac::RCC.apb1enr1().modify(|w| {
            w.set_dac1en(true);
        });
    }

    let p = embassy_stm32::init(Default::default());

    let mut dac = Dac::new_1ch(p.DAC1, p.PA4);

    loop {
        for v in 0..=255 {
            unwrap!(dac.set(Channel::Ch1, Value::Bit8(to_sine_wave(v))));
            unwrap!(dac.trigger(Channel::Ch1));
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
