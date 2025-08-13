use core::cmp;
use core::future::poll_fn;
use core::task::Poll;

use config::{Address, OwnAddresses, OA2};
use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::drop::OnDrop;
use embedded_hal_1::i2c::Operation;
use mode::{Master, MultiMaster};
use stm32_metapac::i2c::vals::{Addmode, Oamsk};

use super::*;
use crate::pac::i2c;

impl From<AddrMask> for Oamsk {
    fn from(value: AddrMask) -> Self {
        match value {
            AddrMask::NOMASK => Oamsk::NO_MASK,
            AddrMask::MASK1 => Oamsk::MASK1,
            AddrMask::MASK2 => Oamsk::MASK2,
            AddrMask::MASK3 => Oamsk::MASK3,
            AddrMask::MASK4 => Oamsk::MASK4,
            AddrMask::MASK5 => Oamsk::MASK5,
            AddrMask::MASK6 => Oamsk::MASK6,
            AddrMask::MASK7 => Oamsk::MASK7,
        }
    }
}

impl Address {
    pub(super) fn add_mode(&self) -> stm32_metapac::i2c::vals::Addmode {
        match self {
            Address::SevenBit(_) => stm32_metapac::i2c::vals::Addmode::BIT7,
            Address::TenBit(_) => stm32_metapac::i2c::vals::Addmode::BIT10,
        }
    }
}

enum ReceiveResult {
    DataAvailable,
    StopReceived,
    NewStart,
}

fn debug_print_interrupts(isr: stm32_metapac::i2c::regs::Isr) {
    if isr.tcr() {
        trace!("interrupt: tcr");
    }
    if isr.tc() {
        trace!("interrupt: tc");
    }
    if isr.addr() {
        trace!("interrupt: addr");
    }
    if isr.stopf() {
        trace!("interrupt: stopf");
    }
    if isr.nackf() {
        trace!("interrupt: nackf");
    }
    if isr.berr() {
        trace!("interrupt: berr");
    }
    if isr.arlo() {
        trace!("interrupt: arlo");
    }
    if isr.ovr() {
        trace!("interrupt: ovr");
    }
}

pub(crate) unsafe fn on_interrupt<T: Instance>() {
    let regs = T::info().regs;
    let isr = regs.isr().read();

    if isr.tcr() || isr.tc() || isr.addr() || isr.stopf() || isr.nackf() || isr.berr() || isr.arlo() || isr.ovr() {
        debug_print_interrupts(isr);

        T::state().waker.wake();
    }

    critical_section::with(|_| {
        regs.cr1().modify(|w| {
            w.set_addrie(false);
            w.set_stopie(false);
            // The flag can only be cleared by writting to nbytes, we won't do that here
            w.set_tcie(false);
            // Error flags are to be read in the routines, so we also don't clear them here
            w.set_nackie(false);
            w.set_errie(false);
        });
    });
}

