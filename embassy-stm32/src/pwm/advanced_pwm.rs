use core::marker::PhantomData;

use embassy_hal_common::{into_ref, PeripheralRef};

use super::*;
#[allow(unused_imports)]
use crate::gpio::sealed::{AFType, Pin};
use crate::gpio::AnyPin;
use crate::time::Hertz;
use crate::Peripheral;

pub enum Source {
    Master,
    ChA,
    ChB,
    ChC,
    ChD,
    ChE,
}

pub struct BurstController<T: HighResolutionCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}
pub struct Master<T: HighResolutionCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}
pub struct ChA<T: HighResolutionCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}
pub struct ChB<T: HighResolutionCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}
pub struct ChC<T: HighResolutionCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}
pub struct ChD<T: HighResolutionCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}
pub struct ChE<T: HighResolutionCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}

mod sealed {
    use crate::pwm::HighResolutionCaptureCompare16bitInstance;

    pub trait AdvancedChannel<T: HighResolutionCaptureCompare16bitInstance> {}
}

pub trait AdvancedChannel<T: HighResolutionCaptureCompare16bitInstance>: sealed::AdvancedChannel<T> {
    fn raw() -> usize;
}

pub struct PwmPin<'d, Perip, Channel> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(Perip, Channel)>,
}

pub struct ComplementaryPwmPin<'d, Perip, Channel> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(Perip, Channel)>,
}

macro_rules! advanced_channel_impl {
    ($new_chx:ident, $channel:tt, $ch_num:expr, $pin_trait:ident, $complementary_pin_trait:ident) => {
        impl<'d, Perip: HighResolutionCaptureCompare16bitInstance> PwmPin<'d, Perip, $channel<Perip>> {
            pub fn $new_chx(pin: impl Peripheral<P = impl $pin_trait<Perip>> + 'd) -> Self {
                into_ref!(pin);
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(pin.af_num(), AFType::OutputPushPull);
                    #[cfg(gpio_v2)]
                    pin.set_speed(crate::gpio::Speed::VeryHigh);
                });
                PwmPin {
                    _pin: pin.map_into(),
                    phantom: PhantomData,
                }
            }
        }

        impl<'d, Perip: HighResolutionCaptureCompare16bitInstance> ComplementaryPwmPin<'d, Perip, $channel<Perip>> {
            pub fn $new_chx(pin: impl Peripheral<P = impl $complementary_pin_trait<Perip>> + 'd) -> Self {
                into_ref!(pin);
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(pin.af_num(), AFType::OutputPushPull);
                    #[cfg(gpio_v2)]
                    pin.set_speed(crate::gpio::Speed::VeryHigh);
                });
                ComplementaryPwmPin {
                    _pin: pin.map_into(),
                    phantom: PhantomData,
                }
            }
        }

        impl<T: HighResolutionCaptureCompare16bitInstance> sealed::AdvancedChannel<T> for $channel<T> {}
        impl<T: HighResolutionCaptureCompare16bitInstance> AdvancedChannel<T> for $channel<T> {
            fn raw() -> usize {
                $ch_num
            }
        }
    };
}

advanced_channel_impl!(new_cha, ChA, 0, ChannelAPin, ChannelAComplementaryPin);
advanced_channel_impl!(new_chb, ChB, 1, ChannelBPin, ChannelBComplementaryPin);
advanced_channel_impl!(new_chc, ChC, 2, ChannelCPin, ChannelCComplementaryPin);
advanced_channel_impl!(new_chd, ChD, 3, ChannelDPin, ChannelDComplementaryPin);
advanced_channel_impl!(new_che, ChE, 4, ChannelEPin, ChannelEComplementaryPin);

/// Struct used to divide a high resolution timer into multiple channels
pub struct AdvancedPwm<'d, T: HighResolutionCaptureCompare16bitInstance> {
    _inner: PeripheralRef<'d, T>,
    pub master: Master<T>,
    pub burst_controller: BurstController<T>,
    pub ch_a: ChA<T>,
    pub ch_b: ChB<T>,
    pub ch_c: ChC<T>,
    pub ch_d: ChD<T>,
    pub ch_e: ChE<T>,
}

impl<'d, T: HighResolutionCaptureCompare16bitInstance> AdvancedPwm<'d, T> {
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
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
    ) -> Self {
        Self::new_inner(tim)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(tim);

        T::enable();
        <T as crate::rcc::sealed::RccPeripheral>::reset();

        Self {
            _inner: tim,
            master: Master { phantom: PhantomData },
            burst_controller: BurstController { phantom: PhantomData },
            ch_a: ChA { phantom: PhantomData },
            ch_b: ChB { phantom: PhantomData },
            ch_c: ChC { phantom: PhantomData },
            ch_d: ChD { phantom: PhantomData },
            ch_e: ChE { phantom: PhantomData },
        }
    }
}

impl<T: HighResolutionCaptureCompare16bitInstance> BurstController<T> {
    pub fn set_source(&mut self, source: Source) {
        let regs = T::regs();
    }
}

