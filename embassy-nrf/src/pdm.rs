#![macro_use]

use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_common::{into_ref, PeripheralRef};
use embassy_util::waitqueue::AtomicWaker;
use futures::future::poll_fn;
use pac::{pdm, PDM};
use pdm::mode::{EDGE_A, OPERATION_A};
use fixed::types::I7F1;

use crate::interrupt::InterruptExt;
use crate::gpio::Pin as GpioPin;
use crate::{interrupt, pac, peripherals, Peripheral};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {}

/// One-shot and continuous PDM.
pub struct Pdm<'d> {
    _p: PeripheralRef<'d, peripherals::PDM>,
}

static WAKER: AtomicWaker = AtomicWaker::new();

/// Used to configure the PDM peripheral.
///
/// See the `Default` impl for suitable default values.
#[non_exhaustive]
pub struct Config {
    /// Clock
    /// Clock ratio
    /// Channels
    pub channels: Channels,
    /// Edge to sample on
    pub left_edge: Edge,
    /// Gain left in dB
    pub gain_left: I7F1,
    /// Gain right in dB
    pub gain_right: I7F1,
}

impl Default for Config {
    /// Default configuration for single channel sampling.
    fn default() -> Self {
        Self {
            channels: Channels::Stereo,
            left_edge: Edge::FallingEdge,
            gain_left: I7F1::ZERO,
            gain_right: I7F1::ZERO,
        }
    }
}

/// The state of a continuously running sampler. While it reflects
/// the progress of a sampler, it also signals what should be done
/// next. For example, if the sampler has stopped then the Pdm implementation
/// can then tear down its infrastructure.
#[derive(PartialEq)]
pub enum SamplerState {
    Sampled,
    Stopped,
}

impl<'d> Pdm<'d> {
    pub fn new(
        pdm: impl Peripheral<P = peripherals::PDM> + 'd,
        irq: impl Peripheral<P = interrupt::PDM> + 'd,
        data: impl Peripheral<P = impl GpioPin> + 'd,
        clock: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(pdm, irq, data, clock);

        let r = unsafe { &*PDM::ptr() };

        let Config { channels, left_edge, gain_left, gain_right } = config;

        // Configure channels
        r.enable.write(|w| w.enable().enabled());
        // TODO: Clock control
        r.mode.write(|w| {
            w.operation().variant(channels.into());
            w.edge().variant(left_edge.into());
            w
        });

        Self::_set_gain(r, gain_left, gain_right);

        r.psel.din.write(|w| unsafe { w.bits(data.psel_bits()) });
        r.psel.clk.write(|w| unsafe { w.bits(clock.psel_bits()) });

        // Disable all events interrupts
        r.intenclr.write(|w| unsafe { w.bits(0x003F_FFFF) });

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self { _p: pdm }
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

    fn _set_gain(r: &pdm::RegisterBlock, gain_left: I7F1, gain_right: I7F1) {
        let gain_left = gain_left.saturating_add(I7F1::from_bits(40)).saturating_to_num::<u8>().clamp(0, 0x50);
        let gain_right = gain_right.saturating_add(I7F1::from_bits(40)).saturating_to_num::<u8>().clamp(0, 0x50);

        r.gainl.write(|w| unsafe { w.gainl().bits(gain_left) });
        r.gainr.write(|w| unsafe { w.gainr().bits(gain_right) });
    }

    pub fn set_gain(&mut self, gain_left: I7F1, gain_right: I7F1) {
        Self::_set_gain(Self::regs(), gain_left, gain_right)
    }

    fn regs() -> &'static pdm::RegisterBlock {
        unsafe { &*PDM::ptr() }
    }

    /// One shot sampling. If the PDM is configured for multiple channels, the samples will be interleaved.
    pub async fn sample<const N: usize>(&mut self, buf: &mut [i16; N]) {
        let r = Self::regs();

        // Set up the DMA
        r.sample.ptr.write(|w| unsafe { w.sampleptr().bits(buf.as_mut_ptr() as u32) });
        r.sample.maxcnt.write(|w| unsafe { w.buffsize().bits(N as _) });

        // Reset and enable the end event
        r.events_end.reset();
        r.intenset.write(|w| w.end().set());

        // Don't reorder the start event before the previous writes. Hopefully self
        // wouldn't happen anyway.
        compiler_fence(Ordering::SeqCst);

        r.tasks_start.write(|w| { w.tasks_start().set_bit() });

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

impl<'d> Drop for Pdm<'d> {
    fn drop(&mut self) {
        let r = Self::regs();
        r.enable.write(|w| w.enable().disabled());
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Edge {
    FallingEdge,
    RisingEdge,
}

impl From<Edge> for EDGE_A {
    fn from(edge: Edge) -> Self {
        match edge {
            Edge::FallingEdge => EDGE_A::LEFTFALLING,
            Edge::RisingEdge => EDGE_A::LEFTRISING,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Channels {
    Stereo,
    Mono,
}

impl From<Channels> for OPERATION_A {
    fn from(ch: Channels) -> Self {
        match ch {
            Channels::Stereo => OPERATION_A::STEREO,
            Channels::Mono => OPERATION_A::MONO,
        }
    }
}