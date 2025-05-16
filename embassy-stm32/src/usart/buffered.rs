use core::future::poll_fn;
use core::marker::PhantomData;
use core::slice;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::atomic_ring_buffer::RingBuffer;
use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;

#[cfg(not(any(usart_v1, usart_v2)))]
use super::DePin;
use super::{
    clear_interrupt_flags, configure, half_duplex_set_rx_tx_before_write, rdr, reconfigure, send_break, set_baudrate,
    sr, tdr, Config, ConfigError, CtsPin, Duplex, Error, HalfDuplexReadback, Info, Instance, Regs, RtsPin, RxPin,
    TxPin,
};
use crate::gpio::{AfType, AnyPin, Pull, SealedPin as _};
use crate::interrupt::{self, InterruptExt};
use crate::time::Hertz;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_interrupt(T::info().regs, T::buffered_state())
    }
}

unsafe fn on_interrupt(r: Regs, state: &'static State) {
    // RX
    let sr_val = sr(r).read();
    // On v1 & v2, reading DR clears the rxne, error and idle interrupt
    // flags. Keep this close to the SR read to reduce the chance of a
    // flag being set in-between.
    let dr = if sr_val.rxne() || cfg!(any(usart_v1, usart_v2)) && (sr_val.ore() || sr_val.idle()) {
        Some(rdr(r).read_volatile())
    } else {
        None
    };
    clear_interrupt_flags(r, sr_val);

    if sr_val.pe() {
        warn!("Parity error");
    }
    if sr_val.fe() {
        warn!("Framing error");
    }
    if sr_val.ne() {
        warn!("Noise error");
    }
    if sr_val.ore() {
        warn!("Overrun error");
    }
    if sr_val.rxne() {
        let mut rx_writer = state.rx_buf.writer();
        let buf = rx_writer.push_slice();
        if !buf.is_empty() {
            if let Some(byte) = dr {
                buf[0] = byte;
                rx_writer.push_done(1);
            }
        } else {
            // FIXME: Should we disable any further RX interrupts when the buffer becomes full.
        }

        if !state.rx_buf.is_empty() {
            state.rx_waker.wake();
        }
    }

    if sr_val.idle() {
        state.rx_waker.wake();
    }

    // With `usart_v4` hardware FIFO is enabled and Transmission complete (TC)
    // indicates that all bytes are pushed out from the FIFO.
    // For other usart variants it shows that last byte from the buffer was just sent.
    if sr_val.tc() {
        // For others it is cleared above with `clear_interrupt_flags`.
        #[cfg(any(usart_v1, usart_v2))]
        sr(r).modify(|w| w.set_tc(false));

        r.cr1().modify(|w| {
            w.set_tcie(false);
        });

        state.tx_done.store(true, Ordering::Release);
        state.tx_waker.wake();
    }

    // TX
    if sr(r).read().txe() {
        let mut tx_reader = state.tx_buf.reader();
        let buf = tx_reader.pop_slice();
        if !buf.is_empty() {
            r.cr1().modify(|w| {
                w.set_txeie(true);
            });

            // Enable transmission complete interrupt when last byte is going to be sent out.
            if buf.len() == 1 {
                r.cr1().modify(|w| {
                    w.set_tcie(true);
                });
            }

            half_duplex_set_rx_tx_before_write(&r, state.half_duplex_readback.load(Ordering::Relaxed));

            tdr(r).write_volatile(buf[0].into());
            tx_reader.pop_done(1);
        } else {
            // Disable interrupt until we have something to transmit again.
            r.cr1().modify(|w| {
                w.set_txeie(false);
            });
        }
    }
}

pub(super) struct State {
    rx_waker: AtomicWaker,
    rx_buf: RingBuffer,
    tx_waker: AtomicWaker,
    tx_buf: RingBuffer,
    tx_done: AtomicBool,
    tx_rx_refcount: AtomicU8,
    half_duplex_readback: AtomicBool,
}

