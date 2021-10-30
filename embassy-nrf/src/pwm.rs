#![macro_use]

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

use crate::gpio::sealed::Pin as _;
use crate::gpio::OptionalPin as GpioOptionalPin;
use crate::interrupt::Interrupt;
use crate::util::slice_in_ram_or;
use crate::{pac, EASY_DMA_SIZE};

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

/// How a sequence is read from RAM and is spread to the compare register
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SequenceLoad {
    /// sequence in buffer will be used across all channels
    Common,
    Grouped,
    /// buffer holds [ch0_0, ch1_0, ch2_0, ch3_0... ch0_n, ch1_n, ch2_n, ch3_n]
    Individual,
    Waveform,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CounterMode {
    Up,
    UpAndDown,
}

/// Interface to the PWM peripheral
pub struct Pwm<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum LoopMode {
    // Repeat n additional times after the first
    Additional(u16),
    /// Repeat until `stop` is called
    Infinite,
}

// Configure an infinite looping sequence for `simple_playback`
pub struct LoopingConfig<'a> {
    /// Selects up mode or up-and-down mode for the counter
    pub counter_mode: CounterMode,
    // Top value to be compared against buffer values
    pub top: u16,
    /// Configuration for PWM_CLK
    pub prescaler: Prescaler,
    /// In ram buffer to be played back
    pub sequence: &'a [u16],
    /// How a sequence is read from RAM and is spread to the compare register
    pub sequence_load: SequenceLoad,
    /// Number of additional PWM periods between samples loaded into compare register
    pub refresh: u32,
    /// Number of additional PWM periods after the sequence ends
    pub enddelay: u32,
    /// How many times to repeat the sequence
    pub additional_loops: LoopMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    Seq0BufferTooLong,
    Seq1BufferTooLong,
    /// EasyDMA can only read from data memory, read only buffers in flash will fail.
    DMABufferNotInDataMemory,
}

impl<'d, T: Instance> Pwm<'d, T> {
    /// Creates the interface to a UARTE instance.
    /// Sets the baud rate, parity and assigns the pins to the UARTE peripheral.
    ///
    /// # Safety
    ///
    /// The returned API is safe unless you use `mem::forget` (or similar safe mechanisms)
    /// on stack allocated buffers which which have been passed to [`send()`](Pwm::send)
    /// or [`receive`](Pwm::receive).
    #[allow(unused_unsafe)]
    pub fn new(
        _pwm: impl Unborrow<Target = T> + 'd,
        ch0: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        ch1: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        ch2: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        ch3: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
    ) -> Self {
        unborrow!(ch0, ch1, ch2, ch3);

        let r = T::regs();
        let s = T::state();

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
        r.psel.out[0].write(|w| unsafe { w.bits(ch0.psel_bits()) });
        r.psel.out[1].write(|w| unsafe { w.bits(ch1.psel_bits()) });
        r.psel.out[2].write(|w| unsafe { w.bits(ch2.psel_bits()) });
        r.psel.out[3].write(|w| unsafe { w.bits(ch3.psel_bits()) });

        // Disable all interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        // Enable
        r.enable.write(|w| w.enable().enabled());

        r.seq0
            .ptr
            .write(|w| unsafe { w.bits(&s.duty as *const _ as u32) });
        r.seq0.cnt.write(|w| unsafe { w.bits(4) });
        r.seq0.refresh.write(|w| unsafe { w.bits(32) });
        r.seq0.enddelay.write(|w| unsafe { w.bits(0) });

        r.decoder.write(|w| {
            w.load().individual();
            w.mode().refresh_count()
        });
        r.mode.write(|w| w.updown().up());
        r.prescaler.write(|w| w.prescaler().div_1());
        r.countertop
            .write(|w| unsafe { w.countertop().bits(32767) });
        r.loop_.write(|w| w.cnt().disabled());

        Self {
            phantom: PhantomData,
        }
    }

