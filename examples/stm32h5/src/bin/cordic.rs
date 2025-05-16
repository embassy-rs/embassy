#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::cordic::{self, utils};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut dp = embassy_stm32::init(Default::default());

    let mut cordic = cordic::Cordic::new(
        dp.CORDIC.reborrow(),
        unwrap!(cordic::Config::new(
            cordic::Function::Sin,
            Default::default(),
            Default::default(),
        )),
    );

    // for output buf, the length is not that strict, larger than minimal required is ok.
    let mut output_f64 = [0f64; 19];
    let mut output_u32 = [0u32; 21];

    // tips:
    // CORDIC peripheral has some strict on input value, you can also use ".check_argX_fXX()" methods
    // to make sure your input values are compatible with current CORDIC setup.
    let arg1 = [-1.0, -0.5, 0.0, 0.5, 1.0]; // for trigonometric function, the ARG1 value [-pi, pi] should be map to [-1, 1]
    let arg2 = [0.5]; // and for Sin function, ARG2 should be in [0, 1]

    let mut input_buf = [0u32; 9];

    // convert input from floating point to fixed point
    input_buf[0] = unwrap!(utils::f64_to_q1_31(arg1[0]));
    input_buf[1] = unwrap!(utils::f64_to_q1_31(arg2[0]));

    // If input length is small, blocking mode can be used to minimize overhead.
    let cnt0 = unwrap!(cordic.blocking_calc_32bit(
        &input_buf[..2], // input length is strict, since driver use its length to detect calculation count
        &mut output_u32,
        false,
        false
    ));

    // convert result from fixed point into floating point
    for (&u32_val, f64_val) in output_u32[..cnt0].iter().zip(output_f64.iter_mut()) {
        *f64_val = utils::q1_31_to_f64(u32_val);
    }

    // convert input from floating point to fixed point
    //
    // first value from arg1 is used, so truncate to arg1[1..]
    for (&f64_val, u32_val) in arg1[1..].iter().zip(input_buf.iter_mut()) {
        *u32_val = unwrap!(utils::f64_to_q1_31(f64_val));
    }

    // If calculation is a little longer, async mode can make use of DMA, and let core do some other stuff.
    let cnt1 = unwrap!(
        cordic
            .async_calc_32bit(
                dp.GPDMA1_CH0.reborrow(),
                dp.GPDMA1_CH1.reborrow(),
                &input_buf[..arg1.len() - 1], // limit input buf to its actual length
                &mut output_u32,
                true,
                false
            )
            .await
    );

    // convert result from fixed point into floating point
    for (&u32_val, f64_val) in output_u32[..cnt1].iter().zip(output_f64[cnt0..cnt0 + cnt1].iter_mut()) {
        *f64_val = utils::q1_31_to_f64(u32_val);
    }

    println!("result: {}", output_f64[..cnt0 + cnt1]);
}
