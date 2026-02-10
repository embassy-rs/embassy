//! SPI Slave driver implementation.

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
use crate::pac::lpspi::vals::{Contc, Cpha, Cpol, Lsbf, Master, Mbf, Outcfg, Pcs, Pcspol, Pincfg, Rxmsk, Txmsk};

/// SPI Slave Driver.
///
/// The CS pin is optional. When `Some(pin)`, the hardware PCS signal is used for chip select.
/// When `None`, users must manage chip select externally. Note: For most slave use cases,
/// a CS pin is required to know when the slave is being addressed.
pub struct SpiSlave<'d, T: SlaveInstance, M: Mode> {
    _peri: Peri<'d, T>,
    _sck: Peri<'d, AnyPin>,
    _mosi: Peri<'d, AnyPin>,
    _miso: Peri<'d, AnyPin>,
    _cs: Option<Peri<'d, AnyPin>>,
    _phantom: PhantomData<M>,
}

impl<'d, T: SlaveInstance> SpiSlave<'d, T, Blocking> {
    /// Create a new blocking instance of the SPI Slave driver.
    ///
    /// # Arguments
    /// * `cs` - Optional chip select pin. When `Some(pin)`, hardware PCS is used.
    ///   When `None`, users must manage CS externally. Note: Most slave applications
    ///   require a CS signal.
    pub fn new_blocking(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Option<Peri<'d, impl CsPin<T>>>,
        config: SlaveConfig,
    ) -> Result<Self> {
        Self::new_inner(peri, sck, mosi, miso, cs, config)
    }
}

impl<'d, T: SlaveInstance> SpiSlave<'d, T, Async> {
    /// Create a new async (interrupt-driven) instance of the SPI Slave driver.
    ///
    /// # Arguments
    /// * `cs` - Optional chip select pin. When `Some(pin)`, hardware PCS is used.
    ///   When `None`, users must manage CS externally. Note: Most slave applications
    ///   require a CS signal.
    pub fn new_async(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Option<Peri<'d, impl CsPin<T>>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, SlaveInterruptHandler<T>> + 'd,
        config: SlaveConfig,
    ) -> Result<Self> {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        Self::new_inner(peri, sck, mosi, miso, cs, config)
    }

    #[inline]
    fn get_rx_fifo_count() -> u8 {
        T::regs().fsr().read().rxcount()
    }

    #[inline]
    fn get_tx_fifo_count() -> u8 {
        T::regs().fsr().read().txcount()
    }

    #[inline]
    fn get_fifo_size() -> u8 {
        LPSPI_FIFO_SIZE
    }

    #[inline]
    fn read_tcr_with_errata_workaround() -> u32 {
        read_tcr_with_errata_workaround(T::regs())
    }

    /// Async read from master.
    pub async fn read(&mut self, rx: &mut [u8]) -> Result<()> {
        if rx.is_empty() {
            return Ok(());
        }

        let spi = T::regs();

        spi.cr().modify(|w| w.set_men(false));
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cfgr1().modify(|w| w.set_nostall(true));
        spi.fcr().write(|w| {
            w.set_txwater(0);
            w.set_rxwater(0);
        });
        spi.cr().modify(|w| w.set_men(true));

        disable_all_interrupts(spi);
        spi.tcr().modify(|w| {
            w.set_rxmsk(Rxmsk::NORMAL);
            w.set_txmsk(Txmsk::MASK);
        });

        T::slave_irq_state().with(|st| {
            st.op = SlaveIrqOp::Rx;
            st.error = None;
            st.rx_ptr = rx.as_mut_ptr();
            st.rx_len = rx.len();
            st.rx_pos = 0;
        });

        spi.ier().write(|w| {
            w.set_rdie(true);
            w.set_reie(true);
        });

        T::wait_cell()
            .wait_for(|| T::slave_irq_state().with(|st| st.op == SlaveIrqOp::Idle))
            .await
            .map_err(|_| Error::Timeout)?;

        let err = T::slave_irq_state().with(|st| st.error.take());
        if let Some(e) = err {
            return Err(e);
        }

        Ok(())
    }

    /// Async write to master.
    pub async fn write(&mut self, tx: &[u8]) -> Result<()> {
        if tx.is_empty() {
            return Ok(());
        }

        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();

        spi.cr().modify(|w| w.set_men(false));
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cfgr1().modify(|w| w.set_nostall(true));
        spi.fcr().write(|w| {
            w.set_txwater(1);
            w.set_rxwater(0);
        });
        spi.cr().modify(|w| w.set_men(true));

        disable_all_interrupts(spi);
        spi.tcr().modify(|w| {
            w.set_rxmsk(Rxmsk::MASK);
            w.set_txmsk(Txmsk::NORMAL);
        });

        let mut prefill = 0usize;
        while prefill < tx.len() && Self::get_tx_fifo_count() < fifo_size {
            spi.tdr().write(|w| w.set_data(tx[prefill] as u32));
            prefill += 1;
        }

        T::slave_irq_state().with(|st| {
            st.op = SlaveIrqOp::Tx;
            st.error = None;
            st.tx_ptr = tx.as_ptr();
            st.tx_len = tx.len();
            st.tx_pos = prefill;
        });

        spi.ier().write(|w| {
            w.set_tdie(true);
            w.set_teie(true);
            w.set_fcie(true);
        });

        T::wait_cell()
            .wait_for(|| T::slave_irq_state().with(|st| st.op == SlaveIrqOp::Idle))
            .await
            .map_err(|_| Error::Timeout)?;

        let err = T::slave_irq_state().with(|st| st.error.take());
        if let Some(e) = err {
            return Err(e);
        }

        spin_wait_while(|| spi.sr().read().mbf() == Mbf::BUSY)?;

        Ok(())
    }

