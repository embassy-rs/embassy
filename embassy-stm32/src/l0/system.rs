use crate::{hal, pac, Peripherals};

pub use hal::{
    prelude::*,
    rcc::{Clocks, Config},
};

/// safety: must only call once.
pub unsafe fn configure(config: Config) {
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.freeze(config);

    let clocks = rcc.clocks;

    unsafe { Peripherals::set_peripherals(clocks) };
}