impl<'d, M: Mode, IM: MasterMode> I2c<'d, M, IM> {
    pub(crate) fn init(&mut self, config: Config) {
        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(false);
            reg.set_anfoff(false);
        });

        let timings = Timings::new(self.kernel_clock, config.frequency.into());

        self.info.regs.timingr().write(|reg| {
            reg.set_presc(timings.prescale);
            reg.set_scll(timings.scll);
            reg.set_sclh(timings.sclh);
            reg.set_sdadel(timings.sdadel);
            reg.set_scldel(timings.scldel);
        });

        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(true);
        });
    }

    fn master_stop(&mut self) {
        self.info.regs.cr2().write(|w| w.set_stop(true));
    }

    fn master_read(
        info: &'static Info,
        address: Address,
        length: usize,
        stop: Stop,
        reload: bool,
        restart: bool,
        timeout: Timeout,
    ) -> Result<(), Error> {
        assert!(length < 256);

        if !restart {
            // Wait for any previous address sequence to end
            // automatically. This could be up to 50% of a bus
            // cycle (ie. up to 0.5/freq)
            while info.regs.cr2().read().start() {
                timeout.check()?;
            }
        }

        // Set START and prepare to receive bytes into
        // `buffer`. The START bit can be set even if the bus
        // is BUSY or I2C is in slave mode.

        let reload = if reload {
            i2c::vals::Reload::NOT_COMPLETED
        } else {
            i2c::vals::Reload::COMPLETED
        };

        info.regs.cr2().modify(|w| {
            w.set_sadd(address.addr() << 1);
            w.set_add10(address.add_mode());
            w.set_dir(i2c::vals::Dir::READ);
            w.set_nbytes(length as u8);
            w.set_start(true);
            w.set_autoend(stop.autoend());
            w.set_reload(reload);
        });

        Ok(())
    }

    fn master_write(
        info: &'static Info,
        address: Address,
        length: usize,
        stop: Stop,
        reload: bool,
        timeout: Timeout,
    ) -> Result<(), Error> {
        assert!(length < 256);

        // Wait for any previous address sequence to end
        // automatically. This could be up to 50% of a bus
        // cycle (ie. up to 0.5/freq)
        while info.regs.cr2().read().start() {
            timeout.check()?;
        }

        // Wait for the bus to be free
        while info.regs.isr().read().busy() {
            timeout.check()?;
        }

        let reload = if reload {
            i2c::vals::Reload::NOT_COMPLETED
        } else {
            i2c::vals::Reload::COMPLETED
        };

        // Set START and prepare to send `bytes`. The
        // START bit can be set even if the bus is BUSY or
        // I2C is in slave mode.
        info.regs.cr2().modify(|w| {
            w.set_sadd(address.addr() << 1);
            w.set_add10(address.add_mode());
            w.set_dir(i2c::vals::Dir::WRITE);
            w.set_nbytes(length as u8);
            w.set_start(true);
            w.set_autoend(stop.autoend());
            w.set_reload(reload);
        });

        Ok(())
    }

    fn reload(info: &'static Info, length: usize, will_reload: bool, timeout: Timeout) -> Result<(), Error> {
        assert!(length < 256 && length > 0);

        while !info.regs.isr().read().tcr() {
            timeout.check()?;
        }

        let will_reload = if will_reload {
            i2c::vals::Reload::NOT_COMPLETED
        } else {
            i2c::vals::Reload::COMPLETED
        };

        info.regs.cr2().modify(|w| {
            w.set_nbytes(length as u8);
            w.set_reload(will_reload);
        });

        Ok(())
    }

    fn flush_txdr(&self) {
        if self.info.regs.isr().read().txis() {
            trace!("Flush TXDATA with zeroes");
            self.info.regs.txdr().modify(|w| w.set_txdata(0));
        }
        if !self.info.regs.isr().read().txe() {
            trace!("Flush TXDR");
            self.info.regs.isr().modify(|w| w.set_txe(true))
        }
    }

    fn error_occurred(&self, isr: &i2c::regs::Isr, timeout: Timeout) -> Result<(), Error> {
        if isr.nackf() {
            trace!("NACK triggered.");
            self.info.regs.icr().modify(|reg| reg.set_nackcf(true));
            // NACK should be followed by STOP
            if let Ok(()) = self.wait_stop(timeout) {
                trace!("Got STOP after NACK, clearing flag.");
                self.info.regs.icr().modify(|reg| reg.set_stopcf(true));
            }
            self.flush_txdr();
            return Err(Error::Nack);
        } else if isr.berr() {
            trace!("BERR triggered.");
            self.info.regs.icr().modify(|reg| reg.set_berrcf(true));
            self.flush_txdr();
            return Err(Error::Bus);
        } else if isr.arlo() {
            trace!("ARLO triggered.");
            self.info.regs.icr().modify(|reg| reg.set_arlocf(true));
            self.flush_txdr();
            return Err(Error::Arbitration);
        } else if isr.ovr() {
            trace!("OVR triggered.");
            self.info.regs.icr().modify(|reg| reg.set_ovrcf(true));
            return Err(Error::Overrun);
        }
        return Ok(());
    }

    fn wait_txis(&self, timeout: Timeout) -> Result<(), Error> {
        let mut first_loop = true;

        loop {
            let isr = self.info.regs.isr().read();
            self.error_occurred(&isr, timeout)?;
            if isr.txis() {
                trace!("TXIS");
                return Ok(());
            }

            {
                if first_loop {
                    trace!("Waiting for TXIS...");
                    first_loop = false;
                }
            }
            timeout.check()?;
        }
    }

    fn wait_stop_or_err(&self, timeout: Timeout) -> Result<(), Error> {
        loop {
            let isr = self.info.regs.isr().read();
            self.error_occurred(&isr, timeout)?;
            if isr.stopf() {
                trace!("STOP triggered.");
                self.info.regs.icr().modify(|reg| reg.set_stopcf(true));
                return Ok(());
            }
            timeout.check()?;
        }
    }
    fn wait_stop(&self, timeout: Timeout) -> Result<(), Error> {
        loop {
            let isr = self.info.regs.isr().read();
            if isr.stopf() {
                trace!("STOP triggered.");
                self.info.regs.icr().modify(|reg| reg.set_stopcf(true));
                return Ok(());
            }
            timeout.check()?;
        }
    }

    fn wait_af(&self, timeout: Timeout) -> Result<(), Error> {
        loop {
            let isr = self.info.regs.isr().read();
            if isr.nackf() {
                trace!("AF triggered.");
                self.info.regs.icr().modify(|reg| reg.set_nackcf(true));
                return Ok(());
            }
            timeout.check()?;
        }
    }

    fn wait_rxne(&self, timeout: Timeout) -> Result<ReceiveResult, Error> {
        let mut first_loop = true;

        loop {
            let isr = self.info.regs.isr().read();
            self.error_occurred(&isr, timeout)?;
            if isr.stopf() {
                trace!("STOP when waiting for RXNE.");
                if self.info.regs.isr().read().rxne() {
                    trace!("Data received with STOP.");
                    return Ok(ReceiveResult::DataAvailable);
                }
                trace!("STOP triggered without data.");
                return Ok(ReceiveResult::StopReceived);
            } else if isr.rxne() {
                trace!("RXNE.");
                return Ok(ReceiveResult::DataAvailable);
            } else if isr.addr() {
                // Another addr event received, which means START was sent again
                // which happens when accessing memory registers (common i2c interface design)
                // e.g. master sends: START, write 1 byte (register index), START, read N bytes (until NACK)
                // Possible to receive this flag at the same time as rxne, so check rxne first
                trace!("START when waiting for RXNE. Ending receive loop.");
                // Return without clearing ADDR so `listen` can catch it
                return Ok(ReceiveResult::NewStart);
            }
            {
                if first_loop {
                    trace!("Waiting for RXNE...");
                    first_loop = false;
                }
            }

            timeout.check()?;
        }
    }

    fn wait_tc(&self, timeout: Timeout) -> Result<(), Error> {
        loop {
            let isr = self.info.regs.isr().read();
            self.error_occurred(&isr, timeout)?;
            if isr.tc() {
                return Ok(());
            }
            timeout.check()?;
        }
    }

    fn read_internal(
        &mut self,
        address: Address,
        read: &mut [u8],
        restart: bool,
        timeout: Timeout,
    ) -> Result<(), Error> {
        let completed_chunks = read.len() / 255;
        let total_chunks = if completed_chunks * 255 == read.len() {
            completed_chunks
        } else {
            completed_chunks + 1
        };
        let last_chunk_idx = total_chunks.saturating_sub(1);

        Self::master_read(
            self.info,
            address,
            read.len().min(255),
            Stop::Automatic,
            last_chunk_idx != 0,
            restart,
            timeout,
        )?;

        for (number, chunk) in read.chunks_mut(255).enumerate() {
            if number != 0 {
                Self::reload(self.info, chunk.len(), number != last_chunk_idx, timeout)?;
            }

            for byte in chunk {
                // Wait until we have received something
                self.wait_rxne(timeout)?;

                *byte = self.info.regs.rxdr().read().rxdata();
            }
        }
        self.wait_stop(timeout)?;
        Ok(())
    }

    fn write_internal(
        &mut self,
        address: Address,
        write: &[u8],
        send_stop: bool,
        timeout: Timeout,
    ) -> Result<(), Error> {
        let completed_chunks = write.len() / 255;
        let total_chunks = if completed_chunks * 255 == write.len() {
            completed_chunks
        } else {
            completed_chunks + 1
        };
        let last_chunk_idx = total_chunks.saturating_sub(1);

        // I2C start
        //
        // ST SAD+W
        if let Err(err) = Self::master_write(
            self.info,
            address,
            write.len().min(255),
            Stop::Software,
            last_chunk_idx != 0,
            timeout,
        ) {
            if send_stop {
                self.master_stop();
            }
            return Err(err);
        }

        for (number, chunk) in write.chunks(255).enumerate() {
            if number != 0 {
                Self::reload(self.info, chunk.len(), number != last_chunk_idx, timeout)?;
            }

            for byte in chunk {
                // Wait until we are allowed to send data
                // (START has been ACKed or last byte when
                // through)
                if let Err(err) = self.wait_txis(timeout) {
                    if send_stop {
                        self.master_stop();
                    }
                    return Err(err);
                }

                self.info.regs.txdr().write(|w| w.set_txdata(*byte));
            }
        }
        // Wait until the write finishes
        self.wait_tc(timeout)?;
        if send_stop {
            self.master_stop();
            self.wait_stop(timeout)?;
        }

        Ok(())
    }

    // =========================
    //  Blocking public API

    /// Blocking read.
    pub fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Error> {
        self.read_internal(address.into(), read, false, self.timeout())
        // Automatic Stop
    }

    /// Blocking write.
    pub fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<(), Error> {
        self.write_internal(address.into(), write, true, self.timeout())
    }

    /// Blocking write, restart, read.
    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        let timeout = self.timeout();
        self.write_internal(address.into(), write, false, timeout)?;
        self.read_internal(address.into(), read, true, timeout)
        // Automatic Stop
    }

    /// Blocking transaction with operations.
    ///
    /// Consecutive operations of same type are merged. See [transaction contract] for details.
    ///
    /// [transaction contract]: embedded_hal_1::i2c::I2c::transaction
    pub fn blocking_transaction(&mut self, addr: u8, operations: &mut [Operation<'_>]) -> Result<(), Error> {
        let _ = addr;
        let _ = operations;
        todo!()
    }

    /// Blocking write multiple buffers.
    ///
    /// The buffers are concatenated in a single write transaction.
    pub fn blocking_write_vectored(&mut self, address: u8, write: &[&[u8]]) -> Result<(), Error> {
        if write.is_empty() {
            return Err(Error::ZeroLengthTransfer);
        }

        let timeout = self.timeout();

        let first_length = write[0].len();
        let last_slice_index = write.len() - 1;

        if let Err(err) = Self::master_write(
            self.info,
            address.into(),
            first_length.min(255),
            Stop::Software,
            (first_length > 255) || (last_slice_index != 0),
            timeout,
        ) {
            self.master_stop();
            return Err(err);
        }

        for (idx, slice) in write.iter().enumerate() {
            let slice_len = slice.len();
            let completed_chunks = slice_len / 255;
            let total_chunks = if completed_chunks * 255 == slice_len {
                completed_chunks
            } else {
                completed_chunks + 1
            };
            let last_chunk_idx = total_chunks.saturating_sub(1);

            if idx != 0 {
                if let Err(err) = Self::reload(
                    self.info,
                    slice_len.min(255),
                    (idx != last_slice_index) || (slice_len > 255),
                    timeout,
                ) {
                    self.master_stop();
                    return Err(err);
                }
            }

            for (number, chunk) in slice.chunks(255).enumerate() {
                if number != 0 {
                    if let Err(err) = Self::reload(
                        self.info,
                        chunk.len(),
                        (number != last_chunk_idx) || (idx != last_slice_index),
                        timeout,
                    ) {
                        self.master_stop();
                        return Err(err);
                    }
                }

                for byte in chunk {
                    // Wait until we are allowed to send data
                    // (START has been ACKed or last byte when
                    // through)
                    if let Err(err) = self.wait_txis(timeout) {
                        self.master_stop();
                        return Err(err);
                    }

                    // Put byte on the wire
                    //self.i2c.txdr.write(|w| w.txdata().bits(*byte));
                    self.info.regs.txdr().write(|w| w.set_txdata(*byte));
                }
            }
        }
        // Wait until the write finishes
        let result = self.wait_tc(timeout);
        self.master_stop();
        result
    }
}

