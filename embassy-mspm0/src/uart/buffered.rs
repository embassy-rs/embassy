use core::future::{Future, poll_fn};
use core::marker::PhantomData;
use core::slice;
use core::sync::atomic::{AtomicU8, Ordering};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::atomic_ring_buffer::RingBuffer;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;
use embedded_hal_nb::nb;

use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt::typelevel::Binding;
use crate::pac::uart::Uart as Regs;
use crate::uart::{Config, ConfigError, CtsPin, Error, Info, Instance, RtsPin, RxPin, State, TxPin};
use crate::{Peri, interrupt};

/// Interrupt handler.
pub struct BufferedInterruptHandler<T: Instance> {
    _uart: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for BufferedInterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_interrupt(T::info().regs, T::buffered_state())
    }
}

/// Bidirectional buffered UART which acts as a combination of [`BufferedUartTx`] and [`BufferedUartRx`].
pub struct BufferedUart<'d> {
    rx: BufferedUartRx<'d>,
    tx: BufferedUartTx<'d>,
}

impl SetConfig for BufferedUart<'_> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d> BufferedUart<'d> {
    /// Create a new bidirectional buffered UART.
    pub fn new<T: Instance>(
        uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            uart,
            new_pin!(rx, config.rx_pf()),
            new_pin!(tx, config.tx_pf()),
            None,
            None,
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    /// Create a new bidirectional buffered UART with request-to-send and clear-to-send pins
    pub fn new_with_rtscts<T: Instance>(
        uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        rts: Peri<'d, impl RtsPin<T>>,
        cts: Peri<'d, impl CtsPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            uart,
            new_pin!(rx, config.rx_pf()),
            new_pin!(tx, config.tx_pf()),
            new_pin!(rts, config.rts_pf()),
            new_pin!(cts, config.cts_pf()),
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        self.tx.set_config(config)?;
        self.rx.set_config(config)
    }

    /// Set baudrate
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        self.rx.set_baudrate(baudrate)
    }

    /// Write to UART TX buffer, blocking execution until done.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        self.tx.blocking_write(buffer)
    }

    /// Flush UART TX buffer, blocking execution until done.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Check if UART is busy.
    pub fn busy(&self) -> bool {
        self.tx.busy()
    }

    /// Read from UART RX buffer, blocking execution until done.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.rx.blocking_read(buffer)
    }

    /// Send break character.
    pub fn send_break(&mut self) {
        self.tx.send_break()
    }

    /// Split into separate RX and TX handles.
    pub fn split(self) -> (BufferedUartTx<'d>, BufferedUartRx<'d>) {
        (self.tx, self.rx)
    }

    /// Split into separate RX and TX handles.
    pub fn split_ref(&mut self) -> (BufferedUartTx<'_>, BufferedUartRx<'_>) {
        (
            BufferedUartTx {
                info: self.tx.info,
                state: self.tx.state,
                tx: self.tx.tx.as_mut().map(Peri::reborrow),
                cts: self.tx.cts.as_mut().map(Peri::reborrow),
                reborrowed: true,
            },
            BufferedUartRx {
                info: self.rx.info,
                state: self.rx.state,
                rx: self.rx.rx.as_mut().map(Peri::reborrow),
                rts: self.rx.rts.as_mut().map(Peri::reborrow),
                reborrowed: true,
            },
        )
    }
}

/// Rx-only buffered UART.
///
/// Can be obtained from [`BufferedUart::split`], or can be constructed independently,
/// if you do not need the transmitting half of the driver.
pub struct BufferedUartRx<'d> {
    info: &'static Info,
    state: &'static BufferedState,
    rx: Option<Peri<'d, AnyPin>>,
    rts: Option<Peri<'d, AnyPin>>,
    reborrowed: bool,
}

impl SetConfig for BufferedUartRx<'_> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d> BufferedUartRx<'d> {
    /// Create a new rx-only buffered UART with no hardware flow control.
    ///
    /// Useful if you only want Uart Rx. It saves 1 pin.
    pub fn new<T: Instance>(
        uart: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(uart, new_pin!(rx, config.rx_pf()), None, rx_buffer, config)
    }