impl State {
    pub(super) const fn new() -> Self {
        Self {
            rx_buf: RingBuffer::new(),
            tx_buf: RingBuffer::new(),
            rx_waker: AtomicWaker::new(),
            tx_waker: AtomicWaker::new(),
            tx_done: AtomicBool::new(true),
            tx_rx_refcount: AtomicU8::new(0),
            half_duplex_readback: AtomicBool::new(false),
        }
    }
}

/// Bidirectional buffered UART
pub struct BufferedUart<'d> {
    rx: BufferedUartRx<'d>,
    tx: BufferedUartTx<'d>,
}

/// Tx-only buffered UART
///
/// Created with [BufferedUart::split]
pub struct BufferedUartTx<'d> {
    info: &'static Info,
    state: &'static State,
    kernel_clock: Hertz,
    tx: Option<Peri<'d, AnyPin>>,
    cts: Option<Peri<'d, AnyPin>>,
    de: Option<Peri<'d, AnyPin>>,
    is_borrowed: bool,
}

/// Rx-only buffered UART
///
/// Created with [BufferedUart::split]
pub struct BufferedUartRx<'d> {
    info: &'static Info,
    state: &'static State,
    kernel_clock: Hertz,
    rx: Option<Peri<'d, AnyPin>>,
    rts: Option<Peri<'d, AnyPin>>,
    is_borrowed: bool,
}

