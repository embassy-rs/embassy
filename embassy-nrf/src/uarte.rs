//! Universal Asynchronous Receiver Transmitter (UART) driver.
//!
//! The UART driver is provided in two flavors - this one and also [crate::buffered_uarte::BufferedUarte].
//! The [Uarte] here is useful for those use-cases where reading the UARTE peripheral is
//! exclusively awaited on. If the [Uarte] is required to be awaited on with some other future,
//! for example when using `futures_util::future::select`, then you should consider
//! [crate::buffered_uarte::BufferedUarte] so that reads may continue while processing these
//! other futures. If you do not then you may lose data between reads.
//!
//! An advantage of the [Uarte] has over [crate::buffered_uarte::BufferedUarte] is that less
//! memory may be used given that buffers are passed in directly to its read and write
//! methods.

#![macro_use]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, PeripheralRef};
use pac::uarte0::RegisterBlock;
// Re-export SVD variants to allow user to directly set values.
pub use pac::uarte0::{baudrate::BAUDRATE_A as Baudrate, config::PARITY_A as Parity};

use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::gpio::sealed::Pin as _;
use crate::gpio::{self, AnyPin, Pin as GpioPin, PselBits};
use crate::interrupt::typelevel::Interrupt;
use crate::ppi::{AnyConfigurableChannel, ConfigurableChannel, Event, Ppi, Task};
use crate::timer::{Frequency, Instance as TimerInstance, Timer};
use crate::util::slice_in_ram_or;
use crate::{interrupt, pac, Peripheral};

/// UARTE config.
#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    /// Parity bit.
    pub parity: Parity,
    /// Baud rate.
    pub baudrate: Baudrate,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            parity: Parity::EXCLUDED,
            baudrate: Baudrate::BAUD115200,
        }
    }
}

/// UART error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Buffer was too long.
    BufferTooLong,
    /// The buffer is not in data RAM. It's most likely in flash, and nRF's DMA cannot access flash.
    BufferNotInRAM,
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        let s = T::state();

        if r.events_endrx.read().bits() != 0 {
            s.endrx_waker.wake();
            r.intenclr.write(|w| w.endrx().clear());
        }
        if r.events_endtx.read().bits() != 0 {
            s.endtx_waker.wake();
            r.intenclr.write(|w| w.endtx().clear());
        }
    }
}

/// UARTE driver.
pub struct Uarte<'d, T: Instance> {
    tx: UarteTx<'d, T>,
    rx: UarteRx<'d, T>,
}

/// Transmitter part of the UARTE driver.
///
/// This can be obtained via [`Uarte::split`], or created directly.
pub struct UarteTx<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