    /// Create a new rx-only buffered UART with a request-to-send pin
    pub fn new_with_rts<T: Instance>(
        uart: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        rts: Peri<'d, impl RtsPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            uart,
            new_pin!(rx, config.rx_pf()),
            new_pin!(rts, config.rts_pf()),
            rx_buffer,
            config,
        )
    }

    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        if let Some(ref rx) = self.rx {
            rx.update_pf(config.rx_pf());
        }

        if let Some(ref rts) = self.rts {
            rts.update_pf(config.rts_pf());
        }

        super::reconfigure(&self.info, &self.state.state, config)
    }

    /// Set baudrate
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), ConfigError> {
        super::set_baudrate(&self.info, self.state.state.clock.load(Ordering::Relaxed), baudrate)
    }

    /// Read from UART RX buffer, blocking execution until done.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.blocking_read_inner(buffer)
    }
}

impl Drop for BufferedUartRx<'_> {
    fn drop(&mut self) {
        if !self.reborrowed {
            let state = self.state;

            // SAFETY: RX is being dropped (and is not reborrowed), so the ring buffer must be deinitialized
            // in order to meet the requirements of init.
            unsafe {
                state.rx_buf.deinit();
            }

            // TX is inactive if the buffer is not available. If this is true, then disable the
            // interrupt handler since we are running in RX only mode.
            if state.tx_buf.len() == 0 {
                self.info.interrupt.disable();
            }

            self.rx.as_ref().map(|x| x.set_as_disconnected());
            self.rts.as_ref().map(|x| x.set_as_disconnected());
        }
    }
}

/// Tx-only buffered UART.
///
/// Can be obtained from [`BufferedUart::split`], or can be constructed independently,
/// if you do not need the receiving half of the driver.
pub struct BufferedUartTx<'d> {
    info: &'static Info,
    state: &'static BufferedState,
    tx: Option<Peri<'d, AnyPin>>,
    cts: Option<Peri<'d, AnyPin>>,
    reborrowed: bool,
}

impl SetConfig for BufferedUartTx<'_> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d> BufferedUartTx<'d> {
    /// Create a new tx-only buffered UART with no hardware flow control.
    ///
    /// Useful if you only want Uart Tx. It saves 1 pin.
    pub fn new<T: Instance>(
        uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(uart, new_pin!(tx, config.tx_pf()), None, tx_buffer, config)
    }

    /// Create a new tx-only buffered UART with a clear-to-send pin
    pub fn new_with_rts<T: Instance>(
        uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        cts: Peri<'d, impl CtsPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            uart,
            new_pin!(tx, config.tx_pf()),
            new_pin!(cts, config.cts_pf()),
            tx_buffer,
            config,
        )
    }

    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        if let Some(ref tx) = self.tx {
            tx.update_pf(config.tx_pf());
        }

        if let Some(ref cts) = self.cts {
            cts.update_pf(config.cts_pf());
        }

        super::reconfigure(self.info, &self.state.state, config)
    }

    /// Set baudrate
    pub fn set_baudrate(&self, baudrate: u32) -> Result<(), ConfigError> {
        super::set_baudrate(&self.info, self.state.state.clock.load(Ordering::Relaxed), baudrate)
    }

    /// Write to UART TX buffer, blocking execution until done.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        self.blocking_write_inner(buffer)
    }

    /// Flush UART TX buffer, blocking execution until done.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        let state = self.state;

        loop {
            if state.tx_buf.is_empty() {
                return Ok(());
            }
        }
    }

    /// Check if UART is busy.
    pub fn busy(&self) -> bool {
        super::busy(self.info.regs)
    }

    /// Send break character
    pub fn send_break(&mut self) {
        let r = self.info.regs;

        r.lcrh().modify(|w| {
            w.set_brk(true);
        });
    }
}

impl Drop for BufferedUartTx<'_> {
    fn drop(&mut self) {
        if !self.reborrowed {
            let state = self.state;

            // SAFETY: TX is being dropped (and is not reborrowed), so the ring buffer must be deinitialized
            // in order to meet the requirements of init.
            unsafe {
                state.tx_buf.deinit();
            }

            // RX is inactive if the buffer is not available. If this is true, then disable the
            // interrupt handler since we are running in TX only mode.
            if state.rx_buf.len() == 0 {
                self.info.interrupt.disable();
            }

            self.tx.as_ref().map(|x| x.set_as_disconnected());
            self.cts.as_ref().map(|x| x.set_as_disconnected());
        }
    }
}

impl embedded_io_async::ErrorType for BufferedUart<'_> {
    type Error = Error;
}

