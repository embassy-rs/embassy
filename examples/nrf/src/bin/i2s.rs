// Example inspired by RTIC's I2S demo: https://github.com/nrf-rs/nrf-hal/blob/master/examples/i2s-controller-demo/src/main.rs

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

//use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::i2s;
use {defmt_rtt as _, panic_probe as _};

#[repr(align(4))]
pub struct Aligned<T: ?Sized>(T);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let config = i2s::Config::default();

    let mut i2s = i2s::I2s::new(p.I2S, p.P0_28, p.P0_29, p.P0_31, p.P0_11, p.P0_30, config);

    let mut signal_buf: Aligned<[i16; 32]> = Aligned([0i16; 32]);
    let len = signal_buf.0.len() / 2;
    for x in 0..len {
        signal_buf.0[2 * x] = triangle_wave(x as i32, len, 2048, 0, 1) as i16;
        signal_buf.0[2 * x + 1] = triangle_wave(x as i32, len, 2048, 0, 1) as i16;
    }

    let ptr = &signal_buf.0 as *const i16 as *const u8;
    let len = signal_buf.0.len() * core::mem::size_of::<i16>();

    i2s.start();
    i2s.set_tx_enabled(true);

    loop {
        match i2s.tx(ptr, len).await {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        };
    }
}

fn triangle_wave(x: i32, length: usize, amplitude: i32, phase: i32, periods: i32) -> i32 {
    let length = length as i32;
    amplitude
        - ((2 * periods * (x + phase + length / (4 * periods)) * amplitude / length) % (2 * amplitude) - amplitude)
            .abs()
        - amplitude / 2
}
