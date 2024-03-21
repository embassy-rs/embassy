#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::cordic;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut dp = embassy_stm32::init(Default::default());

    let mut cordic = cordic::Cordic::new(
        &mut dp.CORDIC,
        unwrap!(cordic::Config::new(
            cordic::Function::Sin,
            Default::default(),
            Default::default(),
            false,
        )),
    );

    let mut output = [0f64; 16];

    let arg1 = [1.0, 0.0, -1.0]; // for trigonometric function, the ARG1 value [-pi, pi] should be map to [-1, 1]
    let arg2 = [0.5, 1.0];

    let cnt = unwrap!(
        cordic
            .async_calc_32bit(&mut dp.GPDMA1_CH0, &mut dp.GPDMA1_CH1, &arg1, Some(&arg2), &mut output,)
            .await
    );

    println!("async calc 32bit: {}", output[..cnt]);
}
