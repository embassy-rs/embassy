//! Serial Peripheral Instance in master mode (SPIM) driver.

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
#[cfg(feature = "_nrf52832_anomaly_109")]
use core::sync::atomic::AtomicU8;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
pub use embedded_hal_02::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};
pub use pac::spim0::config::ORDER_A as BitOrder;
pub use pac::spim0::frequency::FREQUENCY_A as Frequency;

use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::gpio::{self, convert_drive, AnyPin, OutputDrive, Pin as GpioPin, PselBits, SealedPin as _};
use crate::interrupt::typelevel::Interrupt;
use crate::util::{slice_in_ram_or, slice_ptr_len, slice_ptr_parts, slice_ptr_parts_mut};
use crate::{interrupt, pac, Peripheral};

/// SPIM error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// EasyDMA can only read from data memory, read only buffers in flash will fail.
    BufferNotInRAM,
}

/// SPIM configuration.
#[non_exhaustive]
pub struct Config {
    /// Frequency
    pub frequency: Frequency,

    /// SPI mode
    pub mode: Mode,

    /// Bit order
    pub bit_order: BitOrder,

    /// Overread character.
    ///
    /// When doing bidirectional transfers, if the TX buffer is shorter than the RX buffer,
    /// this byte will be transmitted in the MOSI line for the left-over bytes.
    pub orc: u8,

    /// Drive strength for the SCK line.
    pub sck_drive: OutputDrive,

    /// Drive strength for the MOSI line.
    pub mosi_drive: OutputDrive,

    /// Drive strength for the MISO line.
    pub miso_drive: OutputDrive,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::M1,
            mode: MODE_0,
            bit_order: BitOrder::MSB_FIRST,
            orc: 0x00,
            sck_drive: OutputDrive::HighDrive,
            mosi_drive: OutputDrive::HighDrive,
            miso_drive: OutputDrive::HighDrive,
        }
    }
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        let s = T::state();

        #[cfg(feature = "_nrf52832_anomaly_109")]
        {
            // Ideally we should call this only during the first chunk transfer,
            // but so far calling this every time doesn't seem to be causing any issues.
            if r.events_started.read().bits() != 0 {
                s.waker.wake();
                r.intenclr.write(|w| w.started().clear());
            }
        }

        if r.events_end.read().bits() != 0 {
            s.waker.wake();
            r.intenclr.write(|w| w.end().clear());
        }
    }
}

