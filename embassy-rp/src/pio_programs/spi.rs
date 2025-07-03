//! PIO backed SPi drivers

use core::marker::PhantomData;

use embassy_futures::join::join;
use embassy_hal_internal::Peri;
use embedded_hal_02::spi::{Phase, Polarity};
use fixed::{traits::ToFixed, types::extra::U8};

use crate::{
    clocks::clk_sys_freq,
    dma::{AnyChannel, Channel},
    gpio::Level,
    pio::{Common, Direction, Instance, LoadedProgram, PioPin, ShiftDirection, StateMachine},
    spi::{Async, Blocking, Mode},
};

/// This struct represents a uart tx program loaded into pio instruction memory.
pub struct PioSpiProgram<'d, PIO: crate::pio::Instance> {
    prg: LoadedProgram<'d, PIO>,
}

impl<'d, PIO: crate::pio::Instance> PioSpiProgram<'d, PIO> {
    /// Load the spi program into the given pio
    pub fn new(common: &mut crate::pio::Common<'d, PIO>, phase: Phase) -> Self {
        // These PIO programs are taken straight from the datasheet (3.6.1 in
        // RP2040 datasheet, 11.6.1 in RP2350 datasheet)

        // Pin assignments:
        // - SCK is side-set pin 0
        // - MOSI is OUT pin 0
        // - MISO is IN pin 0
        //
        // Autopush and autopull must be enabled, and the serial frame size is set by
        // configuring the push/pull threshold. Shift left/right is fine, but you must
        // justify the data yourself. This is done most conveniently for frame sizes of
        // 8 or 16 bits by using the narrow store replication and narrow load byte
        // picking behaviour of RP2040's IO fabric.

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
                let prg = pio::pio_asm!(
                    r#"
                        .side_set 1

                        ; Clock phase = 1: data transitions on the leading edge of each SCK pulse, and
                        ; is captured on the trailing edge.

                        out x, 1    side 0     ; Stall here on empty (keep SCK deasserted)
                        mov pins, x side 1 [1] ; Output data, assert SCK (mov pins uses OUT mapping)
                        in pins, 1  side 0     ; Input data, deassert SCK
                    "#
                );

                common.load_program(&prg.program)
            }
        };

        Self { prg }
    }
}

/// PIO SPI errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    // No errors for now
}

/// PIO based Spi driver.
///
/// This driver is less flexible than the hardware backed one. Configuration can
/// not be changed at runtime.
pub struct Spi<'d, PIO: Instance, const SM: usize, M: Mode> {
    sm: StateMachine<'d, PIO, SM>,
    tx_dma: Option<Peri<'d, AnyChannel>>,
    rx_dma: Option<Peri<'d, AnyChannel>>,
    phantom: PhantomData<M>,
}

/// PIO SPI configuration.
#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    /// Frequency (Hz).
    pub frequency: u32,
    /// Polarity.
    pub polarity: Polarity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: 1_000_000,
            polarity: Polarity::IdleLow,
        }
    }
}

impl<'d, PIO: Instance, const SM: usize, M: Mode> Spi<'d, PIO, SM, M> {
    #[allow(clippy::too_many_arguments)]
    fn new_inner(
        pio: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, SM>,
        clk_pin: Peri<'d, impl PioPin>,
        mosi_pin: Peri<'d, impl PioPin>,
        miso_pin: Peri<'d, impl PioPin>,
        tx_dma: Option<Peri<'d, AnyChannel>>,
        rx_dma: Option<Peri<'d, AnyChannel>>,
        program: &PioSpiProgram<'d, PIO>,
        config: Config,
    ) -> Self {
        let mut clk_pin = pio.make_pio_pin(clk_pin);
        let mosi_pin = pio.make_pio_pin(mosi_pin);
        let miso_pin = pio.make_pio_pin(miso_pin);

        if let Polarity::IdleHigh = config.polarity {
            clk_pin.set_output_inversion(true);
        } else {
            clk_pin.set_output_inversion(false);
        }

        sm.set_pins(Level::Low, &[&clk_pin, &mosi_pin]);
        sm.set_pin_dirs(Direction::Out, &[&clk_pin, &mosi_pin]);
        sm.set_pin_dirs(Direction::In, &[&miso_pin]);

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

        let sys_freq = clk_sys_freq().to_fixed::<fixed::FixedU64<U8>>();
        let target_freq = (config.frequency * 4).to_fixed::<fixed::FixedU64<U8>>();
        cfg.clock_divider = (sys_freq / target_freq).to_fixed();

        sm.set_config(&cfg);
        sm.set_enable(true);

        Self {
            sm,
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

    /// Read data from SPI blocking execution until done.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        for v in data {
            self.blocking_write_u8(0)?;
            *v = self.blocking_read_u8()?;
        }
        self.flush()?;
        Ok(())
    }

    /// Write data to SPI blocking execution until done.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        for v in data {
            self.blocking_write_u8(*v)?;
            let _ = self.blocking_read_u8()?;
        }
        self.flush()?;
        Ok(())
    }

    /// Transfer data to SPI blocking execution until done.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        let len = read.len().max(write.len());
        for i in 0..len {
            let wb = write.get(i).copied().unwrap_or(0);
            self.blocking_write_u8(wb)?;

            let rb = self.blocking_read_u8()?;
            if let Some(r) = read.get_mut(i) {
                *r = rb;
            }
        }
        self.flush()?;
        Ok(())
    }

    /// Transfer data in place to SPI blocking execution until done.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        for v in data {
            self.blocking_write_u8(*v)?;
            *v = self.blocking_read_u8()?;
        }
        self.flush()?;
        Ok(())
    }

    /// Block execution until SPI is done.
    pub fn flush(&mut self) -> Result<(), Error> {
        // Wait for all words in the FIFO to have been pulled by the SM
        while !self.sm.tx().empty() {}

        // Wait for last value to be written out to the wire
        while !self.sm.tx().stalled() {}

        Ok(())
    }
}