impl<'d> SetConfig for BufferedUart<'d> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d> SetConfig for BufferedUartRx<'d> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d> SetConfig for BufferedUartTx<'d> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d> BufferedUart<'d> {
    /// Create a new bidirectional buffered UART driver
    pub fn new<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            new_pin!(tx, config.tx_af()),
            None,
            None,
            None,
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    /// Create a new bidirectional buffered UART driver with request-to-send and clear-to-send pins
    pub fn new_with_rtscts<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        rts: Peri<'d, impl RtsPin<T>>,
        cts: Peri<'d, impl CtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            new_pin!(tx, config.tx_af()),
            new_pin!(rts, config.rts_config.af_type()),
            new_pin!(cts, AfType::input(config.cts_pull)),
            None,
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    /// Create a new bidirectional buffered UART driver with only the RTS pin as the DE pin
    pub fn new_with_rts_as_de<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        rts: Peri<'d, impl RtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            new_pin!(tx, config.tx_af()),
            None,
            None,
            new_pin!(rts, AfType::input(Pull::None)), // RTS mapped used as DE
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    /// Create a new bidirectional buffered UART driver with only the request-to-send pin
    pub fn new_with_rts<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        rts: Peri<'d, impl RtsPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            new_pin!(tx, config.tx_af()),
            new_pin!(rts, AfType::input(Pull::None)),
            None, // no CTS
            None, // no DE
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    /// Create a new bidirectional buffered UART driver with a driver-enable pin
    #[cfg(not(any(usart_v1, usart_v2)))]
    pub fn new_with_de<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        tx: Peri<'d, impl TxPin<T>>,
        de: Peri<'d, impl DePin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            new_pin!(tx, config.tx_af()),
            None,
            None,
            new_pin!(de, config.de_config.af_type()),
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    /// Create a single-wire half-duplex Uart transceiver on a single Tx pin.
    ///
    /// See [`new_half_duplex_on_rx`][`Self::new_half_duplex_on_rx`] if you would prefer to use an Rx pin
    /// (when it is available for your chip). There is no functional difference between these methods, as both
    /// allow bidirectional communication.
    ///
    /// The TX pin is always released when no data is transmitted. Thus, it acts as a standard
    /// I/O in idle or in reception. It means that the I/O must be configured so that TX is
    /// configured as alternate function open-drain with an external pull-up
    /// Apart from this, the communication protocol is similar to normal USART mode. Any conflict
    /// on the line must be managed by software (for instance by using a centralized arbiter).
    #[doc(alias("HDSEL"))]
    pub fn new_half_duplex<T: Instance>(
        peri: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        mut config: Config,
        readback: HalfDuplexReadback,
    ) -> Result<Self, ConfigError> {
        #[cfg(not(any(usart_v1, usart_v2)))]
        {
            config.swap_rx_tx = false;
        }
        config.duplex = Duplex::Half(readback);

        Self::new_inner(
            peri,
            None,
            new_pin!(tx, config.tx_af()),
            None,
            None,
            None,
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    /// Create a single-wire half-duplex Uart transceiver on a single Rx pin.
    ///
    /// See [`new_half_duplex`][`Self::new_half_duplex`] if you would prefer to use an Tx pin.
    /// There is no functional difference between these methods, as both allow bidirectional communication.
    ///
    /// The pin is always released when no data is transmitted. Thus, it acts as a standard
    /// I/O in idle or in reception.
    /// Apart from this, the communication protocol is similar to normal USART mode. Any conflict
    /// on the line must be managed by software (for instance by using a centralized arbiter).
    #[cfg(not(any(usart_v1, usart_v2)))]
    #[doc(alias("HDSEL"))]
    pub fn new_half_duplex_on_rx<T: Instance>(
        peri: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        mut config: Config,
        readback: HalfDuplexReadback,
    ) -> Result<Self, ConfigError> {
        config.swap_rx_tx = true;
        config.duplex = Duplex::Half(readback);

        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            None,
            None,
            None,
            None,
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        rx: Option<Peri<'d, AnyPin>>,
        tx: Option<Peri<'d, AnyPin>>,
        rts: Option<Peri<'d, AnyPin>>,
        cts: Option<Peri<'d, AnyPin>>,
        de: Option<Peri<'d, AnyPin>>,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Result<Self, ConfigError> {
        let info = T::info();
        let state = T::buffered_state();
        let kernel_clock = T::frequency();

        state.half_duplex_readback.store(
            config.duplex == Duplex::Half(HalfDuplexReadback::Readback),
            Ordering::Relaxed,
        );

        let mut this = Self {
            rx: BufferedUartRx {
                info,
                state,
                kernel_clock,
                rx,
                rts,
                is_borrowed: false,
            },
            tx: BufferedUartTx {
                info,
                state,
                kernel_clock,
                tx,
                cts,
                de,
                is_borrowed: false,
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
        state.tx_rx_refcount.store(2, Ordering::Relaxed);

        info.rcc.enable_and_reset();

        let len = tx_buffer.len();
        unsafe { state.tx_buf.init(tx_buffer.as_mut_ptr(), len) };
        let len = rx_buffer.len();
        unsafe { state.rx_buf.init(rx_buffer.as_mut_ptr(), len) };

        info.regs.cr3().write(|w| {
            w.set_rtse(self.rx.rts.is_some());
            w.set_ctse(self.tx.cts.is_some());
            #[cfg(not(any(usart_v1, usart_v2)))]
            w.set_dem(self.tx.de.is_some());
            w.set_hdsel(config.duplex.is_half());
        });
        configure(info, self.rx.kernel_clock, &config, true, true)?;

        info.regs.cr1().modify(|w| {
            w.set_rxneie(true);
            w.set_idleie(true);

            if config.duplex.is_half() {
                // The te and re bits will be set by write, read and flush methods.
                // Receiver should be enabled by default for Half-Duplex.
                w.set_te(false);
                w.set_re(true);
            }
        });

        info.interrupt.unpend();
        unsafe { info.interrupt.enable() };

        Ok(())
    }

    /// Split the driver into a Tx and Rx part (useful for sending to separate tasks)
    pub fn split(self) -> (BufferedUartTx<'d>, BufferedUartRx<'d>) {
        (self.tx, self.rx)
    }

    /// Split the Uart into a transmitter and receiver,
    /// which is particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split_ref(&mut self) -> (BufferedUartTx<'_>, BufferedUartRx<'_>) {
        (
            BufferedUartTx {
                info: self.tx.info,
                state: self.tx.state,
                kernel_clock: self.tx.kernel_clock,
                tx: self.tx.tx.as_mut().map(Peri::reborrow),
                cts: self.tx.cts.as_mut().map(Peri::reborrow),
                de: self.tx.de.as_mut().map(Peri::reborrow),
                is_borrowed: true,
            },
            BufferedUartRx {
                info: self.rx.info,
                state: self.rx.state,
                kernel_clock: self.rx.kernel_clock,
                rx: self.rx.rx.as_mut().map(Peri::reborrow),
                rts: self.rx.rts.as_mut().map(Peri::reborrow),
                is_borrowed: true,
            },
        )
    }

    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        reconfigure(self.rx.info, self.rx.kernel_clock, config)?;

        self.rx.info.regs.cr1().modify(|w| {
            w.set_rxneie(true);
            w.set_idleie(true);
        });

        Ok(())
    }

    /// Send break character
    pub fn send_break(&self) {
        self.tx.send_break()
    }

    /// Set baudrate
    pub fn set_baudrate(&self, baudrate: u32) -> Result<(), ConfigError> {
        self.tx.set_baudrate(baudrate)?;
        self.rx.set_baudrate(baudrate)?;
        Ok(())
    }
}

impl<'d> BufferedUartRx<'d> {
    async fn read(&self, buf: &mut [u8]) -> Result<usize, Error> {
        poll_fn(move |cx| {
            let state = self.state;
            let mut rx_reader = unsafe { state.rx_buf.reader() };
            let data = rx_reader.pop_slice();

            if !data.is_empty() {
                let len = data.len().min(buf.len());
                buf[..len].copy_from_slice(&data[..len]);

                let do_pend = state.rx_buf.is_full();
                rx_reader.pop_done(len);

                if do_pend {
                    self.info.interrupt.pend();
                }

                return Poll::Ready(Ok(len));
            }

            state.rx_waker.register(cx.waker());
            Poll::Pending
        })
        .await
    }

    fn blocking_read(&self, buf: &mut [u8]) -> Result<usize, Error> {
        loop {
            let state = self.state;
            let mut rx_reader = unsafe { state.rx_buf.reader() };
            let data = rx_reader.pop_slice();

            if !data.is_empty() {
                let len = data.len().min(buf.len());
                buf[..len].copy_from_slice(&data[..len]);

                let do_pend = state.rx_buf.is_full();
                rx_reader.pop_done(len);

                if do_pend {
                    self.info.interrupt.pend();
                }

                return Ok(len);
            }
        }
    }

    async fn fill_buf(&self) -> Result<&[u8], Error> {
        poll_fn(move |cx| {
            let state = self.state;
            let mut rx_reader = unsafe { state.rx_buf.reader() };
            let (p, n) = rx_reader.pop_buf();
            if n == 0 {
                state.rx_waker.register(cx.waker());
                return Poll::Pending;
            }

            let buf = unsafe { slice::from_raw_parts(p, n) };
            Poll::Ready(Ok(buf))
        })
        .await
    }

    fn consume(&self, amt: usize) {
        let state = self.state;
        let mut rx_reader = unsafe { state.rx_buf.reader() };
        let full = state.rx_buf.is_full();
        rx_reader.pop_done(amt);
        if full {
            self.info.interrupt.pend();
        }
    }

    /// we are ready to read if there is data in the buffer
    fn read_ready(&mut self) -> Result<bool, Error> {
        let state = self.state;
        Ok(!state.rx_buf.is_empty())
    }

    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        reconfigure(self.info, self.kernel_clock, config)?;

        self.info.regs.cr1().modify(|w| {
            w.set_rxneie(true);
            w.set_idleie(true);
        });

        Ok(())
    }

    /// Set baudrate
    pub fn set_baudrate(&self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(self.info, self.kernel_clock, baudrate)
    }
}

