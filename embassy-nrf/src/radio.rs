#![macro_use]

use core::future::Future;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};
use core::task::Poll;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::traits;
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::pac;
use crate::util::{slice_in_ram, slice_in_ram_or};

#[non_exhaustive]
pub struct Config {
    pub frequency: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: 2490,
        }
    }
}

/// Interface to a RADIO instance.
pub struct Radio<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Radio<'d, T> {
    pub fn new(
        _radio: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(irq);

        let r = T::regs();

        // Enable RADIO instance.
        r.power.write(|w| w.power().enabled());

        // Configure frequency.
        // r.frequency
        //     .write(|w| unsafe { w.frequency().bits(config.frequency as u32) });

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

        if r.events_end.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.end().clear());
        }
        if r.events_crcerror.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.crcerror().clear());
        }
    }

    // /// Set TX buffer, checking that it is in RAM and has suitable length.
    // unsafe fn set_tx_buffer(&mut self, buffer: &[u8]) -> Result<(), Error> {
    //     slice_in_ram_or(buffer, Error::DMABufferNotInDataMemory)?;

    //     if buffer.len() == 0 {
    //         return Err(Error::TxBufferZeroLength);
    //     }
    //     if buffer.len() > EASY_DMA_SIZE {
    //         return Err(Error::TxBufferTooLong);
    //     }

    //     let r = T::regs();

    //     r.txd.ptr.write(|w|
    //         // We're giving the register a pointer to the stack. Since we're
    //         // waiting for the I2C transaction to end before this stack pointer
    //         // becomes invalid, there's nothing wrong here.
    //         //
    //         // The PTR field is a full 32 bits wide and accepts the full range
    //         // of values.
    //         w.ptr().bits(buffer.as_ptr() as u32));
    //     r.txd.maxcnt.write(|w|
    //         // We're giving it the length of the buffer, so no danger of
    //         // accessing invalid memory. We have verified that the length of the
    //         // buffer fits in an `u8`, so the cast to `u8` is also fine.
    //         //
    //         // The MAXCNT field is 8 bits wide and accepts the full range of
    //         // values.
    //         w.maxcnt().bits(buffer.len() as _));

    //     Ok(())
    // }

    // /// Set RX buffer, checking that it has suitable length.
    // unsafe fn set_rx_buffer(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
    //     // NOTE: RAM slice check is not necessary, as a mutable
    //     // slice can only be built from data located in RAM.

    //     if buffer.len() == 0 {
    //         return Err(Error::RxBufferZeroLength);
    //     }
    //     if buffer.len() > EASY_DMA_SIZE {
    //         return Err(Error::RxBufferTooLong);
    //     }

    //     let r = T::regs();

    //     r.rxd.ptr.write(|w|
    //         // We're giving the register a pointer to the stack. Since we're
    //         // waiting for the I2C transaction to end before this stack pointer
    //         // becomes invalid, there's nothing wrong here.
    //         //
    //         // The PTR field is a full 32 bits wide and accepts the full range
    //         // of values.
    //         w.ptr().bits(buffer.as_mut_ptr() as u32));
    //     r.rxd.maxcnt.write(|w|
    //         // We're giving it the length of the buffer, so no danger of
    //         // accessing invalid memory. We have verified that the length of the
    //         // buffer fits in an `u8`, so the cast to the type of maxcnt
    //         // is also fine.
    //         //
    //         // Note that that nrf52840 maxcnt is a wider
    //         // type than a u8, so we use a `_` cast rather than a `u8` cast.
    //         // The MAXCNT field is thus at least 8 bits wide and accepts the
    //         // full range of values that fit in a `u8`.
    //         w.maxcnt().bits(buffer.len() as _));

    //     Ok(())
    // }

    // fn clear_errorsrc(&mut self) {
    //     let r = T::regs();
    //     r.errorsrc
    //         .write(|w| w.anack().bit(true).dnack().bit(true).overrun().bit(true));
    // }

    // /// Get Error instance, if any occurred.
    // fn read_errorsrc(&self) -> Result<(), Error> {
    //     let r = T::regs();

    //     let err = r.errorsrc.read();
    //     if err.anack().is_received() {
    //         return Err(Error::AddressNack);
    //     }
    //     if err.dnack().is_received() {
    //         return Err(Error::DataNack);
    //     }
    //     if err.overrun().is_received() {
    //         return Err(Error::DataNack);
    //     }
    //     Ok(())
    // }

    // /// Wait for stop or error
    // fn wait(&mut self) {
    //     let r = T::regs();
    //     loop {
    //         if r.events_stopped.read().bits() != 0 {
    //             r.events_stopped.reset();
    //             break;
    //         }
    //         if r.events_error.read().bits() != 0 {
    //             r.events_error.reset();
    //             r.tasks_stop.write(|w| unsafe { w.bits(1) });
    //         }
    //     }
    // }

    // /// Write to an I2C slave.
    // ///
    // /// The buffer must have a length of at most 255 bytes on the nRF52832
    // /// and at most 65535 bytes on the nRF52840.
    // pub fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
    //     let r = T::regs();

    //     // Conservative compiler fence to prevent optimizations that do not
    //     // take in to account actions by DMA. The fence has been placed here,
    //     // before any DMA action has started.
    //     compiler_fence(SeqCst);

    //     r.address.write(|w| unsafe { w.address().bits(address) });

    //     // Set up the DMA write.
    //     unsafe { self.set_tx_buffer(buffer)? };

    //     // Clear events
    //     r.events_stopped.reset();
    //     r.events_error.reset();
    //     r.events_lasttx.reset();
    //     self.clear_errorsrc();

    //     // Start write operation.
    //     r.shorts.write(|w| w.lasttx_stop().enabled());
    //     r.tasks_starttx.write(|w|
    //         // `1` is a valid value to write to task registers.
    //         unsafe { w.bits(1) });

    //     self.wait();

    //     // Conservative compiler fence to prevent optimizations that do not
    //     // take in to account actions by DMA. The fence has been placed here,
    //     // after all possible DMA actions have completed.
    //     compiler_fence(SeqCst);

    //     self.read_errorsrc()?;

    //     if r.txd.amount.read().bits() != buffer.len() as u32 {
    //         return Err(Error::Transmit);
    //     }

    //     Ok(())
    // }

    // /// Read from an I2C slave.
    // ///
    // /// The buffer must have a length of at most 255 bytes on the nRF52832
    // /// and at most 65535 bytes on the nRF52840.
    // pub fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
    //     let r = T::regs();

    //     // Conservative compiler fence to prevent optimizations that do not
    //     // take in to account actions by DMA. The fence has been placed here,
    //     // before any DMA action has started.
    //     compiler_fence(SeqCst);

    //     r.address.write(|w| unsafe { w.address().bits(address) });

    //     // Set up the DMA read.
    //     unsafe { self.set_rx_buffer(buffer)? };

    //     // Clear events
    //     r.events_stopped.reset();
    //     r.events_error.reset();
    //     self.clear_errorsrc();

    //     // Start read operation.
    //     r.shorts.write(|w| w.lastrx_stop().enabled());
    //     r.tasks_startrx.write(|w|
    //         // `1` is a valid value to write to task registers.
    //         unsafe { w.bits(1) });

    //     self.wait();

    //     // Conservative compiler fence to prevent optimizations that do not
    //     // take in to account actions by DMA. The fence has been placed here,
    //     // after all possible DMA actions have completed.
    //     compiler_fence(SeqCst);

    //     self.read_errorsrc()?;

    //     if r.rxd.amount.read().bits() != buffer.len() as u32 {
    //         return Err(Error::Receive);
    //     }

    //     Ok(())
    // }

    // /// Copy data into RAM and write to an I2C slave.
    // ///
    // /// The write buffer must have a length of at most 255 bytes on the nRF52832
    // /// and at most 1024 bytes on the nRF52840.
    // pub fn copy_write(&mut self, address: u8, wr_buffer: &[u8]) -> Result<(), Error> {
    //     if wr_buffer.len() > FORCE_COPY_BUFFER_SIZE {
    //         return Err(Error::TxBufferTooLong);
    //     }

    //     // Copy to RAM
    //     let wr_ram_buffer = &mut [0; FORCE_COPY_BUFFER_SIZE][..wr_buffer.len()];
    //     wr_ram_buffer.copy_from_slice(wr_buffer);

    //     self.write(address, wr_ram_buffer)
    // }

    // /// Copy data into RAM and write to an I2C slave, then read data from the slave without
    // /// triggering a stop condition between the two.
    // ///
    // /// The write buffer must have a length of at most 255 bytes on the nRF52832
    // /// and at most 1024 bytes on the nRF52840.
    // ///
    // /// The read buffer must have a length of at most 255 bytes on the nRF52832
    // /// and at most 65535 bytes on the nRF52840.
    // pub fn copy_write_then_read(
    //     &mut self,
    //     address: u8,
    //     wr_buffer: &[u8],
    //     rd_buffer: &mut [u8],
    // ) -> Result<(), Error> {
    //     if wr_buffer.len() > FORCE_COPY_BUFFER_SIZE {
    //         return Err(Error::TxBufferTooLong);
    //     }

    //     // Copy to RAM
    //     let wr_ram_buffer = &mut [0; FORCE_COPY_BUFFER_SIZE][..wr_buffer.len()];
    //     wr_ram_buffer.copy_from_slice(wr_buffer);

    //     self.write_then_read(address, wr_ram_buffer, rd_buffer)
    // }

    // fn wait_for_stopped_event(cx: &mut core::task::Context) -> Poll<()> {
    //     let r = T::regs();
    //     let s = T::state();

    //     s.end_waker.register(cx.waker());
    //     if r.events_stopped.read().bits() != 0 {
    //         r.events_stopped.reset();

    //         return Poll::Ready(());
    //     }

    //     // stop if an error occured
    //     if r.events_error.read().bits() != 0 {
    //         r.events_error.reset();
    //         r.tasks_stop.write(|w| unsafe { w.bits(1) });
    //     }

    //     Poll::Pending
    // }
}

impl<'a, T: Instance> Drop for Radio<'a, T> {
    fn drop(&mut self) {
        info!("radio drop");

        // TODO when implementing async here, check for abort

        // disable!
        let r = T::regs();
        r.power.write(|w| w.power().disabled());

        info!("radio drop: done");
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
        fn regs() -> &'static pac::radio::RegisterBlock;
        fn state() -> &'static State;
    }
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_radio {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::radio::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::radio::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::radio::sealed::State {
                static STATE: crate::radio::sealed::State = crate::radio::sealed::State::new();
                &STATE
            }
        }
        impl crate::radio::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}
