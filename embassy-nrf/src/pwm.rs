//! Pulse Width Modulation (PWM) driver.

#![macro_use]

use core::sync::atomic::{compiler_fence, Ordering};

use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::gpio::sealed::Pin as _;
use crate::gpio::{AnyPin, Pin as GpioPin, PselBits};
use crate::ppi::{Event, Task};
use crate::util::slice_in_ram_or;
use crate::{interrupt, pac, Peripheral};

/// SimplePwm is the traditional pwm interface you're probably used to, allowing
/// to simply set a duty cycle across up to four channels.
pub struct SimplePwm<'d, T: Instance> {
    _peri: PeripheralRef<'d, T>,
    duty: [u16; 4],
    ch0: Option<PeripheralRef<'d, AnyPin>>,
    ch1: Option<PeripheralRef<'d, AnyPin>>,
    ch2: Option<PeripheralRef<'d, AnyPin>>,
    ch3: Option<PeripheralRef<'d, AnyPin>>,
}

/// SequencePwm allows you to offload the updating of a sequence of duty cycles
/// to up to four channels, as well as repeat that sequence n times.
pub struct SequencePwm<'d, T: Instance> {
    _peri: PeripheralRef<'d, T>,
    ch0: Option<PeripheralRef<'d, AnyPin>>,
    ch1: Option<PeripheralRef<'d, AnyPin>>,
    ch2: Option<PeripheralRef<'d, AnyPin>>,
    ch3: Option<PeripheralRef<'d, AnyPin>>,
}

/// PWM error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Max Sequence size is 32767
    SequenceTooLong,
    /// Min Sequence count is 1
    SequenceTimesAtLeastOne,
    /// EasyDMA can only read from data memory, read only buffers in flash will fail.
    BufferNotInRAM,
}

const MAX_SEQUENCE_LEN: usize = 32767;

