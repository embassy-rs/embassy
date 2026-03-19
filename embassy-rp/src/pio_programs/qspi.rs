//! PIO backed QSPI drivers

use core::marker::PhantomData;

use embassy_futures::join::join;
use embassy_hal_internal::Peri;
use embedded_hal_02::spi::{Phase, Polarity};
use fixed::traits::ToFixed;
use fixed::types::extra::U8;

use crate::clocks::clk_sys_freq;
use crate::gpio::Level;
use crate::pio::{Common, Direction, Instance, LoadedProgram, Pin, PioPin, ShiftDirection, StateMachine};
use crate::spi::{Async, Blocking, Config, Mode};
use crate::{dma, interrupt};

/// This struct represents a QSPI program loaded into pio instruction memory.
struct PioQspiProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
    phase: Phase,
}

impl<'d, PIO: Instance> PioQspiProgram<'d, PIO> {
    /// Load the qspi program into the given pio
    pub fn new(common: &mut Common<'d, PIO>, phase: Phase) -> Self {
        // These PIO programs are taken straight from the datasheet (3.6.1 in
        // RP2040 datasheet, 11.6.1 in RP2350 datasheet)

        // Pin assignments:
        // - SCK is side-set pin 0
        // - MOSI is OUT pin 0
        // - MISO is IN pin 0
        //
        // Auto-push and auto-pull must be enabled, and the serial frame size is set by
        // configuring the push/pull threshold. Shift left/right is fine, but you must
        // justify the data yourself. This is done most conveniently for frame sizes of
        // 8 or 16 bits by using the narrow store replication and narrow load byte
        // picking behavior of RP2040's IO fabric.

        let prg = match phase {
            Phase::CaptureOnFirstTransition => {
                let prg = pio::pio_asm!(
                    r#"
                        .side_set 1

                        ; Clock phase = 0: data is captured on the leading edge of each SCK pulse, and
                        ; transitions on the trailing edge, or some time before the first leading edge.

                        out pins, 1 side 0 [1] ; Stall here on empty (sideset proceeds even if
                        in pins, 1  side 1 [1] ; instruction stalls, so we stall with SCK low)
                    "#
                );

                common.load_program(&prg.program)
            }
            Phase::CaptureOnSecondTransition => {
                todo!()
            }
        };

        Self { prg, phase }
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
    cfg: crate::pio::Config<'d, PIO>,
    program: Option<PioQspiProgram<'d, PIO>>,
    clk_pin: Pin<'d, PIO>,
    tx_dma: Option<dma::Channel<'d>>,
    rx_dma: Option<dma::Channel<'d>>,
    phantom: PhantomData<M>,
}

impl<'d, PIO: Instance, const SM: usize, M: Mode> Qspi<'d, PIO, SM, M> {
    #[allow(clippy::too_many_arguments)]
    fn new_inner(
        pio: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, SM>,
        clk_pin: Peri<'d, impl PioPin>,
        mosi_pin: Peri<'d, impl PioPin>,
        miso_pin: Peri<'d, impl PioPin>,
        tx_dma: Option<dma::Channel<'d>>,
        rx_dma: Option<dma::Channel<'d>>,
        config: Config,
    ) -> Self {
        let program = PioQspiProgram::new(pio, config.phase);

        let mut clk_pin = pio.make_pio_pin(clk_pin);
        let mosi_pin = pio.make_pio_pin(mosi_pin);
        let miso_pin = pio.make_pio_pin(miso_pin);

        if let Polarity::IdleHigh = config.polarity {
            clk_pin.set_output_inversion(true);
        } else {
            clk_pin.set_output_inversion(false);
        }

        let mut cfg = crate::pio::Config::default();

        cfg.use_program(&program.prg, &[&clk_pin]);
        cfg.set_out_pins(&[&mosi_pin]);
        cfg.set_in_pins(&[&miso_pin]);

        cfg.shift_in.auto_fill = true;
        cfg.shift_in.direction = ShiftDirection::Left;
        cfg.shift_in.threshold = 8;

        cfg.shift_out.auto_fill = true;
        cfg.shift_out.direction = ShiftDirection::Left;
        cfg.shift_out.threshold = 8;

        cfg.clock_divider = calculate_clock_divider(config.frequency);

        sm.set_config(&cfg);

        sm.set_pins(Level::Low, &[&clk_pin, &mosi_pin]);
        sm.set_pin_dirs(Direction::Out, &[&clk_pin, &mosi_pin]);
        sm.set_pin_dirs(Direction::In, &[&miso_pin]);

        sm.set_enable(true);

        Self {
            sm,
            program: Some(program),
            cfg,
            clk_pin,
            tx_dma,
            rx_dma,
            phantom: PhantomData,
        }
    }