impl<'d, IM: MasterMode> I2c<'d, Async, IM> {
    async fn write_dma_internal(
        &mut self,
        address: Address,
        write: &[u8],
        first_slice: bool,
        last_slice: bool,
        send_stop: bool,
        timeout: Timeout,
    ) -> Result<(), Error> {
        let total_len = write.len();

        let dma_transfer = unsafe {
            let regs = self.info.regs;
            regs.cr1().modify(|w| {
                w.set_txdmaen(true);
                if first_slice {
                    w.set_tcie(true);
                }
                w.set_nackie(true);
                w.set_errie(true);
            });
            let dst = regs.txdr().as_ptr() as *mut u8;

            self.tx_dma.as_mut().unwrap().write(write, dst, Default::default())
        };

        let mut remaining_len = total_len;

        let on_drop = OnDrop::new(|| {
            let regs = self.info.regs;
            let isr = regs.isr().read();
            regs.cr1().modify(|w| {
                if last_slice || isr.nackf() || isr.arlo() || isr.berr() || isr.ovr() {
                    w.set_txdmaen(false);
                }
                w.set_tcie(false);
                w.set_nackie(false);
                w.set_errie(false);
            });
            regs.icr().write(|w| {
                w.set_nackcf(true);
                w.set_berrcf(true);
                w.set_arlocf(true);
                w.set_ovrcf(true);
            });
        });

        poll_fn(|cx| {
            self.state.waker.register(cx.waker());

            let isr = self.info.regs.isr().read();

            if isr.nackf() {
                return Poll::Ready(Err(Error::Nack));
            }
            if isr.arlo() {
                return Poll::Ready(Err(Error::Arbitration));
            }
            if isr.berr() {
                return Poll::Ready(Err(Error::Bus));
            }
            if isr.ovr() {
                return Poll::Ready(Err(Error::Overrun));
            }

            if remaining_len == total_len {
                if first_slice {
                    Self::master_write(
                        self.info,
                        address,
                        total_len.min(255),
                        Stop::Software,
                        (total_len > 255) || !last_slice,
                        timeout,
                    )?;
                } else {
                    Self::reload(self.info, total_len.min(255), (total_len > 255) || !last_slice, timeout)?;
                    self.info.regs.cr1().modify(|w| w.set_tcie(true));
                }
            } else if !(isr.tcr() || isr.tc()) {
                // poll_fn was woken without an interrupt present
                return Poll::Pending;
            } else if remaining_len == 0 {
                return Poll::Ready(Ok(()));
            } else {
                let last_piece = (remaining_len <= 255) && last_slice;

                if let Err(e) = Self::reload(self.info, remaining_len.min(255), !last_piece, timeout) {
                    return Poll::Ready(Err(e));
                }
                self.info.regs.cr1().modify(|w| w.set_tcie(true));
            }

            remaining_len = remaining_len.saturating_sub(255);
            Poll::Pending
        })
        .await?;

        dma_transfer.await;
        if last_slice {
            // This should be done already
            self.wait_tc(timeout)?;
        }

        if last_slice & send_stop {
            self.master_stop();
        }

        drop(on_drop);

        Ok(())
    }

