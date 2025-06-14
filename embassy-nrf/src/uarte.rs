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
use core::sync::atomic::{compiler_fence, AtomicU8, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
// Re-export SVD variants to allow user to directly set values.
pub use pac::uarte::vals::{Baudrate, ConfigParity as Parity};

use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::gpio::{self, AnyPin, Pin as GpioPin, PselBits, SealedPin as _, DISCONNECTED};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::gpio::vals as gpiovals;
use crate::pac::uarte::vals;
use crate::ppi::{AnyConfigurableChannel, ConfigurableChannel, Event, Ppi, Task};
use crate::timer::{Frequency, Instance as TimerInstance, Timer};
use crate::util::slice_in_ram_or;
use crate::{interrupt, pac};

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

bitflags::bitflags! {
    /// Error source flags
    pub(crate) struct ErrorSource: u32 {
        /// Buffer overrun
        const OVERRUN = 0x01;
        /// Parity error
        const PARITY = 0x02;
        /// Framing error
        const FRAMING = 0x04;
        /// Break condition
        const BREAK = 0x08;
    }
}

impl ErrorSource {
    #[inline]
    fn check(self) -> Result<(), Error> {
        if self.contains(ErrorSource::OVERRUN) {
            Err(Error::Overrun)
        } else if self.contains(ErrorSource::PARITY) {
            Err(Error::Parity)
        } else if self.contains(ErrorSource::FRAMING) {
            Err(Error::Framing)
        } else if self.contains(ErrorSource::BREAK) {
            Err(Error::Break)
        } else {
            Ok(())
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
    /// Framing Error
    Framing,
    /// Parity Error
    Parity,
    /// Buffer Overrun
    Overrun,
    /// Break condition
    Break,
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        let s = T::state();

        let endrx = r.events_endrx().read();
        let error = r.events_error().read();
        if endrx != 0 || error != 0 {
            s.rx_waker.wake();
            if endrx != 0 {
                r.intenclr().write(|w| w.set_endrx(true));
            }
            if error != 0 {
                r.intenclr().write(|w| w.set_error(true));
            }
        }
        if r.events_endtx().read() != 0 {
            s.tx_waker.wake();
            r.intenclr().write(|w| w.set_endtx(true));
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
    _p: Peri<'d, T>,
}

/// Receiver part of the UARTE driver.
///
/// This can be obtained via [`Uarte::split`], or created directly.
pub struct UarteRx<'d, T: Instance> {
    _p: Peri<'d, T>,
}

impl<'d, T: Instance> Uarte<'d, T> {
    /// Create a new UARTE without hardware flow control
    pub fn new(
        uarte: Peri<'d, T>,
        rxd: Peri<'d, impl GpioPin>,
        txd: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(uarte, rxd.into(), txd.into(), None, None, config)
    }

    /// Create a new UARTE with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts(
        uarte: Peri<'d, T>,
        rxd: Peri<'d, impl GpioPin>,
        txd: Peri<'d, impl GpioPin>,
        cts: Peri<'d, impl GpioPin>,
        rts: Peri<'d, impl GpioPin>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Self {
        Self::new_inner(
            uarte,
            rxd.into(),
            txd.into(),
            Some(cts.into()),
            Some(rts.into()),
            config,
        )
    }

    fn new_inner(
        uarte: Peri<'d, T>,
        rxd: Peri<'d, AnyPin>,
        txd: Peri<'d, AnyPin>,
        cts: Option<Peri<'d, AnyPin>>,
        rts: Option<Peri<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        let r = T::regs();

        let hardware_flow_control = match (rts.is_some(), cts.is_some()) {
            (false, false) => false,
            (true, true) => true,
            _ => panic!("RTS and CTS pins must be either both set or none set."),
        };
        configure(r, config, hardware_flow_control);
        configure_rx_pins(r, rxd, rts);
        configure_tx_pins(r, txd, cts);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        r.enable().write(|w| w.set_enable(vals::Enable::ENABLED));

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

    /// Split the UART in reader and writer parts, by reference.
    ///
    /// The returned halves borrow from `self`, so you can drop them and go back to using
    /// the "un-split" `self`. This allows temporarily splitting the UART.
    pub fn split_by_ref(&mut self) -> (&mut UarteTx<'d, T>, &mut UarteRx<'d, T>) {
        (&mut self.tx, &mut self.rx)
    }

    /// Split the Uarte into the transmitter and receiver with idle support parts.
    ///
    /// This is useful to concurrently transmit and receive from independent tasks.
    pub fn split_with_idle<U: TimerInstance>(
        self,
        timer: Peri<'d, U>,
        ppi_ch1: Peri<'d, impl ConfigurableChannel + 'd>,
        ppi_ch2: Peri<'d, impl ConfigurableChannel + 'd>,
    ) -> (UarteTx<'d, T>, UarteRxWithIdle<'d, T, U>) {
        (self.tx, self.rx.with_idle(timer, ppi_ch1, ppi_ch2))
    }

    /// Return the endtx event for use with PPI
    pub fn event_endtx(&self) -> Event {
        let r = T::regs();
        Event::from_reg(r.events_endtx())
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

pub(crate) fn configure_tx_pins(r: pac::uarte::Uarte, txd: Peri<'_, AnyPin>, cts: Option<Peri<'_, AnyPin>>) {
    txd.set_high();
    txd.conf().write(|w| {
        w.set_dir(gpiovals::Dir::OUTPUT);
        w.set_input(gpiovals::Input::DISCONNECT);
        w.set_drive(gpiovals::Drive::H0H1);
    });
    r.psel().txd().write_value(txd.psel_bits());

    if let Some(pin) = &cts {
        pin.conf().write(|w| {
            w.set_dir(gpiovals::Dir::INPUT);
            w.set_input(gpiovals::Input::CONNECT);
            w.set_drive(gpiovals::Drive::H0H1);
        });
    }
    r.psel().cts().write_value(cts.psel_bits());
}

pub(crate) fn configure_rx_pins(r: pac::uarte::Uarte, rxd: Peri<'_, AnyPin>, rts: Option<Peri<'_, AnyPin>>) {
    rxd.conf().write(|w| {
        w.set_dir(gpiovals::Dir::INPUT);
        w.set_input(gpiovals::Input::CONNECT);
        w.set_drive(gpiovals::Drive::H0H1);
    });
    r.psel().rxd().write_value(rxd.psel_bits());

    if let Some(pin) = &rts {
        pin.set_high();
        pin.conf().write(|w| {
            w.set_dir(gpiovals::Dir::OUTPUT);
            w.set_input(gpiovals::Input::DISCONNECT);
            w.set_drive(gpiovals::Drive::H0H1);
        });
    }
    r.psel().rts().write_value(rts.psel_bits());
}

pub(crate) fn configure(r: pac::uarte::Uarte, config: Config, hardware_flow_control: bool) {
    r.config().write(|w| {
        w.set_hwfc(hardware_flow_control);
        w.set_parity(config.parity);
    });
    r.baudrate().write(|w| w.set_baudrate(config.baudrate));

    // Disable all interrupts
    r.intenclr().write(|w| w.0 = 0xFFFF_FFFF);

    // Reset rxstarted, txstarted. These are used by drop to know whether a transfer was
    // stopped midway or not.
    r.events_rxstarted().write_value(0);
    r.events_txstarted().write_value(0);

    // reset all pins
    r.psel().txd().write_value(DISCONNECTED);
    r.psel().rxd().write_value(DISCONNECTED);
    r.psel().cts().write_value(DISCONNECTED);
    r.psel().rts().write_value(DISCONNECTED);

    apply_workaround_for_enable_anomaly(r);
}

impl<'d, T: Instance> UarteTx<'d, T> {
    /// Create a new tx-only UARTE without hardware flow control
    pub fn new(
        uarte: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        txd: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(uarte, txd.into(), None, config)
    }

    /// Create a new tx-only UARTE with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts(
        uarte: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        txd: Peri<'d, impl GpioPin>,
        cts: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(uarte, txd.into(), Some(cts.into()), config)
    }

    fn new_inner(uarte: Peri<'d, T>, txd: Peri<'d, AnyPin>, cts: Option<Peri<'d, AnyPin>>, config: Config) -> Self {
        let r = T::regs();

        configure(r, config, cts.is_some());
        configure_tx_pins(r, txd, cts);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        r.enable().write(|w| w.set_enable(vals::Enable::ENABLED));

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
                self.write_from_ram(ram_buf).await
            }
            Err(error) => Err(error),
        }
    }

    /// Same as [`write`](Self::write) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub async fn write_from_ram(&mut self, buffer: &[u8]) -> Result<(), Error> {
        if buffer.is_empty() {
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

            r.intenclr().write(|w| w.set_endtx(true));
            r.events_txstopped().write_value(0);
            r.tasks_stoptx().write_value(1);

            // TX is stopped almost instantly, spinning is fine.
            while r.events_endtx().read() == 0 {}
            trace!("write drop: stopped");
        });

        r.txd().ptr().write_value(ptr as u32);
        r.txd().maxcnt().write(|w| w.set_maxcnt(len as _));

        r.events_endtx().write_value(0);
        r.intenset().write(|w| w.set_endtx(true));

        compiler_fence(Ordering::SeqCst);

        trace!("starttx");
        r.tasks_starttx().write_value(1);

        poll_fn(|cx| {
            s.tx_waker.register(cx.waker());
            if r.events_endtx().read() != 0 {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        r.events_txstarted().write_value(0);
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
                self.blocking_write_from_ram(ram_buf)
            }
            Err(error) => Err(error),
        }
    }

    /// Same as [`write_from_ram`](Self::write_from_ram) but will fail instead of copying data into RAM. Consult the module level documentation to learn more.
    pub fn blocking_write_from_ram(&mut self, buffer: &[u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Ok(());
        }

        slice_in_ram_or(buffer, Error::BufferNotInRAM)?;
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();

        r.txd().ptr().write_value(ptr as u32);
        r.txd().maxcnt().write(|w| w.set_maxcnt(len as _));

        r.events_endtx().write_value(0);
        r.intenclr().write(|w| w.set_endtx(true));

        compiler_fence(Ordering::SeqCst);

        trace!("starttx");
        r.tasks_starttx().write_value(1);

        while r.events_endtx().read() == 0 {}

        compiler_fence(Ordering::SeqCst);
        r.events_txstarted().write_value(0);

        Ok(())
    }
}

impl<'a, T: Instance> Drop for UarteTx<'a, T> {
    fn drop(&mut self) {
        trace!("uarte tx drop");

        let r = T::regs();

        let did_stoptx = r.events_txstarted().read() != 0;
        trace!("did_stoptx {}", did_stoptx);

        // Wait for txstopped, if needed.
        while did_stoptx && r.events_txstopped().read() == 0 {}

        let s = T::state();

        drop_tx_rx(r, s);
    }
}

impl<'d, T: Instance> UarteRx<'d, T> {
    /// Create a new rx-only UARTE without hardware flow control
    pub fn new(
        uarte: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rxd: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(uarte, rxd.into(), None, config)
    }

    /// Create a new rx-only UARTE with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts(
        uarte: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rxd: Peri<'d, impl GpioPin>,
        rts: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Self {
        Self::new_inner(uarte, rxd.into(), Some(rts.into()), config)
    }

    /// Check for errors and clear the error register if an error occured.
    fn check_and_clear_errors(&mut self) -> Result<(), Error> {
        let r = T::regs();
        let err_bits = r.errorsrc().read();
        r.errorsrc().write_value(err_bits);
        ErrorSource::from_bits_truncate(err_bits.0).check()
    }

    fn new_inner(uarte: Peri<'d, T>, rxd: Peri<'d, AnyPin>, rts: Option<Peri<'d, AnyPin>>, config: Config) -> Self {
        let r = T::regs();

        configure(r, config, rts.is_some());
        configure_rx_pins(r, rxd, rts);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        r.enable().write(|w| w.set_enable(vals::Enable::ENABLED));

        let s = T::state();
        s.tx_rx_refcount.store(1, Ordering::Relaxed);

        Self { _p: uarte }
    }

    /// Upgrade to an instance that supports idle line detection.
    pub fn with_idle<U: TimerInstance>(
        self,
        timer: Peri<'d, U>,
        ppi_ch1: Peri<'d, impl ConfigurableChannel + 'd>,
        ppi_ch2: Peri<'d, impl ConfigurableChannel + 'd>,
    ) -> UarteRxWithIdle<'d, T, U> {
        let timer = Timer::new(timer);

        let r = T::regs();

        // BAUDRATE register values are `baudrate * 2^32 / 16000000`
        // source: https://devzone.nordicsemi.com/f/nordic-q-a/391/uart-baudrate-register-values
        //
        // We want to stop RX if line is idle for 2 bytes worth of time
        // That is 20 bits (each byte is 1 start bit + 8 data bits + 1 stop bit)
        // This gives us the amount of 16M ticks for 20 bits.
        let baudrate = r.baudrate().read().baudrate();
        let timeout = 0x8000_0000 / (baudrate.to_bits() / 40);

        timer.set_frequency(Frequency::F16MHz);
        timer.cc(0).write(timeout);
        timer.cc(0).short_compare_clear();
        timer.cc(0).short_compare_stop();

        let mut ppi_ch1 = Ppi::new_one_to_two(
            ppi_ch1.into(),
            Event::from_reg(r.events_rxdrdy()),
            timer.task_clear(),
            timer.task_start(),
        );
        ppi_ch1.enable();

        let mut ppi_ch2 = Ppi::new_one_to_one(
            ppi_ch2.into(),
            timer.cc(0).event_compare(),
            Task::from_reg(r.tasks_stoprx()),
        );
        ppi_ch2.enable();

        UarteRxWithIdle {
            rx: self,
            timer,
            ppi_ch1,
            _ppi_ch2: ppi_ch2,
        }
    }

    /// Read bytes until the buffer is filled.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.is_empty() {
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

            r.intenclr().write(|w| {
                w.set_endrx(true);
                w.set_error(true);
            });
            r.events_rxto().write_value(0);
            r.events_error().write_value(0);
            r.tasks_stoprx().write_value(1);

            while r.events_endrx().read() == 0 {}

            trace!("read drop: stopped");
        });

        r.rxd().ptr().write_value(ptr as u32);
        r.rxd().maxcnt().write(|w| w.set_maxcnt(len as _));

        r.events_endrx().write_value(0);
        r.events_error().write_value(0);
        r.intenset().write(|w| {
            w.set_endrx(true);
            w.set_error(true);
        });

        compiler_fence(Ordering::SeqCst);

        trace!("startrx");
        r.tasks_startrx().write_value(1);

        let result = poll_fn(|cx| {
            s.rx_waker.register(cx.waker());

            if let Err(e) = self.check_and_clear_errors() {
                r.tasks_stoprx().write_value(1);
                return Poll::Ready(Err(e));
            }
            if r.events_endrx().read() != 0 {
                return Poll::Ready(Ok(()));
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        r.events_rxstarted().write_value(0);
        drop.defuse();

        result
    }

    /// Read bytes until the buffer is filled.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            return Ok(());
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();

        r.rxd().ptr().write_value(ptr as u32);
        r.rxd().maxcnt().write(|w| w.set_maxcnt(len as _));

        r.events_endrx().write_value(0);
        r.events_error().write_value(0);
        r.intenclr().write(|w| {
            w.set_endrx(true);
            w.set_error(true);
        });

        compiler_fence(Ordering::SeqCst);

        trace!("startrx");
        r.tasks_startrx().write_value(1);

        while r.events_endrx().read() == 0 && r.events_error().read() == 0 {}

        compiler_fence(Ordering::SeqCst);
        r.events_rxstarted().write_value(0);

        self.check_and_clear_errors()
    }
}

