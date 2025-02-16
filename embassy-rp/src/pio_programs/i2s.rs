//! Pio backed I2s output

use fixed::traits::ToFixed;

use crate::dma::{AnyChannel, Channel, Transfer};
use crate::pio::{
    Common, Config, Direction, FifoJoin, Instance, LoadedProgram, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use crate::{into_ref, Peripheral, PeripheralRef};

/// This struct represents an i2s output driver program
pub struct PioI2sOutProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> PioI2sOutProgram<'a, PIO> {
    /// Load the program into the given pio
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
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
pub struct PioI2sOut<'a, P: Instance, const S: usize> {
    dma: PeripheralRef<'a, AnyChannel>,
    sm: StateMachine<'a, P, S>,
}

impl<'a, P: Instance, const S: usize> PioI2sOut<'a, P, S> {
    /// Configure a state machine to output I2s
    pub fn new(
        common: &mut Common<'a, P>,
        mut sm: StateMachine<'a, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'a,
        data_pin: impl PioPin,
        bit_clock_pin: impl PioPin,
        lr_clock_pin: impl PioPin,
        sample_rate: u32,
        bit_depth: u32,
        channels: u32,
        program: &PioI2sOutProgram<'a, P>,
    ) -> Self {
        into_ref!(dma);

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

        Self {
            dma: dma.map_into(),
            sm,
        }
    }

    /// Return an in-prograss dma transfer future. Awaiting it will guarentee a complete transfer.
    pub fn write<'b>(&'b mut self, buff: &'b [u32]) -> Transfer<'b, AnyChannel> {
        self.sm.tx().dma_push(self.dma.reborrow(), buff)
    }
}
