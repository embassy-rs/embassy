//! Successive Approximation Analog-to-Digital Converter (SAADC) driver.

#![macro_use]

use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{impl_peripheral, into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use pac::{saadc, SAADC};
use saadc::ch::config::{GAIN_A, REFSEL_A, RESP_A, TACQ_A};
// We treat the positive and negative channels with the same enum values to keep our type tidy and given they are the same
pub(crate) use saadc::ch::pselp::PSELP_A as InputChannel;
use saadc::oversample::OVERSAMPLE_A;
use saadc::resolution::VAL_A;

use self::sealed::Input as _;
use crate::interrupt::InterruptExt;
use crate::ppi::{ConfigurableChannel, Event, Ppi, Task};
use crate::timer::{Frequency, Instance as TimerInstance, Timer};
use crate::{interrupt, pac, peripherals, Peripheral};

/// SAADC error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {}

/// Interrupt handler.
pub struct InterruptHandler {
    _private: (),
}

impl interrupt::typelevel::Handler<interrupt::typelevel::SAADC> for InterruptHandler {
    unsafe fn on_interrupt() {
        let r = unsafe { &*SAADC::ptr() };

        if r.events_calibratedone.read().bits() != 0 {
            r.intenclr.write(|w| w.calibratedone().clear());
            WAKER.wake();
        }

        if r.events_end.read().bits() != 0 {
            r.intenclr.write(|w| w.end().clear());
            WAKER.wake();
        }

        if r.events_started.read().bits() != 0 {
            r.intenclr.write(|w| w.started().clear());
            WAKER.wake();
        }
    }
}

static WAKER: AtomicWaker = AtomicWaker::new();

/// Used to configure the SAADC peripheral.
///
/// See the `Default` impl for suitable default values.
#[non_exhaustive]
pub struct Config {
    /// Output resolution in bits.
    pub resolution: Resolution,
    /// Average 2^`oversample` input samples before transferring the result into memory.
    pub oversample: Oversample,
}

impl Default for Config {
    /// Default configuration for single channel sampling.
    fn default() -> Self {
        Self {
            resolution: Resolution::_12BIT,
            oversample: Oversample::BYPASS,
        }
    }
}

/// Used to configure an individual SAADC peripheral channel.
///
/// See the `Default` impl for suitable default values.
#[non_exhaustive]
pub struct ChannelConfig<'d> {
    /// Reference voltage of the SAADC input.
    pub reference: Reference,
    /// Gain used to control the effective input range of the SAADC.
    pub gain: Gain,
    /// Positive channel resistor control.
    pub resistor: Resistor,
    /// Acquisition time in microseconds.
    pub time: Time,
    /// Positive channel to sample
    p_channel: PeripheralRef<'d, AnyInput>,
    /// An optional negative channel to sample
    n_channel: Option<PeripheralRef<'d, AnyInput>>,
}

impl<'d> ChannelConfig<'d> {
    /// Default configuration for single ended channel sampling.
    pub fn single_ended(input: impl Peripheral<P = impl Input> + 'd) -> Self {
        into_ref!(input);
        Self {
            reference: Reference::INTERNAL,
            gain: Gain::GAIN1_6,
            resistor: Resistor::BYPASS,
            time: Time::_10US,
            p_channel: input.map_into(),
            n_channel: None,
        }
    }
    /// Default configuration for differential channel sampling.
    pub fn differential(
        p_input: impl Peripheral<P = impl Input> + 'd,
        n_input: impl Peripheral<P = impl Input> + 'd,
    ) -> Self {
        into_ref!(p_input, n_input);
        Self {
            reference: Reference::INTERNAL,
            gain: Gain::GAIN1_6,
            resistor: Resistor::BYPASS,
            time: Time::_10US,
            p_channel: p_input.map_into(),
            n_channel: Some(n_input.map_into()),
        }
    }
}

/// Value returned by the SAADC callback, deciding what happens next.
#[derive(PartialEq)]
pub enum CallbackResult {
    /// The SAADC should keep sampling and calling the callback.
    Continue,
    /// The SAADC should stop sampling, and return.
    Stop,
}

/// One-shot and continuous SAADC.
pub struct Saadc<'d, const N: usize> {
    _p: PeripheralRef<'d, peripherals::SAADC>,
}

