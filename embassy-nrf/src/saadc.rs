#![macro_use]

use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;
use embassy::interrupt::InterruptExt;
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

use crate::interrupt;
use crate::ppi::Task;
use crate::{pac, peripherals};

use pac::{saadc, SAADC};

// We treat the positive and negative channels with the same enum values to keep our type tidy and given they are the same
pub(crate) use saadc::ch::pselp::PSELP_A as InputChannel;

pub use saadc::{
    ch::config::{GAIN_A as Gain, REFSEL_A as Reference, RESP_A as Resistor, TACQ_A as Time},
    oversample::OVERSAMPLE_A as Oversample,
    resolution::VAL_A as Resolution,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {}

/// One-shot saadc. Continuous sample mode TODO.
pub struct Saadc<'d, const N: usize> {
    phantom: PhantomData<&'d mut peripherals::SAADC>,
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
    p_channel: InputChannel,
    /// An optional negative channel to sample
    n_channel: Option<InputChannel>,

    phantom: PhantomData<&'d ()>,
}

impl<'d> ChannelConfig<'d> {
    /// Default configuration for single ended channel sampling.
    pub fn single_ended(input: impl Unborrow<Target = impl Input> + 'd) -> Self {
        unborrow!(input);
        Self {
            reference: Reference::INTERNAL,
            gain: Gain::GAIN1_6,
            resistor: Resistor::BYPASS,
            time: Time::_10US,
            p_channel: input.channel(),
            n_channel: None,
            phantom: PhantomData,
        }
    }
    /// Default configuration for differential channel sampling.
    pub fn differential(
        p_input: impl Unborrow<Target = impl Input> + 'd,
        n_input: impl Unborrow<Target = impl Input> + 'd,
    ) -> Self {
        unborrow!(p_input, n_input);
        Self {
            reference: Reference::INTERNAL,
            gain: Gain::GAIN1_6,
            resistor: Resistor::BYPASS,
            time: Time::_10US,
            p_channel: p_input.channel(),
            n_channel: Some(n_input.channel()),
            phantom: PhantomData,
        }
    }
}

/// The state of a continuously running sampler. While it reflects
/// the progress of a sampler, it also signals what should be done
/// next. For example, if the sampler has stopped then the Saadc implementation
/// can then tear down its infrastructure.
#[derive(PartialEq)]
pub enum SamplerState {
    Sampled,
    Stopped,
}

