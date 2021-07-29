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
use embassy::traits;
use embassy::util::{AtomicWaker, Unborrow};
use embassy_hal_common::unborrow;
use futures::future::poll_fn;
use traits::i2c::I2c;

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

/// Interface to a TWIM instance.
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
    fn read_errorsrc(&self) -> Result<(), Error> {
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

    /// Wait for stop or error
    fn wait(&mut self) {
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

    /// Write to an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        let r = T::regs();

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started.
        compiler_fence(SeqCst);

        r.address.write(|w| unsafe { w.address().bits(address) });

        // Set up the DMA write.
        unsafe { self.set_tx_buffer(buffer)? };

        // Clear events
        r.events_stopped.reset();
        r.events_error.reset();
        r.events_lasttx.reset();
        self.clear_errorsrc();

        // Start write operation.
        r.shorts.write(|w| w.lasttx_stop().enabled());
        r.tasks_starttx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

        self.wait();

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);

        self.read_errorsrc()?;

        if r.txd.amount.read().bits() != buffer.len() as u32 {
            return Err(Error::Transmit);
        }

        Ok(())
    }

    /// Read from an I2C slave.
    ///
    /// The buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        let r = T::regs();

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started.
        compiler_fence(SeqCst);

        r.address.write(|w| unsafe { w.address().bits(address) });

        // Set up the DMA read.
        unsafe { self.set_rx_buffer(buffer)? };

        // Clear events
        r.events_stopped.reset();
        r.events_error.reset();
        self.clear_errorsrc();

        // Start read operation.
        r.shorts.write(|w| w.lastrx_stop().enabled());
        r.tasks_startrx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

        self.wait();

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);

        self.read_errorsrc()?;

        if r.rxd.amount.read().bits() != buffer.len() as u32 {
            return Err(Error::Receive);
        }

        Ok(())
    }

    /// Write data to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    ///
    /// The buffers must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn write_then_read(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        let r = T::regs();

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started.
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

        // Start write+read operation.
        r.shorts.write(|w| {
            w.lasttx_startrx().enabled();
            w.lastrx_stop().enabled();
            w
        });
        // `1` is a valid value to write to task registers.
        r.tasks_starttx.write(|w| unsafe { w.bits(1) });

        self.wait();

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // after all possible DMA actions have completed.
        compiler_fence(SeqCst);

        self.read_errorsrc()?;

        let bad_write = r.txd.amount.read().bits() != wr_buffer.len() as u32;
        let bad_read = r.rxd.amount.read().bits() != rd_buffer.len() as u32;

        if bad_write {
            return Err(Error::Transmit);
        }

        if bad_read {
            return Err(Error::Receive);
        }

        Ok(())
    }

    /// Copy data into RAM and write to an I2C slave.
    ///
    /// The write buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 1024 bytes on the nRF52840.
    pub fn copy_write(&mut self, address: u8, wr_buffer: &[u8]) -> Result<(), Error> {
        if wr_buffer.len() > FORCE_COPY_BUFFER_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        // Copy to RAM
        let wr_ram_buffer = &mut [0; FORCE_COPY_BUFFER_SIZE][..wr_buffer.len()];
        wr_ram_buffer.copy_from_slice(wr_buffer);

        self.write(address, wr_ram_buffer)
    }

    /// Copy data into RAM and write to an I2C slave, then read data from the slave without
    /// triggering a stop condition between the two.
    ///
    /// The write buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 1024 bytes on the nRF52840.
    ///
    /// The read buffer must have a length of at most 255 bytes on the nRF52832
    /// and at most 65535 bytes on the nRF52840.
    pub fn copy_write_then_read(
        &mut self,
        address: u8,
        wr_buffer: &[u8],
        rd_buffer: &mut [u8],
    ) -> Result<(), Error> {
        if wr_buffer.len() > FORCE_COPY_BUFFER_SIZE {
            return Err(Error::TxBufferTooLong);
        }

        // Copy to RAM
        let wr_ram_buffer = &mut [0; FORCE_COPY_BUFFER_SIZE][..wr_buffer.len()];
        wr_ram_buffer.copy_from_slice(wr_buffer);

        self.write_then_read(address, wr_ram_buffer, rd_buffer)
    }

    fn wait_for_stopped_event(cx: &mut core::task::Context) -> Poll<()> {
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
    }
}

impl<'a, T: Instance> Drop for Twim<'a, T> {
    fn drop(&mut self) {
        info!("twim drop");

        // TODO when implementing async here, check for abort

        // disable!
        let r = T::regs();
        r.enable.write(|w| w.enable().disabled());

        gpio::deconfigure_pin(r.psel.sda.read().bits());
        gpio::deconfigure_pin(r.psel.scl.read().bits());

        info!("twim drop: done");
    }
}

