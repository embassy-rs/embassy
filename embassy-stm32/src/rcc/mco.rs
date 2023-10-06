use core::marker::PhantomData;

use embassy_hal_internal::into_ref;

use crate::gpio::sealed::AFType;
use crate::gpio::Speed;
#[cfg(not(stm32f1))]
pub use crate::pac::rcc::vals::Mcopre as McoPrescaler;
#[cfg(not(any(rcc_f2, rcc_f410, rcc_f4, rcc_f7, rcc_h50, rcc_h5, rcc_h7ab, rcc_h7rm0433, rcc_h7)))]
pub use crate::pac::rcc::vals::Mcosel as McoSource;
#[cfg(any(rcc_f2, rcc_f410, rcc_f4, rcc_f7, rcc_h50, rcc_h5, rcc_h7ab, rcc_h7rm0433, rcc_h7))]
pub use crate::pac::rcc::vals::{Mco1sel as Mco1Source, Mco2sel as Mco2Source};
use crate::pac::RCC;
use crate::{peripherals, Peripheral};

pub(crate) mod sealed {
    pub trait McoInstance {
        type Source;
        unsafe fn apply_clock_settings(source: Self::Source, #[cfg(not(stm32f1))] prescaler: super::McoPrescaler);
    }
}

pub trait McoInstance: sealed::McoInstance + 'static {}

pin_trait!(McoPin, McoInstance);

macro_rules! impl_peri {
    ($peri:ident, $source:ident, $set_source:ident, $set_prescaler:ident) => {
        impl sealed::McoInstance for peripherals::$peri {
            type Source = $source;

            unsafe fn apply_clock_settings(source: Self::Source, #[cfg(not(stm32f1))] prescaler: McoPrescaler) {
                #[cfg(not(any(stm32u5, stm32wba)))]
                let r = RCC.cfgr();
                #[cfg(any(stm32u5, stm32wba))]
                let r = RCC.cfgr1();

                r.modify(|w| {
                    w.$set_source(source);
                    #[cfg(not(stm32f1))]
                    w.$set_prescaler(prescaler);
                });
            }
        }

        impl McoInstance for peripherals::$peri {}
    };
}

#[cfg(any(rcc_c0, rcc_g0))]
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
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        pin: impl Peripheral<P = impl McoPin<T>> + 'd,
        source: T::Source,
        #[cfg(not(stm32f1))] prescaler: McoPrescaler,
    ) -> Self {
        into_ref!(pin);

        critical_section::with(|_| unsafe {
            T::apply_clock_settings(
                source,
                #[cfg(not(stm32f1))]
                prescaler,
            );
            pin.set_as_af(pin.af_num(), AFType::OutputPushPull);
            pin.set_speed(Speed::VeryHigh);
        });

        Self { phantom: PhantomData }
    }
}
