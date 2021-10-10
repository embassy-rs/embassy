use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;
use embassy::interrupt::InterruptExt;
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

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

/// One-shot saadc. Continuous sample mode TODO.
pub struct OneShot<'d, const N: usize> {
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

/// An input that can be used as either or negative end of a ADC differential in the SAADC periperhal.
pub trait Input {
    fn channel(&self) -> InputChannel;
}

macro_rules! input_mappings {
    ( $($ch:ident => $input:ident,)*) => {
        $(
            impl Input for crate::peripherals::$input {
                fn channel(&self) -> InputChannel {
                    InputChannel::$ch
                }
            }
        )*
    };
}

// TODO the variant names are unchecked
// the inputs are copied from nrf hal
#[cfg(feature = "9160")]
input_mappings! {
    ANALOGINPUT0 => P0_13,
    ANALOGINPUT1 => P0_14,
    ANALOGINPUT2 => P0_15,
    ANALOGINPUT3 => P0_16,
    ANALOGINPUT4 => P0_17,
    ANALOGINPUT5 => P0_18,
    ANALOGINPUT6 => P0_19,
    ANALOGINPUT7 => P0_20,
}

#[cfg(not(feature = "9160"))]
input_mappings! {
    ANALOGINPUT0 => P0_02,
    ANALOGINPUT1 => P0_03,
    ANALOGINPUT2 => P0_04,
    ANALOGINPUT3 => P0_05,
    ANALOGINPUT4 => P0_28,
    ANALOGINPUT5 => P0_29,
    ANALOGINPUT6 => P0_30,
    ANALOGINPUT7 => P0_31,
}
