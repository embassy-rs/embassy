//! Pio backed I2S output and output drivers

use crate::Peri;
use crate::dma::{AnyChannel, Channel, Transfer};
use crate::gpio::Pull;
use crate::pio::{
    Common, Config, Direction, FifoJoin, Instance, LoadedProgram, Pin, PioPin, ShiftConfig, ShiftDirection,
    StateMachine,
};
use crate::pio_programs::clk::PioClkProgram;
use crate::pio_programs::clock_divider::calculate_pio_clock_divider;

/// This struct represents an I2S receiver & controller driver program
pub struct PioI2sInProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
}

impl<'d, PIO: Instance> PioI2sInProgram<'d, PIO> {
    const CRITICAL_LOOP_LEN: u32 = 2;

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

    /// Return number of cycles that the program needs to recive one Bit of data
    pub fn critical_loop_len(&self) -> u32 {
        Self::CRITICAL_LOOP_LEN
    }
}

/// Pio backed I2S input driver
pub struct PioI2sIn<'d, P: Instance, const S: usize> {
    dma: Peri<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
}

impl<'d, P: Instance, const S: usize> PioI2sIn<'d, P, S> {
    /// Configure a state machine to act as both the controller (provider of SCK and WS) and receiver (of SD) for an I2S signal
    pub fn new(
        common: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: Peri<'d, impl Channel>,
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
        let left_right_clock_pin = common.make_pio_pin(lr_clock_pin);

        let cfg = {
            let mut cfg = Config::default();
            cfg.use_program(&program.prg, &[&bit_clock_pin, &left_right_clock_pin]);
            cfg.set_in_pins(&[&data_pin]);
            let clock_frequency = sample_rate * bit_depth * channels;
            cfg.clock_divider = calculate_pio_clock_divider(clock_frequency * program.critical_loop_len());
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
        sm.set_pin_dirs(Direction::Out, &[&left_right_clock_pin, &bit_clock_pin]);
        sm.set_enable(true);

        Self { dma: dma.into(), sm }
    }

    /// Return an in-progress dma transfer future. Awaiting it will guarantee a complete transfer.
    pub fn read<'b>(&'b mut self, buff: &'b mut [u32]) -> Transfer<'b, AnyChannel> {
        self.sm.rx().dma_pull(self.dma.reborrow(), buff, false)
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

impl<'d, PIO: Instance> PioI2sOutProgram<'d, PIO> {
    const CRITICAL_LOOP_LEN: u32 = 2;

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

    /// Return number of cycles that the program needs to generate one Bit Clock
    pub fn critical_loop_len(&self) -> u32 {
        Self::CRITICAL_LOOP_LEN
    }
}

/// Pio backed I2S output driver
pub struct PioI2sOut<'d, P: Instance, const S: usize, const U: usize> {
    dma: Peri<'d, AnyChannel>,
    i2s_sm: StateMachine<'d, P, S>,
    data_pin: Pin<'d, P>,
    bit_clock_pin: Pin<'d, P>,
    lr_clock_pin: Pin<'d, P>,
    mclk: Option<(StateMachine<'d, P, U>, Pin<'d, P>)>,
}

impl<'d, P: Instance, const S: usize, const U: usize> PioI2sOut<'d, P, S, U> {
    /// Configure a state machine to output I2S
    pub fn new(
        common: &mut Common<'d, P>,
        mut i2s_sm: StateMachine<'d, P, S>,
        dma: Peri<'d, impl Channel>,
        data_pin: Peri<'d, impl PioPin>,
        bit_clock_pin: Peri<'d, impl PioPin>,
        lr_clock_pin: Peri<'d, impl PioPin>,
        sample_rate: u32,
        bit_depth: u32,
        i2s_program: &PioI2sOutProgram<'d, P>,
        mclk: Option<(
            StateMachine<'d, P, U>,
            Peri<'d, impl PioPin>,
            u32,
            &PioClkProgram<'d, P>,
        )>,
    ) -> Self {
        let data_pin = common.make_pio_pin(data_pin);
        let bit_clock_pin = common.make_pio_pin(bit_clock_pin);
        let lr_clock_pin = common.make_pio_pin(lr_clock_pin);

        let i2s_bclk_frequency: u32 = sample_rate * bit_depth * 2;

        let mut i2s_sm_cfg = Config::default();
        i2s_sm_cfg.use_program(&i2s_program.prg, &[&bit_clock_pin, &lr_clock_pin]);
        i2s_sm_cfg.set_out_pins(&[&data_pin]);
        i2s_sm_cfg.clock_divider = calculate_pio_clock_divider(i2s_bclk_frequency * i2s_program.critical_loop_len());
        i2s_sm_cfg.shift_out = ShiftConfig {
            threshold: 32,
            direction: ShiftDirection::Left,
            auto_fill: true,
        };
        // join fifos to have twice the time to start the next dma transfer
        i2s_sm_cfg.fifo_join = FifoJoin::TxOnly;
        i2s_sm.set_config(&i2s_sm_cfg);

        // Set the `y` register up to configure the sample depth
        // The SM counts down to 0 and uses one clock cycle to set up the counter,
        // which results in bit_depth - 2 as register value.
        unsafe { i2s_sm.set_y(bit_depth - 2) };

        let mclk = if let Some((mut mclk_sm, mclk_pin, mclk_multiplier, clk_program)) = mclk {
            let mclk_pin = common.make_pio_pin(mclk_pin);

            let mut mclk_sm_cfg = Config::default();
            mclk_sm_cfg.use_program(&clk_program.prg, &[]);
            mclk_sm_cfg.clock_divider =
                calculate_pio_clock_divider(i2s_bclk_frequency * mclk_multiplier * clk_program.critical_loop_len());
            mclk_sm_cfg.set_set_pins(&[&mclk_pin]);
            mclk_sm.set_config(&mclk_sm_cfg);

            i2s_sm.set_pin_dirs(Direction::Out, &[&data_pin, &lr_clock_pin, &bit_clock_pin]);
            mclk_sm.set_pin_dirs(Direction::Out, &[&mclk_pin]);

            Some((mclk_sm, mclk_pin))
        } else {
            None
        };

        Self {
            dma: dma.into(),
            i2s_sm,
            data_pin,
            bit_clock_pin,
            lr_clock_pin,
            mclk,
        }
    }

    /// Return an in-progress dma transfer future. Awaiting it will guarantee a complete transfer.
    pub fn write<'b>(&'b mut self, buff: &'b [u32]) -> Transfer<'b, AnyChannel> {
        self.i2s_sm.tx().dma_push(self.dma.reborrow(), buff, false)
    }

    fn set_enable(&mut self, common: &mut Common<'d, P>, enable: bool) {
        common.apply_sm_batch(|b| {
            b.set_enable(&mut self.i2s_sm, enable);
            if let Some((mclk_sm, _)) = &mut self.mclk {
                b.set_enable(mclk_sm, enable);
            }
        });
    }

    /// Start the i2s interface
    pub fn start(&mut self, common: &mut Common<'d, P>) {
        self.set_enable(common, true);
    }

    /// Stop the i2s interface
    pub fn stop(&mut self, common: &mut Common<'d, P>) {
        self.set_enable(common, false);
    }

    /// Return the state machine and pin.
    pub fn release(
        mut self,
        common: &mut Common<'d, P>,
    ) -> (
        Peri<'d, impl Channel>,
        StateMachine<'d, P, S>,
        Pin<'d, P>,
        Pin<'d, P>,
        Pin<'d, P>,
        Option<(StateMachine<'d, P, U>, Pin<'d, P>)>,
    ) {
        self.stop(common);
        (
            self.dma,
            self.i2s_sm,
            self.data_pin,
            self.bit_clock_pin,
            self.lr_clock_pin,
            self.mclk,
        )
    }
}