impl<'a, T: Instance> Drop for UarteRx<'a, T> {
    fn drop(&mut self) {
        trace!("uarte rx drop");

        let r = T::regs();

        let did_stoprx = r.events_rxstarted().read() != 0;
        trace!("did_stoprx {}", did_stoprx);

        // Wait for rxto, if needed.
        while did_stoprx && r.events_rxto().read() == 0 {}

        let s = T::state();

        drop_tx_rx(r, s);
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
        if buffer.is_empty() {
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

            r.intenclr().write(|w| {
                w.set_endrx(true);
                w.set_error(true);
            });
            r.events_rxto().write_value(0);
            r.events_error().write_value(0);
            r.tasks_stoprx().write_value(1);

            while r.events_endrx().read() == 0 {}
        });

        r.rxd().ptr().write_value(ptr as u32);
        r.rxd().maxcnt().write(|w| w.set_maxcnt(len as _));

        r.events_endrx().write_value(0);
        r.events_error().write_value(0);
        r.intenset().write(|w| {
            w.set_endrx(true);
            w.set_error(true);
        });

        compiler_fence(Ordering::SeqCst);

        r.tasks_startrx().write_value(1);

        let result = poll_fn(|cx| {
            s.rx_waker.register(cx.waker());

            if let Err(e) = self.rx.check_and_clear_errors() {
                r.tasks_stoprx().write_value(1);
                return Poll::Ready(Err(e));
            }
            if r.events_endrx().read() != 0 {
                return Poll::Ready(Ok(()));
            }

            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        let n = r.rxd().amount().read().0 as usize;

        self.timer.stop();
        r.events_rxstarted().write_value(0);

        drop.defuse();

        result.map(|_| n)
    }

    /// Read bytes until the buffer is filled, or the line becomes idle.
    ///
    /// Returns the amount of bytes read.
    pub fn blocking_read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        if buffer.is_empty() {
            return Ok(0);
        }
        if buffer.len() > EASY_DMA_SIZE {
            return Err(Error::BufferTooLong);
        }

        let ptr = buffer.as_ptr();
        let len = buffer.len();

        let r = T::regs();

        self.ppi_ch1.enable();

        r.rxd().ptr().write_value(ptr as u32);
        r.rxd().maxcnt().write(|w| w.set_maxcnt(len as _));

        r.events_endrx().write_value(0);
        r.events_error().write_value(0);
        r.intenclr().write(|w| {
            w.set_endrx(true);
            w.set_error(true);
        });

        compiler_fence(Ordering::SeqCst);

        r.tasks_startrx().write_value(1);

        while r.events_endrx().read() == 0 && r.events_error().read() == 0 {}

        compiler_fence(Ordering::SeqCst);
        let n = r.rxd().amount().read().0 as usize;

        self.timer.stop();
        r.events_rxstarted().write_value(0);

        self.rx.check_and_clear_errors().map(|_| n)
    }
}

