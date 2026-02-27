//! Input Mux driver.

use paste::paste;

use crate::clocks::periph_helpers::NoConfig;
use crate::clocks::{disable, enable, enable_and_reset};
use crate::pac;
use crate::peripherals::INPUTMUX0;

pub(crate) trait SealedValidInputMuxConfig {
    fn mux() {}
}

/// Marker trait for valid Input mux configurations.
#[allow(private_bounds)]
pub trait ValidInputMuxConfig: SealedValidInputMuxConfig {}

pub(crate) fn init() {
    unsafe {
        // Enable the peripheral an deassert reset.
        let _ = enable_and_reset::<INPUTMUX0>(&NoConfig);

        // INPUTMUX only needs to have its clocks ungated when
        // accessing any of the memory mapped registers. Therefore,
        // it's safe to disable clocks here.
        disable::<INPUTMUX0>();
    }
}

macro_rules! impl_input_mux {
    (ctimer, $value:literal, $pin:ident) => {
        impl_input_mux!(ctimer, 0, $value, $pin);
        impl_input_mux!(ctimer, 1, $value, $pin);
        impl_input_mux!(ctimer, 2, $value, $pin);
        impl_input_mux!(ctimer, 3, $value, $pin);
        impl_input_mux!(ctimer, 4, $value, $pin);
    };

    (ctimer, $inst:literal, $value:literal, $pin:ident) => {
        paste! {
            impl_input_mux!([<CTIMER $inst>], [<CTIMER $inst _CH0>], $pin, [<ctimer $inst cap>], 0, $value);
            impl_input_mux!([<CTIMER $inst>], [<CTIMER $inst _CH1>], $pin, [<ctimer $inst cap>], 1, $value);
            impl_input_mux!([<CTIMER $inst>], [<CTIMER $inst _CH2>], $pin, [<ctimer $inst cap>], 2, $value);
            impl_input_mux!([<CTIMER $inst>], [<CTIMER $inst _CH3>], $pin, [<ctimer $inst cap>], 3, $value);
        }
    };

    ($peri:ident, $ch:ident, $pin:ident, $reg:ident, $n:literal, $value:literal) => {
        paste! {
            impl SealedValidInputMuxConfig
                for (
                    crate::peripherals::$peri,
                    crate::peripherals::$ch,
                    crate::peripherals::$pin,
                )
            {
                #[inline(always)]
                fn mux() {
                    let _ = unsafe { enable::<INPUTMUX0>(&NoConfig) };
                    pac::INPUTMUX0.[<$reg>]($n).write(|w| w.set_inp([<$value>].into()));
                    unsafe { disable::<INPUTMUX0>() };

                }
            }

            impl ValidInputMuxConfig
                for (
                    crate::peripherals::$peri,
                    crate::peripherals::$ch,
                    crate::peripherals::$pin,
                )
            {
            }
        }
    };
}

#[cfg(feature = "swd-as-gpio")]
impl_input_mux!(ctimer, 2, P0_1);
#[cfg(feature = "jtag-extras-as-gpio")]
impl_input_mux!(ctimer, 3, P0_6);

impl_input_mux!(ctimer, 1, P0_20);
impl_input_mux!(ctimer, 2, P0_21);
impl_input_mux!(ctimer, 3, P0_22);
impl_input_mux!(ctimer, 4, P0_23);

impl_input_mux!(ctimer, 5, P1_0);
impl_input_mux!(ctimer, 6, P1_1);
impl_input_mux!(ctimer, 1, P1_2);
impl_input_mux!(ctimer, 2, P1_3);
impl_input_mux!(ctimer, 7, P1_6);
impl_input_mux!(ctimer, 8, P1_7);
impl_input_mux!(ctimer, 9, P1_8);
impl_input_mux!(ctimer, 10, P1_9);
impl_input_mux!(ctimer, 11, P1_14);
impl_input_mux!(ctimer, 12, P1_15);

#[cfg(feature = "sosc-as-gpio")]
impl_input_mux!(ctimer, 17, P1_30);
#[cfg(feature = "sosc-as-gpio")]
impl_input_mux!(ctimer, 18, P1_31);

impl_input_mux!(ctimer, 17, P2_0);
impl_input_mux!(ctimer, 18, P2_1);
impl_input_mux!(ctimer, 13, P2_2);
impl_input_mux!(ctimer, 14, P2_3);
impl_input_mux!(ctimer, 15, P2_4);
impl_input_mux!(ctimer, 16, P2_5);
impl_input_mux!(ctimer, 19, P2_6);
impl_input_mux!(ctimer, 20, P2_7);

impl_input_mux!(ctimer, 17, P3_0);
impl_input_mux!(ctimer, 18, P3_1);
impl_input_mux!(ctimer, 5, P3_8);
impl_input_mux!(ctimer, 6, P3_9);
impl_input_mux!(ctimer, 7, P3_14);
impl_input_mux!(ctimer, 8, P3_15);
impl_input_mux!(ctimer, 9, P3_16);
impl_input_mux!(ctimer, 10, P3_17);
impl_input_mux!(ctimer, 11, P3_22);
impl_input_mux!(ctimer, 14, P3_27);
impl_input_mux!(ctimer, 13, P3_28);
impl_input_mux!(ctimer, 4, P3_29);

impl_input_mux!(ctimer, 7, P4_6);
impl_input_mux!(ctimer, 8, P4_7);
