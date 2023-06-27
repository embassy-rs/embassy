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
pub struct Master {
    phantom: PhantomData<bool>,
}
pub struct ChA {
    phantom: PhantomData<bool>,
}
pub struct ChB {
    phantom: PhantomData<bool>,
}
pub struct ChC {
    phantom: PhantomData<bool>,
}
pub struct ChD {
    phantom: PhantomData<bool>,
}
pub struct ChE {
    phantom: PhantomData<bool>,
}

mod sealed {
    pub trait AdvancedChannel {}
}

pub trait AdvancedChannel: sealed::AdvancedChannel {}

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

        impl sealed::AdvancedChannel for $channel {}
        impl AdvancedChannel for $channel {}
    };
}

advanced_channel_impl!(new_cha, ChA, ChannelAPin, ChannelAComplementaryPin);
advanced_channel_impl!(new_chb, ChB, ChannelBPin, ChannelBComplementaryPin);
advanced_channel_impl!(new_chc, ChC, ChannelCPin, ChannelCComplementaryPin);
advanced_channel_impl!(new_chd, ChD, ChannelDPin, ChannelDComplementaryPin);
advanced_channel_impl!(new_che, ChE, ChannelEPin, ChannelEComplementaryPin);

/// Struct used to divide a high resolution timer into multiple channels
pub struct AdvancedPwm<'d, T> {
    inner: PeripheralRef<'d, T>,
    pub master: Master,
    pub ch_a: ChA,
    pub ch_b: ChB,
    pub ch_c: ChC,
    pub ch_d: ChD,
    pub ch_e: ChE,
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
    ) -> Self {
        Self::new_inner(tim)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(tim);

        T::enable();
        <T as crate::rcc::sealed::RccPeripheral>::reset();

        Self {
            inner: tim,
            master: Master { phantom: PhantomData },
            ch_a: ChA { phantom: PhantomData },
            ch_b: ChB { phantom: PhantomData },
            ch_c: ChC { phantom: PhantomData },
            ch_d: ChD { phantom: PhantomData },
            ch_e: ChE { phantom: PhantomData },
        }
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
    }

    //    pub fn enable(&mut self, channel: AdvancedChannel) {
    //        // self.inner.enable_channel(channel, true);
    //        // self.inner.enable_complementary_channel(channel, true);
    //    }
    //
    //    pub fn disable(&mut self, channel: AdvancedChannel) {
    //        // self.inner.enable_complementary_channel(channel, false);
    //        // self.inner.enable_channel(channel, false);
    //    }
    //
    //    pub fn set_freq(&mut self, freq: Hertz) {
    //        // self.inner.set_frequency(freq);
    //    }
    //
    //    pub fn get_max_duty(&self) -> u16 {
    //        todo!()
    //        // self.inner.get_max_compare_value()
    //    }
    //
    //    pub fn set_duty(&mut self, channel: AdvancedChannel, duty: u16) {
    //        // assert!(duty < self.get_max_duty());
    //        // self.inner.set_compare_value(channel, duty)
    //    }

    /// Set the dead time as a proportion of max_duty
    pub fn set_dead_time(&mut self, value: u16) {
        //        let (ckd, value) = compute_dead_time_value(value);
        //
        //        self.inner.set_dead_time_clock_division(ckd);
        //        self.inner.set_dead_time_value(value);
    }
}

// Represents a fixed-frequency bridge converter
pub struct BridgeConverter<T: AdvancedChannel> {
    pub ch: T,
}

impl<T: AdvancedChannel> BridgeConverter<T> {
    pub fn new(channel: T, frequency: Hertz) -> Self {
        Self { ch: channel }
    }

    pub fn set_duty(&mut self, primary: u16, secondary: u16) {
        todo!()
    }
}

// Represents a variable-frequency resonant converter
pub struct ResonantConverter<T: AdvancedChannel> {
    pub ch: T,
}

impl<T: AdvancedChannel> ResonantConverter<T> {
    pub fn new(channel: T, min_frequency: Hertz) -> Self {
        Self { ch: channel }
    }

    pub fn set_frequency(&mut self, frequency: Hertz) {
        todo!()
    }
}
