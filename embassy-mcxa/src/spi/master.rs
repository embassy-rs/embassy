//! SPI Master driver implementation.

use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use embedded_hal_02::spi::{Phase, Polarity};

use super::common::*;
use super::pins::*;
use crate::clocks::periph_helpers::{Div4, LpspiClockSel, LpspiConfig};
use crate::clocks::{PoweredClock, enable_and_reset};
use crate::gpio::AnyPin;
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;

/// SPI Master Driver.
pub struct Spi<'d, T: Instance, M: Mode> {
    _peri: Peri<'d, T>,
    _sck: Peri<'d, AnyPin>,
    _mosi: Peri<'d, AnyPin>,
    _miso: Peri<'d, AnyPin>,
    _cs: Peri<'d, AnyPin>,
    _phantom: PhantomData<M>,
    chip_select: ChipSelect,
}

impl<'d, T: Instance> Spi<'d, T, Blocking> {
    /// Create a new blocking instance of the SPI Master driver.
    pub fn new_blocking(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        config: Config,
    ) -> Result<Self> {
        Self::new_inner(peri, sck, mosi, miso, cs, config)
    }
}

impl<'d, T: Instance> Spi<'d, T, Async> {
    /// Create a new async (interrupt-driven) instance of the SPI Master driver.
    pub fn new_async(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self> {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        Self::new_inner(peri, sck, mosi, miso, cs, config)
    }

    /// Async write (interrupt-driven).
    pub async fn write(&mut self, tx: &[u8]) -> Result<()> {
        if tx.is_empty() {
            return Ok(());
        }

        let spi = T::regs();
        prepare_for_transfer(spi);

        let is_pcs_continuous = tx.len() > 1;

        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        self.apply_transfer_tcr(is_pcs_continuous, true, false);
        Self::wait_tx_fifo_empty()?;

        let mut tx_idx = 0usize;
        while tx_idx < tx.len() && Self::get_tx_fifo_count() < Self::get_fifo_size() {
            spi.tdr().write(|w| unsafe { w.bits(tx[tx_idx] as u32) });
            tx_idx += 1;
        }

        while tx_idx < tx.len() {
            T::wait_cell()
                .wait_for(|| {
                    spi.ier().modify(|_, w| w.tdie().enable());
                    Self::get_tx_fifo_count() < Self::get_fifo_size()
                })
                .await
                .map_err(|_| Error::Timeout)?;

            while tx_idx < tx.len() && Self::get_tx_fifo_count() < Self::get_fifo_size() {
                spi.tdr().write(|w| unsafe { w.bits(tx[tx_idx] as u32) });
                tx_idx += 1;
            }
        }

        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= Self::get_fifo_size())?;
            self.apply_transfer_tcr(false, true, false);
        }

        Self::wait_tx_fifo_empty()?;
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

