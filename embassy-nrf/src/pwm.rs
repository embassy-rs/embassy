#![macro_use]

use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

use crate::gpio::sealed::Pin as _;
use crate::gpio::{AnyPin, OptionalPin as GpioOptionalPin};
use crate::interrupt::Interrupt;
use crate::pac;
use crate::util::slice_in_ram_or;

/// Pwm allows you to offload the updating of a sequence of duty cycles
/// to up to four channels, as well as repeat that sequence n times.
pub struct Pwm<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
    ch0: Option<AnyPin>,
    ch1: Option<AnyPin>,
    ch2: Option<AnyPin>,
    ch3: Option<AnyPin>,
    sequence: &'d mut [u16],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Max Sequence size is 32767
    SequenceTooLong,
    /// Min Sequence count is 1
    SequenceTimesAtLeastOne,
    /// EasyDMA can only read from data memory, read only buffers in flash will fail.
    DMABufferNotInDataMemory,
}

impl<'d, T: Instance> Pwm<'d, T> {
    /// Creates the interface to a `Pwm`.
    ///
    /// Must be started by calling `start`
    ///
    /// # Safety
    ///
    /// The returned API is safe unless you use `mem::forget` (or similar safe
    /// mechanisms) on stack allocated buffers which which have been passed to
    /// [`new()`](Pwm::new).
    #[allow(unused_unsafe)]
    pub fn new(
        _pwm: impl Unborrow<Target = T> + 'd,
        ch0: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        ch1: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        ch2: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        ch3: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        config: Config,
        sequence: &'d mut [u16],
    ) -> Result<Self, Error> {
        slice_in_ram_or(sequence, Error::DMABufferNotInDataMemory)?;

        if sequence.len() > 32767 {
            return Err(Error::SequenceTooLong);
        }

        unborrow!(ch0, ch1, ch2, ch3);

        let r = T::regs();

        if let Some(pin) = ch0.pin_mut() {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }
        if let Some(pin) = ch1.pin_mut() {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }
        if let Some(pin) = ch2.pin_mut() {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }
        if let Some(pin) = ch3.pin_mut() {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }

        // if NoPin provided writes disconnected (top bit 1) 0x80000000 else
        // writes pin number ex 13 (0x0D) which is connected (top bit 0)
        r.psel.out[0].write(|w| unsafe { w.bits(ch0.psel_bits()) });
        r.psel.out[1].write(|w| unsafe { w.bits(ch1.psel_bits()) });
        r.psel.out[2].write(|w| unsafe { w.bits(ch2.psel_bits()) });
        r.psel.out[3].write(|w| unsafe { w.bits(ch3.psel_bits()) });

        // Disable all interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });
        r.shorts.reset();

        // Enable
        r.enable.write(|w| w.enable().enabled());

        r.seq0
            .ptr
            .write(|w| unsafe { w.bits(sequence.as_ptr() as u32) });
        r.seq0
            .cnt
            .write(|w| unsafe { w.bits(sequence.len() as u32) });
        r.seq0.refresh.write(|w| unsafe { w.bits(config.refresh) });
        r.seq0
            .enddelay
            .write(|w| unsafe { w.bits(config.end_delay) });

        r.seq1
            .ptr
            .write(|w| unsafe { w.bits(sequence.as_ptr() as u32) });
        r.seq1
            .cnt
            .write(|w| unsafe { w.bits(sequence.len() as u32) });
        r.seq1.refresh.write(|w| unsafe { w.bits(config.refresh) });
        r.seq1
            .enddelay
            .write(|w| unsafe { w.bits(config.end_delay) });

        r.decoder.write(|w| {
            w.load().bits(config.sequence_load as u8);
            w.mode().refresh_count()
        });

        r.mode.write(|w| match config.counter_mode {
            CounterMode::UpAndDown => w.updown().up_and_down(),
            CounterMode::Up => w.updown().up(),
        });
        r.prescaler
            .write(|w| w.prescaler().bits(config.prescaler as u8));
        r.countertop
            .write(|w| unsafe { w.countertop().bits(config.top) });

        Ok(Self {
            phantom: PhantomData,
            ch0: ch0.degrade_optional(),
            ch1: ch1.degrade_optional(),
            ch2: ch2.degrade_optional(),
            ch3: ch3.degrade_optional(),
            sequence,
        })
    }

    /// Start or restart playback
    #[inline(always)]
    pub fn start(&self, times: SequenceMode) -> Result<(), Error> {
        if let SequenceMode::Times(0) = times {
            return Err(Error::SequenceTimesAtLeastOne);
        }
        let r = T::regs();

        self.stop();

        r.enable.write(|w| w.enable().enabled());

        // defensive before seqstart
        compiler_fence(Ordering::SeqCst);

        match times {
            // just the one time, no loop count
            SequenceMode::Times(1) => {
                r.loop_.write(|w| w.cnt().disabled());
                // tasks_seqstart() doesn't exist in all svds so write its bit instead
                r.tasks_seqstart[0].write(|w| unsafe { w.bits(0x01) });
            }
            // loop count is how many times to play BOTH sequences
            // 2 total  (1 x 2)
            // 3 total, (2 x 2) - 1
            SequenceMode::Times(n) => {
                let odd = n & 1 == 1;
                let times = if odd { (n / 2) + 1 } else { n / 2 };

                r.loop_.write(|w| unsafe { w.cnt().bits(times) });

                // we can subtract 1 by starting at seq1 instead of seq0
                if odd {
                    // tasks_seqstart() doesn't exist in all svds so write its bit instead
                    r.tasks_seqstart[1].write(|w| unsafe { w.bits(0x01) });
                } else {
                    // tasks_seqstart() doesn't exist in all svds so write its bit instead
                    r.tasks_seqstart[0].write(|w| unsafe { w.bits(0x01) });
                }
            }
            // to play infinitely, repeat the sequence one time, then have loops done self trigger seq0 again
            SequenceMode::Infinite => {
                r.loop_.write(|w| unsafe { w.cnt().bits(0x1) });
                r.shorts.write(|w| w.loopsdone_seqstart0().enabled());

                // tasks_seqstart() doesn't exist in all svds so write its bit instead
                r.tasks_seqstart[0].write(|w| unsafe { w.bits(0x01) });
            }
        }

        Ok(())
    }

