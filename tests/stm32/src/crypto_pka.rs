//! Shared setup of the `crypto_ec*` tests: the PKA needs the RNG running, and the STM32WBA52
//! needs its PLL to get through the suites in time.

use defmt_rtt as _;
use embassy_stm32::rng::Rng;
use embassy_stm32::{bind_interrupts, peripherals, rng};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

/// Run every suite, logging each result, and fail at the end if any failed.
macro_rules! suites {
    ($($(#[$meta:meta])* $name:ident),* $(,)?) => {{
        let mut ok = true;
        $(
            $(#[$meta])*
            match embassy_crypto_test::$name() {
                Ok(stats) => defmt::info!("{}: {:?}", stringify!($name), stats),
                Err(e) => {
                    defmt::error!("{}: {:?}", stringify!($name), e);
                    ok = false;
                }
            }
        )*
        defmt::assert!(ok, "some suites failed");
    }};
}
pub(crate) use suites;

/// Initializes the chip and starts the RNG, which the PKA initializes its RAM from.
pub fn init() -> Rng<'static, embassy_stm32::mode::Async> {
    #[allow(unused_mut)]
    let mut config = crate::common::config();

    // The suites run hundreds of point multiplications: at the 16 MHz the board otherwise
    // runs at, that does not fit the time limit.
    #[cfg(feature = "stm32wba52cg")]
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div1,
            mul: PllMul::Mul30,
            divr: Some(PllDiv::Div5),
            divq: None,
            divp: Some(PllDiv::Div30),
            frac: Some(0),
        });
        config.rcc.ahb5_pre = AHB5Prescaler::Div4;
        config.rcc.voltage_scale = VoltageScale::Range1;
        config.rcc.sys = Sysclk::Pll1R;
    }

    let p = crate::common::init_with_config(config);
    Rng::new(p.RNG, Irqs)
}