        Ok(())
    }

    /// Async read (interrupt-driven).
    pub async fn read(&mut self, rx: &mut [u8]) -> Result<()> {
        if rx.is_empty() {
            return Ok(());
        }

        let spi = T::regs();
        let fifo_size = Self::get_fifo_size() as usize;
        prepare_for_transfer(spi);

        let is_pcs_continuous = rx.len() > 1;

        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        self.apply_transfer_tcr(is_pcs_continuous, false, false);
        Self::wait_tx_fifo_empty()?;

        let mut tx_remaining = rx.len();
        let mut rx_idx = 0usize;
        let rx_fifo_max_bytes = fifo_size;

        while tx_remaining > 0 || rx_idx < rx.len() {
            while tx_remaining > 0
                && Self::get_tx_fifo_count() < Self::get_fifo_size()
                && (rx.len() - rx_idx) - tx_remaining < rx_fifo_max_bytes
            {
                spi.tdr().write(|w| unsafe { w.bits(0) });
                tx_remaining -= 1;
            }

            while Self::get_rx_fifo_count() > 0 && rx_idx < rx.len() {
                rx[rx_idx] = spi.rdr().read().bits() as u8;
                rx_idx += 1;
            }

            if tx_remaining > 0 || rx_idx < rx.len() {
                T::wait_cell()
                    .wait_for(|| {
                        spi.ier().modify(|_, w| w.rdie().enable());
                        Self::get_rx_fifo_count() > 0 || Self::get_tx_fifo_count() < Self::get_fifo_size()
                    })
                    .await
                    .map_err(|_| Error::Timeout)?;
            }
        }

        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= Self::get_fifo_size())?;
            self.apply_transfer_tcr(false, false, false);
        }

        Ok(())
    }

    /// Async full-duplex transfer (interrupt-driven).
    pub async fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        let tx_len = tx.len();
        let rx_len = rx.len();
        let len = tx_len.max(rx_len);
        if len == 0 {
            return Ok(());
        }

        let spi = T::regs();
        let fifo_size = Self::get_fifo_size() as usize;
        prepare_for_transfer(spi);

        let is_pcs_continuous = len > 1;

        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        self.apply_transfer_tcr(is_pcs_continuous, false, false);
        Self::wait_tx_fifo_empty()?;

        let mut tx_idx = 0usize;
        let mut rx_idx = 0usize;
        let rx_fifo_max_bytes = fifo_size;

        while tx_idx < len || rx_idx < len {
            while tx_idx < len
                && Self::get_tx_fifo_count() < Self::get_fifo_size()
                && (len - rx_idx) - (len - tx_idx) < rx_fifo_max_bytes
            {
                let byte = if tx_idx < tx_len { tx[tx_idx] } else { 0 };
                spi.tdr().write(|w| unsafe { w.bits(byte as u32) });
                tx_idx += 1;
            }

            while Self::get_rx_fifo_count() > 0 && rx_idx < len {
                let byte = spi.rdr().read().bits() as u8;
                if rx_idx < rx_len {
                    rx[rx_idx] = byte;
                }
                rx_idx += 1;
            }

            if tx_idx < len || rx_idx < len {
                T::wait_cell()
                    .wait_for(|| {
                        spi.ier().modify(|_, w| w.rdie().enable());
                        Self::get_rx_fifo_count() > 0 || Self::get_tx_fifo_count() < Self::get_fifo_size()
                    })
                    .await
                    .map_err(|_| Error::Timeout)?;
            }
        }

        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= Self::get_fifo_size())?;
            self.apply_transfer_tcr(false, false, false);
        }

        Ok(())
    }
}

impl<'d, T: Instance, M: Mode> Spi<'d, T, M> {
    fn new_inner(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        config: Config,
    ) -> Result<Self> {
        let clock_config = LpspiConfig {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: LpspiClockSel::FroHfDiv,
            div: Div4::no_div(),
            instance: T::CLOCK_INSTANCE,
        };

        _ = unsafe { enable_and_reset::<T>(&clock_config).map_err(Error::ClockSetup)? };

        sck.mux();
        mosi.mux();
        miso.mux();
        cs.mux();

        let _sck = sck.into();
        let _mosi = mosi.into();
        let _miso = miso.into();
        let _cs = cs.into();

        Self::set_config(&config)?;

        Ok(Self {
            _peri,
            _sck,
            _mosi,
            _miso,
            _cs,
            _phantom: PhantomData,
            chip_select: config.chip_select,
        })
    }