#[cfg(not(any(feature = "_nrf9160", feature = "_nrf5340")))]
pub(crate) fn apply_workaround_for_enable_anomaly(_r: pac::uarte::Uarte) {
    // Do nothing
}

#[cfg(any(feature = "_nrf9160", feature = "_nrf5340"))]
pub(crate) fn apply_workaround_for_enable_anomaly(r: pac::uarte::Uarte) {
    // Apply workaround for anomalies:
    // - nRF9160 - anomaly 23
    // - nRF5340 - anomaly 44
    let rp = r.as_ptr() as *mut u32;
    let rxenable_reg = unsafe { rp.add(0x564 / 4) };
    let txenable_reg = unsafe { rp.add(0x568 / 4) };

    // NB Safety: This is taken from Nordic's driver -
    // https://github.com/NordicSemiconductor/nrfx/blob/master/drivers/src/nrfx_uarte.c#L197
    if unsafe { core::ptr::read_volatile(txenable_reg) } == 1 {
        r.tasks_stoptx().write_value(1);
    }

    // NB Safety: This is taken from Nordic's driver -
    // https://github.com/NordicSemiconductor/nrfx/blob/master/drivers/src/nrfx_uarte.c#L197
    if unsafe { core::ptr::read_volatile(rxenable_reg) } == 1 {
        r.enable().write(|w| w.set_enable(vals::Enable::ENABLED));
        r.tasks_stoprx().write_value(1);

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

        // write back the bits we just read to clear them
        let errors = r.errorsrc().read();
        r.errorsrc().write_value(errors);
        r.enable().write(|w| w.set_enable(vals::Enable::DISABLED));
    }
}