impl<'d, T: Instance> SequencePwm<'d, T> {
    /// Create a new 1-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_1ch(
        pwm: impl Peripheral<P = T> + 'd,
        ch0: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        into_ref!(ch0);
        Self::new_inner(pwm, Some(ch0.map_into()), None, None, None, config)
    }

    /// Create a new 2-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_2ch(
        pwm: impl Peripheral<P = T> + 'd,
        ch0: impl Peripheral<P = impl GpioPin> + 'd,
        ch1: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        into_ref!(ch0, ch1);
        Self::new_inner(pwm, Some(ch0.map_into()), Some(ch1.map_into()), None, None, config)
    }

    /// Create a new 3-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_3ch(
        pwm: impl Peripheral<P = T> + 'd,
        ch0: impl Peripheral<P = impl GpioPin> + 'd,
        ch1: impl Peripheral<P = impl GpioPin> + 'd,
        ch2: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        into_ref!(ch0, ch1, ch2);
        Self::new_inner(
            pwm,
            Some(ch0.map_into()),
            Some(ch1.map_into()),
            Some(ch2.map_into()),
            None,
            config,
        )
    }

    /// Create a new 4-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_4ch(
        pwm: impl Peripheral<P = T> + 'd,
        ch0: impl Peripheral<P = impl GpioPin> + 'd,
        ch1: impl Peripheral<P = impl GpioPin> + 'd,
        ch2: impl Peripheral<P = impl GpioPin> + 'd,
        ch3: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        into_ref!(ch0, ch1, ch2, ch3);
        Self::new_inner(
            pwm,
            Some(ch0.map_into()),
            Some(ch1.map_into()),
            Some(ch2.map_into()),
            Some(ch3.map_into()),
            config,
        )
    }

    fn new_inner(
        _pwm: impl Peripheral<P = T> + 'd,
        ch0: Option<PeripheralRef<'d, AnyPin>>,
        ch1: Option<PeripheralRef<'d, AnyPin>>,
        ch2: Option<PeripheralRef<'d, AnyPin>>,
        ch3: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Result<Self, Error> {
        into_ref!(_pwm);

        let r = T::regs();

        if let Some(pin) = &ch0 {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }
        if let Some(pin) = &ch1 {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }
        if let Some(pin) = &ch2 {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }
        if let Some(pin) = &ch3 {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }

        r.psel.out[0].write(|w| unsafe { w.bits(ch0.psel_bits()) });
        r.psel.out[1].write(|w| unsafe { w.bits(ch1.psel_bits()) });
        r.psel.out[2].write(|w| unsafe { w.bits(ch2.psel_bits()) });
        r.psel.out[3].write(|w| unsafe { w.bits(ch3.psel_bits()) });

        // Disable all interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });
        r.shorts.reset();
        r.events_stopped.reset();
        r.events_loopsdone.reset();
        r.events_seqend[0].reset();
        r.events_seqend[1].reset();
        r.events_pwmperiodend.reset();
        r.events_seqstarted[0].reset();
        r.events_seqstarted[1].reset();

        r.decoder.write(|w| {
            w.load().bits(config.sequence_load as u8);
            w.mode().refresh_count()
        });

        r.mode.write(|w| match config.counter_mode {
            CounterMode::UpAndDown => w.updown().up_and_down(),
            CounterMode::Up => w.updown().up(),
        });
        r.prescaler.write(|w| w.prescaler().bits(config.prescaler as u8));
        r.countertop.write(|w| unsafe { w.countertop().bits(config.max_duty) });

        Ok(Self {
            _peri: _pwm,
            ch0,
            ch1,
            ch2,
            ch3,
        })
    }

    /// Returns reference to `Stopped` event endpoint for PPI.
    #[inline(always)]
    pub fn event_stopped(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(&r.events_stopped)
    }

    /// Returns reference to `LoopsDone` event endpoint for PPI.
    #[inline(always)]
    pub fn event_loops_done(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(&r.events_loopsdone)
    }

    /// Returns reference to `PwmPeriodEnd` event endpoint for PPI.
    #[inline(always)]
    pub fn event_pwm_period_end(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(&r.events_pwmperiodend)
    }

    /// Returns reference to `Seq0 End` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq_end(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(&r.events_seqend[0])
    }

    /// Returns reference to `Seq1 End` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq1_end(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(&r.events_seqend[1])
    }

    /// Returns reference to `Seq0 Started` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq0_started(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(&r.events_seqstarted[0])
    }

    /// Returns reference to `Seq1 Started` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq1_started(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(&r.events_seqstarted[1])
    }

    /// Returns reference to `Seq0 Start` task endpoint for PPI.
    /// # Safety
    ///
    /// Interacting with the sequence while it runs puts it in an unknown state
    #[inline(always)]
    pub unsafe fn task_start_seq0(&self) -> Task<'d> {
        let r = T::regs();

        Task::from_reg(&r.tasks_seqstart[0])
    }

    /// Returns reference to `Seq1 Started` task endpoint for PPI.
    /// # Safety
    ///
    /// Interacting with the sequence while it runs puts it in an unknown state
    #[inline(always)]
    pub unsafe fn task_start_seq1(&self) -> Task<'d> {
        let r = T::regs();

        Task::from_reg(&r.tasks_seqstart[1])
    }

    /// Returns reference to `NextStep` task endpoint for PPI.
    /// # Safety
    ///
    /// Interacting with the sequence while it runs puts it in an unknown state
    #[inline(always)]
    pub unsafe fn task_next_step(&self) -> Task<'d> {
        let r = T::regs();

        Task::from_reg(&r.tasks_nextstep)
    }

    /// Returns reference to `Stop` task endpoint for PPI.
    /// # Safety
    ///
    /// Interacting with the sequence while it runs puts it in an unknown state
    #[inline(always)]
    pub unsafe fn task_stop(&self) -> Task<'d> {
        let r = T::regs();

        Task::from_reg(&r.tasks_stop)
    }
}