impl embedded_io_async::ErrorType for BufferedUartRx<'_> {
    type Error = Error;
}

impl embedded_io_async::ErrorType for BufferedUartTx<'_> {
    type Error = Error;
}

impl embedded_io_async::Read for BufferedUart<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf).await
    }
}

impl embedded_io_async::Read for BufferedUartRx<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_inner(buf).await
    }
}

impl embedded_io_async::ReadReady for BufferedUart<'_> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        self.rx.read_ready()
    }
}

impl embedded_io_async::ReadReady for BufferedUartRx<'_> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        self.read_ready_inner()
    }
}

impl embedded_io_async::BufRead for BufferedUart<'_> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        self.rx.fill_buf().await
    }

    fn consume(&mut self, amt: usize) {
        self.rx.consume(amt);
    }
}

impl embedded_io_async::BufRead for BufferedUartRx<'_> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        self.fill_buf_inner().await
    }

    fn consume(&mut self, amt: usize) {
        self.consume_inner(amt);
    }
}

impl embedded_io_async::Write for BufferedUart<'_> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write_inner(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush_inner().await
    }
}

impl embedded_io_async::Write for BufferedUartTx<'_> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write_inner(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush_inner().await
    }
}

impl embedded_io::Read for BufferedUart<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf)
    }
}

impl embedded_io::Read for BufferedUartRx<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read_inner(buf)
    }
}

impl embedded_io::Write for BufferedUart<'_> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush()
    }
}

impl embedded_io::Write for BufferedUartTx<'_> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write_inner(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl embedded_hal_nb::serial::Error for Error {
    fn kind(&self) -> embedded_hal_nb::serial::ErrorKind {
        match self {
            Error::Framing => embedded_hal_nb::serial::ErrorKind::FrameFormat,
            Error::Noise => embedded_hal_nb::serial::ErrorKind::Noise,
            Error::Overrun => embedded_hal_nb::serial::ErrorKind::Overrun,
            Error::Parity => embedded_hal_nb::serial::ErrorKind::Parity,
            Error::Break => embedded_hal_nb::serial::ErrorKind::Other,
        }
    }
}

impl embedded_hal_nb::serial::ErrorType for BufferedUart<'_> {
    type Error = Error;
}

impl embedded_hal_nb::serial::ErrorType for BufferedUartRx<'_> {
    type Error = Error;
}

impl embedded_hal_nb::serial::ErrorType for BufferedUartTx<'_> {
    type Error = Error;
}

impl embedded_hal_nb::serial::Read for BufferedUart<'_> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.rx.read()
    }
}

impl embedded_hal_nb::serial::Read for BufferedUartRx<'_> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        if self.info.regs.stat().read().rxfe() {
            return Err(nb::Error::WouldBlock);
        }

        super::read_with_error(self.info.regs).map_err(nb::Error::Other)
    }
}

impl embedded_hal_nb::serial::Write for BufferedUart<'_> {
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.tx.write(word)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.tx.flush()
    }
}

impl embedded_hal_nb::serial::Write for BufferedUartTx<'_> {
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.blocking_write(&[word]).map(drop).map_err(nb::Error::Other)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.blocking_flush().map_err(nb::Error::Other)
    }
}

// Impl details

/// Buffered UART state.
pub(crate) struct BufferedState {
    /// non-buffered UART state. This is inline in order to avoid [`BufferedUartRx`]/Tx
    /// needing to carry around a 2nd static reference and waste another 4 bytes.
    state: State,
    rx_waker: AtomicWaker,
    rx_buf: RingBuffer,
    tx_waker: AtomicWaker,
    tx_buf: RingBuffer,
    rx_error: AtomicU8,
}

// these must match bits 8..12 in RXDATA, but shifted by 8 to the right
const RXE_NOISE: u8 = 16;
const RXE_OVERRUN: u8 = 8;
const RXE_BREAK: u8 = 4;
const RXE_PARITY: u8 = 2;
const RXE_FRAMING: u8 = 1;

impl BufferedState {
    pub const fn new() -> Self {
        Self {
            state: State::new(),
            rx_waker: AtomicWaker::new(),
            rx_buf: RingBuffer::new(),
            tx_waker: AtomicWaker::new(),
            tx_buf: RingBuffer::new(),
            rx_error: AtomicU8::new(0),
        }
    }
}

