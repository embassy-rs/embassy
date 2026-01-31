//! Pio backed I2S output and output drivers

use crate::dma::Transfer;
use crate::gpio::Pull;
use crate::pio::{
    Common, Config, Direction, FifoJoin, Instance, LoadedProgram, PioBatch, PioPin, ShiftConfig, ShiftDirection,
    StateMachine,
};
use crate::pio_programs::clock_divider::calculate_pio_clock_divider;
use crate::{Peri, dma, interrupt};

/// This struct represents an I2S receiver & controller driver program
pub struct PioI2sInProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
}

const PIO_I2S_IN_PROGRAM_CLK_MULTIPLIER: u32 = 2;

impl<'d, PIO: Instance> PioI2sInProgram<'d, PIO> {
    /// Load the input program into the given pio
    pub fn new(common: &mut Common<'d, PIO>) -> Self {
        let prg = pio::pio_asm! {
            ".side_set 2",
            "   set x, 14               side 0b01",
            "left_data:",
            "   in pins, 1              side 0b00", // read one left-channel bit from SD
            "   jmp x-- left_data       side 0b01",
            "   in pins, 1              side 0b10", // ws changes 1 clock before MSB
            "   set x, 14               side 0b11",
            "right_data:",
            "   in pins, 1             side 0b10",
            "   jmp x-- right_data     side 0b11",
            "   in pins, 1             side 0b00" // ws changes 1 clock before ms
        };
        let prg = common.load_program(&prg.program);
        Self { prg }
    }
}

/// Pio backed I2S input driver
pub struct PioI2sIn<'d, P: Instance, const S: usize> {
    dma: dma::Channel<'d>,
    sm: StateMachine<'d, P, S>,
}

impl<'d, P: Instance, const S: usize> PioI2sIn<'d, P, S> {
    /// Configure a state machine to act as both the controller (provider of SCK and WS) and receiver (of SD) for an I2S signal
    pub fn new<D: dma::ChannelInstance>(
        common: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: Peri<'d, D>,
        irq: impl interrupt::typelevel::Binding<D::Interrupt, dma::InterruptHandler<D>> + 'd,
        // Whether or not to use the MCU's internal pull-down resistor, as the
        // Pico 2 is known to have problems with the inbuilt pulldowns, many
        // opt to just use an external pull down resistor to meet requirements of common
        // I2S microphones such as the INMP441
        data_pulldown: bool,
        data_pin: Peri<'d, impl PioPin>,
        bit_clock_pin: Peri<'d, impl PioPin>,
        lr_clock_pin: Peri<'d, impl PioPin>,
        sample_rate: u32,
        bit_depth: u32,
        channels: u32,
        program: &PioI2sInProgram<'d, P>,
    ) -> Self {
        let mut data_pin = common.make_pio_pin(data_pin);
        if data_pulldown {
            data_pin.set_pull(Pull::Down);
        }
        let bit_clock_pin = common.make_pio_pin(bit_clock_pin);
        let lr_clock_pin = common.make_pio_pin(lr_clock_pin);

        let cfg = {
            let mut cfg = Config::default();
            cfg.use_program(&program.prg, &[&bit_clock_pin, &lr_clock_pin]);
            cfg.set_in_pins(&[&data_pin]);
            let clock_frequency = sample_rate * bit_depth * channels;
            cfg.clock_divider = calculate_pio_clock_divider(clock_frequency * PIO_I2S_IN_PROGRAM_CLK_MULTIPLIER);
            cfg.shift_in = ShiftConfig {
                threshold: 32,
                direction: ShiftDirection::Left,
                auto_fill: true,
            };
            // join fifos to have twice the time to start the next dma transfer
            cfg.fifo_join = FifoJoin::RxOnly; // both control signals are sent via side-setting
            cfg
        };
        sm.set_config(&cfg);
        sm.set_pin_dirs(Direction::In, &[&data_pin]);
        sm.set_pin_dirs(Direction::Out, &[&lr_clock_pin, &bit_clock_pin]);

        Self {
            dma: dma::Channel::new(dma, irq),
            sm,
        }
    }

    /// Start the i2s interface
    pub fn start(&mut self) {
        self.sm.set_enable(true);
    }

    /// Stop the i2s interface
    pub fn stop(&mut self) {
        self.sm.set_enable(true);
    }

    /// Start at the the same as other drivers
    pub fn start_batched(&mut self, b: &mut PioBatch<'d, P>) {
        b.set_enable(&mut self.sm, true);
    }

