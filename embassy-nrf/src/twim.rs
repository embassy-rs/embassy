#![macro_use]

//! HAL interface to the TWIM peripheral.
//!
//! See product specification:
//!
//! - nRF52832: Section 33
//! - nRF52840: Section 6.31
use core::future::Future;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};
use core::task::Poll;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::gpio;
use crate::gpio::Pin as GpioPin;
use crate::pac;
use crate::util::{slice_in_ram, slice_in_ram_or};

pub enum Frequency {
    #[doc = "26738688: 100 kbps"]
    K100 = 26738688,
    #[doc = "67108864: 250 kbps"]
    K250 = 67108864,
    #[doc = "104857600: 400 kbps"]
    K400 = 104857600,
}

#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
    pub sda_pullup: bool,
    pub scl_pullup: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::K100,
            sda_pullup: false,
            scl_pullup: false,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    TxBufferZeroLength,
    RxBufferZeroLength,
    Transmit,
    Receive,
    DMABufferNotInDataMemory,
    AddressNack,
    DataNack,
    Overrun,
}

/// Interface to a TWIM instance using EasyDMA to offload the transmission and reception workload.
///
/// For more details about EasyDMA, consult the module documentation.
pub struct Twim<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Twim<'d, T> {
    pub fn new(
        _twim: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        sda: impl Unborrow<Target = impl GpioPin> + 'd,
        scl: impl Unborrow<Target = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(irq, sda, scl);

        let r = T::regs();

        // Configure pins
        sda.conf().write(|w| {
            w.dir().input();
            w.input().connect();
            w.drive().s0d1();
            if config.sda_pullup {
                w.pull().pullup();
            }
            w
        });
        scl.conf().write(|w| {
            w.dir().input();
            w.input().connect();
            w.drive().s0d1();
            if config.scl_pullup {
                w.pull().pullup();
            }
            w
        });

        // Select pins.
        r.psel.sda.write(|w| unsafe { w.bits(sda.psel_bits()) });
        r.psel.scl.write(|w| unsafe { w.bits(scl.psel_bits()) });

        // Enable TWIM instance.
        r.enable.write(|w| w.enable().enabled());

        // Configure frequency.
        r.frequency
            .write(|w| unsafe { w.frequency().bits(config.frequency as u32) });

        // Disable all events interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self {
            phantom: PhantomData,
        }
    }

    fn on_interrupt(_: *mut ()) {
        let r = T::regs();
        let s = T::state();

        if r.events_stopped.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.stopped().clear());
        }
        if r.events_error.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.error().clear());
        }
    }

    /// Set TX buffer, checking that it is in RAM and has suitable length.
    unsafe fn set_tx_buffer(&mut self, buffer: &[u8]) -> Result<(), Error> {
        slice_in_ram_or(buffer, Error::DMABufferNotInDataMemory)?;

        if buffer.len() == 0 {
            return Err(Error::TxBufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        let r = T::regs();

        r.txd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the I2C transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            w.ptr().bits(buffer.as_ptr() as u32));
        r.txd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to `u8` is also fine.
            //
            // The MAXCNT field is 8 bits wide and accepts the full range of
            // values.
            w.maxcnt().bits(buffer.len() as _));

        Ok(())
    }

    /// Set RX buffer, checking that it has suitable length.
    unsafe fn set_rx_buffer(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // NOTE: RAM slice check is not necessary, as a mutable
        // slice can only be built from data located in RAM.

        if buffer.len() == 0 {
            return Err(Error::RxBufferZeroLength);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::RxBufferTooLong);
        }

        let r = T::regs();

        r.rxd.ptr.write(|w|
            // We're giving the register a pointer to the stack. Since we're
            // waiting for the I2C transaction to end before this stack pointer
            // becomes invalid, there's nothing wrong here.
            //
            // The PTR field is a full 32 bits wide and accepts the full range
            // of values.
            w.ptr().bits(buffer.as_mut_ptr() as u32));
        r.rxd.maxcnt.write(|w|
            // We're giving it the length of the buffer, so no danger of
            // accessing invalid memory. We have verified that the length of the
            // buffer fits in an `u8`, so the cast to the type of maxcnt
            // is also fine.
            //
            // Note that that nrf52840 maxcnt is a wider
            // type than a u8, so we use a `_` cast rather than a `u8` cast.
            // The MAXCNT field is thus at least 8 bits wide and accepts the
            // full range of values that fit in a `u8`.
            w.maxcnt().bits(buffer.len() as _));

        Ok(())
    }

    fn clear_errorsrc(&mut self) {
        let r = T::regs();
        r.errorsrc
            .write(|w| w.anack().bit(true).dnack().bit(true).overrun().bit(true));
    }

    /// Get Error instance, if any occurred.
    fn check_errorsrc(&self) -> Result<(), Error> {
        let r = T::regs();

        let err = r.errorsrc.read();
        if err.anack().is_received() {
            return Err(Error::AddressNack);
        }
        if err.dnack().is_received() {
            return Err(Error::DataNack);
        }
        if err.overrun().is_received() {
            return Err(Error::DataNack);
        }
        Ok(())
    }

    fn check_rx(&self, len: usize) -> Result<(), Error> {
        let r = T::regs();
        if r.rxd.amount.read().bits() != len as u32 {
            Err(Error::Receive)
        } else {
            Ok(())
        }
    }

    fn check_tx(&self, len: usize) -> Result<(), Error> {
        let r = T::regs();
        if r.txd.amount.read().bits() != len as u32 {
            Err(Error::Transmit)
        } else {
            Ok(())
        }
    }

    /// Wait for stop or error
    fn blocking_wait(&mut self) {
        let r = T::regs();
        loop {
            if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();
                break;
            }
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
            }
        }
    }

    /// Wait for stop or error
    fn async_wait(&mut self) -> impl Future<Output = ()> {
        poll_fn(move |cx| {
            let r = T::regs();
            let s = T::state();

            s.end_waker.register(cx.waker());
            if r.events_stopped.read().bits() != 0 {
                r.events_stopped.reset();

                return Poll::Ready(());
            }

            // stop if an error occured
            if r.events_error.read().bits() != 0 {
                r.events_error.reset();
                r.tasks_stop.write(|w| unsafe { w.bits(1) });
            }

            Poll::Pending
        })
    }

    fn setup_write_from_ram(
        &mut self,
        address: u8,
        buffer: &[u8],
        inten: bool,
    ) -> Result<(), Error> {
        let r = T::regs();

        compiler_fence(SeqCst);

        r.address.write(|w| unsafe { w.address().bits(address) });

        // Set up the DMA write.
        unsafe { self.set_tx_buffer(buffer)? };

        // Clear events
        r.events_stopped.reset();
        r.events_error.reset();
        r.events_lasttx.reset();
        self.clear_errorsrc();

        if inten {
            r.intenset.write(|w| w.stopped().set().error().set());
        } else {
            r.intenclr.write(|w| w.stopped().clear().error().clear());
        }

        // Start write operation.
        r.shorts.write(|w| w.lasttx_stop().enabled());
        r.tasks_starttx.write(|w| unsafe { w.bits(1) });
        Ok(())
    }

    fn setup_read(&mut self, address: u8, buffer: &mut [u8], inten: bool) -> Result<(), Error> {
        let r = T::regs();

        compiler_fence(SeqCst);

        r.address.write(|w| unsafe { w.address().bits(address) });

        // Set up the DMA read.
        unsafe { self.set_rx_buffer(buffer)? };

        // Clear events
        r.events_stopped.reset();
        r.events_error.reset();
        self.clear_errorsrc();

        if inten {
            r.intenset.write(|w| w.stopped().set().error().set());
        } else {
            r.intenclr.write(|w| w.stopped().clear().error().clear());
        }

        // Start read operation.
        r.shorts.write(|w| w.lastrx_stop().enabled());
        r.tasks_startrx.write(|w| unsafe { w.bits(1) });
        Ok(())
    }

    fn setup_write_read_from_ram(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
        inten: bool,
    ) -> Result<(), Error> {
        let r = T::regs();

        compiler_fence(SeqCst);

        r.address.write(|w| unsafe { w.address().bits(address) });

        // Set up DMA buffers.
        unsafe {
            self.set_tx_buffer(wr_buffer)?;
            self.set_rx_buffer(rd_buffer)?;
        }

        // Clear events
        r.events_stopped.reset();
        r.events_error.reset();
        self.clear_errorsrc();

        if inten {
            r.intenset.write(|w| w.stopped().set().error().set());
        } else {
            r.intenclr.write(|w| w.stopped().clear().error().clear());
        }

        // Start write+read operation.
        r.shorts.write(|w| {
            w.lasttx_startrx().enabled();
            w.lastrx_stop().enabled();
            w
        });
        r.tasks_starttx.write(|w| unsafe { w.bits(1) });
        Ok(())
    }

    fn setup_write_read(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
        inten: bool,
    ) -> Result<(), Error> {
        match self.setup_write_read_from_ram(address, wr_buffer, rd_buffer, inten) {
            Ok(_) => Ok(()),
            Err(Error::DMABufferNotInDataMemory) => {
                trace!("Copying TWIM tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..wr_buffer.len()];
                tx_ram_buf.copy_from_slice(wr_buffer);
                self.setup_write_read_from_ram(address, &tx_ram_buf, rd_buffer, inten)
            }
            Err(error) => Err(error),
        }
    }

    fn setup_write(&mut self, address: u8, wr_buffer: &[u8], inten: bool) -> Result<(), Error> {
        match self.setup_write_from_ram(address, wr_buffer, inten) {
            Ok(_) => Ok(()),
            Err(Error::DMABufferNotInDataMemory) => {
                trace!("Copying TWIM tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..wr_buffer.len()];
                tx_ram_buf.copy_from_slice(wr_buffer);
                self.setup_write_from_ram(address, &tx_ram_buf, inten)
            }
            Err(error) => Err(error),
        }
    }

    /// Write to an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn blocking_write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.setup_write(address, buffer, false)?;
        self.blocking_wait();
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(buffer.len())?;
        Ok(())
    }

    /// Same as [`blocking_write`](Twim::blocking_write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_write_from_ram(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.setup_write_from_ram(address, buffer, false)?;
        self.blocking_wait();
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(buffer.len())?;
        Ok(())
    }

    /// Read from an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn blocking_read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.setup_read(address, buffer, false)?;
        self.blocking_wait();
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_rx(buffer.len())?;
        Ok(())
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    ///
    /// The buffers must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn blocking_write_read(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.setup_write_read(address, wr_buffer, rd_buffer, false)?;
        self.blocking_wait();
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(wr_buffer.len())?;
        self.check_rx(rd_buffer.len())?;
        Ok(())
    }

    /// Same as [`blocking_write_read`](Twim::blocking_write_read) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_write_read_from_ram(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.setup_write_read_from_ram(address, wr_buffer, rd_buffer, false)?;
        self.blocking_wait();
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(wr_buffer.len())?;
        self.check_rx(rd_buffer.len())?;
        Ok(())
    }

    pub async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.setup_read(address, buffer, true)?;
        self.async_wait().await;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_rx(buffer.len())?;
        Ok(())
    }

    pub async fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.setup_write(address, buffer, true)?;
        self.async_wait().await;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(buffer.len())?;
        Ok(())
    }

    /// Same as [`write`](Twim::write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn write_from_ram(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.setup_write_from_ram(address, buffer, true)?;
        self.async_wait().await;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(buffer.len())?;
        Ok(())
    }

    pub async fn write_read(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.setup_write_read(address, wr_buffer, rd_buffer, true)?;
        self.async_wait().await;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(wr_buffer.len())?;
        self.check_rx(rd_buffer.len())?;
        Ok(())
    }

    /// Same as [`write_read`](Twim::write_read) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn write_read_from_ram(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        self.setup_write_read_from_ram(address, wr_buffer, rd_buffer, true)?;
        self.async_wait().await;
        compiler_fence(SeqCst);
        self.check_errorsrc()?;
        self.check_tx(wr_buffer.len())?;
        self.check_rx(rd_buffer.len())?;
        Ok(())
    }
}

