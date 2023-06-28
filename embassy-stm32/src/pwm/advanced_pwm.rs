use core::marker::PhantomData;

use embassy_hal_common::{into_ref, PeripheralRef};

use super::*;
#[allow(unused_imports)]
use crate::gpio::sealed::{AFType, Pin};
use crate::gpio::AnyPin;
use crate::time::Hertz;
use crate::Peripheral;

// Re-implement the channels for hrtim
pub struct Master<T: AdvancedCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}
pub struct ChA<T: AdvancedCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}
pub struct ChB<T: AdvancedCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}
pub struct ChC<T: AdvancedCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}
pub struct ChD<T: AdvancedCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}
pub struct ChE<T: AdvancedCaptureCompare16bitInstance> {
    phantom: PhantomData<T>,
}

mod sealed {
    use crate::pwm::AdvancedCaptureCompare16bitInstance;

    pub trait AdvancedChannel<T: AdvancedCaptureCompare16bitInstance> {}
}

pub trait AdvancedChannel<T: AdvancedCaptureCompare16bitInstance>: sealed::AdvancedChannel<T> {
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
        impl<'d, Perip: AdvancedCaptureCompare16bitInstance> PwmPin<'d, Perip, $channel<Perip>> {
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

        impl<'d, Perip: AdvancedCaptureCompare16bitInstance> ComplementaryPwmPin<'d, Perip, $channel<Perip>> {
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

        impl<T: AdvancedCaptureCompare16bitInstance> sealed::AdvancedChannel<T> for $channel<T> {}
        impl<T: AdvancedCaptureCompare16bitInstance> AdvancedChannel<T> for $channel<T> {
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
pub struct AdvancedPwm<'d, T: AdvancedCaptureCompare16bitInstance> {
    _inner: PeripheralRef<'d, T>,
    pub master: Master<T>,
    pub ch_a: ChA<T>,
    pub ch_b: ChB<T>,
    pub ch_c: ChC<T>,
    pub ch_d: ChD<T>,
    pub ch_e: ChE<T>,
}

impl<'d, T: AdvancedCaptureCompare16bitInstance> AdvancedPwm<'d, T> {
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
            ch_a: ChA { phantom: PhantomData },
            ch_b: ChB { phantom: PhantomData },
            ch_c: ChC { phantom: PhantomData },
            ch_d: ChD { phantom: PhantomData },
            ch_e: ChE { phantom: PhantomData },
        }
    }

    /// Set the dead time as a proportion of max_duty
    pub fn set_dead_time(&mut self, _value: u16) {
        todo!()
        //        let (ckd, value) = compute_dead_time_value(value);
        //
        //        self.inner.set_dead_time_clock_division(ckd);
        //        self.inner.set_dead_time_value(value);
    }
}

// Represents a fixed-frequency bridge converter
pub struct BridgeConverter<T: AdvancedCaptureCompare16bitInstance, C: AdvancedChannel<T>> {
    phantom: PhantomData<T>,
    pub ch: C,
}

impl<T: AdvancedCaptureCompare16bitInstance, C: AdvancedChannel<T>> BridgeConverter<T, C> {
    pub fn new(channel: C, frequency: Hertz) -> Self {
        Self {
            phantom: PhantomData,
            ch: channel,
        }
    }

    pub fn set_duty(&mut self, primary: u16, secondary: u16) {
        let _ = T::regs();
        let _ = C::raw();

        todo!()
    }
}

// Represents a variable-frequency resonant converter
pub struct ResonantConverter<T: AdvancedCaptureCompare16bitInstance, C: AdvancedChannel<T>> {
    phantom: PhantomData<T>,
    pub ch: C,
}

impl<T: AdvancedCaptureCompare16bitInstance, C: AdvancedChannel<T>> ResonantConverter<T, C> {
    pub fn new(channel: C, min_frequency: Hertz) -> Self {
        Self {
            phantom: PhantomData,
            ch: channel,
        }
    }

    pub fn set_frequency(&mut self, frequency: Hertz) {
        todo!()
    }
}