impl<'a, T: Instance> Drop for SequencePwm<'a, T> {
    fn drop(&mut self) {
        let r = T::regs();

        if let Some(pin) = &self.ch0 {
            pin.set_low();
            pin.conf().reset();
            r.psel.out[0].reset();
        }
        if let Some(pin) = &self.ch1 {
            pin.set_low();
            pin.conf().reset();
            r.psel.out[1].reset();
        }
        if let Some(pin) = &self.ch2 {
            pin.set_low();
            pin.conf().reset();
            r.psel.out[2].reset();
        }
        if let Some(pin) = &self.ch3 {
            pin.set_low();
            pin.conf().reset();
            r.psel.out[3].reset();
        }
    }
}

/// Configuration for the PWM as a whole.
#[non_exhaustive]
pub struct Config {
    /// Selects up mode or up-and-down mode for the counter
    pub counter_mode: CounterMode,
    /// Top value to be compared against buffer values
    pub max_duty: u16,
    /// Configuration for PWM_CLK
    pub prescaler: Prescaler,
    /// How a sequence is read from RAM and is spread to the compare register
    pub sequence_load: SequenceLoad,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            counter_mode: CounterMode::Up,
            max_duty: 1000,
            prescaler: Prescaler::Div16,
            sequence_load: SequenceLoad::Common,
        }
    }
}

/// Configuration per sequence
#[non_exhaustive]
#[derive(Clone)]
pub struct SequenceConfig {
    /// Number of PWM periods to delay between each sequence sample
    pub refresh: u32,
    /// Number of PWM periods after the sequence ends before starting the next sequence
    pub end_delay: u32,
}

impl Default for SequenceConfig {
    fn default() -> SequenceConfig {
        SequenceConfig {
            refresh: 0,
            end_delay: 0,
        }
    }
}

/// A composition of a sequence buffer and its configuration.
#[non_exhaustive]
pub struct Sequence<'s> {
    /// The words comprising the sequence. Must not exceed 32767 words.
    pub words: &'s [u16],
    /// Configuration associated with the sequence.
    pub config: SequenceConfig,
}

impl<'s> Sequence<'s> {
    /// Create a new `Sequence`
    pub fn new(words: &'s [u16], config: SequenceConfig) -> Self {
        Self { words, config }
    }
}

/// A single sequence that can be started and stopped.
/// Takes at one sequence along with its configuration.
#[non_exhaustive]
pub struct SingleSequencer<'d, 's, T: Instance> {
    sequencer: Sequencer<'d, 's, T>,
}

impl<'d, 's, T: Instance> SingleSequencer<'d, 's, T> {
    /// Create a new sequencer
    pub fn new(pwm: &'s mut SequencePwm<'d, T>, words: &'s [u16], config: SequenceConfig) -> Self {
        Self {
            sequencer: Sequencer::new(pwm, Sequence::new(words, config), None),
        }
    }

    /// Start or restart playback.
    #[inline(always)]
    pub fn start(&self, times: SingleSequenceMode) -> Result<(), Error> {
        let (start_seq, times) = match times {
            SingleSequenceMode::Times(n) if n == 1 => (StartSequence::One, SequenceMode::Loop(1)),
            SingleSequenceMode::Times(n) if n & 1 == 1 => (StartSequence::One, SequenceMode::Loop((n / 2) + 1)),
            SingleSequenceMode::Times(n) => (StartSequence::Zero, SequenceMode::Loop(n / 2)),
            SingleSequenceMode::Infinite => (StartSequence::Zero, SequenceMode::Infinite),
        };
        self.sequencer.start(start_seq, times)
    }

    /// Stop playback. Disables the peripheral. Does NOT clear the last duty
    /// cycle from the pin. Returns any sequences previously provided to
    /// `start` so that they may be further mutated.
    #[inline(always)]
    pub fn stop(&self) {
        self.sequencer.stop();
    }
}

