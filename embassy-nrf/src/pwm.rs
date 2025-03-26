//! Pulse Width Modulation (PWM) driver.

#![macro_use]

use core::sync::atomic::{compiler_fence, Ordering};

use embassy_hal_internal::{Peri, PeripheralType};

use crate::gpio::{convert_drive, AnyPin, OutputDrive, Pin as GpioPin, PselBits, SealedPin as _, DISCONNECTED};
use crate::pac::gpio::vals as gpiovals;
use crate::pac::pwm::vals;
use crate::ppi::{Event, Task};
use crate::util::slice_in_ram_or;
use crate::{interrupt, pac};

/// SimplePwm is the traditional pwm interface you're probably used to, allowing
/// to simply set a duty cycle across up to four channels.
pub struct SimplePwm<'d, T: Instance> {
    _peri: Peri<'d, T>,
    duty: [u16; 4],
    ch0: Option<Peri<'d, AnyPin>>,
    ch1: Option<Peri<'d, AnyPin>>,
    ch2: Option<Peri<'d, AnyPin>>,
    ch3: Option<Peri<'d, AnyPin>>,
}

/// SequencePwm allows you to offload the updating of a sequence of duty cycles
/// to up to four channels, as well as repeat that sequence n times.
pub struct SequencePwm<'d, T: Instance> {
    _peri: Peri<'d, T>,
    ch0: Option<Peri<'d, AnyPin>>,
    ch1: Option<Peri<'d, AnyPin>>,
    ch2: Option<Peri<'d, AnyPin>>,
    ch3: Option<Peri<'d, AnyPin>>,
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
/// The used pwm clock frequency
pub const PWM_CLK_HZ: u32 = 16_000_000;