pub(crate) fn drop_tx_rx(r: pac::uarte::Uarte, s: &State) {
    if s.tx_rx_refcount.fetch_sub(1, Ordering::Relaxed) == 1 {
        // Finally we can disable, and we do so for the peripheral
        // i.e. not just rx concerns.
        r.enable().write(|w| w.set_enable(vals::Enable::DISABLED));

        gpio::deconfigure_pin(r.psel().rxd().read());
        gpio::deconfigure_pin(r.psel().txd().read());
        gpio::deconfigure_pin(r.psel().rts().read());
        gpio::deconfigure_pin(r.psel().cts().read());

        trace!("uarte tx and rx drop: done");
    }
}

pub(crate) struct State {
    pub(crate) rx_waker: AtomicWaker,
    pub(crate) tx_waker: AtomicWaker,
    pub(crate) tx_rx_refcount: AtomicU8,
}
impl State {
    pub(crate) const fn new() -> Self {
        Self {
            rx_waker: AtomicWaker::new(),
            tx_waker: AtomicWaker::new(),
            tx_rx_refcount: AtomicU8::new(0),
        }
    }
}

pub(crate) trait SealedInstance {
    fn regs() -> pac::uarte::Uarte;
    fn state() -> &'static State;
    fn buffered_state() -> &'static crate::buffered_uarte::State;
}

