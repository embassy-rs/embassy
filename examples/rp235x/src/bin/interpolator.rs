//! Example showing basic usage of the RP235x interpolator peripheral.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::interpolator::{Instance, Interpolator, LaneCtrl};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut interp0 = Interpolator::new(p.INTERP0);
    let mut interp1 = Interpolator::new(p.INTERP1);

    times_table(&mut interp0);
    moving_mask(&mut interp1);
}

fn times_table(interp: &mut Interpolator<'_, impl Instance>) {
    info!("9 times table");

    let mut lane0 = interp.lane0();
    lane0.set_ctrl(LaneCtrl::default());

    // Set the base value
    lane0.set_base(9);

    // Set the accumulator value
    lane0.set_accum(0);

    // Add values to the accumulators
    for i in 1..=10 {
        info!("9 * {} = {}", i, lane0.pop());
    }
}

fn moving_mask(interp: &mut Interpolator<'_, impl Instance>) {
    let mut config = LaneCtrl::default();

    interp.set_base(0);
    let mut lane0 = interp.lane0();
    lane0.set_accum(0x1234abcd);
    info!("Masking:");
    info!("ACCUM0 = {:08x}\n", lane0.get_accum());
    for i in 0..8 {
        // LSB, then MSB. These are inclusive, so 0,31 means "the entire 32 bit register"
        config.mask_lsb = i * 4;
        config.mask_msb = i * 4 + 3;
        lane0.set_ctrl(config);
        // Reading from ACCUMx_ADD returns the raw lane shift and mask value, without BASEx added
        info!("Nibble {}: {:08x}", i, lane0.peek());
    }

    info!("Masking with sign extension:");
    config.signed = true;
    for i in 0..8 {
        config.mask_lsb = i * 4;
        config.mask_msb = i * 4 + 3;
        lane0.set_ctrl(config);
        info!("Nibble {}: {:08x}", i, lane0.peek());
    }
}
