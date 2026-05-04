#![no_std]
#![no_main]

use defmt::*;
use dsp_fixedpoint::Q32;
use embassy_executor::Spawner;
use embassy_stm32::cordic::{self};
use embassy_stm32::{bind_interrupts, dma, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL0 => dma::InterruptHandler<peripherals::GPDMA1_CH0>;
    GPDMA1_CHANNEL1 => dma::InterruptHandler<peripherals::GPDMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut dp = embassy_stm32::init(Default::default());

    // Create CORDIC with 2-arg, 2-result config for the initial call (sets ARG2)
    let mut cordic = cordic::Cordic::new(
        dp.CORDIC.reborrow(),
        &unwrap!(cordic::Config::new(
            cordic::Function::Sin,
            Default::default(),
            Default::default(),
        )),
    );

    let mut cordic_32 = cordic.q1_31(cordic::AccessCount::Two, cordic::AccessCount::Two);

    // for output buf, the length is not that strict, larger than minimal required is ok.
    let mut output_f64 = [0f64; 19];
    let mut output_u32 = [Q32::<31>::new(0); 21];

    // tips:
    // CORDIC peripheral has some strict on input value, you can also use ".check_argX_fXX()" methods
    // to make sure your input values are compatible with current CORDIC setup.
    let arg1 = [-1.0, -0.5, 0.0, 0.5, 1.0]; // for trigonometric function, the ARG1 value [-pi, pi] should be map to [-1, 1]
    let arg2 = [0.5]; // and for Sin function, ARG2 should be in [0, 1]

    let mut input_buf = [Q32::<31>::new(0); 9];

    // convert input from floating point to fixed point
    input_buf[0] = Q32::<31>::from_f64(arg1[0]);
    input_buf[1] = Q32::<31>::from_f64(arg2[0]);

    // If input length is small, blocking mode can be used to minimize overhead.
    let cnt0 = unwrap!(cordic_32.blocking_calc(
        &input_buf[..2], // input length is strict, since driver use its length to detect calculation count
        &mut output_u32,
    ));

    // convert result from fixed point into floating point
    for (&u32_val, f64_val) in output_u32[..cnt0].iter().zip(output_f64.iter_mut()) {
        *f64_val = u32_val.as_f64()
    }

    // convert input from floating point to fixed point
    //
    // first value from arg1 is used, so truncate to arg1[1..]
    for (&f64_val, u32_val) in arg1[1..].iter().zip(input_buf.iter_mut()) {
        *u32_val = Q32::<31>::from_f64(f64_val);
    }

    // Switch to 1-arg mode (reuse ARG2 set above) without resetting ARG2
    cordic_32.set_access_counts(cordic::AccessCount::One, cordic::AccessCount::Two);

    // If calculation is a little longer, async mode can make use of DMA, and let core do some other stuff.
    let cnt1 = unwrap!(
        cordic_32
            .async_calc(
                dp.GPDMA1_CH0.reborrow(),
                dp.GPDMA1_CH1.reborrow(),
                Irqs,
                &input_buf[..arg1.len() - 1], // limit input buf to its actual length
                &mut output_u32,
            )
            .await
    );

    // convert result from fixed point into floating point
    for (&u32_val, f64_val) in output_u32[..cnt1].iter().zip(output_f64[cnt0..cnt0 + cnt1].iter_mut()) {
        *f64_val = u32_val.as_f64();
    }

    println!("result: {}", output_f64[..cnt0 + cnt1]);
}