/// SPIM driver.
pub struct Spim<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Spim<'d, T> {
    /// Create a new SPIM driver.
    pub fn new(
        spim: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        miso: impl Peripheral<P = impl GpioPin> + 'd,
        mosi: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(sck, miso, mosi);
        Self::new_inner(
            spim,
            Some(sck.map_into()),
            Some(miso.map_into()),
            Some(mosi.map_into()),
            config,
        )
    }

    /// Create a new SPIM driver, capable of TX only (MOSI only).
    pub fn new_txonly(
        spim: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        mosi: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(sck, mosi);
        Self::new_inner(spim, Some(sck.map_into()), None, Some(mosi.map_into()), config)
    }

    /// Create a new SPIM driver, capable of RX only (MISO only).
    pub fn new_rxonly(
        spim: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        sck: impl Peripheral<P = impl GpioPin> + 'd,
        miso: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(sck, miso);
        Self::new_inner(spim, Some(sck.map_into()), Some(miso.map_into()), None, config)
    }

    /// Create a new SPIM driver, capable of TX only (MOSI only), without SCK pin.
    pub fn new_txonly_nosck(
        spim: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        mosi: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(mosi);
        Self::new_inner(spim, None, None, Some(mosi.map_into()), config)
    }

    fn new_inner(
        spim: impl Peripheral<P = T> + 'd,
        sck: Option<PeripheralRef<'d, AnyPin>>,
        miso: Option<PeripheralRef<'d, AnyPin>>,
        mosi: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        into_ref!(spim);

        let r = T::regs();

        // Configure pins
        if let Some(sck) = &sck {
            sck.conf()
                .write(|w| w.dir().output().drive().variant(convert_drive(config.sck_drive)));
        }
        if let Some(mosi) = &mosi {
            mosi.conf()
                .write(|w| w.dir().output().drive().variant(convert_drive(config.mosi_drive)));
        }
        if let Some(miso) = &miso {
            miso.conf()
                .write(|w| w.input().connect().drive().variant(convert_drive(config.miso_drive)));
        }

        match config.mode.polarity {
            Polarity::IdleHigh => {
                if let Some(sck) = &sck {
                    sck.set_high();
                }
                if let Some(mosi) = &mosi {
                    mosi.set_high();
                }
            }
            Polarity::IdleLow => {
                if let Some(sck) = &sck {
                    sck.set_low();
                }
                if let Some(mosi) = &mosi {
                    mosi.set_low();
                }
            }
        }

        // Select pins.
        r.psel.sck.write(|w| unsafe { w.bits(sck.psel_bits()) });
        r.psel.mosi.write(|w| unsafe { w.bits(mosi.psel_bits()) });
        r.psel.miso.write(|w| unsafe { w.bits(miso.psel_bits()) });

        // Enable SPIM instance.
        r.enable.write(|w| w.enable().enabled());

        let mut spim = Self { _p: spim };

        // Apply runtime peripheral configuration
        Self::set_config(&mut spim, &config).unwrap();

        // Disable all events interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        spim
    }

    fn prepare_dma_transfer(&mut self, rx: *mut [u8], tx: *const [u8], offset: usize, length: usize) {
        compiler_fence(Ordering::SeqCst);

        let r = T::regs();

        fn xfer_params(ptr: u32, total: usize, offset: usize, length: usize) -> (u32, usize) {
            if total > offset {
                (ptr.wrapping_add(offset as _), core::cmp::min(total - offset, length))
            } else {
                (ptr, 0)
            }
        }

        // Set up the DMA read.
        let (ptr, len) = slice_ptr_parts_mut(rx);
        let (rx_ptr, rx_len) = xfer_params(ptr as _, len as _, offset, length);
        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(rx_ptr) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(rx_len as _) });

        // Set up the DMA write.
        let (ptr, len) = slice_ptr_parts(tx);
        let (tx_ptr, tx_len) = xfer_params(ptr as _, len as _, offset, length);
        r.txd.ptr.write(|w| unsafe { w.ptr().bits(tx_ptr) });
        r.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(tx_len as _) });

        /*
        trace!("XFER: offset: {}, length: {}", offset, length);
        trace!("RX(len: {}, ptr: {=u32:02x})", rx_len, rx_ptr as u32);
        trace!("TX(len: {}, ptr: {=u32:02x})", tx_len, tx_ptr as u32);
        */

        #[cfg(feature = "_nrf52832_anomaly_109")]
        if offset == 0 {
            let s = T::state();

            r.events_started.reset();

            // Set rx/tx buffer lengths to 0...
            r.txd.maxcnt.reset();
            r.rxd.maxcnt.reset();

            // ...and keep track of original buffer lengths...
            s.tx.store(tx_len as _, Ordering::Relaxed);
            s.rx.store(rx_len as _, Ordering::Relaxed);

            // ...signalling the start of the fake transfer.
            r.intenset.write(|w| w.started().bit(true));
        }

        // Reset and enable the event
        r.events_end.reset();
        r.intenset.write(|w| w.end().set());

        // Start SPI transaction.
        r.tasks_start.write(|w| unsafe { w.bits(1) });
    }

    fn blocking_inner_from_ram_chunk(&mut self, rx: *mut [u8], tx: *const [u8], offset: usize, length: usize) {
        self.prepare_dma_transfer(rx, tx, offset, length);

        #[cfg(feature = "_nrf52832_anomaly_109")]
        if offset == 0 {
            while self.nrf52832_dma_workaround_status().is_pending() {}
        }

        // Wait for 'end' event.
        while T::regs().events_end.read().bits() == 0 {}

        compiler_fence(Ordering::SeqCst);
    }

    fn blocking_inner_from_ram(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
        slice_in_ram_or(tx, Error::BufferNotInRAM)?;
        // NOTE: RAM slice check for rx is not necessary, as a mutable
        // slice can only be built from data located in RAM.

        let xfer_len = core::cmp::max(slice_ptr_len(rx), slice_ptr_len(tx));
        for offset in (0..xfer_len).step_by(EASY_DMA_SIZE) {
            let length = core::cmp::min(xfer_len - offset, EASY_DMA_SIZE);
            self.blocking_inner_from_ram_chunk(rx, tx, offset, length);
        }
        Ok(())
    }

    fn blocking_inner(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Error> {
        match self.blocking_inner_from_ram(rx, tx) {
            Ok(_) => Ok(()),
            Err(Error::BufferNotInRAM) => {
                // trace!("Copying SPIM tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..tx.len()];
                tx_ram_buf.copy_from_slice(tx);
                self.blocking_inner_from_ram(rx, tx_ram_buf)
            }
        }
    }

    async fn async_inner_from_ram_chunk(&mut self, rx: *mut [u8], tx: *const [u8], offset: usize, length: usize) {
        self.prepare_dma_transfer(rx, tx, offset, length);

        #[cfg(feature = "_nrf52832_anomaly_109")]
        if offset == 0 {
            poll_fn(|cx| {
                let s = T::state();

                s.waker.register(cx.waker());

                self.nrf52832_dma_workaround_status()
            })
            .await;
        }

        // Wait for 'end' event.
        poll_fn(|cx| {
            T::state().waker.register(cx.waker());
            if T::regs().events_end.read().bits() != 0 {
                return Poll::Ready(());
            }

            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
    }

    async fn async_inner_from_ram(&mut self, rx: *mut [u8], tx: *const [u8]) -> Result<(), Error> {
        slice_in_ram_or(tx, Error::BufferNotInRAM)?;
        // NOTE: RAM slice check for rx is not necessary, as a mutable
        // slice can only be built from data located in RAM.

        let xfer_len = core::cmp::max(slice_ptr_len(rx), slice_ptr_len(tx));
        for offset in (0..xfer_len).step_by(EASY_DMA_SIZE) {
            let length = core::cmp::min(xfer_len - offset, EASY_DMA_SIZE);
            self.async_inner_from_ram_chunk(rx, tx, offset, length).await;
        }
        Ok(())
    }

    async fn async_inner(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Error> {
        match self.async_inner_from_ram(rx, tx).await {
            Ok(_) => Ok(()),
            Err(Error::BufferNotInRAM) => {
                // trace!("Copying SPIM tx buffer into RAM for DMA");
                let tx_ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..tx.len()];
                tx_ram_buf.copy_from_slice(tx);
                self.async_inner_from_ram(rx, tx_ram_buf).await
            }
        }
    }

    /// Reads data from the SPI bus without sending anything. Blocks until the buffer has been filled.
    pub fn blocking_read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.blocking_inner(data, &[])
    }

    /// Simultaneously sends and receives data. Blocks until the transmission is completed.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    pub fn blocking_transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.blocking_inner(read, write)
    }

    /// Same as [`blocking_transfer`](Spim::blocking_transfer) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_transfer_from_ram(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.blocking_inner(read, write)
    }

    /// Simultaneously sends and receives data.
    /// Places the received data into the same buffer and blocks until the transmission is completed.
    pub fn blocking_transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.blocking_inner_from_ram(data, data)
    }

    /// Sends data, discarding any received data. Blocks  until the transmission is completed.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    pub fn blocking_write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.blocking_inner(&mut [], data)
    }

    /// Same as [`blocking_write`](Spim::blocking_write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_write_from_ram(&mut self, data: &[u8]) -> Result<(), Error> {
        self.blocking_inner(&mut [], data)
    }

    /// Reads data from the SPI bus without sending anything.
    pub async fn read(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.async_inner(data, &[]).await
    }

    /// Simultaneously sends and receives data.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    pub async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.async_inner(read, write).await
    }

    /// Same as [`transfer`](Spim::transfer) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn transfer_from_ram(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        self.async_inner_from_ram(read, write).await
    }

    /// Simultaneously sends and receives data. Places the received data into the same buffer.
    pub async fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.async_inner_from_ram(data, data).await
    }

    /// Sends data, discarding any received data.
    /// If necessary, the write buffer will be copied into RAM (see struct description for detail).
    pub async fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.async_inner(&mut [], data).await
    }

    /// Same as [`write`](Spim::write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn write_from_ram(&mut self, data: &[u8]) -> Result<(), Error> {
        self.async_inner_from_ram(&mut [], data).await
    }

    #[cfg(feature = "_nrf52832_anomaly_109")]
    fn nrf52832_dma_workaround_status(&mut self) -> Poll<()> {
        let r = T::regs();
        if r.events_started.read().bits() != 0 {
            let s = T::state();

            // Handle the first "fake" transmission
            r.events_started.reset();
            r.events_end.reset();

            // Update DMA registers with correct rx/tx buffer sizes
            r.rxd
                .maxcnt
                .write(|w| unsafe { w.maxcnt().bits(s.rx.load(Ordering::Relaxed)) });
            r.txd
                .maxcnt
                .write(|w| unsafe { w.maxcnt().bits(s.tx.load(Ordering::Relaxed)) });

            r.intenset.write(|w| w.end().set());
            // ... and start actual, hopefully glitch-free transmission
            r.tasks_start.write(|w| unsafe { w.bits(1) });
            return Poll::Ready(());
        }
        Poll::Pending
    }
}

