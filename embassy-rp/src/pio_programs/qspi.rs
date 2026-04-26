//! PIO backed QSPI drivers

use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use embedded_hal_02::spi::{Phase, Polarity};
use fixed::traits::ToFixed;
use fixed::types::extra::U8;

use crate::clocks::clk_sys_freq;
use crate::gpio::{Level, SlewRate};
use crate::pio::{Common, Direction, Instance, Irq, LoadedProgram, Pin, PioPin, ShiftDirection, StateMachine};
use crate::spi::{Async, Blocking, Config, Mode};
use crate::{dma, interrupt};

/// This struct represents a set of QSPI program loaded into pio instruction memory.
struct PioQspiProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
    read_entry: u8,
    write_entry: u8,
    write_single_line_entry: u8,
    phase: Phase,
}

impl<'d, PIO: Instance> PioQspiProgram<'d, PIO> {
    /// Load the qspi program into the given pio
    pub fn new(common: &mut Common<'d, PIO>, phase: Phase) -> Self {
        // These PIO programs are adapted from the datasheet (3.6.1 in
        // RP2040 datasheet, 11.6.1 in RP2350 datasheet)

        // Pin assignments:
        // - SCK is side-set pin 0
        // - QD0 is pin 0
        // - QD1 is pin 1
        // - QD2 is pin 2
        // - QD3 is pin 3
        //
        // Auto-push and auto-pull must be enabled, and the serial frame size is set by
        // configuring the push/pull threshold. Shift left/right is fine, but you must
        // justify the data yourself. This is done most conveniently for frame sizes of
        // 8 or 16 bits by using the narrow store replication and narrow load byte
        // picking behavior of RP2040's IO fabric.

        match phase {
            Phase::CaptureOnFirstTransition => {
                // Clock phase = 0: data is captured on the leading edge of each SCK pulse, and
                // transitions on the trailing edge, or some time before the first leading edge.

                // TODO: might need to make read hang after completing to prevent clocking and
                // reading rubbish
                let prg = pio::pio_asm!(
                    r#"
                        ; Use 1 bit for side-set for SCK
                        .side_set 1

                        public read_entry:
                            ; Set all data pins to input
                            set pindirs 0b0000 side 0

                            ; Read (num_nibbles_to_read - 1) from output shift register
                            out x, 32 side 0

                            read_nibble:
                                nop side 0 [1]
                                in pins, 4 side 1
                                jmp x-- read_nibble side 1

                                ; Set irq to indicate operation complete
                                irq set 0 rel side 0
                            jmp nothing side 0

                        public write_entry:
                            ; Set all data pins to output
                            set pindirs 0b1111 side 0

                            ; Read (num_nibbles_to_write - 1) from output shift register
                            out x, 32 side 0

                            write_nibble:
                                ; Side set proceeds even if instruction stalls, so we stall with SCK low
                                out pins, 4 side 0 [1]
                                nop side 1
                                jmp x-- write_nibble side 1

                                ; Set irq to indicate operation complete
                                irq set 0 rel side 0
                            jmp nothing side 0

                        public write_single_line_entry:
                            ; Set QD0 pin to output
                            set pindirs 0b0001 side 0

                            ; Read (num_bits_to_write - 1) from output shift register
                            out x, 32 side 0

                            write_bit:
                                ; Side set proceeds even if instruction stalls, so we stall with SCK low
                                out pins, 1 side 0 [1]
                                nop side 1
                                jmp x-- write_bit side 1

                                ; Set irq to indicate operation complete
                                irq set 0 rel side 0
                            jmp nothing side 0

                        nothing:
                        .wrap_target
                            nop side 0
                        .wrap
                    "#
                );

                Self {
                    prg: common.load_program(&prg.program),
                    read_entry: prg.public_defines.read_entry as u8,
                    write_entry: prg.public_defines.write_entry as u8,
                    write_single_line_entry: prg.public_defines.write_single_line_entry as u8,
                    phase,
                }
            }
            Phase::CaptureOnSecondTransition => {
                todo!()
            }
        }
    }
}

/// PIO QSPI errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    // No errors for now
}

/// PIO based QSPI driver.
/// Unlike other PIO programs, the PIO QSPI driver owns and holds a reference to
/// the PIO memory it uses. This is so that it can be reconfigured at runtime if
/// desired.
pub struct Qspi<'d, PIO: Instance, const SM: usize, M: Mode> {
    sm: StateMachine<'d, PIO, SM>,
    pio_irq: Irq<'d, PIO, SM>,
    cfg: crate::pio::Config<'d, PIO>,
    program: Option<PioQspiProgram<'d, PIO>>,
    clk_pin: Pin<'d, PIO>,
    tx_dma: Option<dma::Channel<'d>>,
    rx_dma: Option<dma::Channel<'d>>,
    operation_unflushed: bool,
    phantom: PhantomData<M>,
}

