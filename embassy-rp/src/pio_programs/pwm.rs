//! PIO backed PWM driver

use core::time::Duration;

use pio::InstructionOperands;

use crate::gpio::Level;
use crate::pio::{Common, Config, Direction, Instance, LoadedProgram, Pin, PioPin, StateMachine};
use crate::{clocks, Peri};

/// This converts the duration provided into the number of cycles the PIO needs to run to make it take the same time
fn to_pio_cycles(duration: Duration) -> u32 {
    (clocks::clk_sys_freq() / 1_000_000) / 3 * duration.as_micros() as u32 // parentheses are required to prevent overflow
}

/// This struct represents a PWM program loaded into pio instruction memory.
pub struct PioPwmProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> PioPwmProgram<'a, PIO> {
    /// Load the program into the given pio
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio::pio_asm!(
            ".side_set 1 opt"
                "pull noblock    side 0"
                "mov x, osr"
                "mov y, isr"
            "countloop:"
                "jmp x!=y noset"
                "jmp skip        side 1"
            "noset:"
                "nop"
            "skip:"
                "jmp y-- countloop"
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

/// Pio backed PWM output
pub struct PioPwm<'d, T: Instance, const SM: usize> {
    sm: StateMachine<'d, T, SM>,
    pin: Pin<'d, T>,
}

impl<'d, T: Instance, const SM: usize> PioPwm<'d, T, SM> {
    /// Configure a state machine as a PWM output
    pub fn new(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        pin: Peri<'d, impl PioPin>,
        program: &PioPwmProgram<'d, T>,
    ) -> Self {
        let pin = pio.make_pio_pin(pin);
        sm.set_pins(Level::High, &[&pin]);
        sm.set_pin_dirs(Direction::Out, &[&pin]);

        let mut cfg = Config::default();
        cfg.use_program(&program.prg, &[&pin]);

        sm.set_config(&cfg);

        Self { sm, pin }
    }

    /// Enable's the PIO program, continuing the wave generation from the PIO program.
    pub fn start(&mut self) {
        self.sm.set_enable(true);
    }

    /// Stops the PIO program, ceasing all signals from the PIN that were generated via PIO.
    pub fn stop(&mut self) {
        self.sm.set_enable(false);
    }

    /// Sets the pwm period, which is the length of time for each pio wave until reset.
    pub fn set_period(&mut self, duration: Duration) {
        let is_enabled = self.sm.is_enabled();
        while !self.sm.tx().empty() {} // Make sure that the queue is empty
        self.sm.set_enable(false);
        self.sm.tx().push(to_pio_cycles(duration));
        unsafe {
            self.sm.exec_instr(
                InstructionOperands::PULL {
                    if_empty: false,
                    block: false,
                }
                .encode(),
            );
            self.sm.exec_instr(
                InstructionOperands::OUT {
                    destination: ::pio::OutDestination::ISR,
                    bit_count: 32,
                }
                .encode(),
            );
        };
        if is_enabled {
            self.sm.set_enable(true) // Enable if previously enabled
        }
    }

    /// Set the number of pio cycles to set the wave on high to.
    pub fn set_level(&mut self, level: u32) {
        self.sm.tx().push(level);
    }

    /// Set the pulse width high time
    pub fn write(&mut self, duration: Duration) {
        self.set_level(to_pio_cycles(duration));
    }

    /// Return the state machine and pin.
    pub fn release(self) -> (StateMachine<'d, T, SM>, Pin<'d, T>) {
        (self.sm, self.pin)
    }
}
