// required-features: rng, cordic

// Test Cordic driver, with Q1.31 format, Sin function, at 24 iterations (aka PRECISION = 6), using DMA transfer

// Only test on STM32H563ZI, STM32U585AI and STM32U5a5JI.
// STM32G491RE is not tested, since it memory.x has less memory size than it actually has,
// and the test seems use much memory than memory.x suggest.
// see https://github.com/embassy-rs/stm32-data/issues/301#issuecomment-1925412561

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, cordic, peripherals, rng};
use num_traits::Float;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
   RNG => rng::InterruptHandler<peripherals::RNG>;
});

/* input value control, can be changed */

const ARG1_LENGTH: usize = 9;
const ARG2_LENGTH: usize = 4; // this might not be the exact length of ARG2, since ARG2 need to be inside [0, 1]

const INPUT_Q1_31_LENGHT: usize = ARG1_LENGTH + ARG2_LENGTH;
const INPUT_U8_LENGTH: usize = 4 * INPUT_Q1_31_LENGHT;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let dp = embassy_stm32::init(Default::default());

    //
    // use RNG generate random Q1.31 value
    //
    // we don't generate floating-point value, since not all binary value are valid floating-point value,
    // and Q1.31 only accept a fixed range of value.

    let mut rng = rng::Rng::new(dp.RNG, Irqs);

    let mut input_buf_u8 = [0u8; INPUT_U8_LENGTH];
    unwrap!(rng.async_fill_bytes(&mut input_buf_u8).await);

    // convert every [u8; 4] to a u32, for a Q1.31 value
    let input_q1_31 = unsafe { core::mem::transmute::<[u8; INPUT_U8_LENGTH], [u32; INPUT_Q1_31_LENGHT]>(input_buf_u8) };

    let mut input_f64_buf = [0f64; INPUT_Q1_31_LENGHT];

    let mut cordic_output_f64_buf = [0f64; ARG1_LENGTH * 2];

    // convert Q1.31 value back to f64, for software calculation verify
    for (val_u32, val_f64) in input_q1_31.iter().zip(input_f64_buf.iter_mut()) {
        *val_f64 = cordic::utils::q1_31_to_f64(*val_u32);
    }

    let mut arg2_f64_buf = [0f64; ARG2_LENGTH];
    let mut arg2_f64_len = 0;

    // check if ARG2 is in range [0, 1] (limited by CORDIC peripheral with Sin mode)
    for &arg2 in &input_f64_buf[ARG1_LENGTH..] {
        if arg2 >= 0.0 {
            arg2_f64_buf[arg2_f64_len] = arg2;
            arg2_f64_len += 1;
        }
    }

    // the actal value feed to CORDIC
    let arg1_f64_ls = &input_f64_buf[..ARG1_LENGTH];
    let arg2_f64_ls = &arg2_f64_buf[..arg2_f64_len];

    let mut cordic = cordic::Cordic::new(
        dp.CORDIC,
        unwrap!(cordic::Config::new(
            cordic::Function::Sin,
            Default::default(),
            Default::default(),
            false,
        )),
    );

    //#[cfg(feature = "stm32g491re")]
    //let (mut write_dma, mut read_dma) = (dp.DMA1_CH4, dp.DMA1_CH5);

    #[cfg(any(feature = "stm32h563zi", feature = "stm32u585ai", feature = "stm32u5a5zj"))]
    let (mut write_dma, mut read_dma) = (dp.GPDMA1_CH4, dp.GPDMA1_CH5);

    let cordic_start_point = embassy_time::Instant::now();

    let cnt = unwrap!(
        cordic
            .async_calc_32bit(
                &mut write_dma,
                &mut read_dma,
                arg1_f64_ls,
                Some(arg2_f64_ls),
                &mut cordic_output_f64_buf,
            )
            .await
    );

    let cordic_end_point = embassy_time::Instant::now();

    // since we get 2 output for 1 calculation, the output length should be ARG1_LENGTH * 2
    defmt::assert!(cnt == ARG1_LENGTH * 2);

    let mut software_output_f64_buf = [0f64; ARG1_LENGTH * 2];

    // for software calc, if there is no ARG2 value, insert a 1.0 as value (the reset value for ARG2 in CORDIC)
    let arg2_f64_ls = if arg2_f64_len == 0 { &[1.0] } else { arg2_f64_ls };

    let software_inputs = arg1_f64_ls
        .iter()
        .zip(
            arg2_f64_ls
                .iter()
                .chain(core::iter::repeat(&arg2_f64_ls[arg2_f64_ls.len() - 1])),
        )
        .zip(software_output_f64_buf.chunks_mut(2));

    let software_start_point = embassy_time::Instant::now();

    for ((arg1, arg2), res) in software_inputs {
        let (raw_res1, raw_res2) = (arg1 * core::f64::consts::PI).sin_cos();

        (res[0], res[1]) = (raw_res1 * arg2, raw_res2 * arg2);
    }

    let software_end_point = embassy_time::Instant::now();

    for (cordic_res, software_res) in cordic_output_f64_buf[..cnt]
        .chunks(2)
        .zip(software_output_f64_buf.chunks(2))
    {
        for (cord_res, soft_res) in cordic_res.iter().zip(software_res.iter()) {
            defmt::assert!((cord_res - soft_res).abs() <= 2.0.powi(-19));
        }
    }

    // This comparsion is just for fun. Since it not a equal compare:
    // software use 64-bit floating point, but CORDIC use 32-bit fixed point.
    trace!(
        "calculate count: {}, Cordic time: {} us, software time: {} us",
        ARG1_LENGTH,
        (cordic_end_point - cordic_start_point).as_micros(),
        (software_end_point - software_start_point).as_micros()
    );

    info!("Test OK");
    cortex_m::asm::bkpt();
}
