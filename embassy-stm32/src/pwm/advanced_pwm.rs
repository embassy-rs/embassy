use core::marker::PhantomData;

use embassy_hal_common::{into_ref, PeripheralRef};

use super::simple_pwm::*;
use super::*;
#[allow(unused_imports)]
use crate::gpio::sealed::{AFType, Pin};
use crate::gpio::AnyPin;
use crate::time::Hertz;
use crate::Peripheral;

// Re-implement the channels for hrtim
pub struct ChA;
pub struct ChB;
pub struct ChC;
pub struct ChD;
pub struct ChE;

pub struct PwmPin<'d, Perip, Channel> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(Perip, Channel)>,
}

pub struct ComplementaryPwmPin<'d, Perip, Channel> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(Perip, Channel)>,
}

macro_rules! advanced_channel_impl {
    ($new_chx:ident, $channel:ident, $pin_trait:ident, $complementary_pin_trait:ident) => {
        impl<'d, Perip: AdvancedCaptureCompare16bitInstance> PwmPin<'d, Perip, $channel> {
            pub fn $new_chx(pin: impl Peripheral<P = impl $complementary_pin_trait<Perip>> + 'd) -> Self {
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

        impl<'d, Perip: AdvancedCaptureCompare16bitInstance> ComplementaryPwmPin<'d, Perip, $channel> {
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
    };
}

advanced_channel_impl!(new_cha, ChA, ChannelAPin, ChannelAComplementaryPin);
advanced_channel_impl!(new_chb, ChB, ChannelBPin, ChannelBComplementaryPin);
advanced_channel_impl!(new_chc, ChC, ChannelCPin, ChannelCComplementaryPin);
advanced_channel_impl!(new_chd, ChD, ChannelDPin, ChannelDComplementaryPin);
advanced_channel_impl!(new_che, ChE, ChannelEPin, ChannelEComplementaryPin);

pub struct AdvancedPwm<'d, T> {
    inner: PeripheralRef<'d, T>,
}

impl<'d, T: ComplementaryCaptureCompare16bitInstance> AdvancedPwm<'d, T> {
    pub fn new(
        tim: impl Peripheral<P = T> + 'd,
        _ch1: Option<PwmPin<'d, T, Ch1>>,
        _ch1n: Option<ComplementaryPwmPin<'d, T, Ch1>>,
        _ch2: Option<PwmPin<'d, T, Ch2>>,
        _ch2n: Option<ComplementaryPwmPin<'d, T, Ch2>>,
        _ch3: Option<PwmPin<'d, T, Ch3>>,
        _ch3n: Option<ComplementaryPwmPin<'d, T, Ch3>>,
        _ch4: Option<PwmPin<'d, T, Ch4>>,
        _ch4n: Option<ComplementaryPwmPin<'d, T, Ch4>>,
        freq: Hertz,
    ) -> Self {
        Self::new_inner(tim, freq)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd, freq: Hertz) -> Self {
        into_ref!(tim);

        T::enable();
        <T as crate::rcc::sealed::RccPeripheral>::reset();

        let mut this = Self { inner: tim };
        //
        //        this.inner.set_frequency(freq);
        //        this.inner.start();
        //
        //        this.inner.enable_outputs(true);
        //
        //        this.inner
        //            .set_output_compare_mode(Channel::Ch1, OutputCompareMode::PwmMode1);
        //        this.inner
        //            .set_output_compare_mode(Channel::Ch2, OutputCompareMode::PwmMode1);
        //        this.inner
        //            .set_output_compare_mode(Channel::Ch3, OutputCompareMode::PwmMode1);
        //        this.inner
        //            .set_output_compare_mode(Channel::Ch4, OutputCompareMode::PwmMode1);
        this
    }

    pub fn enable(&mut self, channel: AdvancedChannel) {
        // self.inner.enable_channel(channel, true);
        // self.inner.enable_complementary_channel(channel, true);
    }

    pub fn disable(&mut self, channel: AdvancedChannel) {
        // self.inner.enable_complementary_channel(channel, false);
        // self.inner.enable_channel(channel, false);
    }

    pub fn set_freq(&mut self, freq: Hertz) {
        // self.inner.set_frequency(freq);
    }

    pub fn get_max_duty(&self) -> u16 {
        todo!()
        // self.inner.get_max_compare_value()
    }

    pub fn set_duty(&mut self, channel: AdvancedChannel, duty: u16) {
        // assert!(duty < self.get_max_duty());
        // self.inner.set_compare_value(channel, duty)
    }

    /// Set the dead time as a proportion of max_duty
    pub fn set_dead_time(&mut self, value: u16) {
        //        let (ckd, value) = compute_dead_time_value(value);
        //
        //        self.inner.set_dead_time_clock_division(ckd);
        //        self.inner.set_dead_time_value(value);
    }
}
