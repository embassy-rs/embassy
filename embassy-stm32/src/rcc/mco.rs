use core::marker::PhantomData;

use embassy_hal_internal::into_ref;

use crate::gpio::sealed::AFType;
use crate::gpio::Speed;
pub use crate::pac::rcc::vals::{Mco1 as Mco1Source, Mco2 as Mco2Source};
use crate::pac::RCC;
use crate::{peripherals, Peripheral};

pub(crate) mod sealed {
    pub trait McoInstance {
        type Source;
        unsafe fn apply_clock_settings(source: Self::Source, prescaler: u8);
    }
}

pub trait McoInstance: sealed::McoInstance + 'static {}

pin_trait!(McoPin, McoInstance);

macro_rules! impl_peri {
    ($peri:ident, $source:ident, $set_source:ident, $set_prescaler:ident) => {
        impl sealed::McoInstance for peripherals::$peri {
            type Source = $source;

            unsafe fn apply_clock_settings(source: Self::Source, prescaler: u8) {
                RCC.cfgr().modify(|w| {
                    w.$set_source(source);
                    w.$set_prescaler(prescaler);
                });
            }
        }

        impl McoInstance for peripherals::$peri {}
    };
}

impl_peri!(MCO1, Mco1Source, set_mco1, set_mco1pre);
impl_peri!(MCO2, Mco2Source, set_mco2, set_mco2pre);

pub struct Mco<'d, T: McoInstance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: McoInstance> Mco<'d, T> {
    /// Create a new MCO instance.
    ///
    /// `prescaler` must be between 1 and 15.
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        pin: impl Peripheral<P = impl McoPin<T>> + 'd,
        source: T::Source,
        prescaler: u8,
    ) -> Self {
        into_ref!(pin);

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