    async fn read_dma_internal(
        &mut self,
        address: Address,
        buffer: &mut [u8],
        restart: bool,
        timeout: Timeout,
    ) -> Result<(), Error> {
        let total_len = buffer.len();

        let dma_transfer = unsafe {
            let regs = self.info.regs;
            regs.cr1().modify(|w| {
                w.set_rxdmaen(true);
                w.set_tcie(true);
                w.set_nackie(true);
                w.set_errie(true);
            });
            let src = regs.rxdr().as_ptr() as *mut u8;

            self.rx_dma.as_mut().unwrap().read(src, buffer, Default::default())
        };

        let mut remaining_len = total_len;

        let on_drop = OnDrop::new(|| {
            let regs = self.info.regs;
            regs.cr1().modify(|w| {
                w.set_rxdmaen(false);
                w.set_tcie(false);
                w.set_nackie(false);
                w.set_errie(false);
            });
            regs.icr().write(|w| {
                w.set_nackcf(true);
                w.set_berrcf(true);
                w.set_arlocf(true);
                w.set_ovrcf(true);
            });
        });

        poll_fn(|cx| {
            self.state.waker.register(cx.waker());

            let isr = self.info.regs.isr().read();

            if isr.nackf() {
                return Poll::Ready(Err(Error::Nack));
            }
            if isr.arlo() {
                return Poll::Ready(Err(Error::Arbitration));
            }
            if isr.berr() {
                return Poll::Ready(Err(Error::Bus));
            }
            if isr.ovr() {
                return Poll::Ready(Err(Error::Overrun));
            }

            if remaining_len == total_len {
                Self::master_read(
                    self.info,
                    address,
                    total_len.min(255),
                    Stop::Automatic,
                    total_len > 255,
                    restart,
                    timeout,
                )?;
                if total_len <= 255 {
                    return Poll::Ready(Ok(()));
                }
            } else if isr.tcr() {
                // poll_fn was woken without an interrupt present
                return Poll::Pending;
            } else {
                let last_piece = remaining_len <= 255;

                if let Err(e) = Self::reload(self.info, remaining_len.min(255), !last_piece, timeout) {
                    return Poll::Ready(Err(e));
                }
                // Return here if we are on last chunk,
                // end of transfer will be awaited with the DMA below
                if last_piece {
                    return Poll::Ready(Ok(()));
                }
                self.info.regs.cr1().modify(|w| w.set_tcie(true));
            }

            remaining_len = remaining_len.saturating_sub(255);
            Poll::Pending
        })
        .await?;

        dma_transfer.await;
        drop(on_drop);

        Ok(())
    }
    // =========================
    //  Async public API