/// UARTE peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_uarte {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::uarte::SealedInstance for peripherals::$type {
            fn regs() -> pac::uarte::Uarte {
                pac::$pac_type
            }
            fn state() -> &'static crate::uarte::State {
                static STATE: crate::uarte::State = crate::uarte::State::new();
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

mod _embedded_io {
    use super::*;

    impl embedded_io_async::Error for Error {
        fn kind(&self) -> embedded_io_async::ErrorKind {
            match *self {
                Error::BufferTooLong => embedded_io_async::ErrorKind::InvalidInput,
                Error::BufferNotInRAM => embedded_io_async::ErrorKind::Unsupported,
                Error::Framing => embedded_io_async::ErrorKind::InvalidData,
                Error::Parity => embedded_io_async::ErrorKind::InvalidData,
                Error::Overrun => embedded_io_async::ErrorKind::OutOfMemory,
                Error::Break => embedded_io_async::ErrorKind::ConnectionAborted,
            }
        }
    }

    impl<'d, U: Instance> embedded_io_async::ErrorType for Uarte<'d, U> {
        type Error = Error;
    }

    impl<'d, U: Instance> embedded_io_async::ErrorType for UarteTx<'d, U> {
        type Error = Error;
    }

    impl<'d, U: Instance> embedded_io_async::Write for Uarte<'d, U> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.write(buf).await?;
            Ok(buf.len())
        }
    }

    impl<'d: 'd, U: Instance> embedded_io_async::Write for UarteTx<'d, U> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.write(buf).await?;
            Ok(buf.len())
        }
    }
}
