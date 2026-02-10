//! AdvancedPwm

use core::marker::PhantomData;

use embassy_hal_internal::Peri;

use crate::gpio::{AfType, Flex, OutputType, Speed};
pub use crate::timer::simple_pwm::PwmPinConfig;

use super::{BurstController, ChA, ChB, ChC, ChD, ChE, Instance, Master, Prescaler};
#[cfg(hrtim_v2)]
use super::{ChF, ChannelFComplementaryPin, ChannelFPin};
use super::{
    ChannelAComplementaryPin, ChannelBComplementaryPin, ChannelCComplementaryPin, ChannelDComplementaryPin,
    ChannelEComplementaryPin,
};
use super::{ChannelAPin, ChannelBPin, ChannelCPin, ChannelDPin, ChannelEPin};

use crate::rcc;
use crate::time::Hertz;

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

    /// Set master frequency
    pub fn set_master_frequency(&mut self, frequency: Hertz) {
        let f = frequency.0;

        // TODO: wire up HRTIM to the RCC mux infra.
        //#[cfg(stm32f334)]
        //let timer_f = unsafe { crate::rcc::get_freqs() }.hrtim.unwrap_or(T::frequency()).0;
        //#[cfg(not(stm32f334))]
        let timer_f = T::frequency().0;

        let psc_min = (timer_f / f) / (u16::MAX as u32 / 32);
        let psc = if T::regs().isr().read().dllrdy() {
            Prescaler::compute_min_high_res(psc_min)
        } else {
            Prescaler::compute_min_low_res(psc_min)
        };

        let timer_f = 32 * (timer_f as u64 / psc as u64);
        let per: u16 = (timer_f / f as u64) as u16;

        let regs = T::regs();

        regs.mcr().modify(|w| w.set_ckpsc(psc.into()));
        regs.mper().modify(|w| w.set_mper(per));
    }
}

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

macro_rules! channel_impl {
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
    };
}

channel_impl!(new_cha, new_cha_with_config, ChA, 0, ChannelAPin, ChannelAComplementaryPin);
channel_impl!(new_chb, new_chb_with_config, ChB, 1, ChannelBPin, ChannelBComplementaryPin);
channel_impl!(new_chc, new_chc_with_config, ChC, 2, ChannelCPin, ChannelCComplementaryPin);
channel_impl!(new_chd, new_chd_with_config, ChD, 3, ChannelDPin, ChannelDComplementaryPin);
channel_impl!(new_che, new_che_with_config, ChE, 4, ChannelEPin, ChannelEComplementaryPin);
#[cfg(hrtim_v2)]
channel_impl!(new_chf, new_chf_with_config, ChF, 5, ChannelFPin, ChannelFComplementaryPin);