    /// Write.
    pub async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Error> {
        let timeout = self.timeout();
        if write.is_empty() {
            self.write_internal(address.into(), write, true, timeout)
        } else {
            timeout
                .with(self.write_dma_internal(address.into(), write, true, true, true, timeout))
                .await
        }
    }

    /// Write multiple buffers.
    ///
    /// The buffers are concatenated in a single write transaction.
    pub async fn write_vectored(&mut self, address: Address, write: &[&[u8]]) -> Result<(), Error> {
        let timeout = self.timeout();

        if write.is_empty() {
            return Err(Error::ZeroLengthTransfer);
        }
        let mut iter = write.iter();

        let mut first = true;
        let mut current = iter.next();
        while let Some(c) = current {
            let next = iter.next();
            let is_last = next.is_none();

            let fut = self.write_dma_internal(address, c, first, is_last, is_last, timeout);
            timeout.with(fut).await?;
            first = false;
            current = next;
        }
        Ok(())
    }

    /// Read.
    pub async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        let timeout = self.timeout();

        if buffer.is_empty() {
            self.read_internal(address.into(), buffer, false, timeout)
        } else {
            let fut = self.read_dma_internal(address.into(), buffer, false, timeout);
            timeout.with(fut).await
        }
    }

    /// Write, restart, read.
    pub async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        let timeout = self.timeout();

        if write.is_empty() {
            self.write_internal(address.into(), write, false, timeout)?;
        } else {
            let fut = self.write_dma_internal(address.into(), write, true, true, false, timeout);
            timeout.with(fut).await?;
        }

        if read.is_empty() {
            self.read_internal(address.into(), read, true, timeout)?;
        } else {
            let fut = self.read_dma_internal(address.into(), read, true, timeout);
            timeout.with(fut).await?;
        }

        Ok(())
    }

    /// Transaction with operations.
    ///
    /// Consecutive operations of same type are merged. See [transaction contract] for details.
    ///
    /// [transaction contract]: embedded_hal_1::i2c::I2c::transaction
    pub async fn transaction(&mut self, addr: u8, operations: &mut [Operation<'_>]) -> Result<(), Error> {
        let _ = addr;
        let _ = operations;
        todo!()
    }
}

impl<'d, M: Mode> I2c<'d, M, Master> {
    /// Configure the I2C driver for slave operations, allowing for the driver to be used as a slave and a master (multimaster)
    pub fn into_slave_multimaster(mut self, slave_addr_config: SlaveAddrConfig) -> I2c<'d, M, MultiMaster> {
        let mut slave = I2c {
            info: self.info,
            state: self.state,
            kernel_clock: self.kernel_clock,
            tx_dma: self.tx_dma.take(),
            rx_dma: self.rx_dma.take(),
            #[cfg(feature = "time")]
            timeout: self.timeout,
            _phantom: PhantomData,
            _phantom2: PhantomData,
            _drop_guard: self._drop_guard,
        };
        slave.init_slave(slave_addr_config);
        slave
    }
}

impl<'d, M: Mode> I2c<'d, M, MultiMaster> {
    pub(crate) fn init_slave(&mut self, config: SlaveAddrConfig) {
        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(false);
        });

        self.info.regs.cr1().modify(|reg| {
            reg.set_nostretch(false);
            reg.set_gcen(config.general_call);
            reg.set_sbc(true);
            reg.set_pe(true);
        });

        self.reconfigure_addresses(config.addr);
    }

    /// Configure the slave address.
    pub fn reconfigure_addresses(&mut self, addresses: OwnAddresses) {
        match addresses {
            OwnAddresses::OA1(oa1) => self.configure_oa1(oa1),
            OwnAddresses::OA2(oa2) => self.configure_oa2(oa2),
            OwnAddresses::Both { oa1, oa2 } => {
                self.configure_oa1(oa1);
                self.configure_oa2(oa2);
            }
        }
    }

    fn configure_oa1(&mut self, oa1: Address) {
        match oa1 {
            Address::SevenBit(addr) => self.info.regs.oar1().write(|reg| {
                reg.set_oa1en(false);
                reg.set_oa1((addr << 1) as u16);
                reg.set_oa1mode(Addmode::BIT7);
                reg.set_oa1en(true);
            }),
            Address::TenBit(addr) => self.info.regs.oar1().write(|reg| {
                reg.set_oa1en(false);
                reg.set_oa1(addr);
                reg.set_oa1mode(Addmode::BIT10);
                reg.set_oa1en(true);
            }),
        }
    }

    fn configure_oa2(&mut self, oa2: OA2) {
        self.info.regs.oar2().write(|reg| {
            reg.set_oa2en(false);
            reg.set_oa2msk(oa2.mask.into());
            reg.set_oa2(oa2.addr << 1);
            reg.set_oa2en(true);
        });
    }

    fn determine_matched_address(&self) -> Result<Address, Error> {
        let matched = self.info.regs.isr().read().addcode();

        if matched >> 3 == 0b11110 {
            // is 10-bit address and we need to get the other 8 bits from the rxdr
            // we do this by doing a blocking read of 1 byte
            let mut buffer = [0];
            self.slave_read_internal(&mut buffer, self.timeout())?;
            Ok(Address::TenBit((matched as u16) << 6 | buffer[0] as u16))
        } else {
            Ok(Address::SevenBit(matched))
        }
    }
}

