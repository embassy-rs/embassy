//! Implements Flexcomm interface wrapper for easier usage across modules

pub mod spi;
pub mod uart;

use paste::paste;

use crate::clocks::{SysconPeripheral, enable_and_reset};
use crate::peripherals::{
    FLEXCOMM0, FLEXCOMM1, FLEXCOMM2, FLEXCOMM3, FLEXCOMM4, FLEXCOMM5, FLEXCOMM6, FLEXCOMM7, FLEXCOMM14, FLEXCOMM15,
};
use crate::{PeripheralType, pac};

/// clock selection option
#[derive(Copy, Clone, Debug)]
pub enum Clock {
    /// SFRO
    Sfro,

    /// FFRO
    Ffro,

    /// `AUDIO_PLL`
    AudioPll,

    /// MASTER
    Master,

    /// FCn_FRG with Main clock source
    FcnFrgMain,

    /// FCn_FRG with Pll clock source
    FcnFrgPll,

    /// FCn_FRG with Sfro clock source
    FcnFrgSfro,

    /// FCn_FRG with Ffro clock source
    FcnFrgFfro,

    /// disabled
    None,
}

/// do not allow implementation of trait outside this mod
mod sealed {
    /// trait does not get re-exported outside flexcomm mod, allowing us to safely expose only desired APIs
    pub trait Sealed {}
}

/// primary low-level flexcomm interface
pub(crate) trait FlexcommLowLevel: sealed::Sealed + PeripheralType + SysconPeripheral + 'static + Send {
    // fetch the flexcomm register block for direct manipulation
    fn reg() -> pac::flexcomm::Flexcomm;

    // set the clock select for this flexcomm instance and remove from reset
    fn enable(clk: Clock);
}

macro_rules! impl_flexcomm {
    ($($idx:expr),*) => {
	$(
	    paste!{
		impl sealed::Sealed for crate::peripherals::[<FLEXCOMM $idx>] {}

		impl FlexcommLowLevel for crate::peripherals::[<FLEXCOMM $idx>] {
		    fn reg() -> crate::pac::flexcomm::Flexcomm {
			crate::pac::[<FLEXCOMM $idx>]
		    }

		    fn enable(clk: Clock) {
			use crate::pac::clkctl1::vals::{FcfclkselSel, FrgclkselSel};

			let clkctl1 = crate::pac::CLKCTL1;

			clkctl1.flexcomm($idx).fcfclksel().write(|w| w.set_sel(match clk {
			    Clock::Sfro => FcfclkselSel::SFRO_CLK,
			    Clock::Ffro => FcfclkselSel::FFRO_CLK,
			    Clock::AudioPll => FcfclkselSel::AUDIO_PLL_CLK,
			    Clock::Master => FcfclkselSel::MASTER_CLK,
			    Clock::FcnFrgMain => FcfclkselSel::FCN_FRG_CLK,
			    Clock::FcnFrgPll => FcfclkselSel::FCN_FRG_CLK,
			    Clock::FcnFrgSfro => FcfclkselSel::FCN_FRG_CLK,
			    Clock::FcnFrgFfro => FcfclkselSel::FCN_FRG_CLK,
			    Clock::None => FcfclkselSel::NONE, // no clock? throw an error?
			}));

			clkctl1.flexcomm($idx).frgclksel().write(|w| w.set_sel(match clk {
			    Clock::FcnFrgMain => FrgclkselSel::MAIN_CLK,
			    Clock::FcnFrgPll => FrgclkselSel::FRG_PLL_CLK,
			    Clock::FcnFrgSfro => FrgclkselSel::SFRO_CLK,
			    Clock::FcnFrgFfro => FrgclkselSel::FFRO_CLK,
			    _ => FrgclkselSel::NONE,    // not using frg ...
			}));

			// todo: add support for frg div/mult
			clkctl1
			    .flexcomm($idx)
			    .frgctl()
			    .write(|w| w.set_mult(0));

			enable_and_reset::<[<FLEXCOMM $idx>]>();
		    }
		}
	    }
        )*
    }
}

impl_flexcomm!(0, 1, 2, 3, 4, 5, 6, 7);

// TODO: FLEXCOMM 14 is untested. Enable SPI support on FLEXCOMM14
// Add special case FLEXCOMM14
impl sealed::Sealed for crate::peripherals::FLEXCOMM14 {}

impl FlexcommLowLevel for crate::peripherals::FLEXCOMM14 {
    fn reg() -> crate::pac::flexcomm::Flexcomm {
        crate::pac::FLEXCOMM14
    }