impl<'d, T: Instance> SequencePwm<'d, T> {
    /// Create a new 1-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_1ch(pwm: Peri<'d, T>, ch0: Peri<'d, impl GpioPin>, config: Config) -> Result<Self, Error> {
        Self::new_inner(pwm, Some(ch0.into()), None, None, None, config)
    }

    /// Create a new 2-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_2ch(
        pwm: Peri<'d, T>,
        ch0: Peri<'d, impl GpioPin>,
        ch1: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_inner(pwm, Some(ch0.into()), Some(ch1.into()), None, None, config)
    }

    /// Create a new 3-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_3ch(
        pwm: Peri<'d, T>,
        ch0: Peri<'d, impl GpioPin>,
        ch1: Peri<'d, impl GpioPin>,
        ch2: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_inner(pwm, Some(ch0.into()), Some(ch1.into()), Some(ch2.into()), None, config)
    }

    /// Create a new 4-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_4ch(
        pwm: Peri<'d, T>,
        ch0: Peri<'d, impl GpioPin>,
        ch1: Peri<'d, impl GpioPin>,
        ch2: Peri<'d, impl GpioPin>,
        ch3: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Result<Self, Error> {
        Self::new_inner(
            pwm,
            Some(ch0.into()),
            Some(ch1.into()),
            Some(ch2.into()),
            Some(ch3.into()),
            config,
        )
    }

    fn new_inner(
        _pwm: Peri<'d, T>,
        ch0: Option<Peri<'d, AnyPin>>,
        ch1: Option<Peri<'d, AnyPin>>,
        ch2: Option<Peri<'d, AnyPin>>,
        ch3: Option<Peri<'d, AnyPin>>,
        config: Config,
    ) -> Result<Self, Error> {
        let r = T::regs();

        if let Some(pin) = &ch0 {
            pin.set_low();
            pin.conf().write(|w| {
                w.set_dir(gpiovals::Dir::OUTPUT);
                w.set_input(gpiovals::Input::DISCONNECT);
                convert_drive(w, config.ch0_drive);
            });
        }
        if let Some(pin) = &ch1 {
            pin.set_low();
            pin.conf().write(|w| {
                w.set_dir(gpiovals::Dir::OUTPUT);
                w.set_input(gpiovals::Input::DISCONNECT);
                convert_drive(w, config.ch1_drive);
            });
        }
        if let Some(pin) = &ch2 {
            pin.set_low();
            pin.conf().write(|w| {
                w.set_dir(gpiovals::Dir::OUTPUT);
                w.set_input(gpiovals::Input::DISCONNECT);
                convert_drive(w, config.ch2_drive);
            });
        }
        if let Some(pin) = &ch3 {
            pin.set_low();
            pin.conf().write(|w| {
                w.set_dir(gpiovals::Dir::OUTPUT);
                w.set_input(gpiovals::Input::DISCONNECT);
                convert_drive(w, config.ch3_drive);
            });
        }

        r.psel().out(0).write_value(ch0.psel_bits());
        r.psel().out(1).write_value(ch1.psel_bits());
        r.psel().out(2).write_value(ch2.psel_bits());
        r.psel().out(3).write_value(ch3.psel_bits());

        // Disable all interrupts
        r.intenclr().write(|w| w.0 = 0xFFFF_FFFF);
        r.shorts().write(|_| ());
        r.events_stopped().write_value(0);
        r.events_loopsdone().write_value(0);
        r.events_seqend(0).write_value(0);
        r.events_seqend(1).write_value(0);
        r.events_pwmperiodend().write_value(0);
        r.events_seqstarted(0).write_value(0);
        r.events_seqstarted(1).write_value(0);

        r.decoder().write(|w| {
            w.set_load(vals::Load::from_bits(config.sequence_load as u8));
            w.set_mode(vals::Mode::REFRESH_COUNT);
        });

        r.mode().write(|w| match config.counter_mode {
            CounterMode::UpAndDown => w.set_updown(vals::Updown::UP_AND_DOWN),
            CounterMode::Up => w.set_updown(vals::Updown::UP),
        });
        r.prescaler()
            .write(|w| w.set_prescaler(vals::Prescaler::from_bits(config.prescaler as u8)));
        r.countertop().write(|w| w.set_countertop(config.max_duty));

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

        Event::from_reg(r.events_stopped())
    }

    /// Returns reference to `LoopsDone` event endpoint for PPI.
    #[inline(always)]
    pub fn event_loops_done(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(r.events_loopsdone())
    }

    /// Returns reference to `PwmPeriodEnd` event endpoint for PPI.
    #[inline(always)]
    pub fn event_pwm_period_end(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(r.events_pwmperiodend())
    }

    /// Returns reference to `Seq0 End` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq_end(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(r.events_seqend(0))
    }

    /// Returns reference to `Seq1 End` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq1_end(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(r.events_seqend(1))
    }

    /// Returns reference to `Seq0 Started` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq0_started(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(r.events_seqstarted(0))
    }

    /// Returns reference to `Seq1 Started` event endpoint for PPI.
    #[inline(always)]
    pub fn event_seq1_started(&self) -> Event<'d> {
        let r = T::regs();

        Event::from_reg(r.events_seqstarted(1))
    }

    /// Returns reference to `Seq0 Start` task endpoint for PPI.
    /// # Safety
    ///
    /// Interacting with the sequence while it runs puts it in an unknown state
    #[inline(always)]
    pub unsafe fn task_start_seq0(&self) -> Task<'d> {
        let r = T::regs();

        Task::from_reg(r.tasks_seqstart(0))
    }

    /// Returns reference to `Seq1 Started` task endpoint for PPI.
    /// # Safety
    ///
    /// Interacting with the sequence while it runs puts it in an unknown state
    #[inline(always)]
    pub unsafe fn task_start_seq1(&self) -> Task<'d> {
        let r = T::regs();

        Task::from_reg(r.tasks_seqstart(1))
    }

    /// Returns reference to `NextStep` task endpoint for PPI.
    /// # Safety
    ///
    /// Interacting with the sequence while it runs puts it in an unknown state
    #[inline(always)]
    pub unsafe fn task_next_step(&self) -> Task<'d> {
        let r = T::regs();

        Task::from_reg(r.tasks_nextstep())
    }

    /// Returns reference to `Stop` task endpoint for PPI.
    /// # Safety
    ///
    /// Interacting with the sequence while it runs puts it in an unknown state
    #[inline(always)]
    pub unsafe fn task_stop(&self) -> Task<'d> {
        let r = T::regs();

        Task::from_reg(r.tasks_stop())
    }
}