impl<'d, M: Mode> I2c<'d, M, MultiMaster> {
    /// # Safety
    /// This function will clear the address flag which will stop the clock stretching.
    /// This should only be done after the dma transfer has been set up.
    fn slave_start(info: &'static Info, length: usize, reload: bool) {
        assert!(length < 256);

        let reload = if reload {
            i2c::vals::Reload::NOT_COMPLETED
        } else {
            i2c::vals::Reload::COMPLETED
        };

        info.regs.cr2().modify(|w| {
            w.set_nbytes(length as u8);
            w.set_reload(reload);
        });

        // clear the address flag, will stop the clock stretching.
        // this should only be done after the dma transfer has been set up.
        info.regs.icr().modify(|reg| reg.set_addrcf(true));
        trace!("ADDRCF cleared (ADDR interrupt enabled, clock stretching ended)");
    }

    // A blocking read operation
    fn slave_read_internal(&self, read: &mut [u8], timeout: Timeout) -> Result<usize, Error> {
        let completed_chunks = read.len() / 255;
        let total_chunks = if completed_chunks * 255 == read.len() {
            completed_chunks
        } else {
            completed_chunks + 1
        };
        let last_chunk_idx = total_chunks.saturating_sub(1);
        let total_len = read.len();
        let mut remaining_len = total_len;

        for (number, chunk) in read.chunks_mut(255).enumerate() {
            trace!(
                "--- Slave RX transmission start - chunk: {}, expected (max) size: {}",
                number,
                chunk.len()
            );
            if number == 0 {
                Self::slave_start(self.info, chunk.len(), number != last_chunk_idx);
            } else {
                Self::reload(self.info, chunk.len(), number != last_chunk_idx, timeout)?;
            }

            let mut index = 0;

            for byte in chunk {
                // Wait until we have received something
                match self.wait_rxne(timeout) {
                    Ok(ReceiveResult::StopReceived) | Ok(ReceiveResult::NewStart) => {
                        trace!("--- Slave RX transmission end (early)");
                        return Ok(total_len - remaining_len); // Return N bytes read
                    }
                    Ok(ReceiveResult::DataAvailable) => {
                        *byte = self.info.regs.rxdr().read().rxdata();
                        remaining_len = remaining_len.saturating_sub(1);
                        {
                            trace!("Slave RX data {}: {:#04x}", index, byte);
                            index = index + 1;
                        }
                    }
                    Err(e) => return Err(e),
                };
            }
        }
        self.wait_stop_or_err(timeout)?;

        trace!("--- Slave RX transmission end");
        Ok(total_len - remaining_len) // Return N bytes read
    }

    // A blocking write operation
    fn slave_write_internal(&mut self, write: &[u8], timeout: Timeout) -> Result<(), Error> {
        let completed_chunks = write.len() / 255;
        let total_chunks = if completed_chunks * 255 == write.len() {
            completed_chunks
        } else {
            completed_chunks + 1
        };
        let last_chunk_idx = total_chunks.saturating_sub(1);

        for (number, chunk) in write.chunks(255).enumerate() {
            trace!(
                "--- Slave TX transmission start - chunk: {}, size: {}",
                number,
                chunk.len()
            );
            if number == 0 {
                Self::slave_start(self.info, chunk.len(), number != last_chunk_idx);
            } else {
                Self::reload(self.info, chunk.len(), number != last_chunk_idx, timeout)?;
            }

            let mut index = 0;

            for byte in chunk {
                // Wait until we are allowed to send data
                // (START has been ACKed or last byte when through)
                self.wait_txis(timeout)?;

                {
                    trace!("Slave TX data {}: {:#04x}", index, byte);
                    index = index + 1;
                }
                self.info.regs.txdr().write(|w| w.set_txdata(*byte));
            }
        }
        self.wait_af(timeout)?;
        self.flush_txdr();
        self.wait_stop_or_err(timeout)?;

        trace!("--- Slave TX transmission end");
        Ok(())
    }

    /// Listen for incoming I2C messages.
    ///
    /// The listen method is an asynchronous method but it does not require DMA to be asynchronous.
    pub async fn listen(&mut self) -> Result<SlaveCommand, Error> {
        let state = self.state;
        self.info.regs.cr1().modify(|reg| {
            reg.set_addrie(true);
            trace!("Enable ADDRIE");
        });

        poll_fn(|cx| {
            state.waker.register(cx.waker());
            let isr = self.info.regs.isr().read();
            if !isr.addr() {
                Poll::Pending
            } else {
                trace!("ADDR triggered (address match)");
                // we do not clear the address flag here as it will be cleared by the dma read/write
                // if we clear it here the clock stretching will stop and the master will read in data before the slave is ready to send it
                match isr.dir() {
                    i2c::vals::Dir::WRITE => {
                        trace!("DIR: write");
                        Poll::Ready(Ok(SlaveCommand {
                            kind: SlaveCommandKind::Write,
                            address: self.determine_matched_address()?,
                        }))
                    }
                    i2c::vals::Dir::READ => {
                        trace!("DIR: read");
                        Poll::Ready(Ok(SlaveCommand {
                            kind: SlaveCommandKind::Read,
                            address: self.determine_matched_address()?,
                        }))
                    }
                }
            }
        })
        .await
    }