    /// Stop at the the same as other drivers
    pub fn stop_batched(&mut self, b: &mut PioBatch<'d, P>) {
        b.set_enable(&mut self.sm, false);
    }

    /// Return an in-progress dma transfer future. Awaiting it will guarantee a complete transfer.
    pub fn read<'b>(&'b mut self, buff: &'b mut [u32]) -> Transfer<'b> {
        self.sm.rx().dma_pull(&mut self.dma, buff, false)
    }
}

/// This struct represents an I2S output driver program
///
/// The sample bit-depth is set through scratch register `Y`.
/// `Y` has to be set to sample bit-depth - 2.
/// (14 = 16bit, 22 = 24bit, 30 = 32bit)
pub struct PioI2sOutProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
}

const PIO_I2S_OUT_PROGRAM_CLK_MULTIPLIER: u32 = 2;

impl<'d, PIO: Instance> PioI2sOutProgram<'d, PIO> {
    /// Load the program into the given pio
    pub fn new(common: &mut Common<'d, PIO>) -> Self {
        let prg = pio::pio_asm!(
            ".side_set 2",                      // side 0bWB - W = Word Clock, B = Bit Clock
            "    mov x, y           side 0b01", // y stores sample depth - 2 (14 = 16bit, 22 = 24bit, 30 = 32bit)
            "left_data:",
            "    out pins, 1        side 0b00",
            "    jmp x-- left_data  side 0b01",
            "    out pins, 1        side 0b10",
            "    mov x, y           side 0b11",
            "right_data:",
            "    out pins, 1         side 0b10",
            "    jmp x-- right_data side 0b11",
            "    out pins, 1         side 0b00",
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

/// Pio backed I2S output driver
pub struct PioI2sOut<'d, P: Instance, const S: usize> {
    dma: dma::Channel<'d>,
    sm: StateMachine<'d, P, S>,
}

impl<'d, P: Instance, const S: usize> PioI2sOut<'d, P, S> {
    /// Configure a state machine to output I2S
    pub fn new<D: dma::ChannelInstance>(
        common: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: Peri<'d, D>,
        irq: impl interrupt::typelevel::Binding<D::Interrupt, dma::InterruptHandler<D>> + 'd,
        data_pin: Peri<'d, impl PioPin>,
        bit_clock_pin: Peri<'d, impl PioPin>,
        lr_clock_pin: Peri<'d, impl PioPin>,
        sample_rate: u32,
        bit_depth: u32,
        program: &PioI2sOutProgram<'d, P>,
    ) -> Self {
        let data_pin = common.make_pio_pin(data_pin);
        let bit_clock_pin = common.make_pio_pin(bit_clock_pin);
        let lr_clock_pin = common.make_pio_pin(lr_clock_pin);

        let bclk_frequency: u32 = sample_rate * bit_depth * 2;

        let cfg = {
            let mut cfg = Config::default();
            cfg.use_program(&program.prg, &[&bit_clock_pin, &lr_clock_pin]);
            cfg.set_out_pins(&[&data_pin]);
            cfg.clock_divider = calculate_pio_clock_divider(bclk_frequency * PIO_I2S_OUT_PROGRAM_CLK_MULTIPLIER);
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
        sm.set_pin_dirs(Direction::Out, &[&data_pin, &lr_clock_pin, &bit_clock_pin]);

        // Set the `y` register up to configure the sample depth
        // The SM counts down to 0 and uses one clock cycle to set up the counter,
        // which results in bit_depth - 2 as register value.
        unsafe { sm.set_y(bit_depth - 2) };

        Self {
            dma: dma::Channel::new(dma, irq),
            sm,
        }
    }

    /// Start the i2s interface
    pub fn start(&mut self) {
        self.sm.set_enable(true);
    }

    /// Stop the i2s interface
    pub fn stop(&mut self) {
        self.sm.set_enable(false);
    }

    /// Start at the the same as other drivers
    pub fn start_batched(&mut self, b: &mut PioBatch<'d, P>) {
        b.set_enable(&mut self.sm, true);
    }

    /// Stop at the the same as other drivers
    pub fn stop_batched(&mut self, b: &mut PioBatch<'d, P>) {
        b.set_enable(&mut self.sm, false);
    }

    /// Return an in-progress dma transfer future. Awaiting it will guarantee a complete transfer.
    pub fn write<'b>(&'b mut self, buff: &'b [u32]) -> Transfer<'b> {
        self.sm.tx().dma_push(&mut self.dma, buff, false)
    }
}
