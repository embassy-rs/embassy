//! Serial Peripheral Interface
use core::marker::PhantomData;

use embassy_futures::join::join;
use embassy_hal_internal::Peri;
pub use embedded_hal_02::spi::{Phase, Polarity};

use crate::dma::{Channel, ChannelInstance};
use crate::gpio::AnyPin;
use crate::spi::{
    Info, blocking_read_inner, blocking_transfer_in_place_inner, blocking_write_inner, configure_pins, dma_read_inner,
    flush_inner, instance_info,
};
use crate::{dma, interrupt, mode};

pub use crate::spi::{Async, Blocking, ClkPin, CsPin, Instance, MisoPin, Mode, MosiPin};

/// SPI errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Receive and transmit buffers for a slave transaction must have equal lengths.
    BufferLengthMismatch,
}

/// SPI configuration.
#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    /// Phase.
    pub phase: Phase,
    /// Polarity.
    pub polarity: Polarity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            phase: Phase::CaptureOnFirstTransition,
            polarity: Polarity::IdleLow,
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Config {
    fn format(&self, f: defmt::Formatter) {
        let phase = match self.phase {
            Phase::CaptureOnFirstTransition => {
                defmt::intern!("CaptureOnFirstTransition")
            }
            Phase::CaptureOnSecondTransition => {
                defmt::intern!("CaptureOnSecondTransition")
            }
        };

        let polarity = match self.polarity {
            Polarity::IdleLow => defmt::intern!("IdleLow"),
            Polarity::IdleHigh => defmt::intern!("IdleHigh"),
        };

        defmt::write!(f, "Config {{ phase: {=istr}, polarity: {=istr} }}", phase, polarity,);
    }
}

/// SPI driver.
pub struct Spi<'d, M: Mode> {
    info: &'static Info,
    tx_dma: Option<Channel<'d, mode::Async>>,
    rx_dma: Option<Channel<'d, mode::Async>>,
    phantom: PhantomData<(&'d mut (), M)>,
}

impl<'d, M: Mode> Spi<'d, M> {
    fn new_inner<T: Instance>(
        _spi: Peri<'d, T>,
        clk: Option<Peri<'d, AnyPin>>,
        mosi: Option<Peri<'d, AnyPin>>,
        miso: Option<Peri<'d, AnyPin>>,
        cs: Option<Peri<'d, AnyPin>>,
        tx_dma: Option<Channel<'d, mode::Async>>,
        rx_dma: Option<Channel<'d, mode::Async>>,
        config: Config,
    ) -> Self {
        let info = instance_info::<T>();
        let p = info.regs;

        // The peripheral must be disabled before selecting slave mode.
        p.cr1().write(|w| {
            w.set_ms(true);
            w.set_sse(false)
        });

        Self::apply_config(info, &config);

        // Always enable DREQ signals -- harmless if DMA is not listening
        p.dmacr().write(|reg| {
            reg.set_rxdmae(true);
            reg.set_txdmae(true);
        });

        // finally, enable (preserving MS bit)
        p.cr1().write(|w| {
            w.set_ms(true);
            w.set_sse(true)
        });

        configure_pins(&[clk.as_ref(), mosi.as_ref(), miso.as_ref(), cs.as_ref()]);

        Self {
            info,
            tx_dma,
            rx_dma,
            phantom: PhantomData,
        }
    }

    /// Private function to apply SPI configuration (phase, polarity) settings.
    ///
    /// Driver should be disabled before making changes and re-enabled after the modifications
    /// are applied.
    fn apply_config(info: &Info, config: &Config) {
        let p = info.regs;
        p.cr0().write(|w| {
            w.set_dss(0b0111); // 8bit
            w.set_spo(config.polarity == Polarity::IdleHigh);
            w.set_sph(config.phase == Phase::CaptureOnSecondTransition);
        });
    }

    /// Queue data for a controller-clocked transaction, discarding received bytes.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        blocking_write_inner(self.info, data);
        Ok(())
    }

    /// Exchange data with the controller in place.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        blocking_transfer_in_place_inner(self.info, data);
        Ok(())
    }

    /// Receive data from the controller while transmitting zeroes.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        blocking_read_inner(self.info, data);
        Ok(())
    }

    /// Exchange equally sized receive and transmit buffers with the controller.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        if read.len() != write.len() {
            return Err(Error::BufferLengthMismatch);
        }

        let p = self.info.regs;
        for (r, &wb) in read.iter_mut().zip(write) {
            while !p.sr().read().tnf() {}
            p.dr().write(|w| w.set_data(wb as _));
            while !p.sr().read().rne() {}
            *r = p.dr().read().data() as u8;
        }
        self.flush()?;
        Ok(())
    }

    /// Block execution until SPI is done.
    pub fn flush(&mut self) -> Result<(), Error> {
        flush_inner(self.info);
        Ok(())
    }

    /// Set SPI config.
    pub fn set_config(&mut self, config: &Config) {
        let p = self.info.regs;

        // disable
        p.cr1().write(|w| {
            w.set_ms(true);
            w.set_sse(false)
        });

        // change stuff
        Self::apply_config(self.info, config);

        // enable
        p.cr1().write(|w| {
            w.set_ms(true);
            w.set_sse(true)
        });
    }
}