    /// Respond to a write command.
    ///
    /// Returns total number of bytes received.
    pub fn blocking_respond_to_write(&self, read: &mut [u8]) -> Result<usize, Error> {
        let timeout = self.timeout();
        self.slave_read_internal(read, timeout)
    }

    /// Respond to a read command.
    pub fn blocking_respond_to_read(&mut self, write: &[u8]) -> Result<(), Error> {
        let timeout = self.timeout();
        self.slave_write_internal(write, timeout)
    }
}

impl<'d> I2c<'d, Async, MultiMaster> {
    /// Respond to a write command.
    ///
    /// Returns the total number of bytes received.
    pub async fn respond_to_write(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        let timeout = self.timeout();
        timeout.with(self.read_dma_internal_slave(buffer, timeout)).await
    }

    /// Respond to a read request from an I2C master.
    pub async fn respond_to_read(&mut self, write: &[u8]) -> Result<SendStatus, Error> {
        let timeout = self.timeout();
        timeout.with(self.write_dma_internal_slave(write, timeout)).await
    }

    // for data reception in slave mode
    //
    // returns the total number of bytes received
    async fn read_dma_internal_slave(&mut self, buffer: &mut [u8], timeout: Timeout) -> Result<usize, Error> {
        let total_len = buffer.len();
        let mut remaining_len = total_len;

        let regs = self.info.regs;

        let dma_transfer = unsafe {
            regs.cr1().modify(|w| {
                w.set_rxdmaen(true);
                w.set_stopie(true);
                w.set_tcie(true);
            });
            let src = regs.rxdr().as_ptr() as *mut u8;

            self.rx_dma.as_mut().unwrap().read(src, buffer, Default::default())
        };

        let state = self.state;

        let on_drop = OnDrop::new(|| {
            regs.cr1().modify(|w| {
                w.set_rxdmaen(false);
                w.set_stopie(false);
                w.set_tcie(false);
            });
        });

        let total_received = poll_fn(|cx| {
            state.waker.register(cx.waker());

            let isr = regs.isr().read();

            if remaining_len == total_len {
                Self::slave_start(self.info, total_len.min(255), total_len > 255);
                remaining_len = remaining_len.saturating_sub(255);
                Poll::Pending
            } else if isr.tcr() {
                let is_last_slice = remaining_len <= 255;
                if let Err(e) = Self::reload(self.info, remaining_len.min(255), !is_last_slice, timeout) {
                    return Poll::Ready(Err(e));
                }
                remaining_len = remaining_len.saturating_sub(255);
                regs.cr1().modify(|w| w.set_tcie(true));
                Poll::Pending
            } else if isr.stopf() {
                regs.icr().write(|reg| reg.set_stopcf(true));
                let poll = Poll::Ready(Ok(total_len - remaining_len));
                poll
            } else {
                Poll::Pending
            }
        })
        .await?;

        dma_transfer.await;

        drop(on_drop);

        Ok(total_received)
    }

    async fn write_dma_internal_slave(&mut self, buffer: &[u8], timeout: Timeout) -> Result<SendStatus, Error> {
        let total_len = buffer.len();
        let mut remaining_len = total_len;

        let mut dma_transfer = unsafe {
            let regs = self.info.regs;
            regs.cr1().modify(|w| {
                w.set_txdmaen(true);
                w.set_stopie(true);
                w.set_tcie(true);
            });
            let dst = regs.txdr().as_ptr() as *mut u8;

            self.tx_dma.as_mut().unwrap().write(buffer, dst, Default::default())
        };

        let on_drop = OnDrop::new(|| {
            let regs = self.info.regs;
            regs.cr1().modify(|w| {
                w.set_txdmaen(false);
                w.set_stopie(false);
                w.set_tcie(false);
            })
        });

        let state = self.state;

        let size = poll_fn(|cx| {
            state.waker.register(cx.waker());

            let isr = self.info.regs.isr().read();

            if remaining_len == total_len {
                Self::slave_start(self.info, total_len.min(255), total_len > 255);
                remaining_len = remaining_len.saturating_sub(255);
                Poll::Pending
            } else if isr.tcr() {
                let is_last_slice = remaining_len <= 255;
                if let Err(e) = Self::reload(self.info, remaining_len.min(255), !is_last_slice, timeout) {
                    return Poll::Ready(Err(e));
                }
                remaining_len = remaining_len.saturating_sub(255);
                self.info.regs.cr1().modify(|w| w.set_tcie(true));
                Poll::Pending
            } else if isr.stopf() {
                self.info.regs.icr().write(|reg| reg.set_stopcf(true));
                if remaining_len > 0 {
                    dma_transfer.request_stop();
                    Poll::Ready(Ok(SendStatus::LeftoverBytes(remaining_len as usize)))
                } else {
                    Poll::Ready(Ok(SendStatus::Done))
                }
            } else {
                Poll::Pending
            }
        })
        .await?;

        dma_transfer.await;

        drop(on_drop);

        Ok(size)
    }
}