impl<'d> BufferedUartTx<'d> {
    async fn write(&self, buf: &[u8]) -> Result<usize, Error> {
        poll_fn(move |cx| {
            let state = self.state;
            state.tx_done.store(false, Ordering::Release);

            let empty = state.tx_buf.is_empty();

            let mut tx_writer = unsafe { state.tx_buf.writer() };
            let data = tx_writer.push_slice();
            if data.is_empty() {
                state.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            let n = data.len().min(buf.len());
            data[..n].copy_from_slice(&buf[..n]);
            tx_writer.push_done(n);

            if empty {
                self.info.interrupt.pend();
            }

            Poll::Ready(Ok(n))
        })
        .await
    }

    async fn flush(&self) -> Result<(), Error> {
        poll_fn(move |cx| {
            let state = self.state;

            if !state.tx_done.load(Ordering::Acquire) {
                state.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            Poll::Ready(Ok(()))
        })
        .await
    }

    fn blocking_write(&self, buf: &[u8]) -> Result<usize, Error> {
        loop {
            let state = self.state;
            let empty = state.tx_buf.is_empty();

            let mut tx_writer = unsafe { state.tx_buf.writer() };
            let data = tx_writer.push_slice();
            if !data.is_empty() {
                let n = data.len().min(buf.len());
                data[..n].copy_from_slice(&buf[..n]);
                tx_writer.push_done(n);

                if empty {
                    self.info.interrupt.pend();
                }

                return Ok(n);
            }
        }
    }

    fn blocking_flush(&self) -> Result<(), Error> {
        loop {
            let state = self.state;
            if state.tx_buf.is_empty() {
                return Ok(());
            }
        }
    }

    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        reconfigure(self.info, self.kernel_clock, config)?;

        self.info.regs.cr1().modify(|w| {
            w.set_rxneie(true);
            w.set_idleie(true);
        });

        Ok(())
    }

    /// Send break character
    pub fn send_break(&self) {
        send_break(&self.info.regs);
    }

    /// Set baudrate
    pub fn set_baudrate(&self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(self.info, self.kernel_clock, baudrate)
    }
}

impl<'d> Drop for BufferedUartRx<'d> {
    fn drop(&mut self) {
        if !self.is_borrowed {
            let state = self.state;
            unsafe {
                state.rx_buf.deinit();

                // TX is inactive if the buffer is not available.
                // We can now unregister the interrupt handler
                if state.tx_buf.len() == 0 {
                    self.info.interrupt.disable();
                }
            }

            self.rx.as_ref().map(|x| x.set_as_disconnected());
            self.rts.as_ref().map(|x| x.set_as_disconnected());
            drop_tx_rx(self.info, state);
        }
    }
}

impl<'d> Drop for BufferedUartTx<'d> {
    fn drop(&mut self) {
        if !self.is_borrowed {
            let state = self.state;
            unsafe {
                state.tx_buf.deinit();

                // RX is inactive if the buffer is not available.
                // We can now unregister the interrupt handler
                if state.rx_buf.len() == 0 {
                    self.info.interrupt.disable();
                }
            }

            self.tx.as_ref().map(|x| x.set_as_disconnected());
            self.cts.as_ref().map(|x| x.set_as_disconnected());
            self.de.as_ref().map(|x| x.set_as_disconnected());
            drop_tx_rx(self.info, state);
        }
    }
}

fn drop_tx_rx(info: &Info, state: &State) {
    // We cannot use atomic subtraction here, because it's not supported for all targets
    let is_last_drop = critical_section::with(|_| {
        let refcount = state.tx_rx_refcount.load(Ordering::Relaxed);
        assert!(refcount >= 1);
        state.tx_rx_refcount.store(refcount - 1, Ordering::Relaxed);
        refcount == 1
    });
    if is_last_drop {
        info.rcc.disable();
    }
}

impl<'d> embedded_io_async::ErrorType for BufferedUart<'d> {
    type Error = Error;
}

impl<'d> embedded_io_async::ErrorType for BufferedUartRx<'d> {
    type Error = Error;
}

impl<'d> embedded_io_async::ErrorType for BufferedUartTx<'d> {
    type Error = Error;
}

impl<'d> embedded_io_async::Read for BufferedUart<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf).await
    }
}