impl<'d> BufferedUart<'d> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Option<Peri<'d, AnyPin>>,
        tx: Option<Peri<'d, AnyPin>>,
        rts: Option<Peri<'d, AnyPin>>,
        cts: Option<Peri<'d, AnyPin>>,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        let info = T::info();
        let state = T::buffered_state();

        let mut this = Self {
            tx: BufferedUartTx {
                info,
                state,
                tx,
                cts,
                reborrowed: false,
            },
            rx: BufferedUartRx {
                info,
                state,
                rx,
                rts,
                reborrowed: false,
            },
        };
        this.enable_and_configure(tx_buffer, rx_buffer, &config)?;

        Ok(this)
    }

    fn enable_and_configure(
        &mut self,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: &Config,
    ) -> Result<(), ConfigError> {
        let info = self.rx.info;
        let state = self.rx.state;

        assert!(!tx_buffer.is_empty());
        assert!(!rx_buffer.is_empty());

        init_buffers(info, state, Some(tx_buffer), Some(rx_buffer));
        super::enable(info.regs);
        super::configure(
            info,
            &state.state,
            config,
            true,
            self.rx.rts.is_some(),
            true,
            self.tx.cts.is_some(),
        )?;

        info.regs.cpu_int(0).imask().modify(|w| {
            w.set_rxint(true);
        });

        info.interrupt.unpend();
        unsafe { info.interrupt.enable() };

        Ok(())
    }
}

impl<'d> BufferedUartRx<'d> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Option<Peri<'d, AnyPin>>,
        rts: Option<Peri<'d, AnyPin>>,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        let mut this = Self {
            info: T::info(),
            state: T::buffered_state(),
            rx,
            rts,
            reborrowed: false,
        };
        this.enable_and_configure(rx_buffer, &config)?;

        Ok(this)
    }

    fn enable_and_configure(&mut self, rx_buffer: &'d mut [u8], config: &Config) -> Result<(), ConfigError> {
        let info = self.info;
        let state = self.state;

        init_buffers(info, state, None, Some(rx_buffer));
        super::enable(info.regs);
        super::configure(info, &self.state.state, config, true, self.rts.is_some(), false, false)?;

        info.regs.cpu_int(0).imask().modify(|w| {
            w.set_rxint(true);
        });

        info.interrupt.unpend();
        unsafe { info.interrupt.enable() };

        Ok(())
    }

    async fn read_inner(&self, buf: &mut [u8]) -> Result<usize, Error> {
        poll_fn(move |cx| {
            let state = self.state;

            if let Poll::Ready(r) = self.try_read(buf) {
                return Poll::Ready(r);
            }

            state.rx_waker.register(cx.waker());
            Poll::Pending
        })
        .await
    }

    fn blocking_read_inner(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        loop {
            match self.try_read(buffer) {
                Poll::Ready(res) => return res,
                Poll::Pending => continue,
            }
        }
    }

    fn fill_buf_inner(&self) -> impl Future<Output = Result<&'_ [u8], Error>> {
        poll_fn(move |cx| {
            let mut rx_reader = unsafe { self.state.rx_buf.reader() };
            let (p, n) = rx_reader.pop_buf();
            let result = if n == 0 {
                match Self::get_rx_error(self.state) {
                    None => {
                        self.state.rx_waker.register(cx.waker());
                        return Poll::Pending;
                    }
                    Some(e) => Err(e),
                }
            } else {
                let buf = unsafe { slice::from_raw_parts(p, n) };
                Ok(buf)
            };

            Poll::Ready(result)
        })
    }

    fn consume_inner(&self, amt: usize) {
        let mut rx_reader = unsafe { self.state.rx_buf.reader() };
        rx_reader.pop_done(amt);

        // (Re-)Enable the interrupt to receive more data in case it was
        // disabled because the buffer was full or errors were detected.
        self.info.regs.cpu_int(0).imask().modify(|w| {
            w.set_rxint(true);
            w.set_rtout(true);
        });
    }

    /// we are ready to read if there is data in the buffer
    fn read_ready_inner(&self) -> Result<bool, Error> {
        Ok(!self.state.rx_buf.is_empty())
    }

    fn try_read(&self, buf: &mut [u8]) -> Poll<Result<usize, Error>> {
        let state = self.state;

        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        let mut rx_reader = unsafe { state.rx_buf.reader() };
        let n = rx_reader.pop(|data| {
            let n = data.len().min(buf.len());
            buf[..n].copy_from_slice(&data[..n]);
            n
        });

        let result = if n == 0 {
            match Self::get_rx_error(state) {
                None => return Poll::Pending,
                Some(e) => Err(e),
            }
        } else {
            Ok(n)
        };

        // (Re-)Enable the interrupt to receive more data in case it was
        // disabled because the buffer was full or errors were detected.
        self.info.regs.cpu_int(0).imask().modify(|w| {
            w.set_rxint(true);
            w.set_rtout(true);
        });

        Poll::Ready(result)
    }

    fn get_rx_error(state: &BufferedState) -> Option<Error> {
        // Cortex-M0 has does not support atomic swap, so we must do two operations.
        let errs = critical_section::with(|_cs| {
            let errs = state.rx_error.load(Ordering::Relaxed);
            state.rx_error.store(0, Ordering::Relaxed);

            errs
        });

        if errs & RXE_NOISE != 0 {
            Some(Error::Noise)
        } else if errs & RXE_OVERRUN != 0 {
            Some(Error::Overrun)
        } else if errs & RXE_BREAK != 0 {
            Some(Error::Break)
        } else if errs & RXE_PARITY != 0 {
            Some(Error::Parity)
        } else if errs & RXE_FRAMING != 0 {
            Some(Error::Framing)
        } else {
            None
        }
    }
}