    /// Apply configuration to the SPI peripheral.
    pub(super) fn set_config(config: &Config) -> Result<()> {
        let spi = T::regs();

        spi.cr().write(|w| w.men().disabled());
        spi.cr().modify(|_, w| w.rst().reset());
        spi.cr().modify(|_, w| w.rst().not_reset());
        spi.cr().modify(|_, w| w.rtf().txfifo_rst().rrf().rxfifo_rst());

        spi.cfgr1().write(|w| unsafe {
            w.master().master_mode().pincfg().sin_in_sout_out().pcspol().bits(0)
        });

        spi.ccr().write(|w| unsafe {
            w.sckdiv()
                .bits(config.sck_div)
                .dbt()
                .bits(config.sck_div)
                .pcssck()
                .bits(config.sck_div)
                .sckpcs()
                .bits(config.sck_div)
        });

        spi.fcr().write(|w| unsafe { w.txwater().bits(0).rxwater().bits(0) });

        let framesz = config.bits_per_frame.saturating_sub(1).min(0xFFF);
        spi.tcr().write(|w| unsafe {
            w.framesz().bits(framesz);
            match config.mode.polarity {
                Polarity::IdleLow => w.cpol().inactive_low(),
                Polarity::IdleHigh => w.cpol().inactive_high(),
            };
            match config.mode.phase {
                Phase::CaptureOnFirstTransition => w.cpha().captured(),
                Phase::CaptureOnSecondTransition => w.cpha().changed(),
            };
            match config.bit_order {
                BitOrder::MsbFirst => w.lsbf().msb_first(),
                BitOrder::LsbFirst => w.lsbf().lsb_first(),
            };
            match config.chip_select {
                ChipSelect::Pcs0 => w.pcs().tx_pcs0(),
                ChipSelect::Pcs1 => w.pcs().tx_pcs1(),
                ChipSelect::Pcs2 => w.pcs().tx_pcs2(),
                ChipSelect::Pcs3 => w.pcs().tx_pcs3(),
            };
            w.prescale().bits(config.prescaler as u8)
        });

        spi.cr().write(|w| w.men().enabled());

        Ok(())
    }

    #[inline]
    fn get_tx_fifo_count() -> u8 {
        T::regs().fsr().read().txcount().bits()
    }

    #[inline]
    fn get_rx_fifo_count() -> u8 {
        T::regs().fsr().read().rxcount().bits()
    }

    #[inline]
    fn get_fifo_size() -> u8 {
        LPSPI_FIFO_SIZE
    }

    #[inline]
    fn wait_tx_fifo_empty() -> Result<()> {
        spin_wait_while(|| Self::get_tx_fifo_count() != 0)
    }

    /// Check if the module is busy.
    #[inline]
    pub fn is_busy(&self) -> bool {
        T::regs().sr().read().mbf().is_busy()
    }

    /// Wait for all transfers to complete.
    pub fn flush(&self) -> Result<()> {
        spin_wait_while(|| self.is_busy())
    }

    #[inline]
    fn apply_transfer_tcr(&self, continuous_pcs: bool, rx_mask: bool, tx_mask: bool) {
        let spi = T::regs();
        spi.tcr().modify(|_, w| {
            if tx_mask {
                w.txmsk().mask();
            } else {
                w.txmsk().normal();
            }

            if rx_mask {
                w.rxmsk().mask();
            } else {
                w.rxmsk().normal();
            }

            if continuous_pcs {
                w.contc().continue_();
                w.cont().enabled();
            } else {
                w.contc().start();
                w.cont().disabled();
            }

            match self.chip_select {
                ChipSelect::Pcs0 => {
                    w.pcs().tx_pcs0();
                }
                ChipSelect::Pcs1 => {
                    w.pcs().tx_pcs1();
                }
                ChipSelect::Pcs2 => {
                    w.pcs().tx_pcs2();
                }
                ChipSelect::Pcs3 => {
                    w.pcs().tx_pcs3();
                }
            }

            w
        });
    }

    /// Full-duplex transfer (blocking).
    pub fn blocking_transfer(&self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        let tx_len = tx.len();
        let rx_len = rx.len();
        let len = tx_len.max(rx_len);
        if len == 0 {
            return Ok(());
        }
        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();
        prepare_for_blocking_transfer(spi);

        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        let is_pcs_continuous = len > 1;
        if is_pcs_continuous {
            self.apply_transfer_tcr(true, false, false);
            Self::wait_tx_fifo_empty()?;
        }

        let mut tx_remaining = len;
        let mut rx_remaining = len;
        let mut tx_idx = 0usize;
        let mut rx_idx = 0usize;
        let rx_fifo_max_bytes = fifo_size as usize;

        while tx_remaining > 0 {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;

            if rx_remaining - tx_remaining < rx_fifo_max_bytes {
                let byte = if tx_idx < tx_len { tx[tx_idx] } else { 0 };
                spi.tdr().write(|w| unsafe { w.bits(byte as u32) });
                tx_idx += 1;
                tx_remaining -= 1;
            }

            while Self::get_rx_fifo_count() > 0 && rx_remaining > 0 {
                let byte = spi.rdr().read().bits() as u8;
                if rx_idx < rx_len {
                    rx[rx_idx] = byte;
                }
                rx_idx += 1;
                rx_remaining -= 1;
            }
        }

        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;
            self.apply_transfer_tcr(false, false, false);
        }