    /// Returns a configured pwm that has had start called on it
    pub fn simple_playback(
        _pwm: impl Unborrow<Target = T> + 'd,
        ch0: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        ch1: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        ch2: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        ch3: impl Unborrow<Target = impl GpioOptionalPin> + 'd,
        config: LoopingConfig,
    ) -> Result<Self, Error> {
        slice_in_ram_or(config.sequence, Error::DMABufferNotInDataMemory)?;

        if config.sequence.len() > EASY_DMA_SIZE {
            return Err(Error::Seq0BufferTooLong);
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

        r.enable.write(|w| w.enable().enabled());

        r.mode
            .write(|w| unsafe { w.bits(config.counter_mode as u32) });
        r.prescaler
            .write(|w| w.prescaler().bits(config.prescaler as u8));
        r.countertop
            .write(|w| unsafe { w.countertop().bits(config.top) });

        r.decoder.write(|w| {
            w.load().bits(config.sequence_load as u8);
            w.mode().refresh_count()
        });

        r.seq0
            .ptr
            .write(|w| unsafe { w.bits(config.sequence.as_ptr() as u32) });
        r.seq0
            .cnt
            .write(|w| unsafe { w.bits(config.sequence.len() as u32) });
        r.seq0.refresh.write(|w| unsafe { w.bits(config.refresh) });
        r.seq0
            .enddelay
            .write(|w| unsafe { w.bits(config.enddelay) });

        r.seq1
            .ptr
            .write(|w| unsafe { w.bits(config.sequence.as_ptr() as u32) });
        r.seq1
            .cnt
            .write(|w| unsafe { w.bits(config.sequence.len() as u32) });
        r.seq1.refresh.write(|w| unsafe { w.bits(config.refresh) });
        r.seq1
            .enddelay
            .write(|w| unsafe { w.bits(config.enddelay) });

        match config.additional_loops {
            LoopMode::Additional(0) => {
                r.loop_.write(|w| w.cnt().disabled());
                r.tasks_seqstart[0].write(|w| unsafe { w.bits(0x01) });
            }
            LoopMode::Additional(n) => {
                let times = (n / 2) + 1;

                r.loop_.write(|w| unsafe { w.cnt().bits(times) });
                r.shorts.write(|w| {
                    w.loopsdone_seqstart1().enabled();
                    w.loopsdone_seqstart0().disabled();
                    w.loopsdone_stop().enabled()
                });

                if n & 1 == 1 {
                    // tasks_seqstart doesnt exist in all svds so write its bit instead
                    r.tasks_seqstart[0].write(|w| unsafe { w.bits(0x01) });
                } else {
                    // tasks_seqstart doesnt exist in all svds so write its bit instead
                    r.tasks_seqstart[1].write(|w| unsafe { w.bits(0x01) });
                }
            }

            LoopMode::Infinite => {
                r.loop_.write(|w| unsafe { w.cnt().bits(0x1) });
                r.shorts.write(|w| {
                    w.loopsdone_seqstart1().enabled();
                    w.loopsdone_seqstart0().disabled()
                });
                r.tasks_seqstart[1].write(|w| unsafe { w.bits(0x01) });
            }
        }

        Ok(Self {
            phantom: PhantomData,
        })
    }

    /// Stop playback
    #[inline(always)]
    pub fn stop(&self) {
        let r = T::regs();

        r.shorts.write(|w| unsafe { w.bits(0x0) });

        // tasks_stop doesnt exist in all svds so write its bit instead
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
    pub fn set_duty(&self, channel: usize, duty: u16) {
        let s = T::state();
        unsafe { (*s.duty.get())[channel] = duty & 0x7FFF };

        compiler_fence(Ordering::SeqCst);
        T::regs().tasks_seqstart[0].write(|w| unsafe { w.bits(1) });
    }

    /// Sets the PWM clock prescaler.
    #[inline(always)]
    pub fn set_prescaler(&self, div: Prescaler) {
        T::regs().prescaler.write(|w| w.prescaler().bits(div as u8));
    }

    /// Gets the PWM clock prescaler.
    #[inline(always)]
    pub fn prescaler(&self) -> Prescaler {
        match T::regs().prescaler.read().prescaler().bits() {
            0 => Prescaler::Div1,
            1 => Prescaler::Div2,
            2 => Prescaler::Div4,
            3 => Prescaler::Div8,
            4 => Prescaler::Div16,
            5 => Prescaler::Div32,
            6 => Prescaler::Div64,
            7 => Prescaler::Div128,
            _ => unreachable!(),
        }
    }

    /// Sets the maximum duty cycle value.
    #[inline(always)]
    pub fn set_max_duty(&self, duty: u16) {
        T::regs()
            .countertop
            .write(|w| unsafe { w.countertop().bits(duty.min(32767u16)) });
    }

    /// Returns the maximum duty cycle value.
    #[inline(always)]
    pub fn max_duty(&self) -> u16 {
        T::regs().countertop.read().countertop().bits()
    }

    /// Sets the PWM output frequency.
    #[inline(always)]
    pub fn set_period(&self, freq: u32) {
        let clk = 16_000_000u32 >> (self.prescaler() as u8);
        let duty = clk / freq;
        self.set_max_duty(duty.min(32767) as u16);
    }

    /// Returns the PWM output frequency.
    #[inline(always)]
    pub fn period(&self) -> u32 {
        let clk = 16_000_000u32 >> (self.prescaler() as u8);
        let max_duty = self.max_duty() as u32;
        clk / max_duty
    }
}

impl<'a, T: Instance> Drop for Pwm<'a, T> {
    fn drop(&mut self) {
        self.stop();
        self.disable();

        info!("pwm drop: done");

        // TODO: disable pins
    }
}

pub(crate) mod sealed {
    use super::*;

    pub struct State {
        pub duty: UnsafeCell<[u16; 4]>,
    }
    unsafe impl Sync for State {}

    impl State {
        pub const fn new() -> Self {
            Self {
                duty: UnsafeCell::new([0; 4]),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static pac::pwm0::RegisterBlock;
        fn state() -> &'static State;
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
            fn state() -> &'static crate::pwm::sealed::State {
                static STATE: crate::pwm::sealed::State = crate::pwm::sealed::State::new();
                &STATE
            }
        }
        impl crate::pwm::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}
