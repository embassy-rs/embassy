//! Low-power timer (LPTIM)

mod traits;

use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};
pub use traits::Instance;

use crate::gpio::{AfType, AnyPin, OutputType, Speed};
// use crate::time::Hertz;
use crate::{rcc, Peripheral};

/// LPTIM master instance.
pub struct Master<T: Instance> {
    phantom: PhantomData<T>,
}

/// LPTIM channel 1 instance.
pub struct Ch1<T: Instance> {
    phantom: PhantomData<T>,
}

/// LPTIM channel 2 instance.
pub struct Ch2<T: Instance> {
    phantom: PhantomData<T>,
}

trait SealedChannel<T: Instance> {
    fn raw() -> usize;
}

///  channel instance trait.
#[allow(private_bounds)]
pub trait Channel<T: Instance>: SealedChannel<T> {}

/// LPTIM PWM pin.
pub struct PwmPin<'d, T, C> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(T, C)>,
}

macro_rules! channel_impl {
    ($new_chx:ident, $channel:tt, $ch_num:expr, $pin_trait:ident) => {
        impl<'d, T: Instance> PwmPin<'d, T, $channel<T>> {
            #[doc = concat!("Create a new ", stringify!($channel), " PWM pin instance.")]
            pub fn $new_chx(pin: impl Peripheral<P = impl $pin_trait<T>> + 'd) -> Self {
                into_ref!(pin);
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(
                        pin.af_num(),
                        AfType::output(OutputType::PushPull, Speed::VeryHigh),
                    );
                });
                PwmPin {
                    _pin: pin.map_into(),
                    phantom: PhantomData,
                }
            }
        }

        impl<T: Instance> SealedChannel<T> for $channel<T> {
            fn raw() -> usize {
                $ch_num
            }
        }
        impl<T: Instance> Channel<T> for $channel<T> {}
    };
}

channel_impl!(new_ch1, Ch1, 0, Channel1Pin);
channel_impl!(new_ch2, Ch2, 1, Channel2Pin);

/// Struct used to divide a high resolution timer into multiple channels
pub struct Pwm<'d, T: Instance> {
    _inner: PeripheralRef<'d, T>,
    /// Master instance.
    pub master: Master<T>,
    /// Channel 1.
    pub ch_1: Ch1<T>,
    /// Channel 2.
    pub ch_2: Ch2<T>,
}

impl<'d, T: Instance> Pwm<'d, T> {
    /// Create a new LPTIM driver.
    ///
    /// This splits the LPTIM into its constituent parts, which you can then use individually.
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        _ch1: Option<PwmPin<'d, T, Ch1<T>>>,
        _ch2: Option<PwmPin<'d, T, Ch2<T>>>,
    ) -> Self {
        Self::new_inner(tim)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(tim);

        rcc::enable_and_reset::<T>();

        T::regs().cr().modify(|w| w.set_enable(true));

        // Set frequency. Should be configurable.
        // By default, 16MHz. We want 440Hz.
        // That's 36363 cycles
        T::regs().arr().write_value(stm32_metapac::lptim::regs::Arr(36363));

        // Set duty cycle. Should be configurable. Should take care of channel too (only Ch1 now)
        T::regs().ccr(0).write_value(stm32_metapac::lptim::regs::Ccr(18181));

        // Enable channel as PWM. Default state anyway. Implement later.
        // T::regs().ccmr().modify(|w| {
        //     w.set_ccsel(0, 0);
        //     w.set_ccsel(1, 0);
        // })

        // Enable output on pins. Should care about the channels!
        T::regs().ccmr().modify(|w| {
            w.set_cce(0, true);
            w.set_cce(1, true);
        });

        Self {
            _inner: tim,
            master: Master { phantom: PhantomData },
            ch_1: Ch1 { phantom: PhantomData },
            ch_2: Ch2 { phantom: PhantomData },
        }
    }

    /// Start
    pub fn start(&mut self) {
        T::regs().cr().modify(|w| w.set_cntstrt(true));
    }

    /// Stop
    pub fn stop(&mut self) {
        T::regs().cr().modify(|w| w.set_cntstrt(false));
    }
}

pin_trait!(Channel1Pin, Instance);
pin_trait!(Channel2Pin, Instance);