/// A composition of sequences that can be started and stopped.
/// Takes at least one sequence along with its configuration.
/// Optionally takes a second sequence and its configuration.
/// In the case where no second sequence is provided then the first sequence
/// is used.
#[non_exhaustive]
pub struct Sequencer<'d, 's, T: Instance> {
    _pwm: &'s mut SequencePwm<'d, T>,
    sequence0: Sequence<'s>,
    sequence1: Option<Sequence<'s>>,
}

impl<'d, 's, T: Instance> Sequencer<'d, 's, T> {
    /// Create a new double sequence. In the absence of sequence 1, sequence 0
    /// will be used twice in the one loop.
    pub fn new(pwm: &'s mut SequencePwm<'d, T>, sequence0: Sequence<'s>, sequence1: Option<Sequence<'s>>) -> Self {
        Sequencer {
            _pwm: pwm,
            sequence0,
            sequence1,
        }
    }

    /// Start or restart playback. The sequence mode applies to both sequences combined as one.
    #[inline(always)]
    pub fn start(&self, start_seq: StartSequence, times: SequenceMode) -> Result<(), Error> {
        let sequence0 = &self.sequence0;
        let alt_sequence = self.sequence1.as_ref().unwrap_or(&self.sequence0);

        slice_in_ram_or(sequence0.words, Error::BufferNotInRAM)?;
        slice_in_ram_or(alt_sequence.words, Error::BufferNotInRAM)?;

        if sequence0.words.len() > MAX_SEQUENCE_LEN || alt_sequence.words.len() > MAX_SEQUENCE_LEN {
            return Err(Error::SequenceTooLong);
        }

        if let SequenceMode::Loop(0) = times {
            return Err(Error::SequenceTimesAtLeastOne);
        }

        let _ = self.stop();

        let r = T::regs();

        r.seq0.refresh.write(|w| unsafe { w.bits(sequence0.config.refresh) });
        r.seq0.enddelay.write(|w| unsafe { w.bits(sequence0.config.end_delay) });
        r.seq0.ptr.write(|w| unsafe { w.bits(sequence0.words.as_ptr() as u32) });
        r.seq0.cnt.write(|w| unsafe { w.bits(sequence0.words.len() as u32) });

        r.seq1.refresh.write(|w| unsafe { w.bits(alt_sequence.config.refresh) });
        r.seq1
            .enddelay
            .write(|w| unsafe { w.bits(alt_sequence.config.end_delay) });
        r.seq1
            .ptr
            .write(|w| unsafe { w.bits(alt_sequence.words.as_ptr() as u32) });
        r.seq1.cnt.write(|w| unsafe { w.bits(alt_sequence.words.len() as u32) });

        r.enable.write(|w| w.enable().enabled());

        // defensive before seqstart
        compiler_fence(Ordering::SeqCst);

        let seqstart_index = if start_seq == StartSequence::One { 1 } else { 0 };

        match times {
            // just the one time, no loop count
            SequenceMode::Loop(n) => {
                r.loop_.write(|w| unsafe { w.cnt().bits(n) });
            }
            // to play infinitely, repeat the sequence one time, then have loops done self trigger seq0 again
            SequenceMode::Infinite => {
                r.loop_.write(|w| unsafe { w.cnt().bits(0x1) });
                r.shorts.write(|w| w.loopsdone_seqstart0().enabled());
            }
        }

        // tasks_seqstart() doesn't exist in all svds so write its bit instead
        r.tasks_seqstart[seqstart_index].write(|w| unsafe { w.bits(0x01) });

        Ok(())
    }

    /// Stop playback. Disables the peripheral. Does NOT clear the last duty
    /// cycle from the pin. Returns any sequences previously provided to
    /// `start` so that they may be further mutated.
    #[inline(always)]
    pub fn stop(&self) {
        let r = T::regs();

        r.shorts.reset();

        compiler_fence(Ordering::SeqCst);

        // tasks_stop() doesn't exist in all svds so write its bit instead
        r.tasks_stop.write(|w| unsafe { w.bits(0x01) });

        r.enable.write(|w| w.enable().disabled());
    }
}

