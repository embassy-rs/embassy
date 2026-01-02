// required-features: rng, cordic

// Test Cordic driver, with Q1.31 format, Sin function, at 24 iterations (aka PRECISION = 6), using DMA transfer

#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
use common::*;
use embassy_executor::Spawner;
use embassy_stm32::cordic::utils;
use embassy_stm32::{bind_interrupts, cordic, peripherals, rng};
use num_traits::Float;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
   RNG => rng::InterruptHandler<peripherals::RNG>;
});

/* input value control, can be changed */

const INPUT_U32_COUNT: usize = 9;
const INPUT_U8_COUNT: usize = 4 * INPUT_U32_COUNT;

// Assume first calculation needs 2 arguments, the reset needs 1 argument.
// And all calculation generate 2 results.
const OUTPUT_LENGTH: usize = (INPUT_U32_COUNT - 1) * 2;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let dp = init();

    //
    // use RNG generate random Q1.31 value
    //
    // we don't generate floating-point value, since not all binary value are valid floating-point value,
    // and Q1.31 only accept a fixed range of value.

    let mut rng = rng::Rng::new(dp.RNG, Irqs);

    let mut input_buf_u8 = [0u8; INPUT_U8_COUNT];
    defmt::unwrap!(rng.async_fill_bytes(&mut input_buf_u8).await);

    // convert every [u8; 4] to a u32, for a Q1.31 value
    let mut input_q1_31 = unsafe { core::mem::transmute::<[u8; INPUT_U8_COUNT], [u32; INPUT_U32_COUNT]>(input_buf_u8) };

    // ARG2 for Sin function should be inside [0, 1], set MSB to 0 of a Q1.31 value, will make sure it's no less than 0.
    input_q1_31[1] &= !(1u32 << 31);

    //
    // CORDIC calculation
    //

    let mut output_q1_31 = [0u32; OUTPUT_LENGTH];

    // setup Cordic driver
    let mut cordic = cordic::Cordic::new(
        dp.CORDIC,
        defmt::unwrap!(cordic::Config::new(
            cordic::Function::Sin,
            Default::default(),
            Default::default(),
        )),
    );

    #[cfg(feature = "stm32g491re")]
    let (mut write_dma, mut read_dma) = (dp.DMA1_CH4, dp.DMA1_CH5);

    #[cfg(any(
        feature = "stm32h563zi",
        feature = "stm32u585ai",
        feature = "stm32u5a5zj",
        feature = "stm32h7s3l8"
    ))]
    let (mut write_dma, mut read_dma) = (dp.GPDMA1_CH0, dp.GPDMA1_CH1);

    // calculate first result using blocking mode
    let cnt0 = defmt::unwrap!(cordic.blocking_calc_32bit(&input_q1_31[..2], &mut output_q1_31, false, false));

    // calculate rest results using async mode
    let cnt1 = defmt::unwrap!(
        cordic
            .async_calc_32bit(
                write_dma.reborrow(),
                read_dma.reborrow(),
                &input_q1_31[2..],
                &mut output_q1_31[cnt0..],
                true,
                false,
            )
            .await
    );

    // all output value length should be the same as our output buffer size
    defmt::assert_eq!(cnt0 + cnt1, output_q1_31.len());

    let mut cordic_result_f64 = [0.0f64; OUTPUT_LENGTH];

    for (f64_val, u32_val) in cordic_result_f64.iter_mut().zip(output_q1_31) {
        *f64_val = utils::q1_31_to_f64(u32_val);
    }

    //
    // software calculation
    //

    let mut software_result_f64 = [0.0f64; OUTPUT_LENGTH];

    let arg2 = utils::q1_31_to_f64(input_q1_31[1]);

    for (&arg1, res) in input_q1_31
        .iter()
        .enumerate()
        .filter_map(|(idx, val)| if idx != 1 { Some(val) } else { None })
        .zip(software_result_f64.chunks_mut(2))
    {
        let arg1 = utils::q1_31_to_f64(arg1);

        let (raw_res1, raw_res2) = (arg1 * core::f64::consts::PI).sin_cos();
        (res[0], res[1]) = (raw_res1 * arg2, raw_res2 * arg2);
    }

    //
    // check result are the same
    //

    for (cordic_res, software_res) in cordic_result_f64[..cnt0 + cnt1]
        .chunks(2)
        .zip(software_result_f64.chunks(2))
    {
        for (cord_res, soft_res) in cordic_res.iter().zip(software_res.iter()) {
            // 2.0.powi(-19) is the max residual error for Sin function, in q1.31 format, with 24 iterations (aka PRECISION = 6)
            defmt::assert!((cord_res - soft_res).abs() <= 2.0.powi(-19));
        }
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
