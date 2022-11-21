#![macro_use]

use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_common::{into_ref, PeripheralRef};
pub use embedded_hal_02::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};

use crate::chip::FORCE_COPY_BUFFER_SIZE;
use crate::gpio::sealed::Pin as _;
use crate::gpio::{self, AnyPin, Pin as GpioPin, PselBits};
use crate::interrupt::{Interrupt, InterruptExt};
use crate::util::{slice_in_ram_or, slice_ptr_parts, slice_ptr_parts_mut};
use crate::{pac, Peripheral};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    /// EasyDMA can only read from data memory, read only buffers in flash will fail.
    DMABufferNotInDataMemory,
}

/// Interface for the SPIS peripheral using EasyDMA to offload the transmission and reception workload.
///
/// For more details about EasyDMA, consult the module documentation.
pub struct Spis<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

#[non_exhaustive]
pub struct Config {
    pub mode: Mode,
    pub orc: u8,
    pub def: u8,
    pub auto_acquire: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: MODE_0,
            orc: 0x00,
            def: 0x00,
            auto_acquire: true,
        }
    }
}

impl<'d, T: Instance> Spis<'d, T> {
    pub fn new(
        spis: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        cs: impl Peripheral<P = impl GpioPin> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        miso: impl Peripheral<P = impl GpioPin> + 'd,
        mosi: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(cs, sck, miso, mosi);
        Self::new_inner(
            spis,
            irq,
            cs.map_into(),
            sck.map_into(),
            Some(miso.map_into()),
            Some(mosi.map_into()),
            config,
        )
    }

    pub fn new_txonly(
        spis: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        cs: impl Peripheral<P = impl GpioPin> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        miso: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(cs, sck, mosi);
        Self::new_inner(
            spis,
            irq,
            cs.map_into(),
            sck.map_into(),
            Some(miso.map_into()),
            None,
            config,
        )
    }

    pub fn new_rxonly(
        spis: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        cs: impl Peripheral<P = impl GpioPin> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        mosi: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(cs, sck, miso);
        Self::new_inner(
            spis,
            irq,
            cs.map_into(),
            sck.map_into(),
            None,
            Some(mosi.map_into()),
            config,
        )
    }

    fn new_inner(
        spis: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        cs: PeripheralRef<'d, AnyPin>,
        sck: PeripheralRef<'d, AnyPin>,
        miso: Option<PeripheralRef<'d, AnyPin>>,
        mosi: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        into_ref!(cs, spis, irq);

        let r = T::regs();

        // Configure pins.
        sck.conf().write(|w| w.input().connect().drive().h0h1());
        cs.conf().write(|w| w.input().connect().drive().h0h1());
        if let Some(mosi) = &mosi {
            mosi.conf().write(|w| w.input().connect().drive().h0h1());
        }
        if let Some(miso) = &miso {
            miso.conf().write(|w| w.dir().output().drive().h0h1());
        }

        match config.mode.polarity {
            Polarity::IdleHigh => {
                sck.set_high();
                if let Some(mosi) = &mosi {
                    mosi.set_high();
                }
            }
            Polarity::IdleLow => {
                sck.set_low();
                if let Some(mosi) = &mosi {
                    mosi.set_low();
                }
            }
        }

        // Select pins.
        r.psel.sck.write(|w| unsafe { w.bits(sck.psel_bits()) });
        r.psel.csn.write(|w| unsafe { w.bits(cs.psel_bits()) });
        r.psel.mosi.write(|w| unsafe { w.bits(mosi.psel_bits()) });
        r.psel.miso.write(|w| unsafe { w.bits(miso.psel_bits()) });

        // Enable SPIS instance.
        r.enable.write(|w| w.enable().enabled());

        // Configure mode.
        let mode = config.mode;
        r.config.write(|w| {
            match mode {
                MODE_0 => {
                    w.order().msb_first();
                    w.cpol().active_high();
                    w.cpha().leading();
                }
                MODE_1 => {
                    w.order().msb_first();
                    w.cpol().active_high();
                    w.cpha().trailing();
                }
                MODE_2 => {
                    w.order().msb_first();
                    w.cpol().active_low();
                    w.cpha().leading();
                }
                MODE_3 => {
                    w.order().msb_first();
                    w.cpol().active_low();
                    w.cpha().trailing();
                }
            }

            w
        });

        // Set over-read character.
        let orc = config.orc;
        r.orc.write(|w| unsafe { w.orc().bits(orc) });

        // Set default character.
        let def = config.def;
        r.def.write(|w| unsafe { w.def().bits(def) });

        // Configure auto-acquire on 'transfer end' event.
        if config.auto_acquire {
            r.shorts.write(|w| w.end_acquire().bit(true));
        }

        // Disable all events interrupts.
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self { _p: spis }
    }