impl<'d, 's, T: Instance> Drop for Sequencer<'d, 's, T> {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// How many times to run a single sequence
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SingleSequenceMode {
    /// Run a single sequence n Times total.
    Times(u16),
    /// Repeat until `stop` is called.
    Infinite,
}

/// Which sequence to start a loop with
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum StartSequence {
    /// Start with Sequence 0
    Zero,
    /// Start with Sequence 1
    One,
}

/// How many loops to run two sequences
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SequenceMode {
    /// Run two sequences n loops i.e. (n * (seq0 + seq1.unwrap_or(seq0)))
    Loop(u16),
    /// Repeat until `stop` is called.
    Infinite,
}

/// PWM Base clock is system clock (16MHz) divided by prescaler
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Prescaler {
    /// Divide by 1
    Div1,
    /// Divide by 2
    Div2,
    /// Divide by 4
    Div4,
    /// Divide by 8
    Div8,
    /// Divide by 16
    Div16,
    /// Divide by 32
    Div32,
    /// Divide by 64
    Div64,
    /// Divide by 128
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

impl<'d, T: Instance> SimplePwm<'d, T> {
    /// Create a new 1-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_1ch(pwm: impl Peripheral<P = T> + 'd, ch0: impl Peripheral<P = impl GpioPin> + 'd) -> Self {
        unsafe {
            into_ref!(ch0);
            Self::new_inner(pwm, Some(ch0.map_into()), None, None, None)
        }
    }

    /// Create a new 2-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_2ch(
        pwm: impl Peripheral<P = T> + 'd,
        ch0: impl Peripheral<P = impl GpioPin> + 'd,
        ch1: impl Peripheral<P = impl GpioPin> + 'd,
    ) -> Self {
        into_ref!(ch0, ch1);
        Self::new_inner(pwm, Some(ch0.map_into()), Some(ch1.map_into()), None, None)
    }

