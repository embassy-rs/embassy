//! High Resolution Timer (HRTIM)

mod traits;

use core::marker::PhantomData;

use embassy_hal_internal::Peri;
pub use traits::Instance;

pub mod bridge_converter;
pub mod resonant_converter;
pub use bridge_converter::BridgeConverter;
pub use resonant_converter::ResonantConverter;

use crate::gpio::{AfType, Flex, OutputType, Speed};
use crate::rcc;
use crate::time::Hertz;
pub use crate::timer::simple_pwm::PwmPinConfig;

/// HRTIM burst controller instance.
pub struct BurstController<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM master instance.
pub struct Master<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel A instance.
pub struct ChA<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel B instance.
pub struct ChB<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel C instance.
pub struct ChC<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel D instance.
pub struct ChD<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel E instance.
pub struct ChE<T: Instance> {
    phantom: PhantomData<T>,
}

/// HRTIM channel F instance.
#[cfg(hrtim_v2)]
pub struct ChF<T: Instance> {
    phantom: PhantomData<T>,
}

trait SealedAdvancedChannel<T: Instance> {
    fn raw() -> usize;
}

/// Advanced channel instance trait.
#[allow(private_bounds)]
pub trait AdvancedChannel<T: Instance>: SealedAdvancedChannel<T> {}

/// HRTIM PWM pin.
pub struct PwmPin<'d, T, C> {
    _pin: Flex<'d>,
    phantom: PhantomData<(T, C)>,
}

/// HRTIM complementary PWM pin.
pub struct ComplementaryPwmPin<'d, T, C> {
    _pin: Flex<'d>,
    phantom: PhantomData<(T, C)>,
}

macro_rules! advanced_channel_impl {
    ($new_chx:ident, $new_chx_with_config:ident, $channel:tt, $ch_num:expr, $pin_trait:ident, $complementary_pin_trait:ident) => {
        impl<'d, T: Instance> PwmPin<'d, T, $channel<T>> {
            #[doc = concat!("Create a new ", stringify!($channel), " PWM pin instance.")]
            pub fn $new_chx(pin: Peri<'d, impl $pin_trait<T>>) -> Self {
                critical_section::with(|_| {
                    pin.set_low();
                    set_as_af!(pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
                });
                PwmPin {
                    _pin: Flex::new(pin),
                    phantom: PhantomData,
                }
            }

            #[doc = concat!("Create a new ", stringify!($channel), " PWM pin instance with a specific configuration.")]
            pub fn $new_chx_with_config(
                pin: Peri<'d, impl $pin_trait<T>>,
                pin_config: PwmPinConfig,
            ) -> Self {
                critical_section::with(|_| {
                    pin.set_low();
                    set_as_af!(pin, AfType::output(pin_config.output_type, pin_config.speed));
                });
                PwmPin {
                    _pin: Flex::new(pin),
                    phantom: PhantomData,
                }
            }
        }

        impl<'d, T: Instance> ComplementaryPwmPin<'d, T, $channel<T>> {
            #[doc = concat!("Create a new ", stringify!($channel), " complementary PWM pin instance.")]
            pub fn $new_chx(pin: Peri<'d, impl $complementary_pin_trait<T>>) -> Self {
                critical_section::with(|_| {
                    pin.set_low();
                    set_as_af!(pin, AfType::output(OutputType::PushPull, Speed::VeryHigh));
                });
                ComplementaryPwmPin {
                    _pin: Flex::new(pin),
                    phantom: PhantomData,
                }
            }

            #[doc = concat!("Create a new ", stringify!($channel), " complementary PWM pin instance with a specific configuration.")]
            pub fn $new_chx_with_config(
                pin: Peri<'d, impl $complementary_pin_trait<T>>,
                pin_config: PwmPinConfig,
            ) -> Self {
                critical_section::with(|_| {
                    pin.set_low();
                    set_as_af!(pin, AfType::output(pin_config.output_type, pin_config.speed));
                });
                ComplementaryPwmPin {
                    _pin: Flex::new(pin),
                    phantom: PhantomData,
                }
            }
        }

        impl<T: Instance> SealedAdvancedChannel<T> for $channel<T> {
            fn raw() -> usize {
                $ch_num
            }
        }
        impl<T: Instance> AdvancedChannel<T> for $channel<T> {}
    };
}

advanced_channel_impl!(
    new_cha,
    new_cha_with_config,
    ChA,
    0,
    ChannelAPin,
    ChannelAComplementaryPin
);
advanced_channel_impl!(
    new_chb,
    new_chb_with_config,
    ChB,
    1,
    ChannelBPin,
    ChannelBComplementaryPin
);
advanced_channel_impl!(
    new_chc,
    new_chc_with_config,
    ChC,
    2,
    ChannelCPin,
    ChannelCComplementaryPin
);
advanced_channel_impl!(
    new_chd,
    new_chd_with_config,
    ChD,
    3,
    ChannelDPin,
    ChannelDComplementaryPin
);
advanced_channel_impl!(
    new_che,
    new_che_with_config,
    ChE,
    4,
    ChannelEPin,
    ChannelEComplementaryPin
);
#[cfg(hrtim_v2)]
advanced_channel_impl!(
    new_chf,
    new_chf_with_config,
    ChF,
    5,
    ChannelFPin,
    ChannelFComplementaryPin
);

/// Struct used to divide a high resolution timer into multiple channels
pub struct AdvancedPwm<'d, T: Instance> {
    _inner: Peri<'d, T>,
    /// Master instance.
    pub master: Master<T>,
    /// Burst controller.
    pub burst_controller: BurstController<T>,
    /// Channel A.
    pub ch_a: ChA<T>,
    /// Channel B.
    pub ch_b: ChB<T>,
    /// Channel C.
    pub ch_c: ChC<T>,
    /// Channel D.
    pub ch_d: ChD<T>,
    /// Channel E.
    pub ch_e: ChE<T>,
    /// Channel F.
    #[cfg(hrtim_v2)]
    pub ch_f: ChF<T>,
}

