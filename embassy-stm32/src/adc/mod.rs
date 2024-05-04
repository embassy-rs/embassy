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
#[cfg_attr(any(adc_v3, adc_g0, adc_h5), path = "v3.rs")]
#[cfg_attr(adc_v4, path = "v4.rs")]
mod _version;

#[allow(unused)]
#[cfg(not(adc_f3_v2))]
pub use _version::*;
use embassy_sync::waitqueue::AtomicWaker;
#[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1))]
use embassy_sync::waitqueue::AtomicWaker;

#[cfg(not(any(adc_f1, adc_f3_v2)))]
pub use crate::pac::adc::vals::Res as Resolution;
pub use crate::pac::adc::vals::SampleTime;
use crate::peripherals;

#[cfg_attr(adc_v2, path = "v2.rs")]
dma_trait!(RxDma, Instance);
/// Analog to Digital driver.
pub struct Adc<'d, T: Instance> {
    #[allow(unused)]
    adc: crate::PeripheralRef<'d, T>,
    #[cfg(not(any(adc_f3_v2, adc_f3_v1_1)))]
    sample_time: SampleTime,
}

#[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1, adc_v2))]
pub struct State {
    pub waker: AtomicWaker,
}

#[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1, adc_v2))]
impl State {
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

trait SealedInstance {
    #[allow(unused)]
    fn regs() -> crate::pac::adc::Adc;
    #[cfg(not(any(adc_f1, adc_v1, adc_l0, adc_f3_v2, adc_f3_v1_1, adc_g0)))]
    fn common_regs() -> crate::pac::adccommon::AdcCommon;
    #[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1, adc_v2))]
    fn state() -> &'static State;
}

pub(crate) trait SealedAdcPin<T: Instance> {
    #[cfg(any(adc_v1, adc_l0, adc_v2))]
    fn set_as_analog(&mut self) {}

    #[allow(unused)]
    fn channel(&self) -> u8;
}

trait SealedInternalChannel<T> {
    #[allow(unused)]
    fn channel(&self) -> u8;
}

/// ADC instance.
#[cfg(not(any(adc_f1, adc_v1, adc_l0, adc_v2, adc_v3, adc_v4, adc_f3, adc_f3_v1_1, adc_g0, adc_h5)))]
#[allow(private_bounds)]
pub trait Instance: SealedInstance + crate::Peripheral<P = Self> {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}
/// ADC instance.
#[cfg(any(adc_f1, adc_v1, adc_l0, adc_v2, adc_v3, adc_v4, adc_f3, adc_f3_v1_1, adc_g0, adc_h5))]
#[allow(private_bounds)]
pub trait Instance: SealedInstance + crate::Peripheral<P = Self> + crate::rcc::RccPeripheral {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// ADC pin.
#[allow(private_bounds)]
pub trait AdcPin<T: Instance>: SealedAdcPin<T> {}
/// ADC internal channel.
#[allow(private_bounds)]
pub trait InternalChannel<T>: SealedInternalChannel<T> {}

foreach_adc!(
    ($inst:ident, $common_inst:ident, $clock:ident) => {
        impl crate::adc::SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::adc::Adc {
                crate::pac::$inst
            }

            #[cfg(not(any(adc_f1, adc_v1, adc_l0, adc_f3_v2, adc_f3_v1_1, adc_g0)))]
            fn common_regs() -> crate::pac::adccommon::AdcCommon {
                return crate::pac::$common_inst
            }

            #[cfg(any(adc_f1, adc_f3, adc_v1, adc_v2, adc_l0, adc_f3_v1_1))]
            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl crate::adc::Instance for peripherals::$inst {
            type Interrupt = crate::_generated::peripheral_interrupts::$inst::GLOBAL;
        }
    };
);

macro_rules! impl_adc_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::adc::AdcPin<peripherals::$inst> for crate::peripherals::$pin {}

        impl crate::adc::SealedAdcPin<peripherals::$inst> for crate::peripherals::$pin {
            #[cfg(any(adc_v1, adc_l0, adc_v2))]
            fn set_as_analog(&mut self) {
                <Self as crate::gpio::SealedPin>::set_as_analog(self);
            }

            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}

/// Get the maximum reading value for this resolution.
///
/// This is `2**n - 1`.
#[cfg(not(any(adc_f1, adc_f3_v2)))]
pub const fn resolution_to_max_count(res: Resolution) -> u32 {
    match res {
        #[cfg(adc_v4)]
        Resolution::BITS16 => (1 << 16) - 1,
        #[cfg(adc_v4)]
        Resolution::BITS14 => (1 << 14) - 1,
        #[cfg(adc_v4)]
        Resolution::BITS14V => (1 << 14) - 1,
        #[cfg(adc_v4)]
        Resolution::BITS12V => (1 << 12) - 1,
        Resolution::BITS12 => (1 << 12) - 1,
        Resolution::BITS10 => (1 << 10) - 1,
        Resolution::BITS8 => (1 << 8) - 1,
        #[cfg(any(adc_v1, adc_v2, adc_v3, adc_l0, adc_g0, adc_f3, adc_f3_v1_1, adc_h5))]
        Resolution::BITS6 => (1 << 6) - 1,
        #[allow(unreachable_patterns)]
        _ => core::unreachable!(),
    }
}
