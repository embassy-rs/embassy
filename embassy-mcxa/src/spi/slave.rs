//! SPI Slave driver implementation.

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

/// SPI Slave Driver.
pub struct SpiSlave<'d, T: Instance, M: Mode> {
    _peri: Peri<'d, T>,
    _sck: Peri<'d, AnyPin>,
    _mosi: Peri<'d, AnyPin>,
    _miso: Peri<'d, AnyPin>,
    _cs: Peri<'d, AnyPin>,
    _phantom: PhantomData<M>,
}

impl<'d, T: Instance> SpiSlave<'d, T, Blocking> {
    /// Create a new blocking instance of the SPI Slave driver.
    pub fn new_blocking(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        config: SlaveConfig,
    ) -> Result<Self> {
        Self::new_inner(peri, sck, mosi, miso, cs, config)
    }
}

impl<'d, T: Instance> SpiSlave<'d, T, Async> {
    /// Create a new async (interrupt-driven) instance of the SPI Slave driver.
    pub fn new_async(
        peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: SlaveConfig,
    ) -> Result<Self> {
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        Self::new_inner(peri, sck, mosi, miso, cs, config)
    }

    #[inline]
    fn get_rx_fifo_count() -> u8 {
        T::regs().fsr().read().rxcount().bits()
    }