impl<'d, PIO: Instance, const SM: usize> Spi<'d, PIO, SM, Blocking> {
    /// Create an SPI driver in blocking mode.
    pub fn new_blocking(
        pio: &mut Common<'d, PIO>,
        sm: StateMachine<'d, PIO, SM>,
        clk: Peri<'d, impl PioPin>,
        mosi: Peri<'d, impl PioPin>,
        miso: Peri<'d, impl PioPin>,
        program: &PioSpiProgram<'d, PIO>,
        config: Config,
    ) -> Self {
        Self::new_inner(pio, sm, clk, mosi, miso, None, None, program, config)
    }
}

impl<'d, PIO: Instance, const SM: usize> Spi<'d, PIO, SM, Async> {
    /// Create an SPI driver in async mode supporting DMA operations.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pio: &mut Common<'d, PIO>,
        sm: StateMachine<'d, PIO, SM>,
        clk: Peri<'d, impl PioPin>,
        mosi: Peri<'d, impl PioPin>,
        miso: Peri<'d, impl PioPin>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        program: &PioSpiProgram<'d, PIO>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            pio,
            sm,
            clk,
            mosi,
            miso,
            Some(tx_dma.into()),
            Some(rx_dma.into()),
            program,
            config,
        )
    }

    /// Read data from SPI using DMA.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let (rx, tx) = self.sm.rx_tx();

        let len = buffer.len();

        let rx_ch = self.rx_dma.as_mut().unwrap().reborrow();
        let rx_transfer = rx.dma_pull(rx_ch, buffer, false);

        let tx_ch = self.tx_dma.as_mut().unwrap().reborrow();
        let tx_transfer = tx.dma_push_repeated::<_, u8>(tx_ch, len);

        join(tx_transfer, rx_transfer).await;

        Ok(())
    }

    /// Write data to SPI using DMA.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let (rx, tx) = self.sm.rx_tx();

        let rx_ch = self.rx_dma.as_mut().unwrap().reborrow();
        let rx_transfer = rx.dma_pull_repeated::<_, u8>(rx_ch, buffer.len());

        let tx_ch = self.tx_dma.as_mut().unwrap().reborrow();
        let tx_transfer = tx.dma_push(tx_ch, buffer, false);

        join(tx_transfer, rx_transfer).await;

        Ok(())
    }

    /// Transfer data to SPI using DMA.
    pub async fn transfer(&mut self, rx_buffer: &mut [u8], tx_buffer: &[u8]) -> Result<(), Error> {
        self.transfer_inner(rx_buffer, tx_buffer).await
    }

    /// Transfer data in place to SPI using DMA.
    pub async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Error> {
        self.transfer_inner(words, words).await
    }

    async fn transfer_inner(&mut self, rx_buffer: *mut [u8], tx_buffer: *const [u8]) -> Result<(), Error> {
        let (rx, tx) = self.sm.rx_tx();

        let mut rx_ch = self.rx_dma.as_mut().unwrap().reborrow();
        let rx_transfer = async {
            rx.dma_pull(rx_ch.reborrow(), unsafe { &mut *rx_buffer }, false).await;

            if tx_buffer.len() > rx_buffer.len() {
                let read_bytes_len = tx_buffer.len() - rx_buffer.len();

                rx.dma_pull_repeated::<_, u8>(rx_ch, read_bytes_len).await;
            }
        };

        let mut tx_ch = self.tx_dma.as_mut().unwrap().reborrow();
        let tx_transfer = async {
            tx.dma_push(tx_ch.reborrow(), unsafe { &*tx_buffer }, false).await;

            if rx_buffer.len() > tx_buffer.len() {
                let write_bytes_len = rx_buffer.len() - tx_buffer.len();

                tx.dma_push_repeated::<_, u8>(tx_ch, write_bytes_len).await;
            }
        };

        join(tx_transfer, rx_transfer).await;

        Ok(())
    }
}

// ====================

impl<'d, PIO: Instance, const SM: usize, M: Mode> embedded_hal_02::blocking::spi::Transfer<u8> for Spi<'d, PIO, SM, M> {
    type Error = Error;
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.blocking_transfer_in_place(words)?;
        Ok(words)
    }
}

impl<'d, PIO: Instance, const SM: usize, M: Mode> embedded_hal_02::blocking::spi::Write<u8> for Spi<'d, PIO, SM, M> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }
}

impl embedded_hal_1::spi::Error for Error {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        match *self {}
    }
}

impl<'d, PIO: Instance, const SM: usize, M: Mode> embedded_hal_1::spi::ErrorType for Spi<'d, PIO, SM, M> {
    type Error = Error;
}

impl<'d, PIO: Instance, const SM: usize, M: Mode> embedded_hal_1::spi::SpiBus<u8> for Spi<'d, PIO, SM, M> {
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_transfer(words, &[])
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_transfer(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_transfer_in_place(words)
    }
}

impl<'d, PIO: Instance, const SM: usize> embedded_hal_async::spi::SpiBus<u8> for Spi<'d, PIO, SM, Async> {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.write(words).await
    }

    async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.read(words).await
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.transfer(read, write).await
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.transfer_in_place(words).await
    }
}
