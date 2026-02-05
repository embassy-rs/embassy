//! SPI Master driver implementation.

use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use embedded_hal_02::spi::{Phase, Polarity};

use super::common::*;
use super::pins::*;
use crate::clocks::enable_and_reset;
use crate::clocks::periph_helpers::LpspiConfig;
use crate::gpio::AnyPin;
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::lpspi::vals::{Contc, Cpha, Cpol, Lsbf, Master, Mbf, Pcs, Pcspol, Pincfg, Prescale, Rxmsk, Txmsk};

/// SPI Master Driver.
///
/// The type parameter `C` indicates the chip select mode:
/// - [`HardwareCs`]: The LPSPI hardware controls the PCS signal automatically
/// - [`NoCs`]: The user manages chip select externally via GPIO
///
/// Only `Spi<..., NoCs>` implements [`embedded_hal_1::spi::SpiBus`], as `SpiBus`
/// semantics require that the bus does not manage CS. Use `NoCs` mode when you
/// need to share the SPI bus with [`embassy_embedded_hal::shared_bus::SpiDevice`].
pub struct Spi<'d, T: Instance, M: Mode, C: CsMode> {
    _peri: Peri<'d, T>,
    _sck: Peri<'d, AnyPin>,
    _mosi: Peri<'d, AnyPin>,
    _miso: Peri<'d, AnyPin>,
    _cs: Option<Peri<'d, AnyPin>>,
    _phantom: PhantomData<(M, C)>,
    chip_select: ChipSelect,
}

impl<'d, T: Instance> Spi<'d, T, Blocking, HardwareCs> {
    /// Create a new blocking SPI driver with hardware-managed chip select.
    ///
    /// The LPSPI hardware will automatically control the PCS (Peripheral Chip Select)
    /// signal. Use this when you have a single device on the bus.
    pub fn new_blocking(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        config: Config,
    ) -> Result<Self> {
        Self::new_inner_with_cs(peri, sck, mosi, miso, cs, config)
    }
}

impl<'d, T: Instance> Spi<'d, T, Blocking, NoCs> {
    /// Create a new blocking SPI driver without hardware chip select.
    ///
    /// The user must manage chip select externally via GPIO. Use this when:
    /// - You have multiple devices on the same SPI bus
    /// - You need to use [`embassy_embedded_hal::shared_bus::SpiDevice`]
    /// - You need custom CS timing or behavior
    ///
    /// This variant implements [`embedded_hal_1::spi::SpiBus`].
    pub fn new_blocking_no_cs(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        config: Config,
    ) -> Result<Self> {
        Self::new_inner_no_cs(peri, sck, mosi, miso, config)
    }
}

impl<'d, T: Instance> Spi<'d, T, Async, HardwareCs> {
    /// Create a new async SPI driver with hardware-managed chip select.
    ///
    /// The LPSPI hardware will automatically control the PCS (Peripheral Chip Select)
    /// signal. Use this when you have a single device on the bus.
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
        Self::new_inner_with_cs(peri, sck, mosi, miso, cs, config)
    }
}

