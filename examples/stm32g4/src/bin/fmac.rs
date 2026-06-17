#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::fmac::{self, Q16};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut dp = embassy_stm32::init(Default::default());

    let one_third = Q16::from_f32(1.0 / 3.0);
    let one = Q16::from_f32(1.0);
    let zero = Q16::from_f32(0.0);

    let feedforward_weights = [one_third, one_third, one_third];

    // Create a second order FIR filter
    //
    // This will calculate
    // result = input_history[0] * latest_input +
    //     input_history[1] * older_input +
    //     input_history[2] * oldest_input
    let mut fmac = fmac::Fmac::fir(
        dp.FMAC.reborrow(),
        fmac::Config {
            output_mode: fmac::OutputMode::Saturating,
            read_method: fmac::AccessMethod::Poll,
            write_method: fmac::AccessMethod::Poll,
        },
        None,
        &feedforward_weights,
        fmac::Gain::X1,
    );

    let mut input_buffer_contents = [zero, zero];
    let mut calc = |fmac: &mut fmac::Fmac<'_, _>, x| {
        fmac.write(x).unwrap();
        input_buffer_contents.rotate_right(1);
        input_buffer_contents[0] = x;
        loop {
            if let Some(res) = fmac.read() {
                defmt::assert!(fmac.read().is_none());
                println!(
                    "in_buf: {:?}, x: {}, result: {}",
                    input_buffer_contents.map(|x| x.as_f32()),
                    x.as_f32(),
                    res.as_f32()
                );
                break;
            }
        }
    };

    // Fill up the empty slots
    fmac.write(zero).unwrap();
    fmac.write(zero).unwrap();
    Timer::after_millis(1).await;
    defmt::assert!(fmac.read().is_none()); // <-- Not enough data yet

    // Now we have enough data for a result
    calc(&mut fmac, zero);

    // Note that that the max value is just below 1.0
    for _ in 0..10 {
        calc(&mut fmac, one);
    }
}