    /// Async full-duplex transfer (interrupt-driven).
    pub async fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        if tx.is_empty() && rx.is_empty() {
            return Ok(());
        }

        let spi = T::regs();
        let fifo_size = Self::get_fifo_size();

        let tx_len = tx.len();
        let rx_len = rx.len();
        let total = core::cmp::max(tx_len, rx_len);
        if total == 0 {
            return Ok(());
        }

        spi.cr().modify(|w| w.set_men(false));
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cfgr1().modify(|w| w.set_nostall(true));
        spi.fcr().write(|w| {
            w.set_txwater(1);
            w.set_rxwater(0);
        });
        spi.cr().modify(|w| w.set_men(true));

        disable_all_interrupts(spi);

        // Ensure full duplex: RX and TX unmasked, clear CONT/CONTC/PCS using PAC accessors.
        let mut tcr = crate::pac::lpspi::regs::Tcr(Self::read_tcr_with_errata_workaround());
        tcr.set_cont(false);
        tcr.set_contc(Contc::START);
        tcr.set_rxmsk(Rxmsk::NORMAL);
        tcr.set_txmsk(Txmsk::NORMAL);
        tcr.set_pcs(Pcs::TX_PCS0);
        spi.tcr().write_value(tcr);

        while Self::get_rx_fifo_count() > 0 {
            let _ = spi.rdr().read().data();
        }

        let mut prefill = 0usize;
        while prefill < total && Self::get_tx_fifo_count() < fifo_size {
            let byte = if prefill < tx_len { tx[prefill] } else { 0 };
            spi.tdr().write(|w| w.set_data(byte as u32));
            prefill += 1;
        }

        T::slave_irq_state().with(|st| {
            st.op = SlaveIrqOp::Transfer;
            st.error = None;

            st.rx_ptr = rx.as_mut_ptr();
            st.rx_len = total;
            st.rx_pos = 0;
            st.rx_store_len = rx_len;

            st.tx_ptr = tx.as_ptr();
            st.tx_len = total;
            st.tx_pos = prefill;
            st.tx_source_len = tx_len;
        });

        spi.ier().write(|w| {
            w.set_rdie(true);
            w.set_reie(true);
            w.set_tdie(true);
            w.set_teie(true);
        });

        T::wait_cell()
            .wait_for(|| T::slave_irq_state().with(|st| st.op == SlaveIrqOp::Idle))
            .await
            .map_err(|_| Error::Timeout)?;

        let err = T::slave_irq_state().with(|st| st.error.take());
        if let Some(e) = err {
            return Err(e);
        }

        spin_wait_while(|| Self::get_tx_fifo_count() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf() == Mbf::BUSY)?;

        Ok(())
    }
}

impl<'d, T: SlaveInstance, M: Mode> SpiSlave<'d, T, M> {
    fn new_inner(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Option<Peri<'d, impl CsPin<T>>>,
        config: SlaveConfig,
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
        if let Some(ref pin) = cs {
            pin.mux();
        }

        let _sck = sck.into();
        let _mosi = mosi.into();
        let _miso = miso.into();
        let _cs = cs.map(|p| p.into());

        let spi = T::regs();

        // Slave initialization sequence
        spi.cr().write(|w| w.set_men(false));
        spi.cfgr1().modify(|w| w.set_master(Master::SLAVE_MODE));
        spi.cfgr1().modify(|w| w.set_pcspol(Pcspol::from_bits(0)));
        spi.cfgr1().modify(|w| {
            w.set_outcfg(Outcfg::RETAIN_LASTVALUE);
            w.set_pincfg(Pincfg::SIN_IN_SOUT_OUT);
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
        });

        spi.cr().write(|w| w.set_men(true));

        Ok(Self {
            _peri,
            _sck,
            _mosi,
            _miso,
            _cs,
            _phantom: PhantomData,
        })
    }

    /// RX-only receive from the master (blocking).
    pub fn blocking_read(&self, rx: &mut [u8]) -> Result<()> {
        if rx.is_empty() {
            return Ok(());
        }
        let spi = T::regs();

        spi.cr().modify(|w| w.set_men(false));
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cr().modify(|w| w.set_men(true));

        spi.tcr().modify(|w| {
            w.set_rxmsk(Rxmsk::NORMAL);
            w.set_txmsk(Txmsk::MASK);
        });

        for byte in rx.iter_mut() {
            while spi.fsr().read().rxcount() == 0 {}
            *byte = spi.rdr().read().data() as u8;
        }

        Ok(())
    }

    /// TX-only transmit to the master (blocking).
    pub fn blocking_write(&self, tx: &[u8]) -> Result<()> {
        if tx.is_empty() {
            return Ok(());
        }
        let spi = T::regs();
        let fifo_size = LPSPI_FIFO_SIZE;

        spi.cr().modify(|w| w.set_men(false));
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cr().modify(|w| w.set_men(true));

        spi.tcr().modify(|w| {
            w.set_rxmsk(Rxmsk::MASK);
            w.set_txmsk(Txmsk::NORMAL);
        });

        let mut tx_idx = 0usize;
        while tx_idx < tx.len() && spi.fsr().read().txcount() < fifo_size {
            spi.tdr().write(|w| w.set_data(tx[tx_idx] as u32));
            tx_idx += 1;
        }

        while tx_idx < tx.len() {
            while spi.fsr().read().txcount() >= fifo_size {}
            spi.tdr().write(|w| w.set_data(tx[tx_idx] as u32));
            tx_idx += 1;
        }

        while spi.fsr().read().txcount() != 0 {}
        while spi.sr().read().mbf() == Mbf::BUSY {}

        Ok(())
    }
}