impl<'d, T: Instance> Spi<'d, T, Async, NoCs> {
    /// Create a new async SPI driver without hardware chip select.
    ///
    /// The user must manage chip select externally via GPIO. Use this when:
    /// - You have multiple devices on the same SPI bus
    /// - You need to use [`embassy_embedded_hal::shared_bus::SpiDevice`]
    /// - You need custom CS timing or behavior
    ///
    /// This variant implements [`embedded_hal_1::spi::SpiBus`].
    pub fn new_async_no_cs(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self> {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        Self::new_inner_no_cs(peri, sck, mosi, miso, config)
    }
}

impl<'d, T: Instance, C: CsMode> Spi<'d, T, Async, C> {
    /// Async write (interrupt-driven).
    pub async fn write(&mut self, tx: &[u8]) -> Result<()> {
        if tx.is_empty() {
            return Ok(());
        }

        let spi = T::regs();
        prepare_for_transfer(spi);

        // Use continuous PCS (chip select held active) when transferring multiple bytes.
        // For single-byte transfers, PCS toggles per byte which is correct behavior.
        let is_pcs_continuous = tx.len() > 1;

        self.apply_transfer_tcr(false, false, false);
        Self::wait_tx_fifo_empty()?;

        self.apply_transfer_tcr(is_pcs_continuous, true, false);
        Self::wait_tx_fifo_empty()?;

        // Initial FIFO fill - fill as much as possible without waiting.
        // This avoids an unnecessary wait when the FIFO is empty at the start.
        let mut tx_idx = 0usize;
        while tx_idx < tx.len() && Self::get_tx_fifo_count() < Self::get_fifo_size() {
            spi.tdr().write(|w| w.set_data(tx[tx_idx] as u32));
            tx_idx += 1;
        }

        // Continue filling via interrupt-driven wait for remaining data.
        while tx_idx < tx.len() {
            T::wait_cell()
                .wait_for(|| {
                    spi.ier().modify(|w| w.set_tdie(true));
                    Self::get_tx_fifo_count() < Self::get_fifo_size()
                })
                .await
                .map_err(|_| Error::Timeout)?;

            while tx_idx < tx.len() && Self::get_tx_fifo_count() < Self::get_fifo_size() {
                spi.tdr().write(|w| w.set_data(tx[tx_idx] as u32));
                tx_idx += 1;
            }
        }

        if is_pcs_continuous {
            // Wait for at least one TX FIFO slot to be available (FIFO not full)
            // before writing the final TCR command to de-assert PCS.
            T::wait_cell()
                .wait_for(|| {
                    spi.ier().modify(|w| w.set_tdie(true));
                    Self::get_tx_fifo_count() < Self::get_fifo_size()
                })
                .await
                .map_err(|_| Error::Timeout)?;
            self.apply_transfer_tcr(false, true, false);
        }

        Self::wait_tx_fifo_empty()?;
        // Spin-wait for Module Busy Flag to clear (last bits shifting out).
        // There's no interrupt for MBF - it's a read-only status bit indicating
        // the LPSPI is actively shifting data. This spin is very short in practice.
        spin_wait_while(|| spi.sr().read().mbf() == Mbf::BUSY)?;

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
                spi.tdr().write(|w| w.set_data(0));
                tx_remaining -= 1;
            }

            while Self::get_rx_fifo_count() > 0 && rx_idx < rx.len() {
                rx[rx_idx] = spi.rdr().read().data() as u8;
                rx_idx += 1;
            }

            if tx_remaining > 0 || rx_idx < rx.len() {
                T::wait_cell()
                    .wait_for(|| {
                        spi.ier().modify(|w| w.set_rdie(true));
                        Self::get_rx_fifo_count() > 0 || Self::get_tx_fifo_count() < Self::get_fifo_size()
                    })
                    .await
                    .map_err(|_| Error::Timeout)?;
            }
        }

        if is_pcs_continuous {
            // Wait for at least one TX FIFO slot to be available (FIFO not full)
            // before writing the final TCR command to de-assert PCS.
            T::wait_cell()
                .wait_for(|| {
                    spi.ier().modify(|w| w.set_tdie(true));
                    Self::get_tx_fifo_count() < Self::get_fifo_size()
                })
                .await
                .map_err(|_| Error::Timeout)?;
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
                spi.tdr().write(|w| w.set_data(byte as u32));
                tx_idx += 1;
            }

            while Self::get_rx_fifo_count() > 0 && rx_idx < len {
                let byte = spi.rdr().read().data() as u8;
                if rx_idx < rx_len {
                    rx[rx_idx] = byte;
                }
                rx_idx += 1;
            }

            if tx_idx < len || rx_idx < len {
                T::wait_cell()
                    .wait_for(|| {
                        spi.ier().modify(|w| w.set_rdie(true));
                        Self::get_rx_fifo_count() > 0 || Self::get_tx_fifo_count() < Self::get_fifo_size()
                    })
                    .await
                    .map_err(|_| Error::Timeout)?;
            }
        }

        if is_pcs_continuous {
            // Wait for at least one TX FIFO slot to be available (FIFO not full)
            // before writing the final TCR command to de-assert PCS.
            T::wait_cell()
                .wait_for(|| {
                    spi.ier().modify(|w| w.set_tdie(true));
                    Self::get_tx_fifo_count() < Self::get_fifo_size()
                })
                .await
                .map_err(|_| Error::Timeout)?;
            self.apply_transfer_tcr(false, false, false);
        }

        Ok(())
    }
}