impl<'d> embedded_io_async::Read for BufferedUartRx<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Self::read(self, buf).await
    }
}

impl<'d> embedded_io_async::ReadReady for BufferedUart<'d> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        BufferedUartRx::<'d>::read_ready(&mut self.rx)
    }
}

impl<'d> embedded_io_async::ReadReady for BufferedUartRx<'d> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        Self::read_ready(self)
    }
}

impl<'d> embedded_io_async::BufRead for BufferedUart<'d> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        self.rx.fill_buf().await
    }

    fn consume(&mut self, amt: usize) {
        self.rx.consume(amt)
    }
}

impl<'d> embedded_io_async::BufRead for BufferedUartRx<'d> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        Self::fill_buf(self).await
    }

    fn consume(&mut self, amt: usize) {
        Self::consume(self, amt)
    }
}

impl<'d> embedded_io_async::Write for BufferedUart<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush().await
    }
}

impl<'d> embedded_io_async::Write for BufferedUartTx<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Self::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Self::flush(self).await
    }
}

impl<'d> embedded_io::Read for BufferedUart<'d> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.blocking_read(buf)
    }
}

impl<'d> embedded_io::Read for BufferedUartRx<'d> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf)
    }
}

impl<'d> embedded_io::Write for BufferedUart<'d> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.blocking_write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.blocking_flush()
    }
}