impl<'a, T: Instance> Drop for SequencePwm<'a, T> {
    fn drop(&mut self) {
        let r = T::regs();

        if let Some(pin) = &self.ch0 {
            pin.set_low();
            pin.conf().write(|_| ());
            r.psel().out(0).write_value(DISCONNECTED);
        }
        if let Some(pin) = &self.ch1 {
            pin.set_low();
            pin.conf().write(|_| ());
            r.psel().out(1).write_value(DISCONNECTED);
        }
        if let Some(pin) = &self.ch2 {
            pin.set_low();
            pin.conf().write(|_| ());
            r.psel().out(2).write_value(DISCONNECTED);
        }
        if let Some(pin) = &self.ch3 {
            pin.set_low();
            pin.conf().write(|_| ());
            r.psel().out(3).write_value(DISCONNECTED);
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
    /// Drive strength for the channel 0 line.
    pub ch0_drive: OutputDrive,
    /// Drive strength for the channel 1 line.
    pub ch1_drive: OutputDrive,
    /// Drive strength for the channel 2 line.
    pub ch2_drive: OutputDrive,
    /// Drive strength for the channel 3 line.
    pub ch3_drive: OutputDrive,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            counter_mode: CounterMode::Up,
            max_duty: 1000,
            prescaler: Prescaler::Div16,
            sequence_load: SequenceLoad::Common,
            ch0_drive: OutputDrive::Standard,
            ch1_drive: OutputDrive::Standard,
            ch2_drive: OutputDrive::Standard,
            ch3_drive: OutputDrive::Standard,
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
/// Takes one sequence along with its configuration.
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

        self.stop();

        let r = T::regs();

        r.seq(0).refresh().write(|w| w.0 = sequence0.config.refresh);
        r.seq(0).enddelay().write(|w| w.0 = sequence0.config.end_delay);
        r.seq(0).ptr().write_value(sequence0.words.as_ptr() as u32);
        r.seq(0).cnt().write(|w| w.0 = sequence0.words.len() as u32);

        r.seq(1).refresh().write(|w| w.0 = alt_sequence.config.refresh);
        r.seq(1).enddelay().write(|w| w.0 = alt_sequence.config.end_delay);
        r.seq(1).ptr().write_value(alt_sequence.words.as_ptr() as u32);
        r.seq(1).cnt().write(|w| w.0 = alt_sequence.words.len() as u32);

        r.enable().write(|w| w.set_enable(true));

        // defensive before seqstart
        compiler_fence(Ordering::SeqCst);

        let seqstart_index = if start_seq == StartSequence::One { 1 } else { 0 };

        match times {
            // just the one time, no loop count
            SequenceMode::Loop(_) => {
                r.loop_().write(|w| w.set_cnt(vals::LoopCnt::DISABLED));
            }
            // to play infinitely, repeat the sequence one time, then have loops done self trigger seq0 again
            SequenceMode::Infinite => {
                r.loop_().write(|w| w.set_cnt(vals::LoopCnt::from_bits(1)));
                r.shorts().write(|w| w.set_loopsdone_seqstart0(true));
            }
        }

        r.tasks_seqstart(seqstart_index).write_value(1);

        Ok(())
    }

    /// Stop playback. Disables the peripheral. Does NOT clear the last duty
    /// cycle from the pin. Returns any sequences previously provided to
    /// `start` so that they may be further mutated.
    #[inline(always)]
    pub fn stop(&self) {
        let r = T::regs();

        r.shorts().write(|_| ());

        compiler_fence(Ordering::SeqCst);

        r.tasks_stop().write_value(1);
        r.enable().write(|w| w.set_enable(false));
    }
}

impl<'d, 's, T: Instance> Drop for Sequencer<'d, 's, T> {
    fn drop(&mut self) {
        self.stop();
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
    pub fn new_1ch(pwm: Peri<'d, T>, ch0: Peri<'d, impl GpioPin>) -> Self {
        unsafe { Self::new_inner(pwm, Some(ch0.into()), None, None, None) }
    }

    /// Create a new 2-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_2ch(pwm: Peri<'d, T>, ch0: Peri<'d, impl GpioPin>, ch1: Peri<'d, impl GpioPin>) -> Self {
        Self::new_inner(pwm, Some(ch0.into()), Some(ch1.into()), None, None)
    }