/// Receiver part of the UARTE driver.
///
/// This can be obtained via [`Uarte::split`], or created directly.
pub struct UarteRx<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Uarte<'d, T> {
    /// Create a new UARTE without hardware flow control
    pub fn new(
        uarte: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rxd: impl Peripheral<P = impl GpioPin> + 'd,
        txd: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(rxd, txd);
        Self::new_inner(uarte, rxd.map_into(), txd.map_into(), None, None, config)
    }

    /// Create a new UARTE with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts(
        uarte: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rxd: impl Peripheral<P = impl GpioPin> + 'd,
        txd: impl Peripheral<P = impl GpioPin> + 'd,
        cts: impl Peripheral<P = impl GpioPin> + 'd,
        rts: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(rxd, txd, cts, rts);
        Self::new_inner(
            uarte,
            rxd.map_into(),
            txd.map_into(),
            Some(cts.map_into()),
            Some(rts.map_into()),
            config,
        )
    }

    fn new_inner(
        uarte: impl Peripheral<P = T> + 'd,
        rxd: PeripheralRef<'d, AnyPin>,
        txd: PeripheralRef<'d, AnyPin>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        into_ref!(uarte);

        let r = T::regs();

        rxd.conf().write(|w| w.input().connect().drive().h0h1());
        r.psel.rxd.write(|w| unsafe { w.bits(rxd.psel_bits()) });

        txd.set_high();
        txd.conf().write(|w| w.dir().output().drive().h0h1());
        r.psel.txd.write(|w| unsafe { w.bits(txd.psel_bits()) });

        if let Some(pin) = &cts {
            pin.conf().write(|w| w.input().connect().drive().h0h1());
        }
        r.psel.cts.write(|w| unsafe { w.bits(cts.psel_bits()) });

        if let Some(pin) = &rts {
            pin.set_high();
            pin.conf().write(|w| w.dir().output().drive().h0h1());
        }
        r.psel.rts.write(|w| unsafe { w.bits(rts.psel_bits()) });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let hardware_flow_control = match (rts.is_some(), cts.is_some()) {
            (false, false) => false,
            (true, true) => true,
            _ => panic!("RTS and CTS pins must be either both set or none set."),
        };
        configure(r, config, hardware_flow_control);

        let s = T::state();
        s.tx_rx_refcount.store(2, Ordering::Relaxed);

        Self {
            tx: UarteTx {
                _p: unsafe { uarte.clone_unchecked() },
            },
            rx: UarteRx { _p: uarte },
        }
    }

    /// Split the Uarte into the transmitter and receiver parts.
    ///
    /// This is useful to concurrently transmit and receive from independent tasks.
    pub fn split(self) -> (UarteTx<'d, T>, UarteRx<'d, T>) {
        (self.tx, self.rx)
    }

    /// Split the Uarte into the transmitter and receiver with idle support parts.
    ///
    /// This is useful to concurrently transmit and receive from independent tasks.
    pub fn split_with_idle<U: TimerInstance>(
        self,
        timer: impl Peripheral<P = U> + 'd,
        ppi_ch1: impl Peripheral<P = impl ConfigurableChannel + 'd> + 'd,
        ppi_ch2: impl Peripheral<P = impl ConfigurableChannel + 'd> + 'd,
    ) -> (UarteTx<'d, T>, UarteRxWithIdle<'d, T, U>) {
        (self.tx, self.rx.with_idle(timer, ppi_ch1, ppi_ch2))
    }

    /// Return the endtx event for use with PPI
    pub fn event_endtx(&self) -> Event {
        let r = T::regs();
        Event::from_reg(&r.events_endtx)
    }

    /// Read bytes until the buffer is filled.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }

    /// Write all bytes in the buffer.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    /// Same as [`write`](Uarte::write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn write_from_ram(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write_from_ram(buffer).await
    }

    /// Read bytes until the buffer is filled.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Write all bytes in the buffer.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    /// Same as [`blocking_write`](Uarte::blocking_write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_write_from_ram(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write_from_ram(buffer)
    }
}

fn configure(r: &RegisterBlock, config: Config, hardware_flow_control: bool) {
    r.config.write(|w| {
        w.hwfc().bit(hardware_flow_control);
        w.parity().variant(config.parity);
        w
    });
    r.baudrate.write(|w| w.baudrate().variant(config.baudrate));

    // Disable all interrupts
    r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

    // Reset rxstarted, txstarted. These are used by drop to know whether a transfer was
    // stopped midway or not.
    r.events_rxstarted.reset();
    r.events_txstarted.reset();

    // Enable
    apply_workaround_for_enable_anomaly(&r);
    r.enable.write(|w| w.enable().enabled());
}

impl<'d, T: Instance> UarteTx<'d, T> {
    /// Create a new tx-only UARTE without hardware flow control
    pub fn new(
        uarte: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        txd: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(txd);
        Self::new_inner(uarte, txd.map_into(), None, config)
    }