        while rx_remaining > 0 {
            spin_wait_while(|| Self::get_rx_fifo_count() == 0)?;
            let byte = spi.rdr().read().bits() as u8;
            if rx_idx < rx_len {
                rx[rx_idx] = byte;
            }
            rx_idx += 1;
            rx_remaining -= 1;
        }

        Ok(())
    }

    /// TX-only transfer (blocking).
    pub fn blocking_write(&self, tx: &[u8]) -> Result<()> {
        if tx.is_empty() {
            return Ok(());
        }
        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();
        prepare_for_blocking_transfer(spi);

        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        let is_pcs_continuous = tx.len() > 1;
        self.apply_transfer_tcr(is_pcs_continuous, true, false);
        Self::wait_tx_fifo_empty()?;

        for &byte in tx.iter() {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;
            spi.tdr().write(|w| unsafe { w.bits(byte as u32) });
        }

        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;
            self.apply_transfer_tcr(false, false, false);
        }

        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

        Ok(())
    }

    /// RX-only transfer (blocking).
    pub fn blocking_read(&self, rx: &mut [u8]) -> Result<()> {
        if rx.is_empty() {
            return Ok(());
        }
        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();
        prepare_for_blocking_transfer(spi);

        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        let is_pcs_continuous = rx.len() > 1;
        if is_pcs_continuous {
            self.apply_transfer_tcr(true, false, false);
            Self::wait_tx_fifo_empty()?;
        }

        let mut tx_remaining = rx.len();
        let mut rx_remaining = rx.len();
        let mut rx_idx = 0usize;
        let rx_fifo_max_bytes = fifo_size as usize;

        while tx_remaining > 0 {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;

            if rx_remaining - tx_remaining < rx_fifo_max_bytes {
                spi.tdr().write(|w| unsafe { w.bits(0) });
                tx_remaining -= 1;
            }

            while Self::get_rx_fifo_count() > 0 && rx_remaining > 0 {
                rx[rx_idx] = spi.rdr().read().bits() as u8;
                rx_idx += 1;
                rx_remaining -= 1;
            }
        }

        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;
            self.apply_transfer_tcr(false, false, false);
        }

        while rx_remaining > 0 {
            spin_wait_while(|| Self::get_rx_fifo_count() == 0)?;
            rx[rx_idx] = spi.rdr().read().bits() as u8;
            rx_idx += 1;
            rx_remaining -= 1;
        }

        Ok(())
    }
}

// embedded-hal 1.0 implementations
impl<'d, T: Instance, M: Mode> embedded_hal_1::spi::ErrorType for Spi<'d, T, M> {
    type Error = Error;
}

impl<'d, T: Instance, M: Mode> embedded_hal_1::spi::SpiBus for Spi<'d, T, M> {
    fn read(&mut self, words: &mut [u8]) -> core::result::Result<(), Self::Error> {
        self.blocking_read(words)
    }

    fn write(&mut self, words: &[u8]) -> core::result::Result<(), Self::Error> {
        self.blocking_write(words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> core::result::Result<(), Self::Error> {
        self.blocking_transfer(write, read)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> core::result::Result<(), Self::Error> {
        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();

        for byte in words.iter_mut() {
            let tx_byte = *byte;
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;
            spi.tdr().write(|w| unsafe { w.bits(tx_byte as u32) });
            spin_wait_while(|| Self::get_rx_fifo_count() == 0)?;
            *byte = spi.rdr().read().bits() as u8;
        }
        Ok(())
    }

    fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        Spi::flush(self)
    }
}

impl<'d, T: Instance, M: Mode> embassy_embedded_hal::SetConfig for Spi<'d, T, M> {
    type Config = Config;
    type ConfigError = Error;

    fn set_config(&mut self, config: &Self::Config) -> Result<()> {
        Self::set_config(config)
    }
}