impl<'d, T: Instance> AdvancedPwm<'d, T> {
    /// Create a new HRTIM driver.
    ///
    /// This splits the HRTIM into its constituent parts, which you can then use individually.
    pub fn new(
        tim: Peri<'d, T>,
        _cha: Option<PwmPin<'d, T, ChA<T>>>,
        _chan: Option<ComplementaryPwmPin<'d, T, ChA<T>>>,
        _chb: Option<PwmPin<'d, T, ChB<T>>>,
        _chbn: Option<ComplementaryPwmPin<'d, T, ChB<T>>>,
        _chc: Option<PwmPin<'d, T, ChC<T>>>,
        _chcn: Option<ComplementaryPwmPin<'d, T, ChC<T>>>,
        _chd: Option<PwmPin<'d, T, ChD<T>>>,
        _chdn: Option<ComplementaryPwmPin<'d, T, ChD<T>>>,
        _che: Option<PwmPin<'d, T, ChE<T>>>,
        _chen: Option<ComplementaryPwmPin<'d, T, ChE<T>>>,
        #[cfg(hrtim_v2)] _chf: Option<PwmPin<'d, T, ChF<T>>>,
        #[cfg(hrtim_v2)] _chfn: Option<ComplementaryPwmPin<'d, T, ChF<T>>>,
    ) -> Self {
        Self::new_inner(tim)
    }

    fn new_inner(tim: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();

        #[cfg(stm32f334)]
        if crate::pac::RCC.cfgr3().read().hrtim1sw() == crate::pac::rcc::vals::Timsw::PLL1_P {
            // Enable and and stabilize the DLL
            T::regs().dllcr().modify(|w| {
                w.set_cal(true);
            });

            trace!("hrtim: wait for dll calibration");
            while !T::regs().isr().read().dllrdy() {}

            trace!("hrtim: dll calibration complete");

            // Enable periodic calibration
            // Cal must be disabled before we can enable it
            T::regs().dllcr().modify(|w| {
                w.set_cal(false);
            });

            T::regs().dllcr().modify(|w| {
                w.set_calen(true);
                w.set_calrte(11);
            });
        }

        Self {
            _inner: tim,
            master: Master { phantom: PhantomData },
            burst_controller: BurstController { phantom: PhantomData },
            ch_a: ChA { phantom: PhantomData },
            ch_b: ChB { phantom: PhantomData },
            ch_c: ChC { phantom: PhantomData },
            ch_d: ChD { phantom: PhantomData },
            ch_e: ChE { phantom: PhantomData },
            #[cfg(hrtim_v2)]
            ch_f: ChF { phantom: PhantomData },
        }
    }
}

pin_trait!(ChannelAPin, Instance);
pin_trait!(ChannelAComplementaryPin, Instance);
pin_trait!(ChannelBPin, Instance);
pin_trait!(ChannelBComplementaryPin, Instance);
pin_trait!(ChannelCPin, Instance);
pin_trait!(ChannelCComplementaryPin, Instance);
pin_trait!(ChannelDPin, Instance);
pin_trait!(ChannelDComplementaryPin, Instance);
pin_trait!(ChannelEPin, Instance);
pin_trait!(ChannelEComplementaryPin, Instance);
#[cfg(hrtim_v2)]
pin_trait!(ChannelFPin, Instance);
#[cfg(hrtim_v2)]
pin_trait!(ChannelFComplementaryPin, Instance);