    /// Create a new tx-only UARTE with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts(
        uarte: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        txd: impl Peripheral<P = impl GpioPin> + 'd,
        cts: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(txd, cts);
        Self::new_inner(uarte, txd.map_into(), Some(cts.map_into()), config)
    }

    fn new_inner(
        uarte: impl Peripheral<P = T> + 'd,
        txd: PeripheralRef<'d, AnyPin>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        into_ref!(uarte);

        let r = T::regs();

        txd.set_high();
        txd.conf().write(|w| w.dir().output().drive().s0s1());
        r.psel.txd.write(|w| unsafe { w.bits(txd.psel_bits()) });

        if let Some(pin) = &cts {
            pin.conf().write(|w| w.input().connect().drive().h0h1());
        }
        r.psel.cts.write(|w| unsafe { w.bits(cts.psel_bits()) });

        r.psel.rxd.write(|w| w.connect().disconnected());
        r.psel.rts.write(|w| w.connect().disconnected());

        let hardware_flow_control = cts.is_some();
        configure(r, config, hardware_flow_control);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let s = T::state();
        s.tx_rx_refcount.store(1, Ordering::Relaxed);

        Self { _p: uarte }
    }

    /// Write all bytes in the buffer.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        match self.write_from_ram(buffer).await {
            Ok(_) => Ok(()),
            Err(Error::BufferNotInRAM) => {
                trace!("Copying UARTE tx buffer into RAM for DMA");
                let ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..buffer.len()];
                ram_buf.copy_from_slice(buffer);
                self.write_from_ram(&ram_buf).await
            }
            Err(error) => Err(error),
        }
    }

    /// Same as [`write`](Self::write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn write_from_ram(&mut self, buffer: &[u8]) -> Result<(), Error> {
        if buffer.len() == 0 {
            return Ok(());
        }

        slice_in_ram_or(buffer, Error::BufferNotInRAM)?;
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();
        let s = T::state();

        let drop = OnDrop::new(move || {
            trace!("write drop: stopping");

            r.intenclr.write(|w| w.endtx().clear());
            r.events_txstopped.reset();
            r.tasks_stoptx.write(|w| unsafe { w.bits(1) });

            // TX is stopped almost instantly, spinning is fine.
            while r.events_endtx.read().bits() == 0 {}
            trace!("write drop: stopped");
        });

        r.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endtx.reset();
        r.intenset.write(|w| w.endtx().set());

        compiler_fence(Ordering::SeqCst);

        trace!("starttx");
        r.tasks_starttx.write(|w| unsafe { w.bits(1) });

        poll_fn(|cx| {
            s.endtx_waker.register(cx.waker());
            if r.events_endtx.read().bits() != 0 {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        r.events_txstarted.reset();
        drop.defuse();

        Ok(())
    }

    /// Write all bytes in the buffer.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        match self.blocking_write_from_ram(buffer) {
            Ok(_) => Ok(()),
            Err(Error::BufferNotInRAM) => {
                trace!("Copying UARTE tx buffer into RAM for DMA");
                let ram_buf = &mut [0; FORCE_COPY_BUFFER_SIZE][..buffer.len()];
                ram_buf.copy_from_slice(buffer);
                self.blocking_write_from_ram(&ram_buf)
            }
            Err(error) => Err(error),
        }
    }

    /// Same as [`write_from_ram`](Self::write_from_ram) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_write_from_ram(&mut self, buffer: &[u8]) -> Result<(), Error> {
        if buffer.len() == 0 {
            return Ok(());
        }

        slice_in_ram_or(buffer, Error::BufferNotInRAM)?;
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();

        r.txd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.txd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endtx.reset();
        r.intenclr.write(|w| w.endtx().clear());

        compiler_fence(Ordering::SeqCst);

        trace!("starttx");
        r.tasks_starttx.write(|w| unsafe { w.bits(1) });

        while r.events_endtx.read().bits() == 0 {}

        compiler_fence(Ordering::SeqCst);
        r.events_txstarted.reset();

        Ok(())
    }
}

impl<'a, T: Instance> Drop for UarteTx<'a, T> {
    fn drop(&mut self) {
        trace!("uarte tx drop");

        let r = T::regs();

        let did_stoptx = r.events_txstarted.read().bits() != 0;
        trace!("did_stoptx {}", did_stoptx);

        // Wait for txstopped, if needed.
        while did_stoptx && r.events_txstopped.read().bits() == 0 {}

        let s = T::state();

        drop_tx_rx(&r, &s);
    }
}

impl<'d, T: Instance> UarteRx<'d, T> {
    /// Create a new rx-only UARTE without hardware flow control
    pub fn new(
        uarte: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rxd: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(rxd);
        Self::new_inner(uarte, rxd.map_into(), None, config)
    }

