//! Analog to Digital Converter (ADC)

#![macro_use]
#![allow(missing_docs)] // TODO

#[cfg(not(adc_f3_v2))]
#[cfg_attr(adc_f1, path = "f1.rs")]
#[cfg_attr(adc_f3, path = "f3.rs")]
#[cfg_attr(adc_f3_v1_1, path = "f3_v1_1.rs")]
#[cfg_attr(adc_v1, path = "v1.rs")]
#[cfg_attr(adc_l0, path = "v1.rs")]
#[cfg_attr(adc_v2, path = "v2.rs")]
#[cfg_attr(any(adc_v3, adc_g0), path = "v3.rs")]
#[cfg_attr(adc_v4, path = "v4.rs")]
mod _version;

#[cfg(not(any(adc_f1, adc_f3_v2)))]
mod resolution;
mod sample_time;

#[allow(unused)]
#[cfg(not(adc_f3_v2))]
pub use _version::*;
#[cfg(not(any(adc_f1, adc_f3, adc_f3_v2)))]
pub use resolution::Resolution;
#[cfg(not(adc_f3_v2))]
pub use sample_time::SampleTime;

use crate::peripherals;

/// Analog to Digital driver.
pub struct Adc<'d, T: Instance> {
    #[allow(unused)]
    adc: crate::PeripheralRef<'d, T>,
    #[cfg(not(any(adc_f3_v2, adc_f3_v1_1)))]
    sample_time: SampleTime,
}

pub(crate) mod sealed {
    #[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1))]
    use embassy_sync::waitqueue::AtomicWaker;

    #[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1))]
    pub struct State {
        pub waker: AtomicWaker,
    }

    #[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1))]
    impl State {
        pub const fn new() -> Self {
            Self {
                waker: AtomicWaker::new(),
            }
        }
    }

    pub trait InterruptableInstance {
        type Interrupt: crate::interrupt::typelevel::Interrupt;
    }

    pub trait Instance: InterruptableInstance {
        fn regs() -> crate::pac::adc::Adc;
        #[cfg(not(any(adc_f1, adc_v1, adc_l0, adc_f3_v2, adc_f3_v1_1, adc_g0)))]
        fn common_regs() -> crate::pac::adccommon::AdcCommon;
        #[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1))]
        fn state() -> &'static State;
    }

    pub trait AdcPin<T: Instance> {
        #[cfg(any(adc_v1, adc_l0, adc_v2))]
        fn set_as_analog(&mut self) {}

        fn channel(&self) -> u8;
    }

    pub trait InternalChannel<T> {
        fn channel(&self) -> u8;
    }
}

/// ADC instance.
#[cfg(not(any(adc_f1, adc_v1, adc_l0, adc_v2, adc_v3, adc_v4, adc_f3, adc_f3_v1_1, adc_g0)))]
pub trait Instance: sealed::Instance + crate::Peripheral<P = Self> {}
/// ADC instance.
#[cfg(any(adc_f1, adc_v1, adc_l0, adc_v2, adc_v3, adc_v4, adc_f3, adc_f3_v1_1, adc_g0))]
pub trait Instance: sealed::Instance + crate::Peripheral<P = Self> + crate::rcc::RccPeripheral {}

/// ADC pin.
pub trait AdcPin<T: Instance>: sealed::AdcPin<T> {}
/// ADC internal channel.
pub trait InternalChannel<T>: sealed::InternalChannel<T> {}

foreach_adc!(
    ($inst:ident, $common_inst:ident, $clock:ident) => {
        impl crate::adc::sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::adc::Adc {
                crate::pac::$inst
            }

            #[cfg(not(any(adc_f1, adc_v1, adc_l0, adc_f3_v2, adc_f3_v1_1, adc_g0)))]
            fn common_regs() -> crate::pac::adccommon::AdcCommon {
                return crate::pac::$common_inst
            }

            #[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1))]
            fn state() -> &'static sealed::State {
                static STATE: sealed::State = sealed::State::new();
                &STATE
            }
        }

        foreach_interrupt!(
            ($inst,adc,ADC,GLOBAL,$irq:ident) => {
                impl sealed::InterruptableInstance for peripherals::$inst {
                    type Interrupt = crate::interrupt::typelevel::$irq;
                }
            };
        );

        impl crate::adc::Instance for peripherals::$inst {}
    };
);

macro_rules! impl_adc_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::adc::AdcPin<peripherals::$inst> for crate::peripherals::$pin {}

        impl crate::adc::sealed::AdcPin<peripherals::$inst> for crate::peripherals::$pin {
            #[cfg(any(adc_v1, adc_l0, adc_v2))]
            fn set_as_analog(&mut self) {
                <Self as crate::gpio::sealed::Pin>::set_as_analog(self);
            }

            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}