impl<'d, T> I2c for Twim<'d, T>
where
    T: Instance,
{
    type Error = Error;

    #[rustfmt::skip]
    type WriteFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;
    #[rustfmt::skip]
    type ReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;
    #[rustfmt::skip]
    type WriteReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn read<'a>(&'a mut self, address: u8, buffer: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            // NOTE: RAM slice check for buffer is not necessary, as a mutable
            // slice can only be built from data located in RAM.

            let r = T::regs();

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started.
            compiler_fence(SeqCst);

            r.address.write(|w| unsafe { w.address().bits(address) });

            // Set up the DMA read.
            unsafe { self.set_rx_buffer(buffer)? };

            // Reset events
            r.events_stopped.reset();
            r.events_error.reset();
            self.clear_errorsrc();

            // Enable events
            r.intenset.write(|w| w.stopped().set().error().set());

            // Start read operation.
            r.shorts.write(|w| w.lastrx_stop().enabled());
            r.tasks_startrx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // after all possible DMA actions have completed.
            compiler_fence(SeqCst);

            // Wait for 'stopped' event.
            poll_fn(Self::wait_for_stopped_event).await;

            self.read_errorsrc()?;

            if r.rxd.amount.read().bits() != buffer.len() as u32 {
                return Err(Error::Receive);
            }

            Ok(())
        }
    }

    fn write<'a>(&'a mut self, address: u8, bytes: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            slice_in_ram_or(bytes, Error::DMABufferNotInDataMemory)?;

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started.
            compiler_fence(SeqCst);

            let r = T::regs();

            // Set up current address we're trying to talk to
            r.address.write(|w| unsafe { w.address().bits(address) });

            // Set up DMA write.
            unsafe {
                self.set_tx_buffer(bytes)?;
            }

            // Reset events
            r.events_stopped.reset();
            r.events_error.reset();
            r.events_lasttx.reset();
            self.clear_errorsrc();

            // Enable events
            r.intenset.write(|w| w.stopped().set().error().set());

            // Start write operation.
            r.shorts.write(|w| w.lasttx_stop().enabled());
            r.tasks_starttx.write(|w|
            // `1` is a valid value to write to task registers.
            unsafe { w.bits(1) });

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // after all possible DMA actions have completed.
            compiler_fence(SeqCst);

            // Wait for 'stopped' event.
            poll_fn(Self::wait_for_stopped_event).await;

            self.read_errorsrc()?;

            if r.txd.amount.read().bits() != bytes.len() as u32 {
                return Err(Error::Transmit);
            }

            Ok(())
        }
    }

    fn write_read<'a>(
        &'a mut self,
        address: u8,
        bytes: &'a [u8],
        buffer: &'a mut [u8],
    ) -> Self::WriteReadFuture<'a> {
        async move {
            slice_in_ram_or(bytes, Error::DMABufferNotInDataMemory)?;
            // NOTE: RAM slice check for buffer is not necessary, as a mutable
            // slice can only be built from data located in RAM.

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started.
            compiler_fence(SeqCst);

            let r = T::regs();

            // Set up current address we're trying to talk to
            r.address.write(|w| unsafe { w.address().bits(address) });

            // Set up DMA buffers.
            unsafe {
                self.set_tx_buffer(bytes)?;
                self.set_rx_buffer(buffer)?;
            }

            // Reset events
            r.events_stopped.reset();
            r.events_error.reset();
            r.events_lasttx.reset();
            self.clear_errorsrc();

            // Enable events
            r.intenset.write(|w| w.stopped().set().error().set());

            // Start write+read operation.
            r.shorts.write(|w| {
                w.lasttx_startrx().enabled();
                w.lastrx_stop().enabled();
                w
            });
            // `1` is a valid value to write to task registers.
            r.tasks_starttx.write(|w| unsafe { w.bits(1) });

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // after all possible DMA actions have completed.
            compiler_fence(SeqCst);

            // Wait for 'stopped' event.
            poll_fn(Self::wait_for_stopped_event).await;

            self.read_errorsrc()?;

            let bad_write = r.txd.amount.read().bits() != bytes.len() as u32;
            let bad_read = r.rxd.amount.read().bits() != buffer.len() as u32;

            if bad_write {
                return Err(Error::Transmit);
            }

            if bad_read {
                return Err(Error::Receive);
            }

            Ok(())
        }
    }
}

impl<'a, T: Instance> embedded_hal::blocking::i2c::Write for Twim<'a, T> {
    type Error = Error;

    fn write<'w>(&mut self, addr: u8, bytes: &'w [u8]) -> Result<(), Error> {
        if slice_in_ram(bytes) {
            self.write(addr, bytes)
        } else {
            let buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..];
            for chunk in bytes.chunks(FORCE_COPY_BUFFER_SIZE) {
                buf[..chunk.len()].copy_from_slice(chunk);
                self.write(addr, &buf[..chunk.len()])?;
            }
            Ok(())
        }
    }
}

impl<'a, T: Instance> embedded_hal::blocking::i2c::Read for Twim<'a, T> {
    type Error = Error;

    fn read<'w>(&mut self, addr: u8, bytes: &'w mut [u8]) -> Result<(), Error> {
        self.read(addr, bytes)
    }
}

impl<'a, T: Instance> embedded_hal::blocking::i2c::WriteRead for Twim<'a, T> {
    type Error = Error;

    fn write_read<'w>(
        &mut self,
        addr: u8,
        bytes: &'w [u8],
        buffer: &'w mut [u8],
    ) -> Result<(), Error> {
        if slice_in_ram(bytes) {
            self.write_then_read(addr, bytes, buffer)
        } else {
            self.copy_write_then_read(addr, bytes, buffer)
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