    /// Create a new 3-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_3ch(
        pwm: Peri<'d, T>,
        ch0: Peri<'d, impl GpioPin>,
        ch1: Peri<'d, impl GpioPin>,
        ch2: Peri<'d, impl GpioPin>,
    ) -> Self {
        unsafe { Self::new_inner(pwm, Some(ch0.into()), Some(ch1.into()), Some(ch2.into()), None) }
    }

    /// Create a new 4-channel PWM
    #[allow(unused_unsafe)]
    pub fn new_4ch(
        pwm: Peri<'d, T>,
        ch0: Peri<'d, impl GpioPin>,
        ch1: Peri<'d, impl GpioPin>,
        ch2: Peri<'d, impl GpioPin>,
        ch3: Peri<'d, impl GpioPin>,
    ) -> Self {
        unsafe {
            Self::new_inner(
                pwm,
                Some(ch0.into()),
                Some(ch1.into()),
                Some(ch2.into()),
                Some(ch3.into()),
            )
        }
    }

    fn new_inner(
        _pwm: Peri<'d, T>,
        ch0: Option<Peri<'d, AnyPin>>,
        ch1: Option<Peri<'d, AnyPin>>,
        ch2: Option<Peri<'d, AnyPin>>,
        ch3: Option<Peri<'d, AnyPin>>,
    ) -> Self {
        let r = T::regs();

        for (i, ch) in [&ch0, &ch1, &ch2, &ch3].into_iter().enumerate() {
            if let Some(pin) = ch {
                pin.set_low();

                pin.conf().write(|w| {
                    w.set_dir(gpiovals::Dir::OUTPUT);
                    w.set_input(gpiovals::Input::DISCONNECT);
                    w.set_drive(gpiovals::Drive::S0S1);
                });
            }
            r.psel().out(i).write_value(ch.psel_bits());
        }

        let pwm = Self {
            _peri: _pwm,
            ch0,
            ch1,
            ch2,
            ch3,
            duty: [0; 4],
        };

        // Disable all interrupts
        r.intenclr().write(|w| w.0 = 0xFFFF_FFFF);
        r.shorts().write(|_| ());

        // Enable
        r.enable().write(|w| w.set_enable(true));

        r.seq(0).ptr().write_value((pwm.duty).as_ptr() as u32);
        r.seq(0).cnt().write(|w| w.0 = 4);
        r.seq(0).refresh().write(|w| w.0 = 0);
        r.seq(0).enddelay().write(|w| w.0 = 0);

        r.decoder().write(|w| {
            w.set_load(vals::Load::INDIVIDUAL);
            w.set_mode(vals::Mode::REFRESH_COUNT);
        });
        r.mode().write(|w| w.set_updown(vals::Updown::UP));
        r.prescaler().write(|w| w.set_prescaler(vals::Prescaler::DIV_16));
        r.countertop().write(|w| w.set_countertop(1000));
        r.loop_().write(|w| w.set_cnt(vals::LoopCnt::DISABLED));

        pwm
    }