    fn on_interrupt(_: *mut ()) {
        let r = T::regs();
        let s = T::state();

        if r.events_end.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.end().clear());
        }

        if r.events_acquired.read().bits() != 0 {
            s.acquire_waker.wake();
            r.intenclr.write(|w| w.acquired().clear());
        }
    }

    fn prepare(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
        slice_in_ram_or(tx, Error::DMABufferNotInDataMemory)?;
        // NOTE: RAM slice check for rx is not necessary, as a mutable
        // slice can only be built from data located in RAM.

        compiler_fence(Ordering::SeqCst);

        let r = T::regs();

        // Set up the DMA write.
        let (ptr, len) = slice_ptr_parts(tx);
        r.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as _) });
        r.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        // Set up the DMA read.
        let (ptr, len) = slice_ptr_parts_mut(rx);
        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as _) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        // Reset and enable the end event.
        r.events_end.reset();
        r.intenset.write(|w| w.end().set());

        // Release the semaphore.
        r.tasks_release.write(|w| unsafe { w.bits(1) });

        Ok(())
    }

    fn blocking_inner_from_ram(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(usize, usize), Error> {
        compiler_fence(Ordering::SeqCst);
        let r = T::regs();

        // Acquire semaphore.
        if r.semstat.read().bits() != 1 {
            r.events_acquired.reset();
            r.tasks_acquire.write(|w| unsafe { w.bits(1) });
            // Wait until CPU has acquired the semaphore.
            while r.semstat.read().bits() != 1 {}
        }

        self.prepare(rx, tx)?;

        // Wait for 'end' event.
        while r.events_end.read().bits() == 0 {}

        let n_rx = r.rxd.amount.read().bits() as usize;
        let n_tx = r.txd.amount.read().bits() as usize;

        compiler_fence(Ordering::SeqCst);

        Ok((n_rx, n_tx))
    }

    fn blocking_inner(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(usize, usize), Error> {
        match self.blocking_inner_from_ram(rx, tx) {
            Ok(n) => Ok(n),
            Err(Error::DMABufferNotInDataMemory) => {
                trace!("Copying SPIS tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..tx.len()];
                tx_ram_buf.copy_from_slice(tx);
                self.blocking_inner_from_ram(rx, tx_ram_buf)
            }
            Err(error) => Err(error),
        }
    }

    async fn async_inner_from_ram(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(usize, usize), Error> {
        let r = T::regs();
        let s = T::state();

        // Clear status register.
        r.status.write(|w| w.overflow().clear().overread().clear());

        // Acquire semaphore.
        if r.semstat.read().bits() != 1 {
            // Reset and enable the acquire event.
            r.events_acquired.reset();
            r.intenset.write(|w| w.acquired().set());

            // Request acquiring the SPIS semaphore.
            r.tasks_acquire.write(|w| unsafe { w.bits(1) });

            // Wait until CPU has acquired the semaphore.
            poll_fn(|cx| {
                s.acquire_waker.register(cx.waker());
                if r.semstat.read().bits() == 1 {
                    return Poll::Ready(());
                }
                Poll::Pending
            })
            .await;
        }

        self.prepare(rx, tx)?;

        // Wait for 'end' event.
        poll_fn(|cx| {
            s.end_waker.register(cx.waker());
            if r.events_end.read().bits() != 0 {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;

        let n_rx = r.rxd.amount.read().bits() as usize;
        let n_tx = r.txd.amount.read().bits() as usize;

        compiler_fence(Ordering::SeqCst);

        Ok((n_rx, n_tx))
    }

    async fn async_inner(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(usize, usize), Error> {
        match self.async_inner_from_ram(rx, tx).await {
            Ok(n) => Ok(n),
            Err(Error::DMABufferNotInDataMemory) => {
                trace!("Copying SPIS tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..tx.len()];
                tx_ram_buf.copy_from_slice(tx);
                self.async_inner_from_ram(rx, tx_ram_buf).await
            }
            Err(error) => Err(error),
        }
    }

    /// Reads data from the SPI bus without sending anything. Blocks until `cs` is deasserted.
    /// Returns number of bytes read.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        self.blocking_inner(data, &[]).map(|n| n.0)
    }

    /// Simultaneously sends and receives data. Blocks until the transmission is completed.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    /// Returns number of bytes transferred `(n_rx, n_tx)`.
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(usize, usize), Error> {
        self.blocking_inner(read, write)
    }

    /// Same as [`blocking_transfer`](Spis::blocking_transfer) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    /// Returns number of bytes transferred `(n_rx, n_tx)`.
    pub fn blocking_transfer_from_ram(&mut self, read: &mut [u8], write: &[u8]) -> Result<(usize, usize), Error> {
        self.blocking_inner_from_ram(read, write)
    }

    /// Simultaneously sends and receives data.
    /// Places the received data into the same buffer and blocks until the transmission is completed.
    /// Returns number of bytes transferred.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        self.blocking_inner_from_ram(data, data).map(|n| n.0)
    }

    /// Sends data, discarding any received data. Blocks  until the transmission is completed.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    /// Returns number of bytes written.
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.blocking_inner(&mut [], data).map(|n| n.1)
    }

    /// Same as [`blocking_write`](Spis::blocking_write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    /// Returns number of bytes written.
    pub fn blocking_write_from_ram(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.blocking_inner_from_ram(&mut [], data).map(|n| n.1)
    }

    /// Reads data from the SPI bus without sending anything.
    /// Returns number of bytes read.
    pub async fn read(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        self.async_inner(data, &[]).await.map(|n| n.0)
    }

    /// Simultaneously sends and receives data.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    /// Returns number of bytes transferred `(n_rx, n_tx)`.
    pub async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(usize, usize), Error> {
        self.async_inner(read, write).await
    }

    /// Same as [`transfer`](Spis::transfer) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    /// Returns number of bytes transferred `(n_rx, n_tx)`.
    pub async fn transfer_from_ram(&mut self, read: &mut [u8], write: &[u8]) -> Result<(usize, usize), Error> {
        self.async_inner_from_ram(read, write).await
    }

    /// Simultaneously sends and receives data. Places the received data into the same buffer.
    /// Returns number of bytes transferred.
    pub async fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        self.async_inner_from_ram(data, data).await.map(|n| n.0)
    }

    /// Sends data, discarding any received data.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    /// Returns number of bytes written.
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.async_inner(&mut [], data).await.map(|n| n.1)
    }

    /// Same as [`write`](Spis::write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    /// Returns number of bytes written.
    pub async fn write_from_ram(&mut self, data: &[u8]) -> Result<usize, Error> {
        self.async_inner_from_ram(&mut [], data).await.map(|n| n.1)
    }

    /// Checks if last transaction overread.
    pub fn is_overread(&mut self) -> bool {
        T::regs().status.read().overread().is_present()
    }

    /// Checks if last transaction overflowed.
    pub fn is_overflow(&mut self) -> bool {
        T::regs().status.read().overflow().is_present()
    }
}

impl<'d, T: Instance> Drop for Spis<'d, T> {
    fn drop(&mut self) {
        trace!("spis drop");

        // Disable
        let r = T::regs();
        r.enable.write(|w| w.enable().disabled());

        gpio::deconfigure_pin(r.psel.sck.read().bits());
        gpio::deconfigure_pin(r.psel.csn.read().bits());
        gpio::deconfigure_pin(r.psel.miso.read().bits());
        gpio::deconfigure_pin(r.psel.mosi.read().bits());

        trace!("spis drop: done");
    }
}

pub(crate) mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

    use super::*;

    pub struct State {
        pub end_waker: AtomicWaker,
        pub acquire_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                end_waker: AtomicWaker::new(),
                acquire_waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static pac::spis0::RegisterBlock;
        fn state() -> &'static State;
    }
}

pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_spis {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::spis::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::spis0::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::spis::sealed::State {
                static STATE: crate::spis::sealed::State = crate::spis::sealed::State::new();
                &STATE
            }
        }
        impl crate::spis::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}

// ====================

impl<'d, T: Instance> SetConfig for Spis<'d, T> {
    type Config = Config;
    fn set_config(&mut self, config: &Self::Config) {
        let r = T::regs();
        // Configure mode.
        let mode = config.mode;
        r.config.write(|w| {
            match mode {
                MODE_0 => {
                    w.order().msb_first();
                    w.cpol().active_high();
                    w.cpha().leading();
                }
                MODE_1 => {
                    w.order().msb_first();
                    w.cpol().active_high();
                    w.cpha().trailing();
                }
                MODE_2 => {
                    w.order().msb_first();
                    w.cpol().active_low();
                    w.cpha().leading();
                }
                MODE_3 => {
                    w.order().msb_first();
                    w.cpol().active_low();
                    w.cpha().trailing();
                }
            }

            w
        });

        // Set over-read character.
        let orc = config.orc;
        r.orc.write(|w| unsafe { w.orc().bits(orc) });

        // Set default character.
        let def = config.def;
        r.def.write(|w| unsafe { w.def().bits(def) });

        // Configure auto-acquire on 'transfer end' event.
        let auto_acquire = config.auto_acquire;
        r.shorts.write(|w| w.end_acquire().bit(auto_acquire));
    }
}