/// Represents a fixed-frequency bridge converter
///
/// Our implementation of the bridge converter uses a single channel and two compare registers,
/// allowing implementation of a synchronous buck or boost converter in continuous or discontinuous
/// conduction mode.
///
/// It is important to remember that in synchronous topologies, energy can flow in reverse during
/// light loading conditions, and that the low-side switch must be active for a short time to drive
/// a bootstrapped high-side switch.
pub struct BridgeConverter<T: HighResolutionCaptureCompare16bitInstance, C: AdvancedChannel<T>> {
    phantom: PhantomData<T>,
    pub ch: C,
}

impl<T: HighResolutionCaptureCompare16bitInstance, C: AdvancedChannel<T>> BridgeConverter<T, C> {
    pub fn new(channel: C, frequency: Hertz) -> Self {
        use crate::pac::hrtim::vals::{Activeeffect, Cont, Inactiveeffect};

        T::set_channel_frequency(C::raw(), frequency);

        // Always enable preload
        T::regs().tim(C::raw()).cr().modify(|w| {
            w.set_preen(true);

            // TODO: fix metapac
            w.set_cont(Cont(1));
        });

        // Set output 1 to active on a period event
        T::regs()
            .tim(C::raw())
            .setr(0)
            .modify(|w| w.set_per(Activeeffect::SETACTIVE));

        // Set output 1 to inactive on a compare 1 event
        T::regs()
            .tim(C::raw())
            .rstr(0)
            .modify(|w| w.set_cmp(0, Inactiveeffect::SETINACTIVE));

        // Set output 2 to active on a compare 1 event
        T::regs()
            .tim(C::raw())
            .setr(1)
            .modify(|w| w.set_cmp(0, Activeeffect::SETACTIVE));

        // Set output 2 to inactive on a compare 2 event
        T::regs()
            .tim(C::raw())
            .rstr(1)
            .modify(|w| w.set_cmp(1, Inactiveeffect::SETINACTIVE));

        Self {
            phantom: PhantomData,
            ch: channel,
        }
    }

    pub fn start(&mut self) {
        T::regs().mcr().modify(|w| w.set_tcen(C::raw(), true));
    }

    pub fn stop(&mut self) {
        T::regs().mcr().modify(|w| w.set_tcen(C::raw(), false));
    }

    /// Set the dead time as a proportion of the maximum compare value
    pub fn set_dead_time(&mut self, value: u16) {
        T::set_channel_dead_time(C::raw(), value);
    }

    /// Get the maximum compare value of a duty cycle
    pub fn get_max_compare_value(&mut self) -> u16 {
        T::regs().tim(C::raw()).per().read().per()
    }

    /// The primary duty is the period in which the primary switch is active
    ///
    /// In the case of a buck converter, this is the high-side switch
    /// In the case of a boost converter, this is the low-side switch
    pub fn set_primary_duty(&mut self, primary: u16) {
        T::regs().tim(C::raw()).cmp(0).modify(|w| w.set_cmp(primary));
    }

    /// The primary duty is the period in any switch is active
    ///
    /// If less than or equal to the primary duty, the secondary switch will never be active
    pub fn set_secondary_duty(&mut self, secondary: u16) {
        T::regs().tim(C::raw()).cmp(1).modify(|w| w.set_cmp(secondary));
    }
}

/// Represents a variable-frequency resonant converter
///
/// This implementation of a resonsant converter is appropriate for a half or full bridge,
/// but does not include secondary rectification, which is appropriate for applications
/// with a low-voltage on the secondary side.
pub struct ResonantConverter<T: HighResolutionCaptureCompare16bitInstance, C: AdvancedChannel<T>> {
    phantom: PhantomData<T>,
    min_period: u16,
    max_period: u16,
    pub ch: C,
}

impl<T: HighResolutionCaptureCompare16bitInstance, C: AdvancedChannel<T>> ResonantConverter<T, C> {
    pub fn new(channel: C, min_frequency: Hertz, max_frequency: Hertz) -> Self {
        use crate::pac::hrtim::vals::Cont;

        T::set_channel_frequency(C::raw(), min_frequency);

        // Always enable preload
        T::regs().tim(C::raw()).cr().modify(|w| {
            w.set_preen(true);

            // TODO: fix metapac
            w.set_cont(Cont(1));
            w.set_half(true);
        });

        // TODO: compute min period value

        Self {
            min_period: 0,
            max_period: T::regs().tim(C::raw()).per().read().per(),
            phantom: PhantomData,
            ch: channel,
        }
    }

    /// Set the dead time as a proportion of the maximum compare value
    pub fn set_dead_time(&mut self, value: u16) {
        T::set_channel_dead_time(C::raw(), value);
    }

    pub fn set_period(&mut self, period: u16) {
        assert!(period < self.max_period);
        assert!(period > self.min_period);

        T::regs().tim(C::raw()).per().modify(|w| w.set_per(period));
    }

    /// Get the minimum compare value of a duty cycle
    pub fn get_min_period(&mut self) -> u16 {
        self.min_period
    }

    /// Get the maximum compare value of a duty cycle
    pub fn get_max_period(&mut self) -> u16 {
        self.max_period
    }
}
