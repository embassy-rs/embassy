use core::marker::PhantomData;

use embassy_hal_internal::into_ref;

use crate::gpio::sealed::AFType;
use crate::gpio::Speed;
#[cfg(not(stm32wl))]
pub use crate::pac::rcc::vals::{Mco1 as Mco1Source, Mco2 as Mco2Source};
#[cfg(stm32wl)]
pub use crate::pac::rcc::vals::{Mcopre, Mcosel};
use crate::pac::RCC;
use crate::{peripherals, Peripheral};

pub(crate) mod sealed {
    pub trait McoInstance {
        type Source;
        type Prescaler;
        unsafe fn apply_clock_settings(source: Self::Source, prescaler: Self::Prescaler);
    }
}

pub trait McoInstance: sealed::McoInstance + 'static {}

pin_trait!(McoPin, McoInstance);

macro_rules! impl_peri {
    ($peri:ident, $source:ident, $prescaler:ident, $set_source:ident, $set_prescaler:ident) => {
        impl sealed::McoInstance for peripherals::$peri {
            type Source = $source;
            type Prescaler = $prescaler;

            unsafe fn apply_clock_settings(source: Self::Source, prescaler: Self::Prescaler) {
                RCC.cfgr().modify(|w| {
                    w.$set_source(source);
                    w.$set_prescaler(prescaler);
                });
            }
        }

        impl McoInstance for peripherals::$peri {}
    };
}

#[cfg(not(stm32wl))]
impl_peri!(MCO1, Mco1Source, u8, set_mco1, set_mco1pre);
#[cfg(not(stm32wl))]
impl_peri!(MCO2, Mco2Source, u8, set_mco2, set_mco2pre);
#[cfg(stm32wl)]
impl_peri!(MCO, Mcosel, Mcopre, set_mcosel, set_mcopre);

pub struct Mco<'d, T: McoInstance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: McoInstance> Mco<'d, T> {
    /// Create a new MCO instance.
    ///
    /// `prescaler` must be between 1 and 15 for implementations not using Presel enum.
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        pin: impl Peripheral<P = impl McoPin<T>> + 'd,
        source: T::Source,
        prescaler: T::Prescaler,
    ) -> Self {
        into_ref!(pin);

        #[cfg(not(stm32wl))]
        assert!(
            1 <= prescaler && prescaler <= 15,
            "Mco prescaler must be between 1 and 15. Refer to the reference manual for more information."
        );

        critical_section::with(|_| unsafe {
            T::apply_clock_settings(source, prescaler);
            pin.set_as_af(pin.af_num(), AFType::OutputPushPull);
            pin.set_speed(Speed::VeryHigh);
        });

        Self { phantom: PhantomData }
    }
}