    fn enable(clk: Clock) {
        use crate::pac::clkctl1::vals::{Fc14fclkselSel, Frg14clkselSel};

        let clkctl1 = crate::pac::CLKCTL1;

        clkctl1.fc14fclksel().write(|w| {
            w.set_sel(match clk {
                Clock::Sfro => Fc14fclkselSel::SFRO_CLK,
                Clock::Ffro => Fc14fclkselSel::FFRO_CLK,
                Clock::AudioPll => Fc14fclkselSel::AUDIO_PLL_CLK,
                Clock::Master => Fc14fclkselSel::MASTER_CLK,
                Clock::FcnFrgMain => Fc14fclkselSel::FCN_FRG_CLK,
                Clock::FcnFrgPll => Fc14fclkselSel::FCN_FRG_CLK,
                Clock::FcnFrgSfro => Fc14fclkselSel::FCN_FRG_CLK,
                Clock::FcnFrgFfro => Fc14fclkselSel::FCN_FRG_CLK,
                Clock::None => Fc14fclkselSel::NONE, // no clock? throw an error?
            })
        });

        clkctl1.frg14clksel().write(|w| {
            w.set_sel(match clk {
                Clock::FcnFrgMain => Frg14clkselSel::MAIN_CLK,
                Clock::FcnFrgPll => Frg14clkselSel::FRG_PLL_CLK,
                Clock::FcnFrgSfro => Frg14clkselSel::SFRO_CLK,
                Clock::FcnFrgFfro => Frg14clkselSel::FFRO_CLK,
                _ => Frg14clkselSel::NONE, // not using frg ...
            })
        });

        // todo: add support for frg div/mult
        clkctl1.frg14ctl().write(|w| w.set_mult(0));

        enable_and_reset::<FLEXCOMM14>();
    }
}

// Add special case FLEXCOMM15
impl sealed::Sealed for crate::peripherals::FLEXCOMM15 {}

impl FlexcommLowLevel for crate::peripherals::FLEXCOMM15 {
    fn reg() -> crate::pac::flexcomm::Flexcomm {
        crate::pac::FLEXCOMM15
    }

    fn enable(clk: Clock) {
        use crate::pac::clkctl1::vals::{Fc15fclkselSel, Frg15clkselSel};

        let clkctl1 = crate::pac::CLKCTL1;

        clkctl1.fc15fclksel().write(|w| {
            w.set_sel(match clk {
                Clock::Sfro => Fc15fclkselSel::SFRO_CLK,
                Clock::Ffro => Fc15fclkselSel::FFRO_CLK,
                Clock::AudioPll => Fc15fclkselSel::AUDIO_PLL_CLK,
                Clock::Master => Fc15fclkselSel::MASTER_CLK,
                Clock::FcnFrgMain => Fc15fclkselSel::FCN_FRG_CLK,
                Clock::FcnFrgPll => Fc15fclkselSel::FCN_FRG_CLK,
                Clock::FcnFrgSfro => Fc15fclkselSel::FCN_FRG_CLK,
                Clock::FcnFrgFfro => Fc15fclkselSel::FCN_FRG_CLK,
                Clock::None => Fc15fclkselSel::NONE, // no clock? throw an error?
            })
        });
        clkctl1.frg15clksel().write(|w| {
            w.set_sel(match clk {
                Clock::FcnFrgMain => Frg15clkselSel::MAIN_CLK,
                Clock::FcnFrgPll => Frg15clkselSel::FRG_PLL_CLK,
                Clock::FcnFrgSfro => Frg15clkselSel::SFRO_CLK,
                Clock::FcnFrgFfro => Frg15clkselSel::FFRO_CLK,
                _ => Frg15clkselSel::NONE, // not using frg ...
            })
        });
        // todo: add support for frg div/mult
        clkctl1.frg15ctl().write(|w| w.set_mult(0));

        enable_and_reset::<FLEXCOMM15>();
    }
}

macro_rules! into_mode {
    ($mode:ident, $($fc:ident),*) => {
        paste! {
            /// Sealed Mode trait
            trait [<SealedInto $mode:camel>]: FlexcommLowLevel {}

            /// Select mode of operation
            #[allow(private_bounds)]
            pub trait [<Into $mode:camel>]: [<SealedInto $mode:camel>] {
                /// Set mode of operation
                fn [<into_ $mode>]() {
                    Self::reg().pselid().write(|w| w.set_persel(pac::flexcomm::vals::Persel::[<$mode:upper>]));
                }
            }
        }

	$(
	    paste!{
		impl [<SealedInto $mode:camel>] for crate::peripherals::$fc {}
		impl [<Into $mode:camel>] for crate::peripherals::$fc {}
	    }
	)*
    }
}

into_mode!(
    usart, FLEXCOMM0, FLEXCOMM1, FLEXCOMM2, FLEXCOMM3, FLEXCOMM4, FLEXCOMM5, FLEXCOMM6, FLEXCOMM7
);
into_mode!(
    spi, FLEXCOMM0, FLEXCOMM1, FLEXCOMM2, FLEXCOMM3, FLEXCOMM4, FLEXCOMM5, FLEXCOMM6, FLEXCOMM7, FLEXCOMM14
);
into_mode!(
    i2c, FLEXCOMM0, FLEXCOMM1, FLEXCOMM2, FLEXCOMM3, FLEXCOMM4, FLEXCOMM5, FLEXCOMM6, FLEXCOMM7, FLEXCOMM15
);

into_mode!(
    i2s_transmit,
    FLEXCOMM0,
    FLEXCOMM1,
    FLEXCOMM2,
    FLEXCOMM3,
    FLEXCOMM4,
    FLEXCOMM5,
    FLEXCOMM6,
    FLEXCOMM7
);

into_mode!(
    i2s_receive,
    FLEXCOMM0,
    FLEXCOMM1,
    FLEXCOMM2,
    FLEXCOMM3,
    FLEXCOMM4,
    FLEXCOMM5,
    FLEXCOMM6,
    FLEXCOMM7
);
