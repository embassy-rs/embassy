use core::cmp;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::{NoDma, Transfer};
use crate::gpio::sealed::AFType;
use crate::gpio::Pull;
use crate::i2c::{Error, Instance, SclPin, SdaPin};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::i2c;
use crate::time::Hertz;
use crate::{interrupt, Peripheral};

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        let isr = regs.isr().read();

        if isr.tcr() || isr.tc() {
            T::state().waker.wake();
        }
        // The flag can only be cleared by writting to nbytes, we won't do that here, so disable
        // the interrupt
        critical_section::with(|_| {
            regs.cr1().modify(|w| w.set_tcie(false));
        });
    }
}

#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    pub sda_pullup: bool,
    pub scl_pullup: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sda_pullup: false,
            scl_pullup: false,
        }
    }
}

pub struct State {
    waker: AtomicWaker,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

pub struct I2c<'d, T: Instance, TXDMA = NoDma, RXDMA = NoDma> {
    _peri: PeripheralRef<'d, T>,
    tx_dma: PeripheralRef<'d, TXDMA>,
    #[allow(dead_code)]
    rx_dma: PeripheralRef<'d, RXDMA>,
}

impl<'d, T: Instance, TXDMA, RXDMA> I2c<'d, T, TXDMA, RXDMA> {
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        tx_dma: impl Peripheral<P = TXDMA> + 'd,
        rx_dma: impl Peripheral<P = RXDMA> + 'd,
        freq: Hertz,
        config: Config,
    ) -> Self {
        into_ref!(peri, scl, sda, tx_dma, rx_dma);

        T::enable();
        T::reset();

        scl.set_as_af_pull(
            scl.af_num(),
            AFType::OutputOpenDrain,
            match config.scl_pullup {
                true => Pull::Up,
                false => Pull::None,
            },
        );
        sda.set_as_af_pull(
            sda.af_num(),
            AFType::OutputOpenDrain,
            match config.sda_pullup {
                true => Pull::Up,
                false => Pull::None,
            },
        );

        T::regs().cr1().modify(|reg| {
            reg.set_pe(false);
            reg.set_anfoff(false);
        });

        let timings = Timings::new(T::frequency(), freq.into());

        T::regs().timingr().write(|reg| {
            reg.set_presc(timings.prescale);
            reg.set_scll(timings.scll);
            reg.set_sclh(timings.sclh);
            reg.set_sdadel(timings.sdadel);
            reg.set_scldel(timings.scldel);
        });

        T::regs().cr1().modify(|reg| {
            reg.set_pe(true);
        });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self {
            _peri: peri,
            tx_dma,
            rx_dma,
        }
    }

    fn master_stop(&mut self) {
        T::regs().cr2().write(|w| w.set_stop(true));
    }

    fn master_read(
        address: u8,
        length: usize,
        stop: Stop,
        reload: bool,
        restart: bool,
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error> {
        assert!(length < 256);

        if !restart {
            // Wait for any previous address sequence to end
            // automatically. This could be up to 50% of a bus
            // cycle (ie. up to 0.5/freq)
            while T::regs().cr2().read().start() {
                check_timeout()?;
            }
        }

        // Set START and prepare to receive bytes into
        // `buffer`. The START bit can be set even if the bus
        // is BUSY or I2C is in slave mode.

        let reload = if reload {
            i2c::vals::Reload::NOTCOMPLETED
        } else {
            i2c::vals::Reload::COMPLETED
        };

        T::regs().cr2().modify(|w| {
            w.set_sadd((address << 1 | 0) as u16);
            w.set_add10(i2c::vals::Addmode::BIT7);
            w.set_dir(i2c::vals::Dir::READ);
            w.set_nbytes(length as u8);
            w.set_start(true);
            w.set_autoend(stop.autoend());
            w.set_reload(reload);
        });

        Ok(())
    }

    fn master_write(
        address: u8,
        length: usize,
        stop: Stop,
        reload: bool,
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error> {
        assert!(length < 256);

        // Wait for any previous address sequence to end
        // automatically. This could be up to 50% of a bus
        // cycle (ie. up to 0.5/freq)
        while T::regs().cr2().read().start() {
            check_timeout()?;
        }

        let reload = if reload {
            i2c::vals::Reload::NOTCOMPLETED
        } else {
            i2c::vals::Reload::COMPLETED
        };

        // Set START and prepare to send `bytes`. The
        // START bit can be set even if the bus is BUSY or
        // I2C is in slave mode.
        T::regs().cr2().modify(|w| {
            w.set_sadd((address << 1 | 0) as u16);
            w.set_add10(i2c::vals::Addmode::BIT7);
            w.set_dir(i2c::vals::Dir::WRITE);
            w.set_nbytes(length as u8);
            w.set_start(true);
            w.set_autoend(stop.autoend());
            w.set_reload(reload);
        });

        Ok(())
    }

    fn master_continue(
        length: usize,
        reload: bool,
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error> {
        assert!(length < 256 && length > 0);

        while !T::regs().isr().read().tcr() {
            check_timeout()?;
        }

        let reload = if reload {
            i2c::vals::Reload::NOTCOMPLETED
        } else {
            i2c::vals::Reload::COMPLETED
        };

        T::regs().cr2().modify(|w| {
            w.set_nbytes(length as u8);
            w.set_reload(reload);
        });

        Ok(())
    }

    fn flush_txdr(&self) {
        //if $i2c.isr.read().txis().bit_is_set() {
        //$i2c.txdr.write(|w| w.txdata().bits(0));
        //}

        if T::regs().isr().read().txis() {
            T::regs().txdr().write(|w| w.set_txdata(0));
        }
        if !T::regs().isr().read().txe() {
            T::regs().isr().modify(|w| w.set_txe(true))
        }

        // If TXDR is not flagged as empty, write 1 to flush it
        //if $i2c.isr.read().txe().is_not_empty() {
        //$i2c.isr.write(|w| w.txe().set_bit());
        //}
    }

    fn wait_txe(&self, check_timeout: impl Fn() -> Result<(), Error>) -> Result<(), Error> {
        loop {
            let isr = T::regs().isr().read();
            if isr.txe() {
                return Ok(());
            } else if isr.berr() {
                T::regs().icr().write(|reg| reg.set_berrcf(true));
                return Err(Error::Bus);
            } else if isr.arlo() {
                T::regs().icr().write(|reg| reg.set_arlocf(true));
                return Err(Error::Arbitration);
            } else if isr.nackf() {
                T::regs().icr().write(|reg| reg.set_nackcf(true));
                self.flush_txdr();
                return Err(Error::Nack);
            }

            check_timeout()?;
        }
    }

    fn wait_rxne(&self, check_timeout: impl Fn() -> Result<(), Error>) -> Result<(), Error> {
        loop {
            let isr = T::regs().isr().read();
            if isr.rxne() {
                return Ok(());
            } else if isr.berr() {
                T::regs().icr().write(|reg| reg.set_berrcf(true));
                return Err(Error::Bus);
            } else if isr.arlo() {
                T::regs().icr().write(|reg| reg.set_arlocf(true));
                return Err(Error::Arbitration);
            } else if isr.nackf() {
                T::regs().icr().write(|reg| reg.set_nackcf(true));
                self.flush_txdr();
                return Err(Error::Nack);
            }

            check_timeout()?;
        }
    }

    fn wait_tc(&self, check_timeout: impl Fn() -> Result<(), Error>) -> Result<(), Error> {
        loop {
            let isr = T::regs().isr().read();
            if isr.tc() {
                return Ok(());
            } else if isr.berr() {
                T::regs().icr().write(|reg| reg.set_berrcf(true));
                return Err(Error::Bus);
            } else if isr.arlo() {
                T::regs().icr().write(|reg| reg.set_arlocf(true));
                return Err(Error::Arbitration);
            } else if isr.nackf() {
                T::regs().icr().write(|reg| reg.set_nackcf(true));
                self.flush_txdr();
                return Err(Error::Nack);
            }

            check_timeout()?;
        }
    }

    fn read_internal(
        &mut self,
        address: u8,
        read: &mut [u8],
        restart: bool,
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error> {
        let completed_chunks = read.len() / 255;
        let total_chunks = if completed_chunks * 255 == read.len() {
            completed_chunks
        } else {
            completed_chunks + 1
        };
        let last_chunk_idx = total_chunks.saturating_sub(1);

        Self::master_read(
            address,
            read.len().min(255),
            Stop::Automatic,
            last_chunk_idx != 0,
            restart,
            &check_timeout,
        )?;

        for (number, chunk) in read.chunks_mut(255).enumerate() {
            if number != 0 {
                Self::master_continue(chunk.len(), number != last_chunk_idx, &check_timeout)?;
            }

            for byte in chunk {
                // Wait until we have received something
                self.wait_rxne(&check_timeout)?;

                *byte = T::regs().rxdr().read().rxdata();
            }
        }
        Ok(())
    }

    fn write_internal(
        &mut self,
        address: u8,
        write: &[u8],
        send_stop: bool,
        check_timeout: impl Fn() -> Result<(), Error>,
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
            address,
            write.len().min(255),
            Stop::Software,
            last_chunk_idx != 0,
            &check_timeout,
        ) {
            if send_stop {
                self.master_stop();
            }
            return Err(err);
        }

        for (number, chunk) in write.chunks(255).enumerate() {
            if number != 0 {
                Self::master_continue(chunk.len(), number != last_chunk_idx, &check_timeout)?;
            }

            for byte in chunk {
                // Wait until we are allowed to send data
                // (START has been ACKed or last byte when
                // through)
                if let Err(err) = self.wait_txe(&check_timeout) {
                    if send_stop {
                        self.master_stop();
                    }
                    return Err(err);
                }

                T::regs().txdr().write(|w| w.set_txdata(*byte));
            }
        }
        // Wait until the write finishes
        let result = self.wait_tc(&check_timeout);
        if send_stop {
            self.master_stop();
        }
        result
    }

    async fn write_dma_internal(
        &mut self,
        address: u8,
        write: &[u8],
        first_slice: bool,
        last_slice: bool,
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        TXDMA: crate::i2c::TxDma<T>,
    {
        let total_len = write.len();

        let dma_transfer = unsafe {
            let regs = T::regs();
            regs.cr1().modify(|w| {
                w.set_txdmaen(true);
                if first_slice {
                    w.set_tcie(true);
                }
            });
            let dst = regs.txdr().as_ptr() as *mut u8;

            let ch = &mut self.tx_dma;
            let request = ch.request();
            Transfer::new_write(ch, request, write, dst, Default::default())
        };

        let state = T::state();
        let mut remaining_len = total_len;

        let on_drop = OnDrop::new(|| {
            let regs = T::regs();
            regs.cr1().modify(|w| {
                if last_slice {
                    w.set_txdmaen(false);
                }
                w.set_tcie(false);
            })
        });

        poll_fn(|cx| {
            state.waker.register(cx.waker());

            let isr = T::regs().isr().read();
            if remaining_len == total_len {
                if first_slice {
                    Self::master_write(
                        address,
                        total_len.min(255),
                        Stop::Software,
                        (total_len > 255) || !last_slice,
                        &check_timeout,
                    )?;
                } else {
                    Self::master_continue(total_len.min(255), (total_len > 255) || !last_slice, &check_timeout)?;
                    T::regs().cr1().modify(|w| w.set_tcie(true));
                }
            } else if !(isr.tcr() || isr.tc()) {
                // poll_fn was woken without an interrupt present
                return Poll::Pending;
            } else if remaining_len == 0 {
                return Poll::Ready(Ok(()));
            } else {
                let last_piece = (remaining_len <= 255) && last_slice;

                if let Err(e) = Self::master_continue(remaining_len.min(255), !last_piece, &check_timeout) {
                    return Poll::Ready(Err(e));
                }
                T::regs().cr1().modify(|w| w.set_tcie(true));
            }

            remaining_len = remaining_len.saturating_sub(255);
            Poll::Pending
        })
        .await?;

        dma_transfer.await;

        if last_slice {
            // This should be done already
            self.wait_tc(&check_timeout)?;
            self.master_stop();
        }

        drop(on_drop);

        Ok(())
    }

    async fn read_dma_internal(
        &mut self,
        address: u8,
        buffer: &mut [u8],
        restart: bool,
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        RXDMA: crate::i2c::RxDma<T>,
    {
        let total_len = buffer.len();

        let dma_transfer = unsafe {
            let regs = T::regs();
            regs.cr1().modify(|w| {
                w.set_rxdmaen(true);
                w.set_tcie(true);
            });
            let src = regs.rxdr().as_ptr() as *mut u8;

            let ch = &mut self.rx_dma;
            let request = ch.request();
            Transfer::new_read(ch, request, src, buffer, Default::default())
        };

        let state = T::state();
        let mut remaining_len = total_len;

        let on_drop = OnDrop::new(|| {
            let regs = T::regs();
            regs.cr1().modify(|w| {
                w.set_rxdmaen(false);
                w.set_tcie(false);
            })
        });

        poll_fn(|cx| {
            state.waker.register(cx.waker());

            let isr = T::regs().isr().read();
            if remaining_len == total_len {
                Self::master_read(
                    address,
                    total_len.min(255),
                    Stop::Software,
                    total_len > 255,
                    restart,
                    &check_timeout,
                )?;
            } else if !(isr.tcr() || isr.tc()) {
                // poll_fn was woken without an interrupt present
                return Poll::Pending;
            } else if remaining_len == 0 {
                return Poll::Ready(Ok(()));
            } else {
                let last_piece = remaining_len <= 255;

                if let Err(e) = Self::master_continue(remaining_len.min(255), !last_piece, &check_timeout) {
                    return Poll::Ready(Err(e));
                }
                T::regs().cr1().modify(|w| w.set_tcie(true));
            }

            remaining_len = remaining_len.saturating_sub(255);
            Poll::Pending
        })
        .await?;

        dma_transfer.await;

        // This should be done already
        self.wait_tc(&check_timeout)?;
        self.master_stop();

        drop(on_drop);

        Ok(())
    }

    // =========================
    //  Async public API

    pub async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Error>
    where
        TXDMA: crate::i2c::TxDma<T>,
    {
        self.write_timeout(address, write, || Ok(())).await
    }

    pub async fn write_timeout(
        &mut self,
        address: u8,
        write: &[u8],
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        TXDMA: crate::i2c::TxDma<T>,
    {
        if write.is_empty() {
            self.write_internal(address, write, true, check_timeout)
        } else {
            self.write_dma_internal(address, write, true, true, check_timeout).await
        }
    }

    pub async fn write_vectored(&mut self, address: u8, write: &[&[u8]]) -> Result<(), Error>
    where
        TXDMA: crate::i2c::TxDma<T>,
    {
        self.write_vectored_timeout(address, write, || Ok(())).await
    }

    pub async fn write_vectored_timeout(
        &mut self,
        address: u8,
        write: &[&[u8]],
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        TXDMA: crate::i2c::TxDma<T>,
    {
        if write.is_empty() {
            return Err(Error::ZeroLengthTransfer);
        }
        let mut iter = write.iter();

        let mut first = true;
        let mut current = iter.next();
        while let Some(c) = current {
            let next = iter.next();
            let is_last = next.is_none();

            self.write_dma_internal(address, c, first, is_last, || check_timeout())
                .await?;
            first = false;
            current = next;
        }
        Ok(())
    }

    pub async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error>
    where
        RXDMA: crate::i2c::RxDma<T>,
    {
        self.read_timeout(address, buffer, || Ok(())).await
    }

    pub async fn read_timeout(
        &mut self,
        address: u8,
        buffer: &mut [u8],
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        RXDMA: crate::i2c::RxDma<T>,
    {
        if buffer.is_empty() {
            self.read_internal(address, buffer, false, check_timeout)
        } else {
            self.read_dma_internal(address, buffer, false, check_timeout).await
        }
    }

    pub async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error>
    where
        TXDMA: super::TxDma<T>,
        RXDMA: super::RxDma<T>,
    {
        self.write_read_timeout(address, write, read, || Ok(())).await
    }

    pub async fn write_read_timeout(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        TXDMA: super::TxDma<T>,
        RXDMA: super::RxDma<T>,
    {
        if write.is_empty() {
            self.write_internal(address, write, false, || check_timeout())?;
        } else {
            self.write_dma_internal(address, write, true, true, || check_timeout())
                .await?;
        }

        if read.is_empty() {
            self.read_internal(address, read, true, check_timeout)?;
        } else {
            self.read_dma_internal(address, read, true, check_timeout).await?;
        }

        Ok(())
    }

    // =========================
    //  Blocking public API

    pub fn blocking_read_timeout(
        &mut self,
        address: u8,
        read: &mut [u8],
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.read_internal(address, read, false, &check_timeout)
        // Automatic Stop
    }

    pub fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Error> {
        self.blocking_read_timeout(address, read, || Ok(()))
    }

    pub fn blocking_write_timeout(
        &mut self,
        address: u8,
        write: &[u8],
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.write_internal(address, write, true, &check_timeout)
    }

    pub fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<(), Error> {
        self.blocking_write_timeout(address, write, || Ok(()))
    }

    pub fn blocking_write_read_timeout(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.write_internal(address, write, false, &check_timeout)?;
        self.read_internal(address, read, true, &check_timeout)
        // Automatic Stop
    }

    pub fn blocking_write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Error> {
        self.blocking_write_read_timeout(address, write, read, || Ok(()))
    }

    pub fn blocking_write_vectored_timeout(
        &mut self,
        address: u8,
        write: &[&[u8]],
        check_timeout: impl Fn() -> Result<(), Error>,
    ) -> Result<(), Error> {
        if write.is_empty() {
            return Err(Error::ZeroLengthTransfer);
        }
        let first_length = write[0].len();
        let last_slice_index = write.len() - 1;

        if let Err(err) = Self::master_write(
            address,
            first_length.min(255),
            Stop::Software,
            (first_length > 255) || (last_slice_index != 0),
            &check_timeout,
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
                if let Err(err) = Self::master_continue(
                    slice_len.min(255),
                    (idx != last_slice_index) || (slice_len > 255),
                    &check_timeout,
                ) {
                    self.master_stop();
                    return Err(err);
                }
            }

            for (number, chunk) in slice.chunks(255).enumerate() {
                if number != 0 {
                    if let Err(err) = Self::master_continue(
                        chunk.len(),
                        (number != last_chunk_idx) || (idx != last_slice_index),
                        &check_timeout,
                    ) {
                        self.master_stop();
                        return Err(err);
                    }
                }

                for byte in chunk {
                    // Wait until we are allowed to send data
                    // (START has been ACKed or last byte when
                    // through)
                    if let Err(err) = self.wait_txe(&check_timeout) {
                        self.master_stop();
                        return Err(err);
                    }

                    // Put byte on the wire
                    //self.i2c.txdr.write(|w| w.txdata().bits(*byte));
                    T::regs().txdr().write(|w| w.set_txdata(*byte));
                }
            }
        }
        // Wait until the write finishes
        let result = self.wait_tc(&check_timeout);
        self.master_stop();
        result
    }

    pub fn blocking_write_vectored(&mut self, address: u8, write: &[&[u8]]) -> Result<(), Error> {
        self.blocking_write_vectored_timeout(address, write, || Ok(()))
    }
}

impl<'d, T: Instance, TXDMA, RXDMA> Drop for I2c<'d, T, TXDMA, RXDMA> {
    fn drop(&mut self) {
        T::disable();
    }
}

mod eh02 {
    use super::*;

    impl<'d, T: Instance> embedded_hal_02::blocking::i2c::Read for I2c<'d, T> {
        type Error = Error;

        fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_read(address, buffer)
        }
    }

    impl<'d, T: Instance> embedded_hal_02::blocking::i2c::Write for I2c<'d, T> {
        type Error = Error;

        fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(address, write)
        }
    }

    impl<'d, T: Instance> embedded_hal_02::blocking::i2c::WriteRead for I2c<'d, T> {
        type Error = Error;

        fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_write_read(address, write, read)
        }
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
    fn new(i2cclk: Hertz, freq: Hertz) -> Self {
        let i2cclk = i2cclk.0;
        let freq = freq.0;
        // Refer to RM0433 Rev 7 Figure 539 for setup and hold timing:
        //
        // t_I2CCLK = 1 / PCLK1
        // t_PRESC  = (PRESC + 1) * t_I2CCLK
        // t_SCLL   = (SCLL + 1) * t_PRESC
        // t_SCLH   = (SCLH + 1) * t_PRESC
        //
        // t_SYNC1 + t_SYNC2 > 4 * t_I2CCLK
        // t_SCL ~= t_SYNC1 + t_SYNC2 + t_SCLL + t_SCLH
        let ratio = i2cclk / freq;

        // For the standard-mode configuration method, we must have a ratio of 4
        // or higher
        assert!(ratio >= 4, "The I2C PCLK must be at least 4 times the bus frequency!");

        let (presc_reg, scll, sclh, sdadel, scldel) = if freq > 100_000 {
            // Fast-mode (Fm) or Fast-mode Plus (Fm+)
            // here we pick SCLL + 1 = 2 * (SCLH + 1)

            // Prescaler, 384 ticks for sclh/scll. Round up then subtract 1
            let presc_reg = ((ratio - 1) / 384) as u8;
            // ratio < 1200 by pclk 120MHz max., therefore presc < 16

            // Actual precale value selected
            let presc = (presc_reg + 1) as u32;

            let sclh = ((ratio / presc) - 3) / 3;
            let scll = (2 * (sclh + 1)) - 1;

            let (sdadel, scldel) = if freq > 400_000 {
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

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl embedded_hal_1::i2c::Error for Error {
        fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
            match *self {
                Self::Bus => embedded_hal_1::i2c::ErrorKind::Bus,
                Self::Arbitration => embedded_hal_1::i2c::ErrorKind::ArbitrationLoss,
                Self::Nack => {
                    embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Unknown)
                }
                Self::Timeout => embedded_hal_1::i2c::ErrorKind::Other,
                Self::Crc => embedded_hal_1::i2c::ErrorKind::Other,
                Self::Overrun => embedded_hal_1::i2c::ErrorKind::Overrun,
                Self::ZeroLengthTransfer => embedded_hal_1::i2c::ErrorKind::Other,
            }
        }
    }

    impl<'d, T: Instance, TXDMA, RXDMA> embedded_hal_1::i2c::ErrorType for I2c<'d, T, TXDMA, RXDMA> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_1::i2c::I2c for I2c<'d, T, NoDma, NoDma> {
        fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_read(address, read)
        }

        fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(address, write)
        }

        fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_write_read(address, write, read)
        }

        fn transaction(
            &mut self,
            _address: u8,
            _operations: &mut [embedded_hal_1::i2c::Operation<'_>],
        ) -> Result<(), Self::Error> {
            todo!();
        }
    }
}

#[cfg(all(feature = "unstable-traits", feature = "nightly"))]
mod eha {
    use super::super::{RxDma, TxDma};
    use super::*;

    impl<'d, T: Instance, TXDMA: TxDma<T>, RXDMA: RxDma<T>> embedded_hal_async::i2c::I2c for I2c<'d, T, TXDMA, RXDMA> {
        async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
            self.read(address, read).await
        }

        async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
            self.write(address, write).await
        }

        async fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error> {
            self.write_read(address, write, read).await
        }

        async fn transaction(
            &mut self,
            address: u8,
            operations: &mut [embedded_hal_1::i2c::Operation<'_>],
        ) -> Result<(), Self::Error> {
            let _ = address;
            let _ = operations;
            todo!()
        }
    }
}

impl<'d, T: Instance> SetConfig for I2c<'d, T> {
    type Config = Hertz;
    fn set_config(&mut self, config: &Self::Config) {
        let timings = Timings::new(T::frequency(), *config);
        T::regs().timingr().write(|reg| {
            reg.set_presc(timings.prescale);
            reg.set_scll(timings.scll);
            reg.set_sclh(timings.sclh);
            reg.set_sdadel(timings.sdadel);
            reg.set_scldel(timings.scldel);
        });
    }
}