impl<'d, T: Instance> Drop for Spim<'d, T> {
    fn drop(&mut self) {
        trace!("spim drop");

        // TODO check for abort, wait for xxxstopped

        // disable!
        let r = T::regs();
        r.enable.write(|w| w.enable().disabled());

        gpio::deconfigure_pin(r.psel.sck.read().bits());
        gpio::deconfigure_pin(r.psel.miso.read().bits());
        gpio::deconfigure_pin(r.psel.mosi.read().bits());

        // Disable all events interrupts
        T::Interrupt::disable();

        trace!("spim drop: done");
    }
}

pub(crate) struct State {
    waker: AtomicWaker,
    #[cfg(feature = "_nrf52832_anomaly_109")]
    rx: AtomicU8,
    #[cfg(feature = "_nrf52832_anomaly_109")]
    tx: AtomicU8,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            #[cfg(feature = "_nrf52832_anomaly_109")]
            rx: AtomicU8::new(0),
            #[cfg(feature = "_nrf52832_anomaly_109")]
            tx: AtomicU8::new(0),
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> &'static pac::spim0::RegisterBlock;
    fn state() -> &'static State;
}

/// SPIM peripheral instance
#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + SealedInstance + 'static {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_spim {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::spim::SealedInstance for peripherals::$type {
            fn regs() -> &'static pac::spim0::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::spim::State {
                static STATE: crate::spim::State = crate::spim::State::new();
                &STATE
            }
        }
        impl crate::spim::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

