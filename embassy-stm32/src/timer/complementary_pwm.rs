//! PWM driver with complementary (negated) output support.

use core::array;

use embassy_hal_internal::{into_ref, PeripheralRef};
use stm32_metapac::timer::vals::Ckd;

use super::low_level::{CountingMode, OutputPolarity, Timer};
use super::raw::{RawTimer, RawTimerPin};
use super::{
    Advanced4ChInstance, Advanced4ChTim, Ch1, Ch1N, Ch2, Ch2N, Ch3, Ch3N, Ch4, Ch4N, Channel, ChannelMarker,
    NChannelMarker, TimerPin,
};
use crate::gpio::{AfType, OutputType, Speed};
use crate::time::Hertz;
use crate::timer::low_level::OutputCompareMode;
use crate::Peripheral;

/// Builder for [`ComplementaryPwm`].
///
/// Create the builder using [`Builder::new()`], then attach output pins using methods on the
/// builder, and finally build the [`ComplementaryPwm`] driver using one of the `build` methods().
pub struct Builder<'d, T> {
    tim: PeripheralRef<'d, T>,
    channel_pins: [Option<RawTimerPin<'d>>; 4],
    n_channel_pins: [Option<RawTimerPin<'d>>; 4],
}

impl<'d, T: Advanced4ChInstance> Builder<'d, T> {
    /// Create a builder for the PWM driver using timer peripheral `tim`.
    pub fn new(tim: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(tim);
        Self {
            tim,
            channel_pins: array::from_fn(|_| None),
            n_channel_pins: array::from_fn(|_| None),
        }
    }

    /// Attach an output pin to the PWM driver.
    ///
    /// You may use convenience methods [`ch1_pin()`][Self::ch1_pin()] to `ch4_pin()` to aid type
    /// inference.
    pub fn pin<C: ChannelMarker>(
        mut self,
        pin: impl Peripheral<P = impl TimerPin<T, C>> + 'd,
        output_type: OutputType,
    ) -> Self {
        let pin = RawTimerPin::new(pin, AfType::output(output_type, Speed::VeryHigh));
        self.channel_pins[C::CHANNEL.index()] = Some(pin);
        self
    }

    /// Attach a complementary (negated) output pin to the PWM driver.
    ///
    /// You may use convenience methods [`ch1n_pin()`][Self::ch1n_pin()] to `ch4n_pin()` to aid type
    /// inference.
    pub fn n_pin<C: NChannelMarker>(
        mut self,
        pin: impl Peripheral<P = impl TimerPin<T, C>> + 'd,
        output_type: OutputType,
    ) -> Self {
        let pin = RawTimerPin::new(pin, AfType::output(output_type, Speed::VeryHigh));
        self.n_channel_pins[C::N_CHANNEL.index()] = Some(pin);
        self
    }
}

#[rustfmt::skip]
macro_rules! channel_impl {
    ($chx_pin:ident, $chxn_pin:ident, $channel:ident, $nchannel:ident) => {
        impl<'d, T: Advanced4ChInstance> Builder<'d, T> {
            #[doc = concat!(
                "Attach an output pin for channel ",
                stringify!($channel),
                " to the complementary PWM driver.\n\nSee [`pin()`][Self::pin()] for details.",
            )]
            pub fn $chx_pin(
                self,
                pin: impl Peripheral<P = impl TimerPin<T, $channel>> + 'd,
                output_type: OutputType,
            ) -> Self {
                self.pin::<$channel>(pin, output_type)
            }

            #[doc = concat!(
                "Attach a complementary output pin for channel ",
                stringify!($channel),
                " to the complementary PWM driver.\n\nSee [`n_pin()`][Self::pin()] for details.",
            )]
            pub fn $chxn_pin(
                self,
                pin: impl Peripheral<P = impl TimerPin<T, $nchannel>> + 'd,
                output_type: OutputType,
            ) -> Self {
                self.n_pin::<$nchannel>(pin, output_type)
            }
        }
    };
}
channel_impl!(ch1_pin, ch1n_pin, Ch1, Ch1N);
channel_impl!(ch2_pin, ch2n_pin, Ch2, Ch2N);
channel_impl!(ch3_pin, ch3n_pin, Ch3, Ch3N);
channel_impl!(ch4_pin, ch4n_pin, Ch4, Ch4N);

impl<'d, T: Advanced4ChInstance> Builder<'d, T>
where
    PeripheralRef<'d, T>: Peripheral<P = T> + 'd,
{
    /// Initialize the complementary PWM driver.
    pub fn build(self, freq: Hertz, counting_mode: CountingMode) -> ComplementaryPwm<'d> {
        let raw = RawTimer::new_advanced_4ch(self.tim);
        ComplementaryPwm::new_inner(raw, self.channel_pins, self.n_channel_pins, freq, counting_mode)
    }
}

/// PWM driver with support for standard and complementary (negated) outputs.
///
/// Use [`Builder`] to build an instance of this driver.
pub struct ComplementaryPwm<'d> {
    inner: Timer<'d, Advanced4ChTim>,
    _channel_pins: [Option<RawTimerPin<'d>>; 4],
    _n_channel_pins: [Option<RawTimerPin<'d>>; 4],
}

impl<'d> ComplementaryPwm<'d> {
    fn new_inner(
        raw: RawTimer<'d, Advanced4ChTim>,
        channel_pins: [Option<RawTimerPin<'d>>; 4],
        n_channel_pins: [Option<RawTimerPin<'d>>; 4],
        freq: Hertz,
        counting_mode: CountingMode,
    ) -> Self {
        let mut this = Self {
            inner: Timer::new(raw),
            _channel_pins: channel_pins,
            _n_channel_pins: n_channel_pins,
        };

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
        let multiplier = if self.inner.counting_mode().is_center_aligned() {
            2u8
        } else {
            1u8
        };
        self.inner.set_frequency(freq * multiplier);
    }

    /// Get max duty value.
    ///
    /// This value depends on the configured frequency and the timer's clock rate from RCC.
    pub fn max_duty(&self) -> u16 {
        self.inner.max_compare_value() as u16 + 1
    }

    /// Set the duty for a given channel.
    ///
    /// The value ranges from 0 for 0% duty, to [`max_duty`](Self::max_duty) for 100% duty, both included.
    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        assert!(duty <= self.max_duty());
        self.inner.set_compare_value(channel, duty as _)
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

impl<'d> embedded_hal_02::Pwm for ComplementaryPwm<'d> {
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
        self.inner.frequency()
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        self.inner.compare_value(channel) as u16
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.inner.max_compare_value() as u16 + 1
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        assert!(duty <= self.get_max_duty());
        self.inner.set_compare_value(channel, duty as u32)
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
