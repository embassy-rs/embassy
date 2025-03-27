use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;

use crate::gpio::{AfType, OutputType, Speed};
#[cfg(not(any(stm32f1, rcc_f0v1, rcc_f3v1, rcc_f37)))]
pub use crate::pac::rcc::vals::Mcopre as McoPrescaler;
#[cfg(not(any(
    rcc_f2,
    rcc_f410,
    rcc_f4,
    rcc_f7,
    rcc_h50,
    rcc_h5,
    rcc_h7ab,
    rcc_h7rm0433,
    rcc_h7,
    rcc_h7rs
)))]
pub use crate::pac::rcc::vals::Mcosel as McoSource;
#[cfg(any(
    rcc_f2,
    rcc_f410,
    rcc_f4,
    rcc_f7,
    rcc_h50,
    rcc_h5,
    rcc_h7ab,
    rcc_h7rm0433,
    rcc_h7,
    rcc_h7rs
))]
pub use crate::pac::rcc::vals::{Mco1sel as Mco1Source, Mco2sel as Mco2Source};
use crate::pac::RCC;
use crate::{peripherals, Peri};

#[cfg(any(stm32f1, rcc_f0v1, rcc_f3v1, rcc_f37))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum McoPrescaler {
    DIV1,
}

pub(crate) trait SealedMcoInstance {}

#[allow(private_bounds)]
pub trait McoInstance: PeripheralType + SealedMcoInstance + 'static {
    type Source;

    #[doc(hidden)]
    unsafe fn _apply_clock_settings(source: Self::Source, prescaler: super::McoPrescaler);
}

pin_trait!(McoPin, McoInstance);

macro_rules! impl_peri {
    ($peri:ident, $source:ident, $set_source:ident, $set_prescaler:ident) => {
        impl SealedMcoInstance for peripherals::$peri {}
        impl McoInstance for peripherals::$peri {
            type Source = $source;

            unsafe fn _apply_clock_settings(source: Self::Source, _prescaler: McoPrescaler) {
                #[cfg(not(any(stm32u5, stm32wba)))]
                let r = RCC.cfgr();
                #[cfg(any(stm32u5, stm32wba))]
                let r = RCC.cfgr1();

                r.modify(|w| {
                    w.$set_source(source);
                    #[cfg(not(any(stm32f1, rcc_f0v1, rcc_f3v1, rcc_f37)))]
                    w.$set_prescaler(_prescaler);
                });
            }
        }
    };
}

#[cfg(any(rcc_c0, rcc_g0, rcc_u0))]
#[allow(unused_imports)]
use self::{McoSource as Mco1Source, McoSource as Mco2Source};

#[cfg(mco)]
impl_peri!(MCO, McoSource, set_mcosel, set_mcopre);
#[cfg(mco1)]
impl_peri!(MCO1, Mco1Source, set_mco1sel, set_mco1pre);
#[cfg(mco2)]
impl_peri!(MCO2, Mco2Source, set_mco2sel, set_mco2pre);

pub struct Mco<'d, T: McoInstance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: McoInstance> Mco<'d, T> {
    /// Create a new MCO instance.
    pub fn new(_peri: Peri<'d, T>, pin: Peri<'d, impl McoPin<T>>, source: T::Source, prescaler: McoPrescaler) -> Self {
        critical_section::with(|_| unsafe {
            T::_apply_clock_settings(source, prescaler);
            pin.set_as_af(pin.af_num(), AfType::output(OutputType::PushPull, Speed::VeryHigh));
        });

        Self { phantom: PhantomData }
    }
}