/// I2C Stop Configuration
///
/// Peripheral options for generating the STOP condition
#[derive(Copy, Clone, PartialEq)]
enum Stop {
    /// Software end mode: Must write register to generate STOP condition
    Software,
    /// Automatic end mode: A STOP condition is automatically generated once the
    /// configured number of bytes have been transferred
    Automatic,
}

impl Stop {
    fn autoend(&self) -> i2c::vals::Autoend {
        match self {
            Stop::Software => i2c::vals::Autoend::SOFTWARE,
            Stop::Automatic => i2c::vals::Autoend::AUTOMATIC,
        }
    }
}

struct Timings {
    prescale: u8,
    scll: u8,
    sclh: u8,
    sdadel: u8,
    scldel: u8,
}

impl Timings {
    fn new(i2cclk: Hertz, frequency: Hertz) -> Self {
        let i2cclk = i2cclk.0;
        let frequency = frequency.0;
        // Refer to RM0433 Rev 7 Figure 539 for setup and hold timing:
        //
        // t_I2CCLK = 1 / PCLK1
        // t_PRESC  = (PRESC + 1) * t_I2CCLK
        // t_SCLL   = (SCLL + 1) * t_PRESC
        // t_SCLH   = (SCLH + 1) * t_PRESC
        //
        // t_SYNC1 + t_SYNC2 > 4 * t_I2CCLK
        // t_SCL ~= t_SYNC1 + t_SYNC2 + t_SCLL + t_SCLH
        let ratio = i2cclk / frequency;

        // For the standard-mode configuration method, we must have a ratio of 4
        // or higher
        assert!(ratio >= 4, "The I2C PCLK must be at least 4 times the bus frequency!");

        let (presc_reg, scll, sclh, sdadel, scldel) = if frequency > 100_000 {
            // Fast-mode (Fm) or Fast-mode Plus (Fm+)
            // here we pick SCLL + 1 = 2 * (SCLH + 1)

            // Prescaler, 384 ticks for sclh/scll. Round up then subtract 1
            let presc_reg = ((ratio - 1) / 384) as u8;
            // ratio < 1200 by pclk 120MHz max., therefore presc < 16

            // Actual precale value selected
            let presc = (presc_reg + 1) as u32;

            let sclh = ((ratio / presc) - 3) / 3;
            let scll = (2 * (sclh + 1)) - 1;

            let (sdadel, scldel) = if frequency > 400_000 {
                // Fast-mode Plus (Fm+)
                assert!(i2cclk >= 17_000_000); // See table in datsheet

                let sdadel = i2cclk / 8_000_000 / presc;
                let scldel = i2cclk / 4_000_000 / presc - 1;

                (sdadel, scldel)
            } else {
                // Fast-mode (Fm)
                assert!(i2cclk >= 8_000_000); // See table in datsheet

                let sdadel = i2cclk / 4_000_000 / presc;
                let scldel = i2cclk / 2_000_000 / presc - 1;

                (sdadel, scldel)
            };

            (presc_reg, scll as u8, sclh as u8, sdadel as u8, scldel as u8)
        } else {
            // Standard-mode (Sm)
            // here we pick SCLL = SCLH
            assert!(i2cclk >= 2_000_000); // See table in datsheet

            // Prescaler, 512 ticks for sclh/scll. Round up then
            // subtract 1
            let presc = (ratio - 1) / 512;
            let presc_reg = cmp::min(presc, 15) as u8;

            // Actual prescale value selected
            let presc = (presc_reg + 1) as u32;

            let sclh = ((ratio / presc) - 2) / 2;
            let scll = sclh;

            // Speed check
            assert!(sclh < 256, "The I2C PCLK is too fast for this bus frequency!");

            let sdadel = i2cclk / 2_000_000 / presc;
            let scldel = i2cclk / 500_000 / presc - 1;

            (presc_reg, scll as u8, sclh as u8, sdadel as u8, scldel as u8)
        };

        // Sanity check
        assert!(presc_reg < 16);

        // Keep values within reasonable limits for fast per_ck
        let sdadel = cmp::max(sdadel, 2);
        let scldel = cmp::max(scldel, 4);

        //(presc_reg, scll, sclh, sdadel, scldel)
        Self {
            prescale: presc_reg,
            scll,
            sclh,
            sdadel,
            scldel,
        }
    }
}

impl<'d, M: Mode> SetConfig for I2c<'d, M, Master> {
    type Config = Hertz;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(false);
        });

        let timings = Timings::new(self.kernel_clock, *config);

        self.info.regs.timingr().write(|reg| {
            reg.set_presc(timings.prescale);
            reg.set_scll(timings.scll);
            reg.set_sclh(timings.sclh);
            reg.set_sdadel(timings.sdadel);
            reg.set_scldel(timings.scldel);
        });

        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(true);
        });

        Ok(())
    }
}

impl<'d, M: Mode> SetConfig for I2c<'d, M, MultiMaster> {
    type Config = (Hertz, SlaveAddrConfig);
    type ConfigError = ();
    fn set_config(&mut self, (config, addr_config): &Self::Config) -> Result<(), ()> {
        let timings = Timings::new(self.kernel_clock, *config);
        self.info.regs.timingr().write(|reg| {
            reg.set_presc(timings.prescale);
            reg.set_scll(timings.scll);
            reg.set_sclh(timings.sclh);
            reg.set_sdadel(timings.sdadel);
            reg.set_scldel(timings.scldel);
        });
        self.init_slave(*addr_config);

        Ok(())
    }
}