impl<'d, PIO: Instance, const SM: usize, M: Mode> Qspi<'d, PIO, SM, M> {
    #[allow(clippy::too_many_arguments)]
    fn new_inner(
        pio: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, SM>,
        pio_irq: Irq<'d, PIO, SM>,
        clk_pin: Peri<'d, impl PioPin>,
        qd0_pin: Peri<'d, impl PioPin>,
        qd1_pin: Peri<'d, impl PioPin>,
        qd2_pin: Peri<'d, impl PioPin>,
        qd3_pin: Peri<'d, impl PioPin>,
        tx_dma: Option<dma::Channel<'d>>,
        rx_dma: Option<dma::Channel<'d>>,
        config: Config,
    ) -> Self {
        let program = PioQspiProgram::new(pio, config.phase);

        let mut clk_pin = pio.make_pio_pin(clk_pin);
        let mut qd0_pin = pio.make_pio_pin(qd0_pin);
        let mut qd1_pin = pio.make_pio_pin(qd1_pin);
        let mut qd2_pin = pio.make_pio_pin(qd2_pin);
        let mut qd3_pin = pio.make_pio_pin(qd3_pin);

        if let Polarity::IdleHigh = config.polarity {
            clk_pin.set_output_inversion(true);
        } else {
            clk_pin.set_output_inversion(false);
        }

        clk_pin.set_slew_rate(SlewRate::Fast);

        for pin in [&mut qd0_pin, &mut qd1_pin, &mut qd2_pin, &mut qd3_pin] {
            pin.set_input_sync_bypass(true);
            // pin.set_pull(Pull::Down);
            // pin.set_schmitt(true);
        }

        let mut cfg = crate::pio::Config::default();

        cfg.use_program(&program.prg, &[&clk_pin]);
        cfg.set_in_pins(&[&qd0_pin, &qd1_pin, &qd2_pin, &qd3_pin]);
        cfg.set_out_pins(&[&qd0_pin, &qd1_pin, &qd2_pin, &qd3_pin]);
        cfg.set_set_pins(&[&qd0_pin, &qd1_pin, &qd2_pin, &qd3_pin]);

        cfg.shift_in.auto_fill = true;
        cfg.shift_in.direction = ShiftDirection::Left;
        cfg.shift_in.threshold = 8;

        cfg.shift_out.auto_fill = true;
        cfg.shift_out.direction = ShiftDirection::Left;
        cfg.shift_out.threshold = 8;

        cfg.clock_divider = calculate_clock_divider(config.frequency);
        let bytes = cfg.clock_divider.to_le_bytes();
        defmt::info!("clock divider: {:?}", bytes);

        sm.set_config(&cfg);

        sm.set_pins(Level::Low, &[&clk_pin, &qd0_pin, &qd1_pin, &qd2_pin, &qd3_pin]);
        sm.set_pin_dirs(Direction::Out, &[&clk_pin]);

        sm.set_enable(true);

        Self {
            sm,
            pio_irq,
            program: Some(program),
            cfg,
            clk_pin,
            tx_dma,
            rx_dma,
            operation_unflushed: false,
            phantom: PhantomData,
        }
    }

    /// Block execution until QSPI is done.
    pub fn flush(&mut self) -> Result<(), Error> {
        // Wait for all words in the FIFO to have been pulled by the SM
        while !self.sm.tx().empty() {}

        // Wait for last value to be written out to the wire
        while !self.sm.tx().stalled() {}

        Ok(())
    }

    /// Wait for QSPI operation to complete
    pub async fn async_flush(&mut self) {
        if self.operation_unflushed {
            // IRQ fires when operation finished
            self.pio_irq.wait().await;
            self.operation_unflushed = false;
        }
    }

    /// Set QSPI frequency.
    pub fn set_frequency(&mut self, freq: u32) {
        self.sm.set_enable(false);

        let divider = calculate_clock_divider(freq);

        // save into the config for later but dont use sm.set_config() since
        // that operation is relatively more expensive than just setting the
        // clock divider
        self.cfg.clock_divider = divider;
        self.sm.set_clock_divider(divider);

        self.sm.set_enable(true);
    }

    /// Set QSPI config.
    ///
    /// This operation will panic if the PIO program needs to be reloaded and
    /// there is insufficient room. This is unlikely since the programs for each
    /// phase only differ in size by a single instruction.
    pub fn set_config(&mut self, pio: &mut Common<'d, PIO>, config: &Config) {
        self.flush();
        self.sm.set_enable(false);

        self.cfg.clock_divider = calculate_clock_divider(config.frequency);

        if let Polarity::IdleHigh = config.polarity {
            self.clk_pin.set_output_inversion(true);
        } else {
            self.clk_pin.set_output_inversion(false);
        }

        if self.program.as_ref().unwrap().phase != config.phase {
            let old_program = self.program.take().unwrap();

            // SAFETY: the state machine is disabled while this happens
            unsafe {
                pio.free_instr(old_program.prg.used_memory);
            };

            let new_program = PioQspiProgram::new(pio, config.phase);

            self.cfg.use_program(&new_program.prg, &[&self.clk_pin]);
            self.program = Some(new_program);
        }

        self.sm.set_config(&self.cfg);
        self.sm.restart();

        self.sm.set_enable(true);
    }
}