    fn blocking_read_u8(&mut self) -> Result<u8, Error> {
        while self.sm.rx().empty() {}
        let value = self.sm.rx().pull() as u8;

        Ok(value)
    }

    fn blocking_write_u8(&mut self, v: u8) -> Result<(), Error> {
        let value = u32::from_be_bytes([v, 0, 0, 0]);

        while !self.sm.tx().try_push(value) {}

        // need to clear here for flush to work correctly
        self.sm.tx().stalled();

        Ok(())
    }

    /// Read data from QSPI blocking execution until done.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        for v in data {
            self.blocking_write_u8(0)?;
            *v = self.blocking_read_u8()?;
        }
        self.flush()?;
        Ok(())
    }

    /// Write data to QSPI blocking execution until done.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        for v in data {
            self.blocking_write_u8(*v)?;
            let _ = self.blocking_read_u8()?;
        }
        self.flush()?;
        Ok(())
    }

    /// Block execution until QSPI is done.
    pub fn flush(&mut self) -> Result<(), Error> {
        // Wait for all words in the FIFO to have been pulled by the SM
        while !self.sm.tx().empty() {}

        // Wait for last value to be written out to the wire
        while !self.sm.tx().stalled() {}

        Ok(())
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
            unsafe { pio.free_instr(old_program.prg.used_memory) };

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
        clk: Peri<'d, impl PioPin>,
        mosi: Peri<'d, impl PioPin>,
        miso: Peri<'d, impl PioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(pio, sm, clk, mosi, miso, None, None, config)
    }
}

impl<'d, PIO: Instance, const SM: usize> Qspi<'d, PIO, SM, Async> {
    /// Create an QSPI driver in async mode supporting DMA operations.
    #[allow(clippy::too_many_arguments)]
    pub fn new<TxDma: dma::ChannelInstance, RxDma: dma::ChannelInstance>(
        pio: &mut Common<'d, PIO>,
        sm: StateMachine<'d, PIO, SM>,
        clk: Peri<'d, impl PioPin>,
        mosi: Peri<'d, impl PioPin>,
        miso: Peri<'d, impl PioPin>,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        irq: impl interrupt::typelevel::Binding<TxDma::Interrupt, dma::InterruptHandler<TxDma>>
        + interrupt::typelevel::Binding<RxDma::Interrupt, dma::InterruptHandler<RxDma>>
        + 'd,
        config: Config,
    ) -> Self {
        let tx_dma_ch = dma::Channel::new(tx_dma, irq);
        let rx_dma_ch = dma::Channel::new(rx_dma, irq);
        Self::new_inner(pio, sm, clk, mosi, miso, Some(tx_dma_ch), Some(rx_dma_ch), config)
    }

    /// Read data from QSPI using DMA.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let (rx, tx) = self.sm.rx_tx();

        let len = buffer.len();

        let mut rx_ch = self.rx_dma.as_mut().unwrap().reborrow();
        let rx_transfer = rx.dma_pull(&mut rx_ch, buffer, false);

        let mut tx_ch = self.tx_dma.as_mut().unwrap().reborrow();
        let tx_transfer = tx.dma_push_zeros::<u8>(&mut tx_ch, len);

        join(tx_transfer, rx_transfer).await;

        Ok(())
    }

    /// Write data to QSPI using DMA.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let (rx, tx) = self.sm.rx_tx();

        let mut rx_ch = self.rx_dma.as_mut().unwrap().reborrow();
        let rx_transfer = rx.dma_pull_discard::<u8>(&mut rx_ch, buffer.len());

        let mut tx_ch = self.tx_dma.as_mut().unwrap().reborrow();
        let tx_transfer = tx.dma_push(&mut tx_ch, buffer, false);

        join(tx_transfer, rx_transfer).await;

        Ok(())
    }
}