    #[inline]
    fn get_tx_fifo_count() -> u8 {
        T::regs().fsr().read().txcount().bits()
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

        spi.cr().modify(|_, w| w.men().disabled());
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cfgr1().modify(|_, w| w.nostall().enable());
        spi.fcr().write(|w| unsafe { w.txwater().bits(0).rxwater().bits(0) });
        spi.cr().modify(|_, w| w.men().enabled());

        spi.ier().write(|w| w);
        spi.tcr().modify(|_, w| w.rxmsk().normal().txmsk().mask());

        T::slave_irq_state().with(|st| {
            st.op = SlaveIrqOp::Rx;
            st.error = None;
            st.rx_ptr = rx.as_mut_ptr();
            st.rx_len = rx.len();
            st.rx_pos = 0;
        });

        spi.ier().write(|w| w.rdie().enable().reie().enable());

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

        spi.cr().modify(|_, w| w.men().disabled());
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cfgr1().modify(|_, w| w.nostall().enable());
        spi.fcr().write(|w| unsafe { w.txwater().bits(1).rxwater().bits(0) });
        spi.cr().modify(|_, w| w.men().enabled());

        spi.ier().write(|w| w);
        spi.tcr().modify(|_, w| w.rxmsk().mask().txmsk().normal());

        let mut prefill = 0usize;
        while prefill < tx.len() && Self::get_tx_fifo_count() < fifo_size {
            spi.tdr().write(|w| unsafe { w.bits(tx[prefill] as u32) });
            prefill += 1;
        }

        T::slave_irq_state().with(|st| {
            st.op = SlaveIrqOp::Tx;
            st.error = None;
            st.tx_ptr = tx.as_ptr();
            st.tx_len = tx.len();
            st.tx_pos = prefill;
        });

        spi.ier().write(|w| w.tdie().enable().teie().enable().fcie().enable());

        T::wait_cell()
            .wait_for(|| T::slave_irq_state().with(|st| st.op == SlaveIrqOp::Idle))
            .await
            .map_err(|_| Error::Timeout)?;

        let err = T::slave_irq_state().with(|st| st.error.take());
        if let Some(e) = err {
            return Err(e);
        }

        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

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

        spi.cr().modify(|_, w| w.men().disabled());
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cfgr1().modify(|_, w| w.nostall().enable());
        spi.fcr().write(|w| unsafe { w.txwater().bits(1).rxwater().bits(0) });
        spi.cr().modify(|_, w| w.men().enabled());

        spi.ier().write(|w| w);

        // Ensure full duplex: RX and TX unmasked.
        let tcr = Self::read_tcr_with_errata_workaround();
        let new_tcr = tcr & !(TCR_CONT | TCR_CONTC | TCR_RXMSK | TCR_TXMSK | TCR_PCS_MASK);
        spi.tcr().write(|w| unsafe { w.bits(new_tcr) });

        while Self::get_rx_fifo_count() > 0 {
            let _ = spi.rdr().read().bits();
        }

        let mut prefill = 0usize;
        while prefill < total && Self::get_tx_fifo_count() < fifo_size {
            let byte = if prefill < tx_len { tx[prefill] } else { 0 };
            spi.tdr().write(|w| unsafe { w.bits(byte as u32) });
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

        spi.ier()
            .write(|w| w.rdie().enable().reie().enable().tdie().enable().teie().enable());

        T::wait_cell()
            .wait_for(|| T::slave_irq_state().with(|st| st.op == SlaveIrqOp::Idle))
            .await
            .map_err(|_| Error::Timeout)?;

        let err = T::slave_irq_state().with(|st| st.error.take());
        if let Some(e) = err {
            return Err(e);
        }

        spin_wait_while(|| Self::get_tx_fifo_count() > 0)?;
        spin_wait_while(|| spi.sr().read().mbf().is_busy())?;

        Ok(())
    }
}

impl<'d, T: Instance, M: Mode> SpiSlave<'d, T, M> {
    fn new_inner(
        _peri: Peri<'d, T>,
        sck: Peri<'d, impl SckPin<T>>,
        mosi: Peri<'d, impl MosiPin<T>>,
        miso: Peri<'d, impl MisoPin<T>>,
        cs: Peri<'d, impl CsPin<T>>,
        config: SlaveConfig,
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

        let spi = T::regs();

        // Slave initialization sequence
        spi.cr().write(|w| w.men().disabled());
        spi.cfgr1().modify(|_, w| w.master().slave_mode());
        spi.cfgr1().modify(|_, w| unsafe { w.pcspol().bits(0) });
        spi.cfgr1()
            .modify(|_, w| w.outcfg().retain_lastvalue().pincfg().sin_in_sout_out());
        spi.fcr().write(|w| unsafe { w.txwater().bits(0).rxwater().bits(0) });

        let framesz = config.bits_per_frame.saturating_sub(1).min(0xFFF);
        spi.tcr().write(|w| unsafe {
            w.framesz().bits(framesz);
            match config.polarity {
                Polarity::IdleLow => w.cpol().inactive_low(),
                Polarity::IdleHigh => w.cpol().inactive_high(),
            };
            match config.phase {
                Phase::CaptureOnFirstTransition => w.cpha().captured(),
                Phase::CaptureOnSecondTransition => w.cpha().changed(),
            };
            match config.bit_order {
                BitOrder::MsbFirst => w.lsbf().msb_first(),
                BitOrder::LsbFirst => w.lsbf().lsb_first(),
            }
        });

        spi.cr().write(|w| w.men().enabled());

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

        spi.cr().modify(|_, w| w.men().disabled());
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cr().modify(|_, w| w.men().enabled());

        spi.tcr().modify(|_, w| w.rxmsk().normal().txmsk().mask());

        for byte in rx.iter_mut() {
            while spi.fsr().read().rxcount().bits() == 0 {}
            *byte = spi.rdr().read().bits() as u8;
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

        spi.cr().modify(|_, w| w.men().disabled());
        flush_fifos(spi);
        clear_status_flags(spi);
        spi.cr().modify(|_, w| w.men().enabled());

        spi.tcr().modify(|_, w| w.rxmsk().mask().txmsk().normal());

        let mut tx_idx = 0usize;
        while tx_idx < tx.len() && spi.fsr().read().txcount().bits() < fifo_size {
            spi.tdr().write(|w| unsafe { w.bits(tx[tx_idx] as u32) });
            tx_idx += 1;
        }

        while tx_idx < tx.len() {
            while spi.fsr().read().txcount().bits() >= fifo_size {}
            spi.tdr().write(|w| unsafe { w.bits(tx[tx_idx] as u32) });
            tx_idx += 1;
        }

        while spi.fsr().read().txcount().bits() != 0 {}
        while spi.sr().read().mbf().is_busy() {}

        Ok(())
    }
}