    /// Stop playback.
    #[inline(always)]
    pub fn stop(&self) {
        let r = T::regs();

        r.shorts.reset();

        compiler_fence(Ordering::SeqCst);

        // tasks_stop() doesn't exist in all svds so write its bit instead
        r.tasks_stop.write(|w| unsafe { w.bits(0x01) });
    }

    /// Enables the PWM generator.
    #[inline(always)]
    pub fn enable(&self) {
        let r = T::regs();
        r.enable.write(|w| w.enable().enabled());
    }

    /// Disables the PWM generator.
    #[inline(always)]
    pub fn disable(&self) {
        let r = T::regs();
        r.enable.write(|w| w.enable().disabled());
    }

    /// Sets duty cycle (15 bit) for a PWM channel.
    pub fn set_duty(&mut self, channel: usize, duty: u16) {
        let r = T::regs();

        self.sequence[channel] = duty & 0x7FFF;

        r.seq0
            .ptr
            .write(|w| unsafe { w.bits(self.sequence.as_ptr() as u32) });

        // defensive before seqstart
        compiler_fence(Ordering::SeqCst);

        // tasks_seqstart() doesn't exist in all svds so write its bit instead
        r.tasks_seqstart[0].write(|w| unsafe { w.bits(1) });

        // defensive wait until waveform is loaded after seqstart
        while r.events_seqend[0].read().bits() == 0 {}
        r.events_seqend[0].write(|w| w);
    }
}

impl<'a, T: Instance> Drop for Pwm<'a, T> {
    fn drop(&mut self) {
        let r = T::regs();

        self.stop();
        self.disable();

        if let Some(pin) = &self.ch0 {
            pin.set_low();
            pin.conf().write(|w| w);
            r.psel.out[0].write(|w| unsafe { w.bits(0x80000000) });
        }
        if let Some(pin) = &self.ch1 {
            pin.set_low();
            pin.conf().write(|w| w);
            r.psel.out[1].write(|w| unsafe { w.bits(0x80000000) });
        }
        if let Some(pin) = &self.ch2 {
            pin.set_low();
            pin.conf().write(|w| w);
            r.psel.out[2].write(|w| unsafe { w.bits(0x80000000) });
        }
        if let Some(pin) = &self.ch3 {
            pin.set_low();
            pin.conf().write(|w| w);
            r.psel.out[3].write(|w| unsafe { w.bits(0x80000000) });
        }
    }
}

/// Configure an infinite looping sequence for `Pwm`
#[non_exhaustive]
pub struct Config {
    /// Selects up mode or up-and-down mode for the counter
    pub counter_mode: CounterMode,
    /// Top value to be compared against buffer values
    pub top: u16,
    /// Configuration for PWM_CLK
    pub prescaler: Prescaler,
    /// How a sequence is read from RAM and is spread to the compare register
    pub sequence_load: SequenceLoad,
    /// Number of PWM periods to delay between each sequence sample
    pub refresh: u32,
    /// Number of PWM periods after the sequence ends before starting the next sequence
    pub end_delay: u32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            counter_mode: CounterMode::Up,
            top: 1000,
            prescaler: Prescaler::Div16,
            sequence_load: SequenceLoad::Common,
            refresh: 0,
            end_delay: 0,
        }
    }
}

/// How many times to run the sequence
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SequenceMode {
    /// Run sequence n Times total
    Times(u16),
    /// Repeat until `stop` is called.
    Infinite,
}

/// PWM Base clock is system clock (16MHz) divided by prescaler
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Prescaler {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
    Div32,
    Div64,
    Div128,
}

/// How the sequence values are distributed across the channels
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SequenceLoad {
    /// Provided sequence will be used across all channels
    Common,
    /// Provided sequence contains grouped values for each channel ex:
    /// [ch0_0_and_ch1_0, ch2_0_and_ch3_0, ... ch0_n_and_ch1_n, ch2_n_and_ch3_n]
    Grouped,
    /// Provided sequence contains individual values for each channel ex:
    /// [ch0_0, ch1_0, ch2_0, ch3_0... ch0_n, ch1_n, ch2_n, ch3_n]
    Individual,
    /// Similar to Individual mode, but only three channels are used. The fourth
    /// value is loaded into the pulse generator counter as its top value.
    Waveform,
}

/// Selects up mode or up-and-down mode for the counter
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CounterMode {
    /// Up counter (edge-aligned PWM duty cycle)
    Up,
    /// Up and down counter (center-aligned PWM duty cycle)
    UpAndDown,
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> &'static pac::pwm0::RegisterBlock;
    }
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_pwm {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::pwm::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::pwm0::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
        }
        impl crate::pwm::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}