impl<'d, const N: usize> Saadc<'d, N> {
    /// Create a new SAADC driver.
    pub fn new(
        saadc: impl Peripheral<P = peripherals::SAADC> + 'd,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::SAADC, InterruptHandler> + 'd,
        config: Config,
        channel_configs: [ChannelConfig; N],
    ) -> Self {
        into_ref!(saadc);

        let r = unsafe { &*SAADC::ptr() };

        let Config { resolution, oversample } = config;

        // Configure channels
        r.enable.write(|w| w.enable().enabled());
        r.resolution.write(|w| w.val().variant(resolution.into()));
        r.oversample.write(|w| w.oversample().variant(oversample.into()));

        for (i, cc) in channel_configs.iter().enumerate() {
            r.ch[i].pselp.write(|w| w.pselp().variant(cc.p_channel.channel()));
            if let Some(n_channel) = &cc.n_channel {
                r.ch[i]
                    .pseln
                    .write(|w| unsafe { w.pseln().bits(n_channel.channel() as u8) });
            }
            r.ch[i].config.write(|w| {
                w.refsel().variant(cc.reference.into());
                w.gain().variant(cc.gain.into());
                w.tacq().variant(cc.time.into());
                if cc.n_channel.is_none() {
                    w.mode().se();
                } else {
                    w.mode().diff();
                }
                w.resp().variant(cc.resistor.into());
                w.resn().bypass();
                if !matches!(oversample, Oversample::BYPASS) {
                    w.burst().enabled();
                } else {
                    w.burst().disabled();
                }
                w
            });
        }

        // Disable all events interrupts
        r.intenclr.write(|w| unsafe { w.bits(0x003F_FFFF) });

        interrupt::SAADC.unpend();
        unsafe { interrupt::SAADC.enable() };

        Self { _p: saadc }
    }