    /// Create a new 3-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_3ch(
        pwm: impl Peripheral<P = T> + 'd,
        ch0: impl Peripheral<P = impl GpioPin> + 'd,
        ch1: impl Peripheral<P = impl GpioPin> + 'd,
        ch2: impl Peripheral<P = impl GpioPin> + 'd,
    ) -> Self {
        unsafe {
            into_ref!(ch0, ch1, ch2);
            Self::new_inner(
                pwm,
                Some(ch0.map_into()),
                Some(ch1.map_into()),
                Some(ch2.map_into()),
                None,
            )
        }
    }

    /// Create a new 4-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_4ch(
        pwm: impl Peripheral<P = T> + 'd,
        ch0: impl Peripheral<P = impl GpioPin> + 'd,
        ch1: impl Peripheral<P = impl GpioPin> + 'd,
        ch2: impl Peripheral<P = impl GpioPin> + 'd,
        ch3: impl Peripheral<P = impl GpioPin> + 'd,
    ) -> Self {
        unsafe {
            into_ref!(ch0, ch1, ch2, ch3);
            Self::new_inner(
                pwm,
                Some(ch0.map_into()),
                Some(ch1.map_into()),
                Some(ch2.map_into()),
                Some(ch3.map_into()),
            )
        }
    }

    fn new_inner(
        _pwm: impl Peripheral<P = T> + 'd,
        ch0: Option<PeripheralRef<'d, AnyPin>>,
        ch1: Option<PeripheralRef<'d, AnyPin>>,
        ch2: Option<PeripheralRef<'d, AnyPin>>,
        ch3: Option<PeripheralRef<'d, AnyPin>>,
    ) -> Self {
        into_ref!(_pwm);

        let r = T::regs();

        if let Some(pin) = &ch0 {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }
        if let Some(pin) = &ch1 {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }
        if let Some(pin) = &ch2 {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }
        if let Some(pin) = &ch3 {
            pin.set_low();
            pin.conf().write(|w| w.dir().output());
        }

        // if NoPin provided writes disconnected (top bit 1) 0x80000000 else
        // writes pin number ex 13 (0x0D) which is connected (top bit 0)
        r.psel.out[0].write(|w| unsafe { w.bits(ch0.psel_bits()) });
        r.psel.out[1].write(|w| unsafe { w.bits(ch1.psel_bits()) });
        r.psel.out[2].write(|w| unsafe { w.bits(ch2.psel_bits()) });
        r.psel.out[3].write(|w| unsafe { w.bits(ch3.psel_bits()) });

        let pwm = Self {
            _peri: _pwm,
            ch0,
            ch1,
            ch2,
            ch3,
            duty: [0; 4],
        };

        // Disable all interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });
        r.shorts.reset();

        // Enable
        r.enable.write(|w| w.enable().enabled());

        r.seq0.ptr.write(|w| unsafe { w.bits((&pwm.duty).as_ptr() as u32) });

        r.seq0.cnt.write(|w| unsafe { w.bits(4) });
        r.seq0.refresh.write(|w| unsafe { w.bits(0) });
        r.seq0.enddelay.write(|w| unsafe { w.bits(0) });

        r.decoder.write(|w| {
            w.load().individual();
            w.mode().refresh_count()
        });
        r.mode.write(|w| w.updown().up());
        r.prescaler.write(|w| w.prescaler().div_16());
        r.countertop.write(|w| unsafe { w.countertop().bits(1000) });
        r.loop_.write(|w| w.cnt().disabled());

        pwm
    }

    /// Enables the PWM generator.
    #[inline(always)]
    pub fn enable(&self) {
        let r = T::regs();
        r.enable.write(|w| w.enable().enabled());
    }

    /// Disables the PWM generator. Does NOT clear the last duty cycle from the pin.
    #[inline(always)]
    pub fn disable(&self) {
        let r = T::regs();
        r.enable.write(|w| w.enable().disabled());
    }

    /// Sets duty cycle (15 bit) for a PWM channel.
    pub fn set_duty(&mut self, channel: usize, duty: u16) {
        let r = T::regs();

        self.duty[channel] = duty & 0x7FFF;

        // reload ptr in case self was moved
        r.seq0.ptr.write(|w| unsafe { w.bits((&self.duty).as_ptr() as u32) });

        // defensive before seqstart
        compiler_fence(Ordering::SeqCst);

        r.events_seqend[0].reset();

        // tasks_seqstart() doesn't exist in all svds so write its bit instead
        r.tasks_seqstart[0].write(|w| unsafe { w.bits(1) });

        // defensive wait until waveform is loaded after seqstart so set_duty
        // can't be called again while dma is still reading
        while r.events_seqend[0].read().bits() == 0 {}
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

impl<'a, T: Instance> Drop for SimplePwm<'a, T> {
    fn drop(&mut self) {
        let r = T::regs();

        self.disable();

        if let Some(pin) = &self.ch0 {
            pin.set_low();
            pin.conf().reset();
            r.psel.out[0].reset();
        }
        if let Some(pin) = &self.ch1 {
            pin.set_low();
            pin.conf().reset();
            r.psel.out[1].reset();
        }
        if let Some(pin) = &self.ch2 {
            pin.set_low();
            pin.conf().reset();
            r.psel.out[2].reset();
        }
        if let Some(pin) = &self.ch3 {
            pin.set_low();
            pin.conf().reset();
            r.psel.out[3].reset();
        }
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> &'static pac::pwm0::RegisterBlock;
    }
}

/// PWM peripheral instance.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_pwm {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::pwm::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::pwm0::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
        }
        impl crate::pwm::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