    /// Create a new rx-only UARTE with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts(
        uarte: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rxd: impl Peripheral<P = impl GpioPin> + 'd,
        rts: impl Peripheral<P = impl GpioPin> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(rxd, rts);
        Self::new_inner(uarte, rxd.map_into(), Some(rts.map_into()), config)
    }

    fn new_inner(
        uarte: impl Peripheral<P = T> + 'd,
        rxd: PeripheralRef<'d, AnyPin>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        into_ref!(uarte);

        let r = T::regs();

        rxd.conf().write(|w| w.input().connect().drive().h0h1());
        r.psel.rxd.write(|w| unsafe { w.bits(rxd.psel_bits()) });

        if let Some(pin) = &rts {
            pin.set_high();
            pin.conf().write(|w| w.dir().output().drive().h0h1());
        }
        r.psel.rts.write(|w| unsafe { w.bits(rts.psel_bits()) });

        r.psel.txd.write(|w| w.connect().disconnected());
        r.psel.cts.write(|w| w.connect().disconnected());

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let hardware_flow_control = rts.is_some();
        configure(r, config, hardware_flow_control);

        let s = T::state();
        s.tx_rx_refcount.store(1, Ordering::Relaxed);

        Self { _p: uarte }
    }

    /// Upgrade to an instance that supports idle line detection.
    pub fn with_idle<U: TimerInstance>(
        self,
        timer: impl Peripheral<P = U> + 'd,
        ppi_ch1: impl Peripheral<P = impl ConfigurableChannel + 'd> + 'd,
        ppi_ch2: impl Peripheral<P = impl ConfigurableChannel + 'd> + 'd,
    ) -> UarteRxWithIdle<'d, T, U> {
        let timer = Timer::new(timer);

        into_ref!(ppi_ch1, ppi_ch2);

        let r = T::regs();

        // BAUDRATE register values are `baudrate * 2^32 / 16000000`
        // source: https://devzone.nordicsemi.com/f/nordic-q-a/391/uart-baudrate-register-values
        //
        // We want to stop RX if line is idle for 2 bytes worth of time
        // That is 20 bits (each byte is 1 start bit + 8 data bits + 1 stop bit)
        // This gives us the amount of 16M ticks for 20 bits.
        let baudrate = r.baudrate.read().baudrate().variant().unwrap();
        let timeout = 0x8000_0000 / (baudrate as u32 / 40);

        timer.set_frequency(Frequency::F16MHz);
        timer.cc(0).write(timeout);
        timer.cc(0).short_compare_clear();
        timer.cc(0).short_compare_stop();

        let mut ppi_ch1 = Ppi::new_one_to_two(
            ppi_ch1.map_into(),
            Event::from_reg(&r.events_rxdrdy),
            timer.task_clear(),
            timer.task_start(),
        );
        ppi_ch1.enable();

        let mut ppi_ch2 = Ppi::new_one_to_one(
            ppi_ch2.map_into(),
            timer.cc(0).event_compare(),
            Task::from_reg(&r.tasks_stoprx),
        );
        ppi_ch2.enable();

        UarteRxWithIdle {
            rx: self,
            timer,
            ppi_ch1: ppi_ch1,
            _ppi_ch2: ppi_ch2,
        }
    }

    /// Read bytes until the buffer is filled.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.len() == 0 {
            return Ok(());
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();
        let s = T::state();

        let drop = OnDrop::new(move || {
            trace!("read drop: stopping");

            r.intenclr.write(|w| w.endrx().clear());
            r.events_rxto.reset();
            r.tasks_stoprx.write(|w| unsafe { w.bits(1) });

            while r.events_endrx.read().bits() == 0 {}

            trace!("read drop: stopped");
        });

        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endrx.reset();
        r.intenset.write(|w| w.endrx().set());

        compiler_fence(Ordering::SeqCst);

        trace!("startrx");
        r.tasks_startrx.write(|w| unsafe { w.bits(1) });

        poll_fn(|cx| {
            s.endrx_waker.register(cx.waker());
            if r.events_endrx.read().bits() != 0 {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        r.events_rxstarted.reset();
        drop.defuse();

        Ok(())
    }

    /// Read bytes until the buffer is filled.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.len() == 0 {
            return Ok(());
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();

        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endrx.reset();
        r.intenclr.write(|w| w.endrx().clear());

        compiler_fence(Ordering::SeqCst);

        trace!("startrx");
        r.tasks_startrx.write(|w| unsafe { w.bits(1) });

        while r.events_endrx.read().bits() == 0 {}

        compiler_fence(Ordering::SeqCst);
        r.events_rxstarted.reset();

        Ok(())
    }
}