    fn regs() -> &'static saadc::RegisterBlock {
        unsafe { &*SAADC::ptr() }
    }

    /// Perform SAADC calibration. Completes when done.
    pub async fn calibrate(&self) {
        let r = Self::regs();

        // Reset and enable the end event
        r.events_calibratedone.reset();
        r.intenset.write(|w| w.calibratedone().set());

        // Order is important
        compiler_fence(Ordering::SeqCst);

        r.tasks_calibrateoffset.write(|w| unsafe { w.bits(1) });

        // Wait for 'calibratedone' event.
        poll_fn(|cx| {
            let r = Self::regs();

            WAKER.register(cx.waker());

            if r.events_calibratedone.read().bits() != 0 {
                r.events_calibratedone.reset();
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;
    }

    /// One shot sampling. The buffer must be the same size as the number of channels configured.
    /// The sampling is stopped prior to returning in order to reduce power consumption (power
    /// consumption remains higher if sampling is not stopped explicitly). Cancellation will
    /// also cause the sampling to be stopped.
    pub async fn sample(&mut self, buf: &mut [i16; N]) {
        // In case the future is dropped, stop the task and wait for it to end.
        let on_drop = OnDrop::new(Self::stop_sampling_immediately);

        let r = Self::regs();

        // Set up the DMA
        r.result.ptr.write(|w| unsafe { w.ptr().bits(buf.as_mut_ptr() as u32) });
        r.result.maxcnt.write(|w| unsafe { w.maxcnt().bits(N as _) });

        // Reset and enable the end event
        r.events_end.reset();
        r.intenset.write(|w| w.end().set());

        // Don't reorder the ADC start event before the previous writes. Hopefully self
        // wouldn't happen anyway.
        compiler_fence(Ordering::SeqCst);

        r.tasks_start.write(|w| unsafe { w.bits(1) });
        r.tasks_sample.write(|w| unsafe { w.bits(1) });

        // Wait for 'end' event.
        poll_fn(|cx| {
            let r = Self::regs();

            WAKER.register(cx.waker());

            if r.events_end.read().bits() != 0 {
                r.events_end.reset();
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;

        drop(on_drop);
    }

    /// Continuous sampling with double buffers.
    ///
    /// A TIMER and two PPI peripherals are passed in so that precise sampling
    /// can be attained. The sampling interval is expressed by selecting a
    /// timer clock frequency to use along with a counter threshold to be reached.
    /// For example, 1KHz can be achieved using a frequency of 1MHz and a counter
    /// threshold of 1000.
    ///
    /// A sampler closure is provided that receives the buffer of samples, noting
    /// that the size of this buffer can be less than the original buffer's size.
    /// A command is return from the closure that indicates whether the sampling
    /// should continue or stop.
    ///
    /// NOTE: The time spent within the callback supplied should not exceed the time
    /// taken to acquire the samples into a single buffer. You should measure the
    /// time taken by the callback and set the sample buffer size accordingly.
    /// Exceeding this time can lead to samples becoming dropped.
    ///
    /// The sampling is stopped prior to returning in order to reduce power consumption (power
    /// consumption remains higher if sampling is not stopped explicitly), and to
    /// free the buffers from being used by the peripheral. Cancellation will
    /// also cause the sampling to be stopped.

    pub async fn run_task_sampler<F, T: TimerInstance, const N0: usize>(
        &mut self,
        timer: &mut T,
        ppi_ch1: &mut impl ConfigurableChannel,
        ppi_ch2: &mut impl ConfigurableChannel,
        frequency: Frequency,
        sample_counter: u32,
        bufs: &mut [[[i16; N]; N0]; 2],
        callback: F,
    ) where
        F: FnMut(&[[i16; N]]) -> CallbackResult,
    {
        let r = Self::regs();

        // We want the task start to effectively short with the last one ending so
        // we don't miss any samples. It'd be great for the SAADC to offer a SHORTS
        // register instead, but it doesn't, so we must use PPI.
        let mut start_ppi =
            Ppi::new_one_to_one(ppi_ch1, Event::from_reg(&r.events_end), Task::from_reg(&r.tasks_start));
        start_ppi.enable();

        let timer = Timer::new(timer);
        timer.set_frequency(frequency);
        timer.cc(0).write(sample_counter);
        timer.cc(0).short_compare_clear();

        let timer_cc = timer.cc(0);

        let mut sample_ppi = Ppi::new_one_to_one(ppi_ch2, timer_cc.event_compare(), Task::from_reg(&r.tasks_sample));

        timer.start();

        self.run_sampler(
            bufs,
            None,
            || {
                sample_ppi.enable();
            },
            callback,
        )
        .await;
    }

    async fn run_sampler<I, F, const N0: usize>(
        &mut self,
        bufs: &mut [[[i16; N]; N0]; 2],
        sample_rate_divisor: Option<u16>,
        mut init: I,
        mut callback: F,
    ) where
        I: FnMut(),
        F: FnMut(&[[i16; N]]) -> CallbackResult,
    {
        // In case the future is dropped, stop the task and wait for it to end.
        let on_drop = OnDrop::new(Self::stop_sampling_immediately);

        let r = Self::regs();

        // Establish mode and sample rate
        match sample_rate_divisor {
            Some(sr) => {
                r.samplerate.write(|w| unsafe {
                    w.cc().bits(sr);
                    w.mode().timers();
                    w
                });
                r.tasks_sample.write(|w| unsafe { w.bits(1) }); // Need to kick-start the internal timer
            }
            None => r.samplerate.write(|w| unsafe {
                w.cc().bits(0);
                w.mode().task();
                w
            }),
        }

        // Set up the initial DMA
        r.result
            .ptr
            .write(|w| unsafe { w.ptr().bits(bufs[0].as_mut_ptr() as u32) });
        r.result.maxcnt.write(|w| unsafe { w.maxcnt().bits((N0 * N) as _) });

        // Reset and enable the events
        r.events_end.reset();
        r.events_started.reset();
        r.intenset.write(|w| {
            w.end().set();
            w.started().set();
            w
        });

        // Don't reorder the ADC start event before the previous writes. Hopefully self
        // wouldn't happen anyway.
        compiler_fence(Ordering::SeqCst);

        r.tasks_start.write(|w| unsafe { w.bits(1) });

        let mut inited = false;

        let mut current_buffer = 0;

        // Wait for events and complete when the sampler indicates it has had enough.
        let r = poll_fn(|cx| {
            let r = Self::regs();

            WAKER.register(cx.waker());

            if r.events_end.read().bits() != 0 {
                compiler_fence(Ordering::SeqCst);

                r.events_end.reset();
                r.intenset.write(|w| w.end().set());

                match callback(&bufs[current_buffer]) {
                    CallbackResult::Continue => {
                        let next_buffer = 1 - current_buffer;
                        current_buffer = next_buffer;
                    }
                    CallbackResult::Stop => {
                        return Poll::Ready(());
                    }
                }
            }

            if r.events_started.read().bits() != 0 {
                r.events_started.reset();
                r.intenset.write(|w| w.started().set());

                if !inited {
                    init();
                    inited = true;
                }

                let next_buffer = 1 - current_buffer;
                r.result
                    .ptr
                    .write(|w| unsafe { w.ptr().bits(bufs[next_buffer].as_mut_ptr() as u32) });
            }

            Poll::Pending
        })
        .await;

        drop(on_drop);

        r
    }

    // Stop sampling and wait for it to stop in a blocking fashion
    fn stop_sampling_immediately() {
        let r = Self::regs();

        compiler_fence(Ordering::SeqCst);

        r.events_stopped.reset();
        r.tasks_stop.write(|w| unsafe { w.bits(1) });

        while r.events_stopped.read().bits() == 0 {}
        r.events_stopped.reset();
    }
}

impl<'d> Saadc<'d, 1> {
    /// Continuous sampling on a single channel with double buffers.
    ///
    /// The internal clock is to be used with a sample rate expressed as a divisor of
    /// 16MHz, ranging from 80..2047. For example, 1600 represents a sample rate of 10KHz
    /// given 16_000_000 / 10_000_000 = 1600.
    ///
    /// A sampler closure is provided that receives the buffer of samples, noting
    /// that the size of this buffer can be less than the original buffer's size.
    /// A command is return from the closure that indicates whether the sampling
    /// should continue or stop.
    pub async fn run_timer_sampler<I, S, const N0: usize>(
        &mut self,
        bufs: &mut [[[i16; 1]; N0]; 2],
        sample_rate_divisor: u16,
        sampler: S,
    ) where
        S: FnMut(&[[i16; 1]]) -> CallbackResult,
    {
        self.run_sampler(bufs, Some(sample_rate_divisor), || {}, sampler).await;
    }
}

impl<'d, const N: usize> Drop for Saadc<'d, N> {
    fn drop(&mut self) {
        let r = Self::regs();
        r.enable.write(|w| w.enable().disabled());
    }
}

impl From<Gain> for GAIN_A {
    fn from(gain: Gain) -> Self {
        match gain {
            Gain::GAIN1_6 => GAIN_A::GAIN1_6,
            Gain::GAIN1_5 => GAIN_A::GAIN1_5,
            Gain::GAIN1_4 => GAIN_A::GAIN1_4,
            Gain::GAIN1_3 => GAIN_A::GAIN1_3,
            Gain::GAIN1_2 => GAIN_A::GAIN1_2,
            Gain::GAIN1 => GAIN_A::GAIN1,
            Gain::GAIN2 => GAIN_A::GAIN2,
            Gain::GAIN4 => GAIN_A::GAIN4,
        }
    }
}

/// Gain control
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Gain {
    /// 1/6
    GAIN1_6 = 0,
    /// 1/5
    GAIN1_5 = 1,
    /// 1/4
    GAIN1_4 = 2,
    /// 1/3
    GAIN1_3 = 3,
    /// 1/2
    GAIN1_2 = 4,
    /// 1
    GAIN1 = 5,
    /// 2
    GAIN2 = 6,
    /// 4
    GAIN4 = 7,
}

impl From<Reference> for REFSEL_A {
    fn from(reference: Reference) -> Self {
        match reference {
            Reference::INTERNAL => REFSEL_A::INTERNAL,
            Reference::VDD1_4 => REFSEL_A::VDD1_4,
        }
    }
}

/// Reference control
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Reference {
    /// Internal reference (0.6 V)
    INTERNAL = 0,
    /// VDD/4 as reference
    VDD1_4 = 1,
}

impl From<Resistor> for RESP_A {
    fn from(resistor: Resistor) -> Self {
        match resistor {
            Resistor::BYPASS => RESP_A::BYPASS,
            Resistor::PULLDOWN => RESP_A::PULLDOWN,
            Resistor::PULLUP => RESP_A::PULLUP,
            Resistor::VDD1_2 => RESP_A::VDD1_2,
        }
    }
}

/// Positive channel resistor control
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Resistor {
    /// Bypass resistor ladder
    BYPASS = 0,
    /// Pull-down to GND
    PULLDOWN = 1,
    /// Pull-up to VDD
    PULLUP = 2,
    /// Set input at VDD/2
    VDD1_2 = 3,
}

impl From<Time> for TACQ_A {
    fn from(time: Time) -> Self {
        match time {
            Time::_3US => TACQ_A::_3US,
            Time::_5US => TACQ_A::_5US,
            Time::_10US => TACQ_A::_10US,
            Time::_15US => TACQ_A::_15US,
            Time::_20US => TACQ_A::_20US,
            Time::_40US => TACQ_A::_40US,
        }
    }
}

/// Acquisition time, the time the SAADC uses to sample the input voltage
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Time {
    /// 3 us
    _3US = 0,
    ///  5 us
    _5US = 1,
    /// 10 us
    _10US = 2,
    /// 15 us
    _15US = 3,
    /// 20 us
    _20US = 4,
    /// 40 us
    _40US = 5,
}

impl From<Oversample> for OVERSAMPLE_A {
    fn from(oversample: Oversample) -> Self {
        match oversample {
            Oversample::BYPASS => OVERSAMPLE_A::BYPASS,
            Oversample::OVER2X => OVERSAMPLE_A::OVER2X,
            Oversample::OVER4X => OVERSAMPLE_A::OVER4X,
            Oversample::OVER8X => OVERSAMPLE_A::OVER8X,
            Oversample::OVER16X => OVERSAMPLE_A::OVER16X,
            Oversample::OVER32X => OVERSAMPLE_A::OVER32X,
            Oversample::OVER64X => OVERSAMPLE_A::OVER64X,
            Oversample::OVER128X => OVERSAMPLE_A::OVER128X,
            Oversample::OVER256X => OVERSAMPLE_A::OVER256X,
        }
    }
}

/// Oversample control
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Oversample {
    /// Bypass oversampling
    BYPASS = 0,
    /// Oversample 2x
    OVER2X = 1,
    /// Oversample 4x
    OVER4X = 2,
    /// Oversample 8x
    OVER8X = 3,
    /// Oversample 16x
    OVER16X = 4,
    /// Oversample 32x
    OVER32X = 5,
    /// Oversample 64x
    OVER64X = 6,
    /// Oversample 128x
    OVER128X = 7,
    /// Oversample 256x
    OVER256X = 8,
}

impl From<Resolution> for VAL_A {
    fn from(resolution: Resolution) -> Self {
        match resolution {
            Resolution::_8BIT => VAL_A::_8BIT,
            Resolution::_10BIT => VAL_A::_10BIT,
            Resolution::_12BIT => VAL_A::_12BIT,
            Resolution::_14BIT => VAL_A::_14BIT,
        }
    }
}

/// Set the resolution
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Resolution {
    /// 8 bits
    _8BIT = 0,
    /// 10 bits
    _10BIT = 1,
    /// 12 bits
    _12BIT = 2,
    /// 14 bits
    _14BIT = 3,
}

pub(crate) mod sealed {
    use super::*;

    pub trait Input {
        fn channel(&self) -> InputChannel;
    }
}

/// An input that can be used as either or negative end of a ADC differential in the SAADC periperhal.
pub trait Input: sealed::Input + Into<AnyInput> + Peripheral<P = Self> + Sized + 'static {
    /// Convert this SAADC input to a type-erased `AnyInput`.
    ///
    /// This allows using several inputs  in situations that might require
    /// them to be the same type, like putting them in an array.
    fn degrade_saadc(self) -> AnyInput {
        AnyInput {
            channel: self.channel(),
        }
    }
}

/// A type-erased SAADC input.
///
/// This allows using several inputs  in situations that might require
/// them to be the same type, like putting them in an array.
pub struct AnyInput {
    channel: InputChannel,
}

impl_peripheral!(AnyInput);

impl sealed::Input for AnyInput {
    fn channel(&self) -> InputChannel {
        self.channel
    }
}

impl Input for AnyInput {}

macro_rules! impl_saadc_input {
    ($pin:ident, $ch:ident) => {
        impl_saadc_input!(@local, crate::peripherals::$pin, $ch);
    };
    (@local, $pin:ty, $ch:ident) => {
        impl crate::saadc::sealed::Input for $pin {
            fn channel(&self) -> crate::saadc::InputChannel {
                crate::saadc::InputChannel::$ch
            }
        }
        impl crate::saadc::Input for $pin {}

        impl From<$pin> for crate::saadc::AnyInput {
            fn from(val: $pin) -> Self {
                crate::saadc::Input::degrade_saadc(val)
            }
        }
    };
}

/// A dummy `Input` pin implementation for SAADC peripheral sampling from the
/// internal voltage.
pub struct VddInput;

impl_peripheral!(VddInput);
#[cfg(not(feature = "_nrf9160"))]
impl_saadc_input!(@local, VddInput, VDD);
#[cfg(feature = "_nrf9160")]
impl_saadc_input!(@local, VddInput, VDDGPIO);

/// A dummy `Input` pin implementation for SAADC peripheral sampling from the
/// VDDH / 5 voltage.
#[cfg(any(feature = "_nrf5340-app", feature = "nrf52833", feature = "nrf52840"))]
pub struct VddhDiv5Input;

#[cfg(any(feature = "_nrf5340-app", feature = "nrf52833", feature = "nrf52840"))]
impl_peripheral!(VddhDiv5Input);

#[cfg(any(feature = "_nrf5340-app", feature = "nrf52833", feature = "nrf52840"))]
impl_saadc_input!(@local, VddhDiv5Input, VDDHDIV5);