impl<'a, T: Instance> Drop for Twim<'a, T> {
    fn drop(&mut self) {
        trace!("twim drop");

        // TODO: check for abort

        // disable!
        let r = T::regs();
        r.enable.write(|w| w.enable().disabled());

        gpio::deconfigure_pin(r.psel.sda.read().bits());
        gpio::deconfigure_pin(r.psel.scl.read().bits());

        trace!("twim drop: done");
    }
}

pub(crate) mod sealed {
    use super::*;

    pub struct State {
        pub end_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                end_waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static pac::twim0::RegisterBlock;
        fn state() -> &'static State;
    }
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_twim {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::twim::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::twim0::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::twim::sealed::State {
                static STATE: crate::twim::sealed::State = crate::twim::sealed::State::new();
                &STATE
            }
        }
        impl crate::twim::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}

// ====================

mod eh02 {
    use super::*;

    impl<'a, T: Instance> embedded_hal_02::blocking::i2c::Write for Twim<'a, T> {
        type Error = Error;

        fn write<'w>(&mut self, addr: u8, bytes: &'w [u8]) -> Result<(), Error> {
            if slice_in_ram(bytes) {
                self.blocking_write(addr, bytes)
            } else {
                let buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..];
                for chunk in bytes.chunks(FORCE_COPY_BUFFER_SIZE) {
                    buf[..chunk.len()].copy_from_slice(chunk);
                    self.blocking_write(addr, &buf[..chunk.len()])?;
                }
                Ok(())
            }
        }
    }

    impl<'a, T: Instance> embedded_hal_02::blocking::i2c::Read for Twim<'a, T> {
        type Error = Error;

        fn read<'w>(&mut self, addr: u8, bytes: &'w mut [u8]) -> Result<(), Error> {
            self.blocking_read(addr, bytes)
        }
    }

    impl<'a, T: Instance> embedded_hal_02::blocking::i2c::WriteRead for Twim<'a, T> {
        type Error = Error;

        fn write_read<'w>(
            &mut self,
            addr: u8,
            bytes: &'w [u8],
            buffer: &'w mut [u8],
        ) -> Result<(), Error> {
            self.blocking_write_read(addr, bytes, buffer)
        }
    }
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl embedded_hal_1::i2c::Error for Error {
        fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
            match *self {
                Self::TxBufferTooLong => embedded_hal_1::i2c::ErrorKind::Other,
                Self::RxBufferTooLong => embedded_hal_1::i2c::ErrorKind::Other,
                Self::TxBufferZeroLength => embedded_hal_1::i2c::ErrorKind::Other,
                Self::RxBufferZeroLength => embedded_hal_1::i2c::ErrorKind::Other,
                Self::Transmit => embedded_hal_1::i2c::ErrorKind::Other,
                Self::Receive => embedded_hal_1::i2c::ErrorKind::Other,
                Self::DMABufferNotInDataMemory => embedded_hal_1::i2c::ErrorKind::Other,
                Self::AddressNack => embedded_hal_1::i2c::ErrorKind::NoAcknowledge(
                    embedded_hal_1::i2c::NoAcknowledgeSource::Address,
                ),
                Self::DataNack => embedded_hal_1::i2c::ErrorKind::NoAcknowledge(
                    embedded_hal_1::i2c::NoAcknowledgeSource::Data,
                ),
                Self::Overrun => embedded_hal_1::i2c::ErrorKind::Overrun,
            }
        }
    }

    impl<'d, T: Instance> embedded_hal_1::i2c::ErrorType for Twim<'d, T> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_1::i2c::blocking::I2c for Twim<'d, T> {
        fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
            self.blocking_read(address, buffer)
        }

        fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(address, buffer)
        }

        fn write_iter<B>(&mut self, _address: u8, _bytes: B) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>,
        {
            todo!();
        }

        fn write_iter_read<B>(
            &mut self,
            _address: u8,
            _bytes: B,
            _buffer: &mut [u8],
        ) -> Result<(), Self::Error>
        where
            B: IntoIterator<Item = u8>,
        {
            todo!();
        }

        fn write_read(
            &mut self,
            address: u8,
            wr_buffer: &[u8],
            rd_buffer: &mut [u8],
        ) -> Result<(), Self::Error> {
            self.blocking_write_read(address, wr_buffer, rd_buffer)
        }

        fn transaction<'a>(
            &mut self,
            _address: u8,
            _operations: &mut [embedded_hal_1::i2c::blocking::Operation<'a>],
        ) -> Result<(), Self::Error> {
            todo!();
        }

        fn transaction_iter<'a, O>(
            &mut self,
            _address: u8,
            _operations: O,
        ) -> Result<(), Self::Error>
        where
            O: IntoIterator<Item = embedded_hal_1::i2c::blocking::Operation<'a>>,
        {
            todo!();
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "unstable-traits", feature = "nightly"))] {
        impl<'d, T: Instance> embedded_hal_async::i2c::I2c for Twim<'d, T> {
            type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn read<'a>(&'a mut self, address: u8, buffer: &'a mut [u8]) -> Self::ReadFuture<'a> {
                self.read(address, buffer)
            }

            type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn write<'a>(&'a mut self, address: u8, bytes: &'a [u8]) -> Self::WriteFuture<'a> {
                self.write(address, bytes)
            }

            type WriteReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

            fn write_read<'a>(
                &'a mut self,
                address: u8,
                wr_buffer: &'a [u8],
                rd_buffer: &'a mut [u8],
            ) -> Self::WriteReadFuture<'a> {
                self.write_read(address, wr_buffer, rd_buffer)
            }

            type TransactionFuture<'a, 'b> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a, 'b: 'a;

            fn transaction<'a, 'b>(
                &'a mut self,
                address: u8,
                operations: &'a mut [embedded_hal_async::i2c::Operation<'b>],
            ) -> Self::TransactionFuture<'a, 'b> {
                let _ = address;
                let _ = operations;
                async move { todo!() }
            }
        }
    }
}