impl<'d> BufferedUartTx<'d> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        tx: Option<Peri<'d, AnyPin>>,
        cts: Option<Peri<'d, AnyPin>>,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        let mut this = Self {
            info: T::info(),
            state: T::buffered_state(),
            tx,
            cts,
            reborrowed: false,
        };

        this.enable_and_configure(tx_buffer, &config)?;

        Ok(this)
    }

    async fn write_inner(&self, buf: &[u8]) -> Result<usize, Error> {
        poll_fn(move |cx| {
            let state = self.state;

            if buf.is_empty() {
                return Poll::Ready(Ok(0));
            }

            let mut tx_writer = unsafe { state.tx_buf.writer() };
            let n = tx_writer.push(|data| {
                let n = data.len().min(buf.len());
                data[..n].copy_from_slice(&buf[..n]);
                n
            });

            if n == 0 {
                state.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            // The TX interrupt only triggers when the there was data in the
            // FIFO and the number of bytes drops below a threshold. When the
            // FIFO was empty we have to manually pend the interrupt to shovel
            // TX data from the buffer into the FIFO.
            self.info.interrupt.pend();
            Poll::Ready(Ok(n))
        })
        .await
    }

    fn blocking_write_inner(&self, buffer: &[u8]) -> Result<usize, Error> {
        let state = self.state;

        loop {
            let empty = state.tx_buf.is_empty();

            // SAFETY: tx buf must be initialized if BufferedUartTx exists.
            let mut tx_writer = unsafe { state.tx_buf.writer() };
            let data = tx_writer.push_slice();

            if !data.is_empty() {
                let n = data.len().min(buffer.len());
                data[..n].copy_from_slice(&buffer[..n]);
                tx_writer.push_done(n);

                if empty {
                    self.info.interrupt.pend();
                }

                return Ok(n);
            }
        }
    }

    async fn flush_inner(&self) -> Result<(), Error> {
        poll_fn(move |cx| {
            let state = self.state;

            if !state.tx_buf.is_empty() {
                state.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            Poll::Ready(Ok(()))
        })
        .await
    }

    fn enable_and_configure(&mut self, tx_buffer: &'d mut [u8], config: &Config) -> Result<(), ConfigError> {
        let info = self.info;
        let state = self.state;

        init_buffers(info, state, Some(tx_buffer), None);
        super::enable(info.regs);
        super::configure(info, &state.state, config, false, false, true, self.cts.is_some())?;

        info.regs.cpu_int(0).imask().modify(|w| {
            w.set_rxint(true);
        });

        info.interrupt.unpend();
        unsafe { info.interrupt.enable() };

        Ok(())
    }
}

fn init_buffers<'d>(
    info: &Info,
    state: &BufferedState,
    tx_buffer: Option<&'d mut [u8]>,
    rx_buffer: Option<&'d mut [u8]>,
) {
    if let Some(tx_buffer) = tx_buffer {
        let len = tx_buffer.len();
        unsafe { state.tx_buf.init(tx_buffer.as_mut_ptr(), len) };
    }

    if let Some(rx_buffer) = rx_buffer {
        let len = rx_buffer.len();
        unsafe { state.rx_buf.init(rx_buffer.as_mut_ptr(), len) };
    }

    info.regs.cpu_int(0).imask().modify(|w| {
        w.set_nerr(true);
        w.set_frmerr(true);
        w.set_parerr(true);
        w.set_brkerr(true);
        w.set_ovrerr(true);
    });
}

