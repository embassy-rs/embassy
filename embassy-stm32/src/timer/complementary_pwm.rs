//! PWM driver with complementary output support.

use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};
use stm32_metapac::timer::vals::Ckd;

use super::simple_pwm::*;
use super::*;
#[allow(unused_imports)]
use crate::gpio::sealed::{AFType, Pin};
use crate::gpio::{AnyPin, OutputType};
use crate::time::Hertz;
use crate::Peripheral;

/// Complementary PWM pin wrapper.
///
/// This wraps a pin to make it usable with PWM.
pub struct ComplementaryPwmPin<'d, T, C> {
    _pin: PeripheralRef<'d, AnyPin>,
    phantom: PhantomData<(T, C)>,
}

macro_rules! complementary_channel_impl {
    ($new_chx:ident, $channel:ident, $pin_trait:ident) => {
        impl<'d, T: ComplementaryCaptureCompare16bitInstance> ComplementaryPwmPin<'d, T, $channel> {
            #[doc = concat!("Create a new ", stringify!($channel), " complementary PWM pin instance.")]
            pub fn $new_chx(pin: impl Peripheral<P = impl $pin_trait<T>> + 'd, output_type: OutputType) -> Self {
                into_ref!(pin);
                critical_section::with(|_| {
                    pin.set_low();
                    pin.set_as_af(pin.af_num(), output_type.into());
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

complementary_channel_impl!(new_ch1, Ch1, Channel1ComplementaryPin);
complementary_channel_impl!(new_ch2, Ch2, Channel2ComplementaryPin);
complementary_channel_impl!(new_ch3, Ch3, Channel3ComplementaryPin);
complementary_channel_impl!(new_ch4, Ch4, Channel4ComplementaryPin);

/// PWM driver with support for standard and complementary outputs.
pub struct ComplementaryPwm<'d, T> {
    inner: PeripheralRef<'d, T>,
}

impl<'d, T: ComplementaryCaptureCompare16bitInstance> ComplementaryPwm<'d, T> {
    /// Create a new complementary PWM driver.
    #[allow(clippy::too_many_arguments)]
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
        counting_mode: CountingMode,
    ) -> Self {
        Self::new_inner(tim, freq, counting_mode)
    }

    fn new_inner(tim: impl Peripheral<P = T> + 'd, freq: Hertz, counting_mode: CountingMode) -> Self {
        into_ref!(tim);

        T::enable_and_reset();

        let mut this = Self { inner: tim };

        this.inner.set_counting_mode(counting_mode);
        this.set_frequency(freq);
        this.inner.start();

        this.inner.enable_outputs();

        [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4]
            .iter()
            .for_each(|&channel| {
                this.inner.set_output_compare_mode(channel, OutputCompareMode::PwmMode1);
                this.inner.set_output_compare_preload(channel, true);
            });

        this
    }

    /// Enable the given channel.
    pub fn enable(&mut self, channel: Channel) {
        self.inner.enable_channel(channel, true);
        self.inner.enable_complementary_channel(channel, true);
    }

    /// Disable the given channel.
    pub fn disable(&mut self, channel: Channel) {
        self.inner.enable_complementary_channel(channel, false);
        self.inner.enable_channel(channel, false);
    }

    /// Set PWM frequency.
    ///
    /// Note: when you call this, the max duty value changes, so you will have to
    /// call `set_duty` on all channels with the duty calculated based on the new max duty.
    pub fn set_frequency(&mut self, freq: Hertz) {
        let multiplier = if self.inner.get_counting_mode().is_center_aligned() {
            2u8
        } else {
            1u8
        };
        self.inner.set_frequency(freq * multiplier);
    }

    /// Get max duty value.
    ///
    /// This value depends on the configured frequency and the timer's clock rate from RCC.
    pub fn get_max_duty(&self) -> u16 {
        self.inner.get_max_compare_value() + 1
    }

    /// Set the duty for a given channel.
    ///
    /// The value ranges from 0 for 0% duty, to [`get_max_duty`](Self::get_max_duty) for 100% duty, both included.
    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        assert!(duty <= self.get_max_duty());
        self.inner.set_compare_value(channel, duty)
    }

    /// Set the output polarity for a given channel.
    pub fn set_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
        self.inner.set_output_polarity(channel, polarity);
        self.inner.set_complementary_output_polarity(channel, polarity);
    }

    /// Set the dead time as a proportion of max_duty
    pub fn set_dead_time(&mut self, value: u16) {
        let (ckd, value) = compute_dead_time_value(value);

        self.inner.set_dead_time_clock_division(ckd);
        self.inner.set_dead_time_value(value);
    }
}

impl<'d, T: ComplementaryCaptureCompare16bitInstance> embedded_hal_02::Pwm for ComplementaryPwm<'d, T> {
    type Channel = Channel;
    type Time = Hertz;
    type Duty = u16;

    fn disable(&mut self, channel: Self::Channel) {
        self.inner.enable_complementary_channel(channel, false);
        self.inner.enable_channel(channel, false);
    }

    fn enable(&mut self, channel: Self::Channel) {
        self.inner.enable_channel(channel, true);
        self.inner.enable_complementary_channel(channel, true);
    }

    fn get_period(&self) -> Self::Time {
        self.inner.get_frequency()
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        self.inner.get_compare_value(channel)
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.inner.get_max_compare_value() + 1
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        assert!(duty <= self.get_max_duty());
        self.inner.set_compare_value(channel, duty)
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        self.inner.set_frequency(period.into());
    }
}

fn compute_dead_time_value(value: u16) -> (Ckd, u8) {
    /*
        Dead-time = T_clk * T_dts * T_dtg

        T_dts:
        This bit-field indicates the division ratio between the timer clock (CK_INT) frequency and the
        dead-time and sampling clock (tDTS)used by the dead-time generators and the digital filters
        (ETR, TIx),
        00: tDTS=tCK_INT
        01: tDTS=2*tCK_INT
        10: tDTS=4*tCK_INT

        T_dtg:
        This bit-field defines the duration of the dead-time inserted between the complementary
        outputs. DT correspond to this duration.
        DTG[7:5]=0xx => DT=DTG[7:0]x tdtg with tdtg=tDTS.
        DTG[7:5]=10x => DT=(64+DTG[5:0])xtdtg with Tdtg=2xtDTS.
        DTG[7:5]=110 => DT=(32+DTG[4:0])xtdtg with Tdtg=8xtDTS.
        DTG[7:5]=111 => DT=(32+DTG[4:0])xtdtg with Tdtg=16xtDTS.
        Example if TDTS=125ns (8MHz), dead-time possible values are:
        0 to 15875 ns by 125 ns steps,
        16 us to 31750 ns by 250 ns steps,
        32 us to 63us by 1 us steps,
        64 us to 126 us by 2 us steps
    */

    let mut error = u16::MAX;
    let mut ckd = Ckd::DIV1;
    let mut bits = 0u8;

    for this_ckd in [Ckd::DIV1, Ckd::DIV2, Ckd::DIV4] {
        let outdiv = match this_ckd {
            Ckd::DIV1 => 1,
            Ckd::DIV2 => 2,
            Ckd::DIV4 => 4,
            _ => unreachable!(),
        };

        // 127
        // 128
        // ..
        // 254
        // 256
        // ..
        // 504
        // 512
        // ..
        // 1008

        let target = value / outdiv;
        let (these_bits, result) = if target < 128 {
            (target as u8, target)
        } else if target < 255 {
            (64 + (target / 2) as u8, (target - target % 2))
        } else if target < 508 {
            (32 + (target / 8) as u8, (target - target % 8))
        } else if target < 1008 {
            (32 + (target / 16) as u8, (target - target % 16))
        } else {
            (u8::MAX, 1008)
        };

        let this_error = value.abs_diff(result * outdiv);
        if error > this_error {
            ckd = this_ckd;
            bits = these_bits;
            error = this_error;
        }

        if error == 0 {
            break;
        }
    }

    (ckd, bits)
}

#[cfg(test)]
mod tests {
    use super::{compute_dead_time_value, Ckd};

    #[test]
    fn test_compute_dead_time_value() {
        struct TestRun {
            value: u16,
            ckd: Ckd,
            bits: u8,
        }

        let fn_results = [
            TestRun {
                value: 1,
                ckd: Ckd::DIV1,
                bits: 1,
            },
            TestRun {
                value: 125,
                ckd: Ckd::DIV1,
                bits: 125,
            },
            TestRun {
                value: 245,
                ckd: Ckd::DIV1,
                bits: 64 + 245 / 2,
            },
            TestRun {
                value: 255,
                ckd: Ckd::DIV2,
                bits: 127,
            },
            TestRun {
                value: 400,
                ckd: Ckd::DIV1,
                bits: 32 + (400u16 / 8) as u8,
            },
            TestRun {
                value: 600,
                ckd: Ckd::DIV4,
                bits: 64 + (600u16 / 8) as u8,
            },
        ];

        for test_run in fn_results {
            let (ckd, bits) = compute_dead_time_value(test_run.value);

            assert_eq!(ckd.to_bits(), test_run.ckd.to_bits());
            assert_eq!(bits, test_run.bits);
        }
    }
}