// ====================

mod eh02 {
    use super::*;

    impl<'d, T: Instance> embedded_hal_02::blocking::spi::Transfer<u8> for Spim<'d, T> {
        type Error = Error;
        fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
            self.blocking_transfer_in_place(words)?;
            Ok(words)
        }
    }

    impl<'d, T: Instance> embedded_hal_02::blocking::spi::Write<u8> for Spim<'d, T> {
        type Error = Error;

        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(words)
        }
    }
}

impl embedded_hal_1::spi::Error for Error {
    fn kind(&self) -> embedded_hal_1::spi::ErrorKind {
        match *self {
            Self::BufferNotInRAM => embedded_hal_1::spi::ErrorKind::Other,
        }
    }
}

impl<'d, T: Instance> embedded_hal_1::spi::ErrorType for Spim<'d, T> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_hal_1::spi::SpiBus<u8> for Spim<'d, T> {
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_transfer(words, &[])
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(words)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_transfer(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_transfer_in_place(words)
    }
}

impl<'d, T: Instance> embedded_hal_async::spi::SpiBus<u8> for Spim<'d, T> {
    async fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }

    async fn read(&mut self, words: &mut [u8]) -> Result<(), Error> {
        self.read(words).await
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.write(data).await
    }

    async fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Error> {
        self.transfer(rx, tx).await
    }

    async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Error> {
        self.transfer_in_place(words).await
    }
}

impl<'d, T: Instance> SetConfig for Spim<'d, T> {
    type Config = Config;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        let r = T::regs();
        // Configure mode.
        let mode = config.mode;
        r.config.write(|w| {
            match mode {
                MODE_0 => {
                    w.order().variant(config.bit_order);
                    w.cpol().active_high();
                    w.cpha().leading();
                }
                MODE_1 => {
                    w.order().variant(config.bit_order);
                    w.cpol().active_high();
                    w.cpha().trailing();
                }
                MODE_2 => {
                    w.order().variant(config.bit_order);
                    w.cpol().active_low();
                    w.cpha().leading();
                }
                MODE_3 => {
                    w.order().variant(config.bit_order);
                    w.cpol().active_low();
                    w.cpha().trailing();
                }
            }

            w
        });

        // Configure frequency.
        let frequency = config.frequency;
        r.frequency.write(|w| w.frequency().variant(frequency));

        // Set over-read character
        let orc = config.orc;
        r.orc.write(|w| unsafe { w.orc().bits(orc) });

        Ok(())
    }
}