impl<'d> embedded_io::Write for BufferedUartTx<'d> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Self::blocking_write(self, buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Self::blocking_flush(self)
    }
}

impl<'d> embedded_hal_02::serial::Read<u8> for BufferedUartRx<'d> {
    type Error = Error;

    fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
        let state = self.state;
        let mut rx_reader = unsafe { state.rx_buf.reader() };

        let do_pend = state.rx_buf.is_full();
        if let Some(data) = rx_reader.pop_one() {
            if do_pend {
                self.info.interrupt.pend();
            }
            Ok(data)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<'d> embedded_hal_02::blocking::serial::Write<u8> for BufferedUartTx<'d> {
    type Error = Error;

    fn bwrite_all(&mut self, mut buffer: &[u8]) -> Result<(), Self::Error> {
        while !buffer.is_empty() {
            match self.blocking_write(buffer) {
                Ok(0) => panic!("zero-length write."),
                Ok(n) => buffer = &buffer[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d> embedded_hal_02::serial::Read<u8> for BufferedUart<'d> {
    type Error = Error;

    fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
        embedded_hal_02::serial::Read::read(&mut self.rx)
    }
}

impl<'d> embedded_hal_02::blocking::serial::Write<u8> for BufferedUart<'d> {
    type Error = Error;

    fn bwrite_all(&mut self, mut buffer: &[u8]) -> Result<(), Self::Error> {
        while !buffer.is_empty() {
            match self.tx.blocking_write(buffer) {
                Ok(0) => panic!("zero-length write."),
                Ok(n) => buffer = &buffer[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.tx.blocking_flush()
    }
}

impl<'d> embedded_hal_nb::serial::ErrorType for BufferedUart<'d> {
    type Error = Error;
}

impl<'d> embedded_hal_nb::serial::ErrorType for BufferedUartTx<'d> {
    type Error = Error;
}

impl<'d> embedded_hal_nb::serial::ErrorType for BufferedUartRx<'d> {
    type Error = Error;
}

impl<'d> embedded_hal_nb::serial::Read for BufferedUartRx<'d> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        embedded_hal_02::serial::Read::read(self)
    }
}

impl<'d> embedded_hal_nb::serial::Write for BufferedUartTx<'d> {
    fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
        self.blocking_write(&[char]).map(drop).map_err(nb::Error::Other)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.blocking_flush().map_err(nb::Error::Other)
    }
}

impl<'d> embedded_hal_nb::serial::Read for BufferedUart<'d> {
    fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
        embedded_hal_02::serial::Read::read(&mut self.rx)
    }
}

impl<'d> embedded_hal_nb::serial::Write for BufferedUart<'d> {
    fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
        self.tx.blocking_write(&[char]).map(drop).map_err(nb::Error::Other)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.tx.blocking_flush().map_err(nb::Error::Other)
    }
}
