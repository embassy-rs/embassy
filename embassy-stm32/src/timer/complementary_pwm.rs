//! PWM driver with complementary output support.

use core::marker::PhantomData;

use super::low_level::{CountingMode, OutputPolarity, RoundTo, Timer};
use super::simple_pwm::PwmPin;
use super::{AdvancedInstance4Channel, Ch1, Ch2, Ch3, Ch4, Channel, TimerComplementaryPin};
use crate::Peri;
use crate::dma::word::Word;
use crate::gpio::{AfType, AnyPin, OutputType};
pub use crate::pac::timer::vals::{Ccds, Ckd, Mms2, Ossi, Ossr};
use crate::time::Hertz;
use crate::timer::TimerChannel;
use crate::timer::low_level::OutputCompareMode;
use crate::timer::simple_pwm::PwmPinConfig;

/// Complementary PWM pin wrapper.
///
/// This wraps a pin to make it usable with PWM.
pub struct ComplementaryPwmPin<'d, T, C, #[cfg(afio)] A> {
    #[allow(unused)]
    pin: Peri<'d, AnyPin>,
    phantom: PhantomData<if_afio!((T, C, A))>,
}

impl<'d, T: AdvancedInstance4Channel, C: TimerChannel, #[cfg(afio)] A> if_afio!(ComplementaryPwmPin<'d, T, C, A>) {
    /// Create a new  complementary PWM pin instance.
    pub fn new(pin: Peri<'d, if_afio!(impl TimerComplementaryPin<T, C, A>)>, output_type: OutputType) -> Self {
        critical_section::with(|_| {
            pin.set_low();
            set_as_af!(pin, AfType::output(output_type, crate::gpio::Speed::VeryHigh));
        });
        ComplementaryPwmPin {
            pin: pin.into(),
            phantom: PhantomData,
        }
    }

    /// Create a new PWM pin instance with config.
    pub fn new_with_config(
        pin: Peri<'d, if_afio!(impl TimerComplementaryPin<T, C, A>)>,
        pin_config: PwmPinConfig,
    ) -> Self {
        critical_section::with(|_| {
            pin.set_low();
            #[cfg(gpio_v1)]
            set_as_af!(pin, AfType::output(pin_config.output_type, pin_config.speed));
            #[cfg(gpio_v2)]
            pin.set_as_af(
                pin.af_num(),
                AfType::output_pull(pin_config.output_type, pin_config.speed, pin_config.pull),
            );
        });
        ComplementaryPwmPin {
            pin: pin.into(),
            phantom: PhantomData,
        }
    }
}

/// PWM driver with support for standard and complementary outputs.
pub struct ComplementaryPwm<'d, T: AdvancedInstance4Channel> {
    inner: Timer<'d, T>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Determines which outputs are active when PWM is in idle mode
pub enum IdlePolarity {
    /// Normal channels are forced active and complementary channels are forced inactive
    OisActive,
    /// Normal channels are forced inactive and complementary channels are forced active
    OisnActive,
}