impl<'d, T: Instance, M: Mode> Spi<'d, T, M, HardwareCs> {
    fn new_inner_with_cs(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        config: Config,
    ) -> Result<Self> {
        let clock_config = LpspiConfig {
            power: config.clock_power,
            source: config.clock_source,
            div: config.clock_div,
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
        let _cs = Some(cs.into());

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
}

impl<'d, T: Instance, M: Mode> Spi<'d, T, M, NoCs> {
    fn new_inner_no_cs(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        config: Config,
    ) -> Result<Self> {
        let clock_config = LpspiConfig {
            power: config.clock_power,
            source: config.clock_source,
            div: config.clock_div,
            instance: T::CLOCK_INSTANCE,
        };

        _ = unsafe { enable_and_reset::<T>(&clock_config).map_err(Error::ClockSetup)? };

        sck.mux();
        mosi.mux();
        miso.mux();

        let _sck = sck.into();
        let _mosi = mosi.into();
        let _miso = miso.into();

        Self::set_config(&config)?;

        Ok(Self {
            _peri,
            _sck,
            _mosi,
            _miso,
            _cs: None,
            _phantom: PhantomData,
            chip_select: config.chip_select,
        })
    }
}

impl<'d, T: Instance, M: Mode, C: CsMode> Spi<'d, T, M, C> {
    /// Apply configuration to the SPI peripheral.
    pub(super) fn set_config(config: &Config) -> Result<()> {
        let spi = T::regs();

        spi.cr().write(|w| w.set_men(false));
        spi.cr().modify(|w| w.set_rst(true));
        spi.cr().modify(|w| w.set_rst(false));
        flush_fifos(spi);

        spi.cfgr1().write(|w| {
            w.set_master(Master::MASTER_MODE);
            w.set_pincfg(Pincfg::SIN_IN_SOUT_OUT);
            w.set_pcspol(Pcspol::from_bits(0));
        });

        // sck_div is written to DBT (Delay Between Transfers), PCSSCK (PCS-to-SCK Delay),
        // and SCKPCS (SCK-to-PCS Delay) to maintain symmetric timing around the clock signal.
        spi.ccr().write(|w| {
            w.set_sckdiv(config.sck_div);
            w.set_dbt(config.sck_div);
            w.set_pcssck(config.sck_div);
            w.set_sckpcs(config.sck_div);
        });

        spi.fcr().write(|w| {
            w.set_txwater(0);
            w.set_rxwater(0);
        });

        let framesz = config.bits_per_frame.saturating_sub(1).min(0xFFF);
        spi.tcr().write(|w| {
            w.set_framesz(framesz);
            w.set_cpol(match config.mode.polarity {
                Polarity::IdleLow => Cpol::INACTIVE_LOW,
                Polarity::IdleHigh => Cpol::INACTIVE_HIGH,
            });
            w.set_cpha(match config.mode.phase {
                Phase::CaptureOnFirstTransition => Cpha::CAPTURED,
                Phase::CaptureOnSecondTransition => Cpha::CHANGED,
            });
            w.set_lsbf(match config.bit_order {
                BitOrder::MsbFirst => Lsbf::MSB_FIRST,
                BitOrder::LsbFirst => Lsbf::LSB_FIRST,
            });
            w.set_pcs(match config.chip_select {
                ChipSelect::Pcs0 => Pcs::TX_PCS0,
                ChipSelect::Pcs1 => Pcs::TX_PCS1,
                ChipSelect::Pcs2 => Pcs::TX_PCS2,
                ChipSelect::Pcs3 => Pcs::TX_PCS3,
            });
            w.set_prescale(Prescale::from_bits(config.prescaler as u8));
        });

        spi.cr().write(|w| w.set_men(true));

        Ok(())
    }

    #[inline]
    fn get_tx_fifo_count() -> u8 {
        T::regs().fsr().read().txcount()
    }

    #[inline]
    fn get_rx_fifo_count() -> u8 {
        T::regs().fsr().read().rxcount()
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
        T::regs().sr().read().mbf() == Mbf::BUSY
    }

    /// Wait for all transfers to complete.
    pub fn flush(&self) -> Result<()> {
        spin_wait_while(|| self.is_busy())
    }

    #[inline]
    fn apply_transfer_tcr(&self, continuous_pcs: bool, rx_mask: bool, tx_mask: bool) {
        let spi = T::regs();
        spi.tcr().modify(|w| {
            w.set_txmsk(if tx_mask { Txmsk::MASK } else { Txmsk::NORMAL });
            w.set_rxmsk(if rx_mask { Rxmsk::MASK } else { Rxmsk::NORMAL });

            if continuous_pcs {
                w.set_contc(Contc::CONTINUE);
                w.set_cont(true);
            } else {
                w.set_contc(Contc::START);
                w.set_cont(false);
            }

            w.set_pcs(match self.chip_select {
                ChipSelect::Pcs0 => Pcs::TX_PCS0,
                ChipSelect::Pcs1 => Pcs::TX_PCS1,
                ChipSelect::Pcs2 => Pcs::TX_PCS2,
                ChipSelect::Pcs3 => Pcs::TX_PCS3,
            });
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

        // Use continuous PCS (chip select held active) when transferring multiple bytes.
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
                spi.tdr().write(|w| w.set_data(byte as u32));
                tx_idx += 1;
                tx_remaining -= 1;
            }

            while Self::get_rx_fifo_count() > 0 && rx_remaining > 0 {
                let byte = spi.rdr().read().data() as u8;
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
            let byte = spi.rdr().read().data() as u8;
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
            spi.tdr().write(|w| w.set_data(byte as u32));
        }

        if is_pcs_continuous {
            spin_wait_while(|| Self::get_tx_fifo_count() >= fifo_size)?;
            self.apply_transfer_tcr(false, false, false);
        }

        // Spin-wait for Module Busy Flag to clear (last bits shifting out).
        // There's no interrupt for MBF - it's a read-only status bit indicating
        // the LPSPI is actively shifting data. This spin is very short in practice.
        spin_wait_while(|| spi.sr().read().mbf() == Mbf::BUSY)?;

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
                spi.tdr().write(|w| w.set_data(0));
                tx_remaining -= 1;
            }

            while Self::get_rx_fifo_count() > 0 && rx_remaining > 0 {
                rx[rx_idx] = spi.rdr().read().data() as u8;
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
            rx[rx_idx] = spi.rdr().read().data() as u8;
            rx_idx += 1;
            rx_remaining -= 1;
        }

        Ok(())
    }
}

// embedded-hal 1.0 implementations
impl<'d, T: Instance, M: Mode, C: CsMode> embedded_hal_1::spi::ErrorType for Spi<'d, T, M, C> {
    type Error = Error;
}

// SpiBus is only implemented for NoCs (externally-managed chip select).
// This follows embedded-hal semantics where SpiBus represents exclusive bus access
// without CS management. For hardware CS, the LPSPI toggles PCS on each transfer,
// which doesn't match SpiBus expectations.
//
// To use SpiBus semantics (e.g., with embassy-embedded-hal::shared_bus::SpiDevice):
// 1. Create the SPI driver with `new_blocking_no_cs()` or `new_async_no_cs()`
// 2. Wrap with `SpiDevice` which manages CS via GPIO
impl<'d, T: Instance, M: Mode> embedded_hal_1::spi::SpiBus for Spi<'d, T, M, NoCs> {
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
            spi.tdr().write(|w| w.set_data(tx_byte as u32));
            spin_wait_while(|| Self::get_rx_fifo_count() == 0)?;
            *byte = spi.rdr().read().data() as u8;
        }
        Ok(())
    }

    fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        Spi::flush(self)
    }
}

impl<'d, T: Instance, M: Mode, C: CsMode> embassy_embedded_hal::SetConfig for Spi<'d, T, M, C> {
    type Config = Config;
    type ConfigError = Error;

    fn set_config(&mut self, config: &Self::Config) -> Result<()> {
        Self::set_config(config)
    }
}