fn on_interrupt(r: Regs, state: &'static BufferedState) {
    let int = r.cpu_int(0).mis().read();

    // Per https://github.com/embassy-rs/embassy/pull/1458, both buffered and unbuffered handlers may be bound.
    if super::dma_enabled(r) {
        return;
    }

    // RX
    if state.rx_buf.is_available() {
        // SAFETY: RX must have been initialized if RXE is set.
        let mut rx_writer = unsafe { state.rx_buf.writer() };
        let rx_buf = rx_writer.push_slice();
        let mut n_read = 0;
        let mut error = false;

        for rx_byte in rx_buf {
            let stat = r.stat().read();

            if stat.rxfe() {
                break;
            }

            let data = r.rxdata().read();

            if (data.0 >> 8) != 0 {
                // Cortex-M0 does not support atomic fetch_or, must do 2 operations.
                critical_section::with(|_cs| {
                    let mut value = state.rx_error.load(Ordering::Relaxed);
                    value |= (data.0 >> 8) as u8;
                    state.rx_error.store(value, Ordering::Relaxed);
                });
                error = true;

                // only fill the buffer with valid characters. the current character is fine
                // if the error is an overrun, but if we add it to the buffer we'll report
                // the overrun one character too late. drop it instead and pretend we were
                // a bit slower at draining the rx fifo than we actually were.
                // this is consistent with blocking uart error reporting.
                break;
            }

            *rx_byte = data.data();
            n_read += 1;
        }

        if n_read > 0 {
            rx_writer.push_done(n_read);
            state.rx_waker.wake();
        } else if error {
            state.rx_waker.wake();
        }

        // Disable any further RX interrupts when the buffer becomes full or
        // errors have occurred. This lets us buffer additional errors in the
        // fifo without needing more error storage locations, and most applications
        // will want to do a full reset of their uart state anyway once an error
        // has happened.
        if state.rx_buf.is_full() || error {
            r.cpu_int(0).imask().modify(|w| {
                w.set_rxint(false);
                w.set_rtout(false);
            });
        }
    }

    if int.eot() {
        r.cpu_int(0).imask().modify(|w| {
            w.set_eot(false);
        });

        r.cpu_int(0).iclr().write(|w| {
            w.set_eot(true);
        });

        state.tx_waker.wake();
    }

    // TX
    if state.tx_buf.is_available() {
        // SAFETY: TX must have been initialized if TXE is set.
        let mut tx_reader = unsafe { state.tx_buf.reader() };
        let buf = tx_reader.pop_slice();
        let mut n_written = 0;

        for tx_byte in buf.iter_mut() {
            let stat = r.stat().read();

            if stat.txff() {
                break;
            }

            r.txdata().write(|w| {
                w.set_data(*tx_byte);
            });
            n_written += 1;
        }

        if n_written > 0 {
            // EOT will wake.
            r.cpu_int(0).imask().modify(|w| {
                w.set_eot(true);
            });

            tx_reader.pop_done(n_written);
        }
    }

    // Clear TX and error interrupt flags
    // RX interrupt flags are cleared by writing to ICLR.
    let mis = r.cpu_int(0).mis().read();
    r.cpu_int(0).iclr().write(|w| {
        w.set_nerr(mis.nerr());
        w.set_frmerr(mis.frmerr());
        w.set_parerr(mis.parerr());
        w.set_brkerr(mis.brkerr());
        w.set_ovrerr(mis.ovrerr());
    });

    // Errors
    if mis.nerr() {
        warn!("Noise error");
    }
    if mis.frmerr() {
        warn!("Framing error");
    }
    if mis.parerr() {
        warn!("Parity error");
    }
    if mis.brkerr() {
        warn!("Break error");
    }
    if mis.ovrerr() {
        warn!("Overrun error");
    }
}