impl<'d> Spi<'d, Blocking> {
    /// Create an SPI slave driver in blocking mode.
    pub fn new_blocking<T: Instance>(
        spi: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        tx: Peri<'d, impl MosiPin<T> + 'd>,
        rx: Peri<'d, impl MisoPin<T> + 'd>,
        cs: Peri<'d, impl CsPin<T> + 'd>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            spi,
            Some(clk.into()),
            Some(tx.into()),
            Some(rx.into()),
            Some(cs.into()),
            None,
            None,
            config,
        )
    }

    /// Create an SPI slave driver in blocking mode, supporting reads only.
    pub fn new_blocking_rxonly<T: Instance>(
        spi: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        rx: Peri<'d, impl MisoPin<T> + 'd>,
        cs: Peri<'d, impl CsPin<T> + 'd>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            spi,
            Some(clk.into()),
            None,
            Some(rx.into()),
            Some(cs.into()),
            None,
            None,
            config,
        )
    }
}

impl<'d> Spi<'d, Async> {
    /// Create an SPI slave driver in async mode supporting DMA operations.
    pub fn new<T: Instance, TxDma: ChannelInstance, RxDma: ChannelInstance>(
        spi: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        tx: Peri<'d, impl MosiPin<T> + 'd>,
        rx: Peri<'d, impl MisoPin<T> + 'd>,
        cs: Peri<'d, impl CsPin<T> + 'd>,
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
            spi,
            Some(clk.into()),
            Some(tx.into()),
            Some(rx.into()),
            Some(cs.into()),
            Some(tx_dma_ch),
            Some(rx_dma_ch),
            config,
        )
    }

    /// Create an SPI slave driver in async mode supporting DMA operations.
    pub fn new_rxonly<T: Instance, TxDma: ChannelInstance, RxDma: ChannelInstance>(
        spi: Peri<'d, T>,
        clk: Peri<'d, impl ClkPin<T> + 'd>,
        rx: Peri<'d, impl MisoPin<T> + 'd>,
        cs: Peri<'d, impl CsPin<T> + 'd>,
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
            spi,
            Some(clk.into()),
            None,
            Some(rx.into()),
            Some(cs.into()),
            Some(tx_dma_ch),
            Some(rx_dma_ch),
            config,
        )
    }

    /// Queue data for a controller-clocked transaction, discarding received bytes.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let rx_transfer = unsafe {
            self.rx_dma.as_mut().unwrap().read_discard(
                self.info.regs.dr().as_ptr() as *mut u8,
                buffer.len(),
                self.info.rx_dreq,
            )
        };
        let tx_transfer = unsafe {
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            self.tx_dma.as_mut().unwrap().write(
                buffer,
                self.info.regs.dr().as_ptr() as *mut _,
                self.info.tx_dreq,
                false,
            )
        };
        join(tx_transfer, rx_transfer).await;

        Ok(())
    }

    /// Receive data from the controller while transmitting zeroes using DMA.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        dma_read_inner(
            self.info,
            self.tx_dma.as_mut().unwrap(),
            self.rx_dma.as_mut().unwrap(),
            buffer,
        )
        .await;
        Ok(())
    }

    /// Exchange equally sized receive and transmit buffers with the controller using DMA.
    pub async fn transfer(&mut self, rx_buffer: &mut [u8], tx_buffer: &[u8]) -> Result<(), Error> {
        if rx_buffer.len() != tx_buffer.len() {
            return Err(Error::BufferLengthMismatch);
        }
        self.transfer_inner(rx_buffer, tx_buffer).await
    }

    /// Exchange data with the controller in place using DMA.
    pub async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Error> {
        self.transfer_inner(words, words).await
    }

    async fn transfer_inner(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
        // Arm RX first so incoming controller bytes are not lost.
        let rx_transfer = unsafe {
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            self.rx_dma
                .as_mut()
                .unwrap()
                .read(self.info.regs.dr().as_ptr() as *const _, rx, self.info.rx_dreq, false)
        };

        let tx_ch = self.tx_dma.as_mut().unwrap();
        // If we don't assign future to a variable, the data register pointer
        // is held across an await and makes the future non-Send.
        let tx_transfer = async {
            let p = self.info.regs;
            unsafe {
                tx_ch
                    .write(tx, p.dr().as_ptr() as *mut _, self.info.tx_dreq, false)
                    .await;
            }
        };
        join(tx_transfer, rx_transfer).await;

        Ok(())
    }
}
