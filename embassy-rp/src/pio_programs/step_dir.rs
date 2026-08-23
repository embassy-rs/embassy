//! PIO-backed Step/Dir pulse generator for stepper motor drivers.
//!
//! This module targets drivers that expect a STEP pulse input and a DIR level input,
//! such as TMC2209 and similar.

use core::mem::{self, MaybeUninit};

use crate::gpio::{Level, Output, Pin as GpioPin};
use crate::pio::{Common, Config, Direction, Instance, Irq, LoadedProgram, Pin as PioPinHandle, PioPin, StateMachine};
use crate::pio_programs::clock_divider::calculate_pio_clock_divider;
use crate::{Peri, clocks};

/// Default pulse frequency used during initialization.
const DEFAULT_FREQUENCY_HZ: u32 = 100;

/// Configure the STEP pulse timing (cycles per pulse).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepPulseTiming {
    /// 34 cycles high + 34 cycles low + 2 jump cycles = 68 total.
    Cycles68,
    /// 66 cycles high + 66 cycles low + 2 jump cycles = 132 total.
    Cycles132,
    /// 130 cycles high + 130 cycles low + 2 jump cycles = 260 total.
    Cycles260,
}

impl Default for StepPulseTiming {
    fn default() -> Self {
        StepPulseTiming::Cycles68
    }
}

impl StepPulseTiming {
    fn cycles_per_pulse(self) -> u32 {
        match self {
            StepPulseTiming::Cycles68 => 68,
            StepPulseTiming::Cycles132 => 132,
            StepPulseTiming::Cycles260 => 260,
        }
    }
}

/// Program loaded into PIO instruction memory for STEP pulse generation.
pub struct PioStepDirProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
    cycles_per_pulse: u32,
}

impl<'a, PIO: Instance> PioStepDirProgram<'a, PIO> {
    /// Load the STEP pulse program into the given PIO using the default timing.
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        Self::new_with_timing(common, StepPulseTiming::default())
    }

    /// Load the STEP pulse program into the given PIO using the selected timing.
    pub fn new_with_timing(common: &mut Common<'a, PIO>, timing: StepPulseTiming) -> Self {
        let (prg, cycles_per_pulse) = match timing {
            StepPulseTiming::Cycles68 => {
                let prg = pio::pio_asm!(
                    "pull block        ; wait for step count word",
                    "mov x, osr         ; X = steps",
                    "loop:",
                    "jmp !x end         ; if 0 steps, finish",
                    "set pins, 1 [31]   ; STEP high, hold for 32 cycles",
                    "nop [1]            ; extend high by 2 cycles",
                    "set pins, 0 [31]   ; STEP low, hold for 32 cycles",
                    "jmp x-- loop       ; decrement steps and loop",
                    "end:",
                    "irq 0 rel          ; signal completion"
                );
                (common.load_program(&prg.program), timing.cycles_per_pulse())
            }
            StepPulseTiming::Cycles132 => {
                let prg = pio::pio_asm!(
                    "pull block        ; wait for step count word",
                    "mov x, osr         ; X = steps",
                    "loop:",
                    "jmp !x end         ; if 0 steps, finish",
                    "set pins, 1 [31]   ; STEP high, hold for 32 cycles",
                    "nop [31]           ; extend high by 32 cycles",
                    "nop [1]            ; extend high by 2 cycles",
                    "set pins, 0 [31]   ; STEP low, hold for 32 cycles",
                    "nop [31]           ; extend low by 32 cycles",
                    "jmp x-- loop       ; decrement steps and loop",
                    "end:",
                    "irq 0 rel          ; signal completion"
                );
                (common.load_program(&prg.program), timing.cycles_per_pulse())
            }
            StepPulseTiming::Cycles260 => {
                let prg = pio::pio_asm!(
                    "pull block        ; wait for step count word",
                    "mov x, osr         ; X = steps",
                    "loop:",
                    "jmp !x end         ; if 0 steps, finish",
                    "set pins, 1 [31]   ; STEP high, hold for 32 cycles",
                    "nop [31]           ; extend high by 32 cycles",
                    "nop [31]           ; extend high by 32 cycles",
                    "nop [31]           ; extend high by 32 cycles",
                    "nop [1]            ; extend high by 2 cycles",
                    "set pins, 0 [31]   ; STEP low, hold for 32 cycles",
                    "nop [31]           ; extend low by 32 cycles",
                    "nop [31]           ; extend low by 32 cycles",
                    "nop [31]           ; extend low by 32 cycles",
                    "jmp x-- loop       ; decrement steps and loop",
                    "end:",
                    "irq 0 rel          ; signal completion"
                );
                (common.load_program(&prg.program), timing.cycles_per_pulse())
            }
        };

        Self { prg, cycles_per_pulse }
    }

    /// Return the cycles per pulse for this program.
    pub fn cycles_per_pulse(&self) -> u32 {
        self.cycles_per_pulse
    }
}

/// Direction for motion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepDir {
    /// Forward direction.
    Forward,
    /// Reverse direction.
    Reverse,
}

/// Errors returned when setting the STEP pulse frequency.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepDirFrequencyError {
    /// Frequency was zero.
    Zero,
    /// Frequency is below the minimum supported.
    TooLow {
        /// The minimum supported frequency in Hz.
        min_hz: u32,
    },
    /// Frequency exceeds the maximum supported.
    TooHigh {
        /// The maximum supported frequency in Hz.
        max_hz: u32,
    },
}

