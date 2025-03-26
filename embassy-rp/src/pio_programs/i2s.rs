//! Pio backed I2s output

use fixed::traits::ToFixed;

use crate::dma::{AnyChannel, Channel, Transfer};
use crate::pio::{
    Common, Config, Direction, FifoJoin, Instance, LoadedProgram, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use crate::Peri;

/// This struct represents an i2s output driver program
pub struct PioI2sOutProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
}

impl<'d, PIO: Instance> PioI2sOutProgram<'d, PIO> {
    /// Load the program into the given pio
    pub fn new(common: &mut Common<'d, PIO>) -> Self {
        let prg = pio::pio_asm!(
            ".side_set 2",
            "    set x, 14          side 0b01", // side 0bWB - W = Word Clock, B = Bit Clock
            "left_data:",
            "    out pins, 1        side 0b00",
            "    jmp x-- left_data  side 0b01",
            "    out pins 1         side 0b10",
            "    set x, 14          side 0b11",
            "right_data:",
            "    out pins 1         side 0b10",
            "    jmp x-- right_data side 0b11",
            "    out pins 1         side 0b00",
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

/// Pio backed I2s output driver
pub struct PioI2sOut<'d, P: Instance, const S: usize> {
    dma: Peri<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
}

impl<'d, P: Instance, const S: usize> PioI2sOut<'d, P, S> {
    /// Configure a state machine to output I2s
    pub fn new(
        common: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: Peri<'d, impl Channel>,
        data_pin: Peri<'d, impl PioPin>,
        bit_clock_pin: Peri<'d, impl PioPin>,
        lr_clock_pin: Peri<'d, impl PioPin>,
        sample_rate: u32,
        bit_depth: u32,
        channels: u32,
        program: &PioI2sOutProgram<'d, P>,
    ) -> Self {
        let data_pin = common.make_pio_pin(data_pin);
        let bit_clock_pin = common.make_pio_pin(bit_clock_pin);
        let left_right_clock_pin = common.make_pio_pin(lr_clock_pin);

        let cfg = {
            let mut cfg = Config::default();
            cfg.use_program(&program.prg, &[&bit_clock_pin, &left_right_clock_pin]);
            cfg.set_out_pins(&[&data_pin]);
            let clock_frequency = sample_rate * bit_depth * channels;
            cfg.clock_divider = (crate::clocks::clk_sys_freq() as f64 / clock_frequency as f64 / 2.).to_fixed();
            cfg.shift_out = ShiftConfig {
                threshold: 32,
                direction: ShiftDirection::Left,
                auto_fill: true,
            };
            // join fifos to have twice the time to start the next dma transfer
            cfg.fifo_join = FifoJoin::TxOnly;
            cfg
        };
        sm.set_config(&cfg);
        sm.set_pin_dirs(Direction::Out, &[&data_pin, &left_right_clock_pin, &bit_clock_pin]);

        sm.set_enable(true);

        Self { dma: dma.into(), sm }
    }

    /// Return an in-prograss dma transfer future. Awaiting it will guarentee a complete transfer.
    pub fn write<'b>(&'b mut self, buff: &'b [u32]) -> Transfer<'b, AnyChannel> {
        self.sm.tx().dma_push(self.dma.reborrow(), buff, false)
    }
}
