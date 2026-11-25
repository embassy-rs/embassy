//! PIO backed quadrature encoder

use crate::Peri;
use crate::gpio::Pull;
use crate::pio::{
    Common, Config, Direction as PioDirection, FifoJoin, Instance, LoadedProgram, PioPin, ShiftDirection, StateMachine,
};
use crate::pio_programs::clock_divider::calculate_pio_clock_divider_x256;

/// This struct represents an Encoder program loaded into pio instruction memory.
pub struct PioEncoderProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> PioEncoderProgram<'a, PIO> {
    /// Load the program into the given pio
    ///
    /// Set `alternating` to `false` for the common encoders whose rest
    /// position is `11` on every step (full quadrature cycle per step).
    ///
    /// Set `alternating` to `true` for encoders whose rest position
    /// alternates between `11` and `00` from one step to the next
    /// (half quadrature cycle per step).
    ///
    /// Note: in alternating mode, if the encoder rests between two
    /// positions at startup, its first movement may report a spurious step.
    pub fn new(common: &mut Common<'a, PIO>, alternating: bool) -> Self {
        let prg = if !alternating {
            let prg = pio::pio_asm!(
                // reset count. if it was odd, we report a step
                "complete:",
                "mov y, isr",
                "mov isr, null",
                "jmp !y, invalid",
                "in x 1",
                "push noblock",
                "invalid:",
                // loop while pins are [11] (home state). outputs x
                "while_11:",
                "mov osr, ~ pins",
                "out x, 2",
                "jmp !x, while_11", // pins == [11]
                // then, count how many [01] and [10] states come after that one
                // (ignoring [00]) until we see [11] again
                ".wrap_target",
                "mov osr, ~ x",
                "out x, 2", // decide next state to search for
                "until_x:",
                "mov osr, ~ pins",
                "out y, 2",
                "jmp !y, complete", // pins == [11]
                "jmp x!=y until_x",
                // if pins matched next state:
                "mov isr, ~ isr", // count even/odd flag
                ".wrap",
            );
            common.load_program(&prg.program)
        } else {
            // Each step moves between the two rest states [11] and [00],
            // passing through exactly one of the middle states [01] / [10].
            // The program has two symmetric halves, one per rest state.
            // The [11] half reads `~pins`, the [00] half reads `pins`, so in
            // both halves x == 0 means "still at the starting rest state",
            // x == 3 means "reached the other rest state", and bit0 of a
            // middle state encodes the direction (0 = CW, 1 = CCW).
            // The last middle state seen is kept in ISR and pushed once the
            // other rest state is reached. Returning to the starting rest
            // state (bounce) reports nothing.
            let prg = pio::pio_asm!(
                "set y, 3", // constant: "other rest state"
                ".wrap_target",
                // resting at [11]
                "wait_11:",
                "mov osr, ~ pins",
                "out x, 2",
                "jmp !x, wait_11",  // pins == [11]
                "jmp x!=y, mid_11", // pins == [01] or [10]
                "jmp wait_00",      // jumped [11] -> [00] directly: resync, no step
                "mid_11:",
                "mov isr, x", // remember last middle state
                "mov osr, ~ pins",
                "out x, 2",
                "jmp !x, wait_11",  // bounced back to [11]: no step
                "jmp x!=y, mid_11", // still a middle state
                "push noblock",     // reached [00]: report step
                // resting at [00]
                "wait_00:",
                "mov osr, pins",
                "out x, 2",
                "jmp !x, wait_00",  // pins == [00]
                "jmp x!=y, mid_00", // pins == [01] or [10]
                "jmp wait_11",      // jumped [00] -> [11] directly: resync, no step
                "mid_00:",
                "mov isr, x", // remember last middle state
                "mov osr, pins",
                "out x, 2",
                "jmp !x, wait_00",  // bounced back to [00]: no step
                "jmp x!=y, mid_00", // still a middle state
                "push noblock",     // reached [11]: report step
                ".wrap",
            );
            common.load_program(&prg.program)
        };

        Self { prg }
    }
}

/// Pio Backed quadrature encoder reader
pub struct PioEncoder<'d, T: Instance, const SM: usize> {
    sm: StateMachine<'d, T, SM>,
}

impl<'d, T: Instance, const SM: usize> PioEncoder<'d, T, SM> {
    /// Configure a state machine with the loaded [PioEncoderProgram]
    pub fn new(
        pio: &mut Common<'d, T>,
        mut sm: StateMachine<'d, T, SM>,
        pin_a: Peri<'d, impl PioPin>,
        pin_b: Peri<'d, impl PioPin>,
        program: &PioEncoderProgram<'d, T>,
    ) -> Self {
        let mut pin_a = pio.make_pio_pin(pin_a);
        let mut pin_b = pio.make_pio_pin(pin_b);
        pin_a.set_pull(Pull::Up);
        pin_b.set_pull(Pull::Up);

        let mut cfg = Config::default();
        cfg.set_in_pins(&[&pin_a, &pin_b]);
        cfg.fifo_join = FifoJoin::RxOnly;
        cfg.shift_in.direction = ShiftDirection::Left;

        // Target 12 MHz PIO clock
        cfg.clock_divider_x256 = calculate_pio_clock_divider_x256(12_000_000);

        cfg.use_program(&program.prg, &[]);
        sm.set_config(&cfg);
        sm.set_pin_dirs(PioDirection::In, &[&pin_a, &pin_b]);
        sm.set_enable(true);
        Self { sm }
    }
    /// Change rotary encoder target frequency at runtime
    pub fn set_target_frequency(&mut self, target_frequency: u32) {
        let clock_divider = calculate_pio_clock_divider_x256(target_frequency);
        self.sm.set_enable(false);
        self.sm.clear_fifos();
        self.sm.restart();
        self.sm.set_clock_divider_x256(clock_divider);
        self.sm.set_enable(true);
    }

    /// Read a single count from the encoder
    pub async fn read(&mut self) -> Direction {
        // The default program pushes 0/1, the alternating program pushes
        // the raw middle state (1/2). In both cases bit0 is the direction.
        match self.sm.rx().wait_pull().await & 1 {
            0 => Direction::Clockwise,
            _ => Direction::CounterClockwise,
        }
    }
}

/// Encoder Count Direction
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Encoder turned clockwise
    Clockwise,
    /// Encoder turned counter clockwise
    CounterClockwise,
}
