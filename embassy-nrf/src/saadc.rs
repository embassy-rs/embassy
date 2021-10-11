#![macro_use]

use core::marker::PhantomData;
use core::slice;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;
use embassy::interrupt::InterruptExt;
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;
use futures::Stream;

use crate::interrupt;
use crate::{pac, peripherals};

use pac::{saadc, SAADC};

pub use saadc::{
    ch::{
        config::{GAIN_A as Gain, REFSEL_A as Reference, RESP_A as Resistor, TACQ_A as Time},
        pselp::PSELP_A as InputChannel, // We treat the positive and negative channels with the same enum values to keep our type tidy and given they are the same
    },
    oversample::OVERSAMPLE_A as Oversample,
    resolution::VAL_A as Resolution,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {}

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
            resolution: Resolution::_14BIT,
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
            reference: Reference::VDD1_4,
            gain: Gain::GAIN1_6,
            resistor: Resistor::BYPASS,
            time: Time::_10US,
            p_channel: p_input.channel(),
            n_channel: Some(n_input.channel()),
            phantom: PhantomData,
        }
    }
}

/// One-shot SAADC.
pub struct OneShot<'d, const N: usize> {
    phantom: PhantomData<&'d mut peripherals::SAADC>,
}

impl<'d, const N: usize> OneShot<'d, N> {
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
    }

    fn regs() -> &'static saadc::RegisterBlock {
        unsafe { &*SAADC::ptr() }
    }

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
}

impl<'d, const N: usize> Drop for OneShot<'d, N> {
    fn drop(&mut self) {
        let r = Self::regs();
        r.enable.write(|w| w.enable().disabled());
    }
}

/// Continuous SAADC.
pub struct Continuous<'d, const N: usize> {
    phantom: PhantomData<&'d mut peripherals::SAADC>,
}

impl<'d, const N: usize> Continuous<'d, N> {
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

            // FIXME I think we need to move the IRQ setup into the sample method so that
            // we can provide it with the raw double buffers and fire off the next sample
            // request.
        }
    }

    fn regs() -> &'static saadc::RegisterBlock {
        unsafe { &*SAADC::ptr() }
    }

    /// Start sampling by providing a double buffer and a sample rate expressed in
    /// terms of 16MHz divided by it e.g. 16_000_000 / 80 = 200_000 samples per second.
    /// The sample rate ranges from 80..2047.
    pub fn sample<'b, const N0: usize>(
        &mut self,
        buf: &'b mut [[i16; N0]; 2],
        sample_rate: u16,
    ) -> ContinuousStream {
        let r = Self::regs();

        // Set up the DMA
        r.result
            .ptr
            .write(|w| unsafe { w.ptr().bits(buf[0].as_mut_ptr() as u32) });
        r.result
            .maxcnt
            .write(|w| unsafe { w.maxcnt().bits(N0 as _) });

        // Set up the sample rate
        r.samplerate.write(|w| {
            unsafe {
                w.cc().bits(sample_rate);
                w.mode().bit(true); // FIXME: only set this and the above sample rate when N == 1. If N != 1, we'll need to setup the timer and PPI.
            }
            w
        });

        // Reset and enable the end event
        r.events_end.reset();
        r.intenset.write(|w| w.end().set());

        // Don't reorder the ADC start event before the previous writes. Hopefully self
        // wouldn't happen anyway.
        compiler_fence(Ordering::SeqCst);

        r.tasks_start.write(|w| unsafe { w.bits(1) });
        r.tasks_sample.write(|w| unsafe { w.bits(1) });

        ContinuousStream {
            regs: Self::regs(),
            phantom: PhantomData,
        }
    }

    // FIXME: Provide an explicit stop method?
}

pub struct ContinuousStream<'b> {
    regs: &'static saadc::RegisterBlock,
    phantom: PhantomData<&'b ()>,
}

impl<'b> Stream for ContinuousStream<'b> {
    type Item = &'b [i16];

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let r = self.regs;

        WAKER.register(cx.waker());

        if r.events_end.read().bits() != 0 {
            r.events_end.reset();
            return Poll::Ready(Some(unsafe {
                slice::from_raw_parts(
                    self.regs.result.ptr.read().bits() as *const i16,
                    self.regs.result.amount.read().bits() as usize,
                )
            }));
        }

        if r.events_stopped.read().bits() != 0 {
            return Poll::Ready(None);
        }

        Poll::Pending
    }
}

impl<'d, const N: usize> Drop for Continuous<'d, N> {
    fn drop(&mut self) {
        let r = Self::regs();
        r.enable.write(|w| w.enable().disabled());
    }
}

/// An input that can be used as either or negative end of a ADC differential in the SAADC periperhal.
pub trait Input {
    fn channel(&self) -> InputChannel;
}

macro_rules! impl_saadc_input {
    ($pin:ident, $ch:ident) => {
        impl crate::saadc::Input for crate::peripherals::$pin {
            fn channel(&self) -> crate::saadc::InputChannel {
                crate::saadc::InputChannel::$ch
            }
        }
    };
}