impl<'a, T: Instance> Drop for UarteRx<'a, T> {
    fn drop(&mut self) {
        trace!("uarte rx drop");

        let r = T::regs();

        let did_stoprx = r.events_rxstarted.read().bits() != 0;
        trace!("did_stoprx {}", did_stoprx);

        // Wait for rxto, if needed.
        while did_stoprx && r.events_rxto.read().bits() == 0 {}

        let s = T::state();

        drop_tx_rx(&r, &s);
    }
}

/// Receiver part of the UARTE driver, with `read_until_idle` support.
///
/// This can be obtained via [`Uarte::split_with_idle`].
pub struct UarteRxWithIdle<'d, T: Instance, U: TimerInstance> {
    rx: UarteRx<'d, T>,
    timer: Timer<'d, U>,
    ppi_ch1: Ppi<'d, AnyConfigurableChannel, 1, 2>,
    _ppi_ch2: Ppi<'d, AnyConfigurableChannel, 1, 1>,
}

impl<'d, T: Instance, U: TimerInstance> UarteRxWithIdle<'d, T, U> {
    /// Read bytes until the buffer is filled.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.ppi_ch1.disable();
        self.rx.read(buffer).await
    }

    /// Read bytes until the buffer is filled.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.ppi_ch1.disable();
        self.rx.blocking_read(buffer)
    }

    /// Read bytes until the buffer is filled, or the line becomes idle.
    ///
    /// Returns the amount of bytes read.
    pub async fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.len() == 0 {
            return Ok(0);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();
        let s = T::state();

        self.ppi_ch1.enable();

        let drop = OnDrop::new(|| {
            self.timer.stop();

            r.intenclr.write(|w| w.endrx().clear());
            r.events_rxto.reset();
            r.tasks_stoprx.write(|w| unsafe { w.bits(1) });

            while r.events_endrx.read().bits() == 0 {}
        });

        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endrx.reset();
        r.intenset.write(|w| w.endrx().set());

        compiler_fence(Ordering::SeqCst);

        r.tasks_startrx.write(|w| unsafe { w.bits(1) });

        poll_fn(|cx| {
            s.endrx_waker.register(cx.waker());
            if r.events_endrx.read().bits() != 0 {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        let n = r.rxd.amount.read().amount().bits() as usize;

        self.timer.stop();
        r.events_rxstarted.reset();

        drop.defuse();

        Ok(n)
    }

    /// Read bytes until the buffer is filled, or the line becomes idle.
    ///
    /// Returns the amount of bytes read.
    pub fn blocking_read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.len() == 0 {
            return Ok(0);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();

        self.ppi_ch1.enable();

        r.rxd.ptr.write(|w| unsafe { w.ptr().bits(ptr as u32) });
        r.rxd.maxcnt.write(|w| unsafe { w.maxcnt().bits(len as _) });

        r.events_endrx.reset();
        r.intenclr.write(|w| w.endrx().clear());

        compiler_fence(Ordering::SeqCst);

        r.tasks_startrx.write(|w| unsafe { w.bits(1) });

        while r.events_endrx.read().bits() == 0 {}

        compiler_fence(Ordering::SeqCst);
        let n = r.rxd.amount.read().amount().bits() as usize;

        self.timer.stop();
        r.events_rxstarted.reset();

        Ok(n)
    }
}

#[cfg(not(any(feature = "_nrf9160", feature = "_nrf5340")))]
pub(crate) fn apply_workaround_for_enable_anomaly(_r: &crate::pac::uarte0::RegisterBlock) {
    // Do nothing
}

#[cfg(any(feature = "_nrf9160", feature = "_nrf5340"))]
pub(crate) fn apply_workaround_for_enable_anomaly(r: &crate::pac::uarte0::RegisterBlock) {
    // Apply workaround for anomalies:
    // - nRF9160 - anomaly 23
    // - nRF5340 - anomaly 44
    let rxenable_reg: *const u32 = ((r as *const _ as usize) + 0x564) as *const u32;
    let txenable_reg: *const u32 = ((r as *const _ as usize) + 0x568) as *const u32;

    // NB Safety: This is taken from Nordic's driver -
    // https://github.com/NordicSemiconductor/nrfx/blob/master/drivers/src/nrfx_uarte.c#L197
    if unsafe { core::ptr::read_volatile(txenable_reg) } == 1 {
        r.tasks_stoptx.write(|w| unsafe { w.bits(1) });
    }

    // NB Safety: This is taken from Nordic's driver -
    // https://github.com/NordicSemiconductor/nrfx/blob/master/drivers/src/nrfx_uarte.c#L197
    if unsafe { core::ptr::read_volatile(rxenable_reg) } == 1 {
        r.enable.write(|w| w.enable().enabled());
        r.tasks_stoprx.write(|w| unsafe { w.bits(1) });

        let mut workaround_succeded = false;
        // The UARTE is able to receive up to four bytes after the STOPRX task has been triggered.
        // On lowest supported baud rate (1200 baud), with parity bit and two stop bits configured
        // (resulting in 12 bits per data byte sent), this may take up to 40 ms.
        for _ in 0..40000 {
            // NB Safety: This is taken from Nordic's driver -
            // https://github.com/NordicSemiconductor/nrfx/blob/master/drivers/src/nrfx_uarte.c#L197
            if unsafe { core::ptr::read_volatile(rxenable_reg) } == 0 {
                workaround_succeded = true;
                break;
            } else {
                // Need to sleep for 1us here
            }
        }

        if !workaround_succeded {
            panic!("Failed to apply workaround for UART");
        }

        let errors = r.errorsrc.read().bits();
        // NB Safety: safe to write back the bits we just read to clear them
        r.errorsrc.write(|w| unsafe { w.bits(errors) });
        r.enable.write(|w| w.enable().disabled());
    }
}

pub(crate) fn drop_tx_rx(r: &pac::uarte0::RegisterBlock, s: &sealed::State) {
    if s.tx_rx_refcount.fetch_sub(1, Ordering::Relaxed) == 1 {
        // Finally we can disable, and we do so for the peripheral
        // i.e. not just rx concerns.
        r.enable.write(|w| w.enable().disabled());

        gpio::deconfigure_pin(r.psel.rxd.read().bits());
        gpio::deconfigure_pin(r.psel.txd.read().bits());
        gpio::deconfigure_pin(r.psel.rts.read().bits());
        gpio::deconfigure_pin(r.psel.cts.read().bits());

        trace!("uarte tx and rx drop: done");
    }
}

pub(crate) mod sealed {
    use core::sync::atomic::AtomicU8;

    use embassy_sync::waitqueue::AtomicWaker;

    use super::*;

    pub struct State {
        pub endrx_waker: AtomicWaker,
        pub endtx_waker: AtomicWaker,
        pub tx_rx_refcount: AtomicU8,
    }
    impl State {
        pub const fn new() -> Self {
            Self {
                endrx_waker: AtomicWaker::new(),
                endtx_waker: AtomicWaker::new(),
                tx_rx_refcount: AtomicU8::new(0),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static pac::uarte0::RegisterBlock;
        fn state() -> &'static State;
        fn buffered_state() -> &'static crate::buffered_uarte::State;
    }
}

/// UARTE peripheral instance.
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_uarte {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::uarte::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::uarte0::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::uarte::sealed::State {
                static STATE: crate::uarte::sealed::State = crate::uarte::sealed::State::new();
                &STATE
            }
            fn buffered_state() -> &'static crate::buffered_uarte::State {
                static STATE: crate::buffered_uarte::State = crate::buffered_uarte::State::new();
                &STATE
            }
        }
        impl crate::uarte::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

// ====================

mod eh02 {
    use super::*;

    impl<'d, T: Instance> embedded_hal_02::blocking::serial::Write<u8> for Uarte<'d, T> {
        type Error = Error;

        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn bflush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl<'d, T: Instance> embedded_hal_02::blocking::serial::Write<u8> for UarteTx<'d, T> {
        type Error = Error;

        fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer)
        }

        fn bflush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }
}