    /// Returns the enable state of the pwm counter
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        let r = T::regs();
        r.enable().read().enable()
    }

    /// Enables the PWM generator.
    #[inline(always)]
    pub fn enable(&self) {
        let r = T::regs();
        r.enable().write(|w| w.set_enable(true));
    }

    /// Disables the PWM generator. Does NOT clear the last duty cycle from the pin.
    #[inline(always)]
    pub fn disable(&self) {
        let r = T::regs();
        r.enable().write(|w| w.set_enable(false));
    }

    /// Returns the current duty of the channel
    pub fn duty(&self, channel: usize) -> u16 {
        self.duty[channel]
    }

    /// Sets duty cycle (15 bit) for a PWM channel.
    pub fn set_duty(&mut self, channel: usize, duty: u16) {
        let r = T::regs();

        self.duty[channel] = duty & 0x7FFF;

        // reload ptr in case self was moved
        r.seq(0).ptr().write_value((self.duty).as_ptr() as u32);

        // defensive before seqstart
        compiler_fence(Ordering::SeqCst);

        r.events_seqend(0).write_value(0);

        // tasks_seqstart() doesn't exist in all svds so write its bit instead
        r.tasks_seqstart(0).write_value(1);

        // defensive wait until waveform is loaded after seqstart so set_duty
        // can't be called again while dma is still reading
        if self.is_enabled() {
            while r.events_seqend(0).read() == 0 {}
        }
    }

    /// Sets the PWM clock prescaler.
    #[inline(always)]
    pub fn set_prescaler(&self, div: Prescaler) {
        T::regs()
            .prescaler()
            .write(|w| w.set_prescaler(vals::Prescaler::from_bits(div as u8)));
    }

    /// Gets the PWM clock prescaler.
    #[inline(always)]
    pub fn prescaler(&self) -> Prescaler {
        match T::regs().prescaler().read().prescaler().to_bits() {
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
        T::regs().countertop().write(|w| w.set_countertop(duty.min(32767u16)));
    }

    /// Returns the maximum duty cycle value.
    #[inline(always)]
    pub fn max_duty(&self) -> u16 {
        T::regs().countertop().read().countertop()
    }

    /// Sets the PWM output frequency.
    #[inline(always)]
    pub fn set_period(&self, freq: u32) {
        let clk = PWM_CLK_HZ >> (self.prescaler() as u8);
        let duty = clk / freq;
        self.set_max_duty(duty.min(32767) as u16);
    }

    /// Returns the PWM output frequency.
    #[inline(always)]
    pub fn period(&self) -> u32 {
        let clk = PWM_CLK_HZ >> (self.prescaler() as u8);
        let max_duty = self.max_duty() as u32;
        clk / max_duty
    }

    /// Sets the PWM-Channel0 output drive strength
    #[inline(always)]
    pub fn set_ch0_drive(&self, drive: OutputDrive) {
        if let Some(pin) = &self.ch0 {
            pin.conf().modify(|w| convert_drive(w, drive));
        }
    }

    /// Sets the PWM-Channel1 output drive strength
    #[inline(always)]
    pub fn set_ch1_drive(&self, drive: OutputDrive) {
        if let Some(pin) = &self.ch1 {
            pin.conf().modify(|w| convert_drive(w, drive));
        }
    }

    /// Sets the PWM-Channel2 output drive strength
    #[inline(always)]
    pub fn set_ch2_drive(&self, drive: OutputDrive) {
        if let Some(pin) = &self.ch2 {
            pin.conf().modify(|w| convert_drive(w, drive));
        }
    }

    /// Sets the PWM-Channel3 output drive strength
    #[inline(always)]
    pub fn set_ch3_drive(&self, drive: OutputDrive) {
        if let Some(pin) = &self.ch3 {
            pin.conf().modify(|w| convert_drive(w, drive));
        }
    }
}

impl<'a, T: Instance> Drop for SimplePwm<'a, T> {
    fn drop(&mut self) {
        let r = T::regs();

        self.disable();

        if let Some(pin) = &self.ch0 {
            pin.set_low();
            pin.conf().write(|_| ());
            r.psel().out(0).write_value(DISCONNECTED);
        }
        if let Some(pin) = &self.ch1 {
            pin.set_low();
            pin.conf().write(|_| ());
            r.psel().out(1).write_value(DISCONNECTED);
        }
        if let Some(pin) = &self.ch2 {
            pin.set_low();
            pin.conf().write(|_| ());
            r.psel().out(2).write_value(DISCONNECTED);
        }
        if let Some(pin) = &self.ch3 {
            pin.set_low();
            pin.conf().write(|_| ());
            r.psel().out(3).write_value(DISCONNECTED);
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::pwm::Pwm;
}

/// PWM peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_pwm {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::pwm::SealedInstance for peripherals::$type {
            fn regs() -> pac::pwm::Pwm {
                pac::$pac_type
            }
        }
        impl crate::pwm::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
