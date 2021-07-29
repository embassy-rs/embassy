use core::future::Future;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;
use embassy::util::{wake_on_interrupt, Unborrow};
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

use crate::interrupt;
use crate::{pac, peripherals};

#[cfg(feature = "9160")]
use pac::{saadc_ns as saadc, SAADC_NS as SAADC};

#[cfg(not(feature = "9160"))]
use pac::{saadc, SAADC};

pub use saadc::{
    ch::{
        config::{GAIN_A as Gain, REFSEL_A as Reference, RESP_A as Resistor, TACQ_A as Time},
        pselp::PSELP_A as PositiveChannel,
    },
    oversample::OVERSAMPLE_A as Oversample,
    resolution::VAL_A as Resolution,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {}

/// One-shot saadc. Continuous sample mode TODO.
pub struct OneShot<'d> {
    irq: interrupt::SAADC,
    phantom: PhantomData<&'d mut peripherals::SAADC>,
}

/// Used to configure the SAADC peripheral.
///
/// See the `Default` impl for suitable default values.
pub struct Config {
    /// Output resolution in bits.
    pub resolution: Resolution,
    /// Average 2^`oversample` input samples before transferring the result into memory.
    pub oversample: Oversample,
    /// Reference voltage of the SAADC input.
    pub reference: Reference,
    /// Gain used to control the effective input range of the SAADC.
    pub gain: Gain,
    /// Positive channel resistor control.
    pub resistor: Resistor,
    /// Acquisition time in microseconds.
    pub time: Time,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: Resolution::_14BIT,
            oversample: Oversample::OVER8X,
            reference: Reference::VDD1_4,
            gain: Gain::GAIN1_4,
            resistor: Resistor::BYPASS,
            time: Time::_20US,
        }
    }
}

impl<'d> OneShot<'d> {
    pub fn new(
        _saadc: impl Unborrow<Target = peripherals::SAADC> + 'd,
        irq: impl Unborrow<Target = interrupt::SAADC> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(irq);

        let r = unsafe { &*SAADC::ptr() };

        let Config {
            resolution,
            oversample,
            reference,
            gain,
            resistor,
            time,
        } = config;

        // Configure pins
        r.enable.write(|w| w.enable().enabled());
        r.resolution.write(|w| w.val().variant(resolution));
        r.oversample.write(|w| w.oversample().variant(oversample));

        r.ch[0].config.write(|w| {
            w.refsel().variant(reference);
            w.gain().variant(gain);
            w.tacq().variant(time);
            w.mode().se();
            w.resp().variant(resistor);
            w.resn().bypass();
            if !matches!(oversample, Oversample::BYPASS) {
                w.burst().enabled();
            } else {
                w.burst().disabled();
            }
            w
        });

        // Disable all events interrupts
        r.intenclr.write(|w| unsafe { w.bits(0x003F_FFFF) });

        Self {
            irq,
            phantom: PhantomData,
        }
    }

    fn regs(&self) -> &saadc::RegisterBlock {
        unsafe { &*SAADC::ptr() }
    }

    async fn sample_inner(&mut self, pin: PositiveChannel) -> i16 {
        let r = self.regs();

        // Set positive channel
        r.ch[0].pselp.write(|w| w.pselp().variant(pin));

        // Set up the DMA
        let mut val: i16 = 0;
        r.result
            .ptr
            .write(|w| unsafe { w.ptr().bits(((&mut val) as *mut _) as u32) });
        r.result.maxcnt.write(|w| unsafe { w.maxcnt().bits(1) });

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
            let r = self.regs();

            if r.events_end.read().bits() != 0 {
                r.events_end.reset();
                return Poll::Ready(());
            }

            wake_on_interrupt(&mut self.irq, cx.waker());

            Poll::Pending
        })
        .await;

        // The DMA wrote the sampled value to `val`.
        val
    }
}

impl<'d> Drop for OneShot<'d> {
    fn drop(&mut self) {
        let r = self.regs();
        r.enable.write(|w| w.enable().disabled());
    }
}

pub trait Sample {
    type SampleFuture<'a>: Future<Output = i16> + 'a
    where
        Self: 'a;

    fn sample<'a, T: PositivePin>(&'a mut self, pin: &mut T) -> Self::SampleFuture<'a>;
}

impl<'d> Sample for OneShot<'d> {
    #[rustfmt::skip]
    type SampleFuture<'a> where Self: 'a = impl Future<Output = i16> + 'a;

    fn sample<'a, T: PositivePin>(&'a mut self, pin: &mut T) -> Self::SampleFuture<'a> {
        self.sample_inner(pin.channel())
    }
}

/// A pin that can be used as the positive end of a ADC differential in the SAADC periperhal.
///
/// Currently negative is always shorted to ground (0V).
pub trait PositivePin {
    fn channel(&self) -> PositiveChannel;
}

macro_rules! positive_pin_mappings {
    ( $($ch:ident => $pin:ident,)*) => {
        $(
            impl PositivePin for crate::peripherals::$pin {
                fn channel(&self) -> PositiveChannel {
                    PositiveChannel::$ch
                }
            }
        )*
    };
}

// TODO the variant names are unchecked
// the pins are copied from nrf hal
#[cfg(feature = "9160")]
positive_pin_mappings! {
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
positive_pin_mappings! {
    ANALOGINPUT0 => P0_02,
    ANALOGINPUT1 => P0_03,
    ANALOGINPUT2 => P0_04,
    ANALOGINPUT3 => P0_05,
    ANALOGINPUT4 => P0_28,
    ANALOGINPUT5 => P0_29,
    ANALOGINPUT6 => P0_30,
    ANALOGINPUT7 => P0_31,
}