/// PIO-backed Step/Dir stepper pulse generator.
///
/// One pin is driven by PIO for STEP pulses, and one GPIO pin is controlled by software for DIR.
pub struct PioStepDir<'d, T: Instance, const SM: usize> {
    irq: Irq<'d, T, SM>,
    sm: StateMachine<'d, T, SM>,
    step: PioPinHandle<'d, T>,
    dir: Output<'d>,
    frequency_hz: u32,
    cycles_per_pulse: u32,
}

impl<'d, T: Instance, const SM: usize> PioStepDir<'d, T, SM> {
    /// Configure a state machine to generate STEP pulses and bind a DIR pin.
    pub fn new(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        irq: Irq<'d, T, SM>,
        step_pin: Peri<'d, impl PioPin>,
        dir_pin: Peri<'d, impl GpioPin>,
        program: &PioStepDirProgram<'d, T>,
    ) -> Self {
        let step = pio.make_pio_pin(step_pin);
        sm.set_pins(Level::Low, &[&step]);
        sm.set_pin_dirs(Direction::Out, &[&step]);

        let mut cfg = Config::default();
        cfg.set_set_pins(&[&step]);
        cfg.clock_divider = calculate_pio_clock_divider(DEFAULT_FREQUENCY_HZ * program.cycles_per_pulse());
        cfg.use_program(&program.prg, &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);

        let dir = Output::new(dir_pin, Level::Low);

        Self {
            irq,
            sm,
            step,
            dir,
            frequency_hz: DEFAULT_FREQUENCY_HZ,
            cycles_per_pulse: program.cycles_per_pulse(),
        }
    }

    /// Return the minimum supported pulse frequency in Hz for this program.
    pub fn min_frequency(&self) -> u32 {
        Self::calc_min_frequency(self.cycles_per_pulse)
    }

    fn calc_min_frequency(cycles_per_pulse: u32) -> u32 {
        let max_clkdiv_int: u32 = 0xffff;
        let min_pio_hz = (clocks::clk_sys_freq() + max_clkdiv_int - 1) / max_clkdiv_int;
        (min_pio_hz + cycles_per_pulse - 1) / cycles_per_pulse
    }

    /// Set output pulse frequency in Hz.
    pub fn set_frequency(&mut self, freq_hz: u32) -> Result<(), StepDirFrequencyError> {
        if freq_hz == 0 {
            return Err(StepDirFrequencyError::Zero);
        }
        let min_freq = Self::calc_min_frequency(self.cycles_per_pulse);
        if freq_hz < min_freq {
            return Err(StepDirFrequencyError::TooLow { min_hz: min_freq });
        }
        let max_freq = (clocks::clk_sys_freq() as u64 / self.cycles_per_pulse as u64) as u32;
        if freq_hz > max_freq {
            return Err(StepDirFrequencyError::TooHigh { max_hz: max_freq });
        }

        self.frequency_hz = freq_hz;
        let clock_divider = calculate_pio_clock_divider(freq_hz * self.cycles_per_pulse);
        self.sm.set_clock_divider(clock_divider);
        self.sm.clkdiv_restart();
        Ok(())
    }

    /// Get the currently configured pulse frequency in Hz.
    pub fn frequency(&self) -> u32 {
        self.frequency_hz
    }

    /// Set motion direction.
    pub fn set_direction(&mut self, direction: StepDir) {
        match direction {
            StepDir::Forward => self.dir.set_low(),
            StepDir::Reverse => self.dir.set_high(),
        }
    }

    /// Generate an exact number of STEP pulses in the given direction and wait until completion.
    pub async fn move_with_dir(&mut self, steps: u32, direction: StepDir) {
        self.set_direction(direction);
        self.move_steps(steps).await;
    }

    /// Generate an exact number of STEP pulses and wait until completion.
    ///
    /// If this future is dropped before completion (e.g. cancelled via `select!`),
    /// the PIO state machine is automatically reset and the STEP pin is driven low.
    pub async fn move_steps(&mut self, steps: u32) {
        if steps == 0 {
            return;
        }

        self.sm.tx().wait_push(steps).await;
        let drop = OnDrop::new(|| {
            self.sm.clear_fifos();
            unsafe {
                self.sm.exec_instr(
                    pio::InstructionOperands::JMP {
                        address: 0,
                        condition: pio::JmpCondition::Always,
                    }
                    .encode(),
                );
            }
            self.sm.set_pins(Level::Low, &[&self.step]);
        });
        self.irq.wait().await;
        drop.defuse();
    }

    /// Stop pulse generation immediately and clear the state machine FIFOs.
    pub fn stop(&mut self) {
        self.sm.clear_fifos();
        unsafe {
            self.sm.exec_instr(
                pio::InstructionOperands::JMP {
                    address: 0,
                    condition: pio::JmpCondition::Always,
                }
                .encode(),
            );
        }
        self.sm.set_pins(Level::Low, &[&self.step]);
    }

    /// Release owned resources.
    pub fn release(self) -> (Irq<'d, T, SM>, StateMachine<'d, T, SM>, PioPinHandle<'d, T>, Output<'d>) {
        (self.irq, self.sm, self.step, self.dir)
    }
}

struct OnDrop<F: FnOnce()> {
    f: MaybeUninit<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    pub fn new(f: F) -> Self {
        Self { f: MaybeUninit::new(f) }
    }

    pub fn defuse(self) {
        mem::forget(self)
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        unsafe { self.f.as_ptr().read()() }
    }
}