fn calculate_clock_divider(frequency_hz: u32) -> fixed::FixedU32<U8> {
    // we multiply by 4 since each clock period is equal to 4 instructions

    let sys_freq = clk_sys_freq().to_fixed::<fixed::FixedU64<U8>>();
    let target_freq = (frequency_hz * 4).to_fixed::<fixed::FixedU64<U8>>();
    (sys_freq / target_freq).to_fixed()
}

impl<'d, PIO: Instance, const SM: usize> Qspi<'d, PIO, SM, Blocking> {
    /// Create an QSPI driver in blocking mode.
    pub fn new_blocking(
        pio: &mut Common<'d, PIO>,
        sm: StateMachine<'d, PIO, SM>,
        pio_irq: Irq<'d, PIO, SM>,
        clk: Peri<'d, impl PioPin>,
        qd0: Peri<'d, impl PioPin>,
        qd1: Peri<'d, impl PioPin>,
        qd2: Peri<'d, impl PioPin>,
        qd3: Peri<'d, impl PioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(pio, sm, pio_irq, clk, qd0, qd1, qd2, qd3, None, None, config)
    }
}

impl<'d, PIO: Instance, const SM: usize> Qspi<'d, PIO, SM, Async> {
    /// Create an QSPI driver in async mode supporting DMA operations.
    #[allow(clippy::too_many_arguments)]
    pub fn new<TxDma: dma::ChannelInstance, RxDma: dma::ChannelInstance>(
        pio: &mut Common<'d, PIO>,
        sm: StateMachine<'d, PIO, SM>,
        pio_irq: Irq<'d, PIO, SM>,
        clk: Peri<'d, impl PioPin>,
        qd0: Peri<'d, impl PioPin>,
        qd1: Peri<'d, impl PioPin>,
        qd2: Peri<'d, impl PioPin>,
        qd3: Peri<'d, impl PioPin>,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        irq: impl interrupt::typelevel::Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
        + interrupt::typelevel::Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
        + 'd,
        config: Config,
    ) -> Self {
        let tx_dma_ch = dma::Channel::new(tx_dma, irq);
        let rx_dma_ch = dma::Channel::new(rx_dma, irq);
        Self::new_inner(
            pio,
            sm,
            pio_irq,
            clk,
            qd0,
            qd1,
            qd2,
            qd3,
            Some(tx_dma_ch),
            Some(rx_dma_ch),
            config,
        )
    }

    /// Read data from QSPI using DMA.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.async_flush().await;

        // jump to read
        unsafe {
            self.sm.exec_jmp(self.program.as_ref().unwrap().read_entry);
        }

        self.operation_unflushed = true;

        let (rx, tx) = self.sm.rx_tx();

        let num_nibbles: u32 = 2 * (buffer.len() as u32);
        // force push to FIFO since it will be empty after flush
        tx.push(num_nibbles - 1);

        let mut rx_ch = self.rx_dma.as_mut().unwrap().reborrow();
        let rx_transfer = rx.dma_pull(&mut rx_ch, buffer, false);

        rx_transfer.await;
        defmt::debug!("read: {}", &buffer);

        Ok(())
    }

    /// Write data to QSPI using DMA.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.async_flush().await;

        // jump to write
        unsafe {
            self.sm.exec_jmp(self.program.as_ref().unwrap().write_entry);
        }

        self.operation_unflushed = true;

        let tx = self.sm.tx();

        let num_nibbles: u32 = 2 * (buffer.len() as u32);
        // force push to FIFO since it will be empty after flush
        tx.push(num_nibbles - 1);

        let mut tx_ch = self.tx_dma.as_mut().unwrap().reborrow();
        let tx_transfer = tx.dma_push(&mut tx_ch, buffer, false);

        tx_transfer.await;
        defmt::debug!("wrote: {}", &buffer);

        Ok(())
    }

    /// Write data using a single line to QSPI using DMA.
    pub async fn write_single_line(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.async_flush().await;

        // jump to write
        unsafe {
            self.sm.exec_jmp(self.program.as_ref().unwrap().write_single_line_entry);
        }

        self.operation_unflushed = true;

        let tx = self.sm.tx();

        let num_bits: u32 = 8 * (buffer.len() as u32);
        // force push to FIFO since it will be empty after flush
        tx.push(num_bits - 1);

        let mut tx_ch = self.tx_dma.as_mut().unwrap().reborrow();
        let tx_transfer = tx.dma_push(&mut tx_ch, buffer, false);

        tx_transfer.await;
        defmt::debug!("wrote single line: {}", &buffer);

        Ok(())
    }
}

// HAL traits:

impl embassy_embedded_hal::qspi::traits::Error for Error {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        match *self {}
    }
}

impl<'d, PIO: Instance, const SM: usize, M: Mode> embassy_embedded_hal::qspi::traits::ErrorType
    for Qspi<'d, PIO, SM, M>
{
    type Error = Error;
}

impl<'d, PIO: Instance, const SM: usize> embassy_embedded_hal::qspi::traits::QspiBus<u8> for Qspi<'d, PIO, SM, Async> {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.async_flush().await;
        Ok(())
    }

    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.read(words).await
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.write(words).await
    }

    async fn write_single_line(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.write_single_line(words).await
    }
}