impl<'d, T: AdvancedInstance4Channel> ComplementaryPwm<'d, T> {
    /// Create a new complementary PWM driver.
    #[allow(clippy::too_many_arguments, unused)]
    pub fn new<#[cfg(afio)] A>(
        tim: Peri<'d, T>,
        ch1: Option<if_afio!(PwmPin<'d, T, Ch1, A>)>,
        ch1n: Option<if_afio!(ComplementaryPwmPin<'d, T, Ch1, A>)>,
        ch2: Option<if_afio!(PwmPin<'d, T, Ch2, A>)>,
        ch2n: Option<if_afio!(ComplementaryPwmPin<'d, T, Ch2, A>)>,
        ch3: Option<if_afio!(PwmPin<'d, T, Ch3, A>)>,
        ch3n: Option<if_afio!(ComplementaryPwmPin<'d, T, Ch3, A>)>,
        ch4: Option<if_afio!(PwmPin<'d, T, Ch4, A>)>,
        ch4n: Option<if_afio!(ComplementaryPwmPin<'d, T, Ch4, A>)>,
        freq: Hertz,
        counting_mode: CountingMode,
    ) -> Self {
        Self::new_inner(tim, freq, counting_mode)
    }

    fn new_inner(tim: Peri<'d, T>, freq: Hertz, counting_mode: CountingMode) -> Self {
        let mut this = Self { inner: Timer::new(tim) };

        this.inner.set_counting_mode(counting_mode);
        this.set_frequency(freq);
        this.inner.enable_outputs();

        [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4]
            .iter()
            .for_each(|&channel| {
                this.inner.set_output_compare_mode(channel, OutputCompareMode::PwmMode1);
                this.inner.set_output_compare_preload(channel, true);
            });
        this.inner.set_autoreload_preload(true);

        // Generate update event so pre-load registers are written to the shadow registers
        this.inner.generate_update_event();
        this.inner.start();

        this
    }

    /// Sets the idle output state for the given channels.
    pub fn set_output_idle_state(&mut self, channels: &[Channel], polarity: IdlePolarity) {
        let ois_active = matches!(polarity, IdlePolarity::OisActive);
        for &channel in channels {
            self.inner.set_ois(channel, ois_active);
            self.inner.set_oisn(channel, !ois_active);
        }
    }

    /// Set state of OSSI-bit in BDTR register
    pub fn set_off_state_selection_idle(&mut self, val: Ossi) {
        self.inner.set_ossi(val);
    }

    /// Get state of OSSI-bit in BDTR register
    pub fn get_off_state_selection_idle(&self) -> Ossi {
        self.inner.get_ossi()
    }

    /// Set state of OSSR-bit in BDTR register
    pub fn set_off_state_selection_run(&mut self, val: Ossr) {
        self.inner.set_ossr(val);
    }

    /// Get state of OSSR-bit in BDTR register
    pub fn get_off_state_selection_run(&self) -> Ossr {
        self.inner.get_ossr()
    }

    /// Trigger break input from software
    pub fn trigger_software_break(&mut self, n: usize) {
        self.inner.trigger_software_break(n);
    }

    /// Set Master Output Enable
    pub fn set_master_output_enable(&mut self, enable: bool) {
        self.inner.set_moe(enable);
    }

    /// Get Master Output Enable
    pub fn get_master_output_enable(&self) -> bool {
        self.inner.get_moe()
    }

    /// Set Master Slave Mode 2
    pub fn set_mms2(&mut self, mms2: Mms2) {
        self.inner.set_mms2_selection(mms2);
    }

    /// Set Repetition Counter
    pub fn set_repetition_counter(&mut self, val: u16) {
        self.inner.set_repetition_counter(val);
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
    /// The actual frequency may differ from the requested value due to hardware
    /// limitations. The timer will round towards a slower (longer) period.
    ///
    /// Note: that the frequency will not be applied in the timer until an update event
    /// occurs.
    pub fn set_frequency(&mut self, freq: Hertz) {
        let multiplier = if self.inner.get_counting_mode().is_center_aligned() {
            2u64
        } else {
            1u64
        };
        let timer_f = T::frequency().0 as u64;
        let clocks = timer_f / (freq.0 as u64 * multiplier);
        self.inner.set_period_clocks_internal(clocks, RoundTo::Slower, 16);
    }

    /// Set the PWM period in milliseconds.
    ///
    /// The actual period may differ from the requested value due to hardware
    /// limitations. The timer will round towards a slower (longer) period.
    ///
    /// Note: that the period will not be applied in the timer until an update event
    /// occurs.
    pub fn set_period_ms(&mut self, ms: u32) {
        let timer_f = T::frequency().0 as u64;
        let mut clocks = timer_f * ms as u64 / 1_000;
        if self.inner.get_counting_mode().is_center_aligned() {
            clocks = clocks / 2;
        }
        self.inner.set_period_clocks(clocks, RoundTo::Slower);
    }

    /// Set the PWM period in microseconds.
    ///
    /// The actual period may differ from the requested value due to hardware
    /// limitations. The timer will round towards a slower (longer) period.
    ///
    /// Note: that the period will not be applied in the timer until an update event
    /// occurs.
    pub fn set_period_us(&mut self, us: u32) {
        let timer_f = T::frequency().0 as u64;
        let mut clocks = timer_f * us as u64 / 1_000_000;
        if self.inner.get_counting_mode().is_center_aligned() {
            clocks = clocks / 2;
        }
        self.inner.set_period_clocks(clocks, RoundTo::Slower);
    }

    /// Set the PWM period in seconds.
    ///
    /// The actual period may differ from the requested value due to hardware
    /// limitations. The timer will round towards a slower (longer) period.
    ///
    /// Note: that the period will not be applied in the timer until an update event
    /// occurs.
    pub fn set_period_secs(&mut self, secs: u32) {
        let timer_f = T::frequency().0 as u64;
        let mut clocks = timer_f * secs as u64;
        if self.inner.get_counting_mode().is_center_aligned() {
            clocks = clocks / 2;
        }
        self.inner.set_period_clocks(clocks, RoundTo::Slower);
    }

    /// Set the PWM period using an `embassy_time::Duration`.
    ///
    /// The actual period may differ from the requested value due to hardware
    /// limitations. The timer will round towards a slower (longer) period.
    ///
    /// Note: that the period will not be applied in the timer until an update event
    /// occurs.
    #[cfg(feature = "time")]
    pub fn set_period(&mut self, period: embassy_time::Duration) {
        let timer_f = T::frequency().0 as u64;
        let mut clocks = timer_f * period.as_ticks() / embassy_time::TICK_HZ;
        if self.inner.get_counting_mode().is_center_aligned() {
            clocks = clocks / 2;
        }
        self.inner.set_period_clocks(clocks, RoundTo::Slower);
    }

    /// Get max duty value.
    ///
    /// This value depends on the configured frequency and the timer's clock rate from RCC.
    pub fn get_max_duty(&self) -> u32 {
        if self.inner.get_counting_mode().is_center_aligned() {
            self.inner.get_max_compare_value().into()
        } else {
            self.inner.get_max_compare_value().into() + 1
        }
    }

    /// Set the duty for a given channel.
    ///
    /// The value ranges from 0 for 0% duty, to [`get_max_duty`](Self::get_max_duty) for 100% duty, both included.
    pub fn set_duty(&mut self, channel: Channel, duty: u32) {
        assert!(duty <= self.get_max_duty());
        self.inner.set_compare_value(channel, unwrap!(duty.try_into()))
    }

    /// Set the output polarity for a given channel.
    pub fn set_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
        self.inner.set_output_polarity(channel, polarity);
        self.inner.set_complementary_output_polarity(channel, polarity);
    }

    /// Set the main output polarity for a given channel.
    pub fn set_main_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
        self.inner.set_output_polarity(channel, polarity);
    }

    /// Set the complementary output polarity for a given channel.
    pub fn set_complementary_polarity(&mut self, channel: Channel, polarity: OutputPolarity) {
        self.inner.set_complementary_output_polarity(channel, polarity);
    }

    /// Set the dead time as a proportion of max_duty
    pub fn set_dead_time(&mut self, value: u16) {
        let (ckd, value) = compute_dead_time_value(value);

        self.inner.set_dead_time_clock_division(ckd);
        self.inner.set_dead_time_value(value);
    }

    /// Generate a sequence of PWM waveform
    ///
    /// Note:
    /// The DMA channel provided does not need to correspond to the requested channel.
    pub async fn waveform<
        C: TimerChannel,
        W: Word + Into<T::Word>,
        D: super::Dma<T, C> + crate::dma::ChannelInterrupt,
    >(
        &mut self,
        dma: Peri<'_, D>,
        _irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + '_,
        channel: Channel,
        duty: &[W],
    ) {
        crate::dma::assert_dma_binding(&*dma, &_irq);
        self.inner.enable_channel(channel, true);
        self.inner.enable_channel(C::CHANNEL, true);
        self.inner.clamp_compare_value::<W>(channel);
        self.inner.set_cc_dma_selection(Ccds::ON_UPDATE);
        self.inner.set_cc_dma_enable_state(C::CHANNEL, true);
        self.inner.setup_channel_update_dma(dma, channel, duty).await;
        self.inner.set_cc_dma_enable_state(C::CHANNEL, false);
    }

    /// Generate a sequence of PWM waveform
    ///
    /// Note:
    /// you will need to provide corresponding TIMx_UP DMA channel to use this method.
    pub async fn waveform_up<W: Word + Into<T::Word>, D: super::UpDma<T> + crate::dma::ChannelInterrupt>(
        &mut self,
        dma: Peri<'_, D>,
        _irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + '_,
        channel: Channel,
        duty: &[W],
    ) {
        crate::dma::assert_dma_binding(&*dma, &_irq);
        self.inner.enable_channel(channel, true);
        self.inner.clamp_compare_value::<W>(channel);
        self.inner.enable_update_dma(true);
        self.inner.setup_update_dma(dma, channel, duty).await;
        self.inner.enable_update_dma(false);
    }

    /// Generate a multichannel sequence of PWM waveforms using DMA triggered by timer update events.
    ///
    /// This method utilizes the timer's DMA burst transfer capability to update multiple CCRx registers
    /// in sequence on each update event (UEV). The data is written via the DMAR register using the
    /// DMA base address (DBA) and burst length (DBL) configured in the DCR register.
    ///
    /// The `duty` buffer must be structured as a flattened 2D array in row-major order, where each row
    /// represents a single update event and each column corresponds to a specific timer channel (starting
    /// from `starting_channel` up to and including `ending_channel`).
    ///
    /// For example, if using channels 1 through 4, a buffer of 4 update steps might look like:
    ///
    /// ```rust,ignore
    /// let dma_buf: [u16; 16] = [
    ///     ch1_duty_1, ch2_duty_1, ch3_duty_1, ch4_duty_1, // update 1
    ///     ch1_duty_2, ch2_duty_2, ch3_duty_2, ch4_duty_2, // update 2
    ///     ch1_duty_3, ch2_duty_3, ch3_duty_3, ch4_duty_3, // update 3
    ///     ch1_duty_4, ch2_duty_4, ch3_duty_4, ch4_duty_4, // update 4
    /// ];
    /// ```
    ///
    /// Each group of `N` values (where `N` is number of channels) is transferred on one update event,
    /// updating the duty cycles of all selected channels simultaneously.
    ///
    /// Note:
    /// You will need to provide corresponding `TIMx_UP` DMA channel to use this method.
    /// Also be aware that embassy timers use one of timers internally. It is possible to
    /// switch this timer by using `time-driver-timX` feature.
    ///
    pub async fn waveform_up_multi_channel<
        W: Word + Into<T::Word>,
        D: super::UpDma<T> + crate::dma::ChannelInterrupt,
    >(
        &mut self,
        dma: Peri<'_, D>,
        _irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + '_,
        starting_channel: Channel,
        ending_channel: Channel,
        duty: &[W],
    ) {
        crate::dma::assert_dma_binding(&*dma, &_irq);
        [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4]
            .iter()
            .filter(|ch| ch.index() >= starting_channel.index())
            .filter(|ch| ch.index() <= ending_channel.index())
            .for_each(|ch| {
                self.inner.enable_channel(*ch, true);
                self.inner.clamp_compare_value::<W>(*ch);
            });
        self.inner.enable_update_dma(true);
        self.inner
            .setup_update_dma_burst(dma, starting_channel, ending_channel, duty)
            .await;
        self.inner.enable_update_dma(false);
    }
}

impl<'d, T: AdvancedInstance4Channel> embedded_hal_02::Pwm for ComplementaryPwm<'d, T> {
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
        unwrap!(self.inner.get_compare_value(channel).try_into())
    }

    fn get_max_duty(&self) -> Self::Duty {
        if self.inner.get_counting_mode().is_center_aligned() {
            unwrap!(self.inner.get_max_compare_value().try_into())
        } else {
            unwrap!(self.inner.get_max_compare_value().try_into()) + 1
        }
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        assert!(duty <= unwrap!(self.get_max_duty().try_into()));
        self.inner.set_compare_value(channel, unwrap!(duty.try_into()))
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        self.inner.set_frequency(period.into(), RoundTo::Slower);
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
            ((64 + (target / 2) as u8) | 128, (target - target % 2))
        } else if target < 508 {
            ((32 + (target / 8) as u8) | 192, (target - target % 8))
        } else if target < 1008 {
            ((32 + (target / 16) as u8) | 224, (target - target % 16))
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
    use super::{Ckd, compute_dead_time_value};

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
                bits: 210,
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