impl<'d, const N: usize> Saadc<'d, N> {
    pub fn new(
        _saadc: impl Unborrow<Target = peripherals::SAADC> + 'd,
        irq: impl Unborrow<Target = interrupt::SAADC> + 'd,
        config: Config,
        channel_configs: [ChannelConfig; N],
    ) -> Self {
        unborrow!(irq);

        let r = unsafe { &*SAADC::ptr() };

        let Config {
            resolution,
            oversample,
        } = config;

        // Configure channels
        r.enable.write(|w| w.enable().enabled());
        r.resolution.write(|w| w.val().variant(resolution));
        r.oversample.write(|w| w.oversample().variant(oversample));

        for (i, cc) in channel_configs.iter().enumerate() {
            r.ch[i].pselp.write(|w| w.pselp().variant(cc.p_channel));
            if let Some(n_channel) = cc.n_channel {
                r.ch[i]
                    .pseln
                    .write(|w| unsafe { w.pseln().bits(n_channel as u8) });
            }
            r.ch[i].config.write(|w| {
                w.refsel().variant(cc.reference);
                w.gain().variant(cc.gain);
                w.tacq().variant(cc.time);
                if cc.n_channel.is_none() {
                    w.mode().se();
                } else {
                    w.mode().diff();
                }
                w.resp().variant(cc.resistor);
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

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self {
            phantom: PhantomData,
        }
    }

    fn on_interrupt(_ctx: *mut ()) {
        let r = Self::regs();

        if r.events_end.read().bits() != 0 {
            r.intenclr.write(|w| w.end().clear());
            WAKER.wake();
        }

        if r.events_started.read().bits() != 0 {
            r.intenclr.write(|w| w.started().clear());
            WAKER.wake();
        }
    }

    fn regs() -> &'static saadc::RegisterBlock {
        unsafe { &*SAADC::ptr() }
    }

    /// One shot sampling. The buffer must be the same size as the number of channels configured.
    pub async fn sample(&mut self, buf: &mut [i16; N]) {
        let r = Self::regs();

        // Set up the DMA
        r.result
            .ptr
            .write(|w| unsafe { w.ptr().bits(buf.as_mut_ptr() as u32) });
        r.result
            .maxcnt
            .write(|w| unsafe { w.maxcnt().bits(N as _) });

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
    }

    /// Continuous sampling with double buffers. The sample buffers generally
    /// should be a multiple of the number of channels configured.
    ///
    /// A task-driven approach to driving TASK_SAMPLE is expected. With a task
    /// driven approach, multiple channels can be used.
    ///
    /// A sampler closure is provided that receives the buffer of samples, noting
    /// that the size of this buffer can be less than the original buffer's size.
    /// A command is return from the closure that indicates whether the sampling
    /// should continue or stop.
    pub async fn run_task_sampler<S, const N0: usize>(
        &mut self,
        bufs: &mut [[[i16; N]; N0]; 2],
        sampler: S,
    ) where
        S: FnMut(&[[i16; N]]) -> SamplerState,
    {
        self.run_sampler(bufs, None, sampler).await;
    }

    async fn run_sampler<S, const N0: usize>(
        &mut self,
        bufs: &mut [[[i16; N]; N0]; 2],
        sample_rate: Option<u16>,
        mut sampler: S,
    ) where
        S: FnMut(&[[i16; N]]) -> SamplerState,
    {
        let r = Self::regs();

        // Establish mode and sample rate
        match sample_rate {
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
        r.result
            .maxcnt
            .write(|w| unsafe { w.maxcnt().bits((N0 * N) as _) });

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

        let mut current_buffer = 0;

        // Wait for events and complete when the sampler indicates it has had enough.
        poll_fn(|cx| {
            let r = Self::regs();

            WAKER.register(cx.waker());

            if r.events_end.read().bits() != 0 {
                compiler_fence(Ordering::SeqCst);

                r.events_end.reset();
                r.intenset.write(|w| w.end().set());

                if sampler(&bufs[current_buffer][0..r.result.amount.read().bits() as usize / N])
                    == SamplerState::Sampled
                {
                    let next_buffer = 1 - current_buffer;
                    current_buffer = next_buffer;
                    r.tasks_start.write(|w| unsafe { w.bits(1) });
                } else {
                    return Poll::Ready(());
                };
            }

            if r.events_started.read().bits() != 0 {
                r.events_started.reset();
                r.intenset.write(|w| w.started().set());

                let next_buffer = 1 - current_buffer;
                r.result
                    .ptr
                    .write(|w| unsafe { w.ptr().bits(bufs[next_buffer].as_mut_ptr() as u32) });
            }

            Poll::Pending
        })
        .await;
    }

    /// Return the sample task for use with PPI
    pub fn task_sample(&self) -> Task {
        let r = Self::regs();
        Task::from_reg(&r.tasks_sample)
    }
}

impl<'d> Saadc<'d, 1> {
    /// Continuous sampling on a single channel with double buffers. The sample
    /// buffers generally should be a multiple of the number of channels configured.
    ///
    /// The internal clock is to be used with a sample rate expressed as a divisor of
    /// 16MHz, ranging from 80..2047. For example, 1600 represnts a sample rate of 10KHz
    /// given 16_000_000 / 10_000_000 = 1600.
    ///
    /// A sampler closure is provided that receives the buffer of samples, noting
    /// that the size of this buffer can be less than the original buffer's size.
    /// A command is return from the closure that indicates whether the sampling
    /// should continue or stop.
    pub async fn run_timer_sampler<S, const N0: usize>(
        &mut self,
        bufs: &mut [[[i16; 1]; N0]; 2],
        sample_rate: u16,
        sampler: S,
    ) where
        S: FnMut(&[[i16; 1]]) -> SamplerState,
    {
        self.run_sampler(bufs, Some(sample_rate), sampler).await;
    }
}

impl<'d, const N: usize> Drop for Saadc<'d, N> {
    fn drop(&mut self) {
        let r = Self::regs();
        r.enable.write(|w| w.enable().disabled());
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Input {
        fn channel(&self) -> InputChannel;
    }
}

/// An input that can be used as either or negative end of a ADC differential in the SAADC periperhal.
pub trait Input: sealed::Input + Unborrow<Target = Self> {}

macro_rules! impl_saadc_input {
    ($pin:ident, $ch:ident) => {
        impl crate::saadc::sealed::Input for crate::peripherals::$pin {
            fn channel(&self) -> crate::saadc::InputChannel {
                crate::saadc::InputChannel::$ch
            }
        }
        impl crate::saadc::Input for crate::peripherals::$pin {}
    };
}
