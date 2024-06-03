//! Universal Synchronous/Asynchronous Receiver Transmitter (USART, UART, LPUART)
#![macro_use]
#![warn(missing_docs)]

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, AtomicU8, Ordering};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::PeripheralRef;
use embassy_sync::waitqueue::AtomicWaker;
use futures_util::future::{select, Either};

use crate::dma::ChannelAndRequest;
use crate::gpio::{AFType, AnyPin, SealedPin};
use crate::interrupt::typelevel::Interrupt as _;
use crate::interrupt::{self, Interrupt, InterruptExt};
use crate::mode::{Async, Blocking, Mode};
#[allow(unused_imports)]
#[cfg(not(any(usart_v1, usart_v2)))]
use crate::pac::usart::regs::Isr as Sr;
#[cfg(any(usart_v1, usart_v2))]
use crate::pac::usart::regs::Sr;
#[cfg(not(any(usart_v1, usart_v2)))]
use crate::pac::usart::Lpuart as Regs;
#[cfg(any(usart_v1, usart_v2))]
use crate::pac::usart::Usart as Regs;
use crate::pac::usart::{regs, vals};
use crate::rcc::{self, RccInfo, SealedRccPeripheral};
use crate::time::Hertz;
use crate::Peripheral;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        on_interrupt(T::info().regs, T::state())
    }
}

unsafe fn on_interrupt(r: Regs, s: &'static State) {
    let (sr, cr1, cr3) = (sr(r).read(), r.cr1().read(), r.cr3().read());

    let has_errors = (sr.pe() && cr1.peie()) || ((sr.fe() || sr.ne() || sr.ore()) && cr3.eie());
    if has_errors {
        // clear all interrupts and DMA Rx Request
        r.cr1().modify(|w| {
            // disable RXNE interrupt
            w.set_rxneie(false);
            // disable parity interrupt
            w.set_peie(false);
            // disable idle line interrupt
            w.set_idleie(false);
        });
        r.cr3().modify(|w| {
            // disable Error Interrupt: (Frame error, Noise error, Overrun error)
            w.set_eie(false);
            // disable DMA Rx Request
            w.set_dmar(false);
        });
    } else if cr1.idleie() && sr.idle() {
        // IDLE detected: no more data will come
        r.cr1().modify(|w| {
            // disable idle line detection
            w.set_idleie(false);
        });
    } else if cr1.rxneie() {
        // We cannot check the RXNE flag as it is auto-cleared by the DMA controller

        // It is up to the listener to determine if this in fact was a RX event and disable the RXNE detection
    } else {
        return;
    }

    compiler_fence(Ordering::SeqCst);
    s.rx_waker.wake();
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Number of data bits
pub enum DataBits {
    /// 8 Data Bits
    DataBits8,
    /// 9 Data Bits
    DataBits9,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Parity
pub enum Parity {
    /// No parity
    ParityNone,
    /// Even Parity
    ParityEven,
    /// Odd Parity
    ParityOdd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Number of stop bits
pub enum StopBits {
    #[doc = "1 stop bit"]
    STOP1,
    #[doc = "0.5 stop bits"]
    STOP0P5,
    #[doc = "2 stop bits"]
    STOP2,
    #[doc = "1.5 stop bits"]
    STOP1P5,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Config Error
pub enum ConfigError {
    /// Baudrate too low
    BaudrateTooLow,
    /// Baudrate too high
    BaudrateTooHigh,
    /// Rx or Tx not enabled
    RxOrTxNotEnabled,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Config
pub struct Config {
    /// Baud rate
    pub baudrate: u32,
    /// Number of data bits
    pub data_bits: DataBits,
    /// Number of stop bits
    pub stop_bits: StopBits,
    /// Parity type
    pub parity: Parity,

    /// If true: on a read-like method, if there is a latent error pending,
    /// the read will abort and the error will be reported and cleared
    ///
    /// If false: the error is ignored and cleared
    pub detect_previous_overrun: bool,

    /// Set this to true if the line is considered noise free.
    /// This will increase the receiver’s tolerance to clock deviations,
    /// but will effectively disable noise detection.
    #[cfg(not(usart_v1))]
    pub assume_noise_free: bool,

    /// Set this to true to swap the RX and TX pins.
    #[cfg(any(usart_v3, usart_v4))]
    pub swap_rx_tx: bool,

    /// Set this to true to invert TX pin signal values (V<sub>DD</sub> =0/mark, Gnd = 1/idle).
    #[cfg(any(usart_v3, usart_v4))]
    pub invert_tx: bool,

    /// Set this to true to invert RX pin signal values (V<sub>DD</sub> =0/mark, Gnd = 1/idle).
    #[cfg(any(usart_v3, usart_v4))]
    pub invert_rx: bool,

    // private: set by new_half_duplex, not by the user.
    half_duplex: bool,
}

impl Config {
    fn tx_af(&self) -> AFType {
        #[cfg(any(usart_v3, usart_v4))]
        if self.swap_rx_tx {
            return AFType::Input;
        };
        AFType::OutputPushPull
    }
    fn rx_af(&self) -> AFType {
        #[cfg(any(usart_v3, usart_v4))]
        if self.swap_rx_tx {
            return AFType::OutputPushPull;
        };
        AFType::Input
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
            // historical behavior
            detect_previous_overrun: false,
            #[cfg(not(usart_v1))]
            assume_noise_free: false,
            #[cfg(any(usart_v3, usart_v4))]
            swap_rx_tx: false,
            #[cfg(any(usart_v3, usart_v4))]
            invert_tx: false,
            #[cfg(any(usart_v3, usart_v4))]
            invert_rx: false,
            half_duplex: false,
        }
    }
}

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
    /// Buffer too large for DMA
    BufferTooLong,
}

enum ReadCompletionEvent {
    // DMA Read transfer completed first
    DmaCompleted,
    // Idle line detected first
    Idle(usize),
}

/// Bidirectional UART Driver, which acts as a combination of [`UartTx`] and [`UartRx`].
///
/// ### Notes on [`embedded_io::Read`]
///
/// `embedded_io::Read` requires guarantees that the base [`UartRx`] cannot provide.
///
/// See [`UartRx`] for more details, and see [`BufferedUart`] and [`RingBufferedUartRx`]
/// as alternatives that do provide the necessary guarantees for `embedded_io::Read`.
pub struct Uart<'d, M: Mode> {
    tx: UartTx<'d, M>,
    rx: UartRx<'d, M>,
}

impl<'d, M: Mode> SetConfig for Uart<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.tx.set_config(config)?;
        self.rx.set_config(config)
    }
}

/// Tx-only UART Driver.
///
/// Can be obtained from [`Uart::split`], or can be constructed independently,
/// if you do not need the receiving half of the driver.
pub struct UartTx<'d, M: Mode> {
    info: &'static Info,
    state: &'static State,
    kernel_clock: Hertz,
    tx: Option<PeripheralRef<'d, AnyPin>>,
    cts: Option<PeripheralRef<'d, AnyPin>>,
    de: Option<PeripheralRef<'d, AnyPin>>,
    tx_dma: Option<ChannelAndRequest<'d>>,
    _phantom: PhantomData<M>,
}

impl<'d, M: Mode> SetConfig for UartTx<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

/// Rx-only UART Driver.
///
/// Can be obtained from [`Uart::split`], or can be constructed independently,
/// if you do not need the transmitting half of the driver.
///
/// ### Notes on [`embedded_io::Read`]
///
/// `embedded_io::Read` requires guarantees that this struct cannot provide:
///
/// - Any data received between calls to [`UartRx::read`] or [`UartRx::blocking_read`]
/// will be thrown away, as `UartRx` is unbuffered.
/// Users of `embedded_io::Read` are likely to not expect this behavior
/// (for instance if they read multiple small chunks in a row).
/// - [`UartRx::read`] and [`UartRx::blocking_read`] only return once the entire buffer has been
/// filled, whereas `embedded_io::Read` requires us to fill the buffer with what we already
/// received, and only block/wait until the first byte arrived.
/// <br />
/// While [`UartRx::read_until_idle`] does return early, it will still eagerly wait for data until
/// the buffer is full or no data has been transmitted in a while,
/// which may not be what users of `embedded_io::Read` expect.
///
/// [`UartRx::into_ring_buffered`] can be called to equip `UartRx` with a buffer,
/// that it can then use to store data received between calls to `read`,
/// provided you are using DMA already.
///
/// Alternatively, you can use [`BufferedUartRx`], which is interrupt-based and which can also
/// store data received between calls.
///
/// Also see [this github comment](https://github.com/embassy-rs/embassy/pull/2185#issuecomment-1810047043).
pub struct UartRx<'d, M: Mode> {
    info: &'static Info,
    state: &'static State,
    kernel_clock: Hertz,
    rx: Option<PeripheralRef<'d, AnyPin>>,
    rts: Option<PeripheralRef<'d, AnyPin>>,
    rx_dma: Option<ChannelAndRequest<'d>>,
    detect_previous_overrun: bool,
    #[cfg(any(usart_v1, usart_v2))]
    buffered_sr: stm32_metapac::usart::regs::Sr,
    _phantom: PhantomData<M>,
}

impl<'d, M: Mode> SetConfig for UartRx<'d, M> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d> UartTx<'d, Async> {
    /// Useful if you only want Uart Tx. It saves 1 pin and consumes a little less power.
    pub fn new<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(tx, AFType::OutputPushPull),
            None,
            new_dma!(tx_dma),
            config,
        )
    }

    /// Create a new tx-only UART with a clear-to-send pin
    pub fn new_with_cts<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(tx, AFType::OutputPushPull),
            new_pin!(cts, AFType::Input),
            new_dma!(tx_dma),
            config,
        )
    }

    /// Initiate an asynchronous UART write
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs;

        // Disable Receiver for Half-Duplex mode
        if r.cr3().read().hdsel() {
            r.cr1().modify(|reg| reg.set_re(false));
        }

        let ch = self.tx_dma.as_mut().unwrap();
        r.cr3().modify(|reg| {
            reg.set_dmat(true);
        });
        // If we don't assign future to a variable, the data register pointer
        // is held across an await and makes the future non-Send.
        let transfer = unsafe { ch.write(buffer, tdr(r), Default::default()) };
        transfer.await;
        Ok(())
    }

    /// Wait until transmission complete
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.blocking_flush()
    }
}

impl<'d> UartTx<'d, Blocking> {
    /// Create a new blocking tx-only UART with no hardware flow control.
    ///
    /// Useful if you only want Uart Tx. It saves 1 pin and consumes a little less power.
    pub fn new_blocking<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(peri, new_pin!(tx, AFType::OutputPushPull), None, None, config)
    }

    /// Create a new blocking tx-only UART with a clear-to-send pin
    pub fn new_blocking_with_cts<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(tx, AFType::OutputPushPull),
            new_pin!(cts, AFType::Input),
            None,
            config,
        )
    }
}

impl<'d, M: Mode> UartTx<'d, M> {
    fn new_inner<T: Instance>(
        _peri: impl Peripheral<P = T> + 'd,
        tx: Option<PeripheralRef<'d, AnyPin>>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
        tx_dma: Option<ChannelAndRequest<'d>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        rcc::enable_and_reset::<T>();

        let info = T::info();
        let state = T::state();
        let kernel_clock = T::frequency();
        let r = info.regs;
        r.cr3().modify(|w| {
            w.set_ctse(cts.is_some());
        });
        configure(info, kernel_clock, &config, false, true)?;

        state.tx_rx_refcount.store(1, Ordering::Relaxed);

        Ok(Self {
            info,
            state,
            kernel_clock,
            tx,
            cts,
            de: None,
            tx_dma,
            _phantom: PhantomData,
        })
    }

    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        reconfigure(self.info, self.kernel_clock, config)
    }

    /// Perform a blocking UART write
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs;

        // Disable Receiver for Half-Duplex mode
        if r.cr3().read().hdsel() {
            r.cr1().modify(|reg| reg.set_re(false));
        }

        for &b in buffer {
            while !sr(r).read().txe() {}
            unsafe { tdr(r).write_volatile(b) };
        }
        Ok(())
    }

    /// Block until transmission complete
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        blocking_flush(self.info)
    }
}

fn blocking_flush(info: &Info) -> Result<(), Error> {
    let r = info.regs;
    while !sr(r).read().tc() {}

    // Enable Receiver after transmission complete for Half-Duplex mode
    if r.cr3().read().hdsel() {
        r.cr1().modify(|reg| reg.set_re(true));
    }

    Ok(())
}

impl<'d> UartRx<'d, Async> {
    /// Create a new rx-only UART with no hardware flow control.
    ///
    /// Useful if you only want Uart Rx. It saves 1 pin and consumes a little less power.
    pub fn new<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(peri, new_pin!(rx, AFType::Input), None, new_dma!(rx_dma), config)
    }

    /// Create a new rx-only UART with a request-to-send pin
    pub fn new_with_rts<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, AFType::Input),
            new_pin!(rts, AFType::OutputPushPull),
            new_dma!(rx_dma),
            config,
        )
    }

    /// Initiate an asynchronous UART read
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.inner_read(buffer, false).await?;

        Ok(())
    }

    /// Initiate an asynchronous read with idle line detection enabled
    pub async fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.inner_read(buffer, true).await
    }

    async fn inner_read_run(
        &mut self,
        buffer: &mut [u8],
        enable_idle_line_detection: bool,
    ) -> Result<ReadCompletionEvent, Error> {
        let r = self.info.regs;

        // Call flush for Half-Duplex mode. It prevents reading of bytes which have just been written.
        if r.cr3().read().hdsel() {
            blocking_flush(self.info)?;
        }

        // make sure USART state is restored to neutral state when this future is dropped
        let on_drop = OnDrop::new(move || {
            // defmt::trace!("Clear all USART interrupts and DMA Read Request");
            // clear all interrupts and DMA Rx Request
            r.cr1().modify(|w| {
                // disable RXNE interrupt
                w.set_rxneie(false);
                // disable parity interrupt
                w.set_peie(false);
                // disable idle line interrupt
                w.set_idleie(false);
            });
            r.cr3().modify(|w| {
                // disable Error Interrupt: (Frame error, Noise error, Overrun error)
                w.set_eie(false);
                // disable DMA Rx Request
                w.set_dmar(false);
            });
        });

        let ch = self.rx_dma.as_mut().unwrap();

        let buffer_len = buffer.len();

        // Start USART DMA
        // will not do anything yet because DMAR is not yet set
        // future which will complete when DMA Read request completes
        let transfer = unsafe { ch.read(rdr(r), buffer, Default::default()) };

        // clear ORE flag just before enabling DMA Rx Request: can be mandatory for the second transfer
        if !self.detect_previous_overrun {
            let sr = sr(r).read();
            // This read also clears the error and idle interrupt flags on v1.
            unsafe { rdr(r).read_volatile() };
            clear_interrupt_flags(r, sr);
        }

        r.cr1().modify(|w| {
            // disable RXNE interrupt
            w.set_rxneie(false);
            // enable parity interrupt if not ParityNone
            w.set_peie(w.pce());
        });

        r.cr3().modify(|w| {
            // enable Error Interrupt: (Frame error, Noise error, Overrun error)
            w.set_eie(true);
            // enable DMA Rx Request
            w.set_dmar(true);
        });

        compiler_fence(Ordering::SeqCst);

        // In case of errors already pending when reception started, interrupts may have already been raised
        // and lead to reception abortion (Overrun error for instance). In such a case, all interrupts
        // have been disabled in interrupt handler and DMA Rx Request has been disabled.

        let cr3 = r.cr3().read();

        if !cr3.dmar() {
            // something went wrong
            // because the only way to get this flag cleared is to have an interrupt

            // DMA will be stopped when transfer is dropped

            let sr = sr(r).read();
            // This read also clears the error and idle interrupt flags on v1.
            unsafe { rdr(r).read_volatile() };
            clear_interrupt_flags(r, sr);

            if sr.pe() {
                return Err(Error::Parity);
            }
            if sr.fe() {
                return Err(Error::Framing);
            }
            if sr.ne() {
                return Err(Error::Noise);
            }
            if sr.ore() {
                return Err(Error::Overrun);
            }

            unreachable!();
        }

        if enable_idle_line_detection {
            // clear idle flag
            let sr = sr(r).read();
            // This read also clears the error and idle interrupt flags on v1.
            unsafe { rdr(r).read_volatile() };
            clear_interrupt_flags(r, sr);

            // enable idle interrupt
            r.cr1().modify(|w| {
                w.set_idleie(true);
            });
        }

        compiler_fence(Ordering::SeqCst);

        // future which completes when idle line or error is detected
        let s = self.state;
        let abort = poll_fn(move |cx| {
            s.rx_waker.register(cx.waker());

            let sr = sr(r).read();

            // This read also clears the error and idle interrupt flags on v1.
            unsafe { rdr(r).read_volatile() };
            clear_interrupt_flags(r, sr);

            if enable_idle_line_detection {
                // enable idle interrupt
                r.cr1().modify(|w| {
                    w.set_idleie(true);
                });
            }

            compiler_fence(Ordering::SeqCst);

            let has_errors = sr.pe() || sr.fe() || sr.ne() || sr.ore();

            if has_errors {
                // all Rx interrupts and Rx DMA Request have already been cleared in interrupt handler

                if sr.pe() {
                    return Poll::Ready(Err(Error::Parity));
                }
                if sr.fe() {
                    return Poll::Ready(Err(Error::Framing));
                }
                if sr.ne() {
                    return Poll::Ready(Err(Error::Noise));
                }
                if sr.ore() {
                    return Poll::Ready(Err(Error::Overrun));
                }
            }

            if enable_idle_line_detection && sr.idle() {
                // Idle line detected
                return Poll::Ready(Ok(()));
            }

            Poll::Pending
        });

        // wait for the first of DMA request or idle line detected to completes
        // select consumes its arguments
        // when transfer is dropped, it will stop the DMA request
        let r = match select(transfer, abort).await {
            // DMA transfer completed first
            Either::Left(((), _)) => Ok(ReadCompletionEvent::DmaCompleted),

            // Idle line detected first
            Either::Right((Ok(()), transfer)) => Ok(ReadCompletionEvent::Idle(
                buffer_len - transfer.get_remaining_transfers() as usize,
            )),

            // error occurred
            Either::Right((Err(e), _)) => Err(e),
        };

        drop(on_drop);

        r
    }

    async fn inner_read(&mut self, buffer: &mut [u8], enable_idle_line_detection: bool) -> Result<usize, Error> {
        if buffer.is_empty() {
            return Ok(0);
        } else if buffer.len() > 0xFFFF {
            return Err(Error::BufferTooLong);
        }

        let buffer_len = buffer.len();

        // wait for DMA to complete or IDLE line detection if requested
        let res = self.inner_read_run(buffer, enable_idle_line_detection).await;

        match res {
            Ok(ReadCompletionEvent::DmaCompleted) => Ok(buffer_len),
            Ok(ReadCompletionEvent::Idle(n)) => Ok(n),
            Err(e) => Err(e),
        }
    }
}

impl<'d> UartRx<'d, Blocking> {
    /// Create a new rx-only UART with no hardware flow control.
    ///
    /// Useful if you only want Uart Rx. It saves 1 pin and consumes a little less power.
    pub fn new_blocking<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(peri, new_pin!(rx, AFType::Input), None, None, config)
    }

    /// Create a new rx-only UART with a request-to-send pin
    pub fn new_blocking_with_rts<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, AFType::Input),
            new_pin!(rts, AFType::OutputPushPull),
            None,
            config,
        )
    }
}

impl<'d, M: Mode> UartRx<'d, M> {
    fn new_inner<T: Instance>(
        _peri: impl Peripheral<P = T> + 'd,
        rx: Option<PeripheralRef<'d, AnyPin>>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        rx_dma: Option<ChannelAndRequest<'d>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        rcc::enable_and_reset::<T>();

        let info = T::info();
        let state = T::state();
        let kernel_clock = T::frequency();
        let r = info.regs;
        r.cr3().write(|w| {
            w.set_rtse(rts.is_some());
        });
        configure(info, kernel_clock, &config, true, false)?;

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        state.tx_rx_refcount.store(1, Ordering::Relaxed);

        Ok(Self {
            _phantom: PhantomData,
            info,
            state,
            kernel_clock,
            rx,
            rts,
            rx_dma,
            detect_previous_overrun: config.detect_previous_overrun,
            #[cfg(any(usart_v1, usart_v2))]
            buffered_sr: stm32_metapac::usart::regs::Sr(0),
        })
    }

    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        reconfigure(self.info, self.kernel_clock, config)
    }

    #[cfg(any(usart_v1, usart_v2))]
    fn check_rx_flags(&mut self) -> Result<bool, Error> {
        let r = self.info.regs;
        loop {
            // Handle all buffered error flags.
            if self.buffered_sr.pe() {
                self.buffered_sr.set_pe(false);
                return Err(Error::Parity);
            } else if self.buffered_sr.fe() {
                self.buffered_sr.set_fe(false);
                return Err(Error::Framing);
            } else if self.buffered_sr.ne() {
                self.buffered_sr.set_ne(false);
                return Err(Error::Noise);
            } else if self.buffered_sr.ore() {
                self.buffered_sr.set_ore(false);
                return Err(Error::Overrun);
            } else if self.buffered_sr.rxne() {
                self.buffered_sr.set_rxne(false);
                return Ok(true);
            } else {
                // No error flags from previous iterations were set: Check the actual status register
                let sr = r.sr().read();
                if !sr.rxne() {
                    return Ok(false);
                }

                // Buffer the status register and let the loop handle the error flags.
                self.buffered_sr = sr;
            }
        }
    }

    #[cfg(any(usart_v3, usart_v4))]
    fn check_rx_flags(&mut self) -> Result<bool, Error> {
        let r = self.info.regs;
        let sr = r.isr().read();
        if sr.pe() {
            r.icr().write(|w| w.set_pe(true));
            return Err(Error::Parity);
        } else if sr.fe() {
            r.icr().write(|w| w.set_fe(true));
            return Err(Error::Framing);
        } else if sr.ne() {
            r.icr().write(|w| w.set_ne(true));
            return Err(Error::Noise);
        } else if sr.ore() {
            r.icr().write(|w| w.set_ore(true));
            return Err(Error::Overrun);
        }
        Ok(sr.rxne())
    }

    /// Read a single u8 if there is one available, otherwise return WouldBlock
    pub(crate) fn nb_read(&mut self) -> Result<u8, nb::Error<Error>> {
        let r = self.info.regs;
        if self.check_rx_flags()? {
            Ok(unsafe { rdr(r).read_volatile() })
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    /// Perform a blocking read into `buffer`
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        let r = self.info.regs;

        // Call flush for Half-Duplex mode. It prevents reading of bytes which have just been written.
        if r.cr3().read().hdsel() {
            blocking_flush(self.info)?;
        }

        for b in buffer {
            while !self.check_rx_flags()? {}
            unsafe { *b = rdr(r).read_volatile() }
        }
        Ok(())
    }
}

impl<'d, M: Mode> Drop for UartTx<'d, M> {
    fn drop(&mut self) {
        self.tx.as_ref().map(|x| x.set_as_disconnected());
        self.cts.as_ref().map(|x| x.set_as_disconnected());
        self.de.as_ref().map(|x| x.set_as_disconnected());
        drop_tx_rx(self.info, self.state);
    }
}

impl<'d, M: Mode> Drop for UartRx<'d, M> {
    fn drop(&mut self) {
        self.rx.as_ref().map(|x| x.set_as_disconnected());
        self.rts.as_ref().map(|x| x.set_as_disconnected());
        drop_tx_rx(self.info, self.state);
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

impl<'d> Uart<'d, Async> {
    /// Create a new bidirectional UART
    pub fn new<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
        rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            new_pin!(tx, config.tx_af()),
            None,
            None,
            None,
            new_dma!(tx_dma),
            new_dma!(rx_dma),
            config,
        )
    }

    /// Create a new bidirectional UART with request-to-send and clear-to-send pins
    pub fn new_with_rtscts<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
        rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            new_pin!(tx, config.tx_af()),
            new_pin!(rts, AFType::OutputPushPull),
            new_pin!(cts, AFType::Input),
            None,
            new_dma!(tx_dma),
            new_dma!(rx_dma),
            config,
        )
    }

    #[cfg(not(any(usart_v1, usart_v2)))]
    /// Create a new bidirectional UART with a driver-enable pin
    pub fn new_with_de<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        de: impl Peripheral<P = impl DePin<T>> + 'd,
        tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
        rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            new_pin!(tx, config.tx_af()),
            None,
            None,
            new_pin!(de, AFType::OutputPushPull),
            new_dma!(tx_dma),
            new_dma!(rx_dma),
            config,
        )
    }

    /// Create a single-wire half-duplex Uart transceiver on a single Tx pin.
    ///
    /// See [`new_half_duplex_on_rx`][`Self::new_half_duplex_on_rx`] if you would prefer to use an Rx pin
    /// (when it is available for your chip). There is no functional difference between these methods, as both
    /// allow bidirectional communication.
    ///
    /// The pin is always released when no data is transmitted. Thus, it acts as a standard
    /// I/O in idle or in reception.
    /// Apart from this, the communication protocol is similar to normal USART mode. Any conflict
    /// on the line must be managed by software (for instance by using a centralized arbiter).
    #[doc(alias("HDSEL"))]
    pub fn new_half_duplex<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
        rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
        mut config: Config,
    ) -> Result<Self, ConfigError> {
        #[cfg(not(any(usart_v1, usart_v2)))]
        {
            config.swap_rx_tx = false;
        }
        config.half_duplex = true;

        Self::new_inner(
            peri,
            None,
            new_pin!(tx, AFType::OutputPushPull),
            None,
            None,
            None,
            new_dma!(tx_dma),
            new_dma!(rx_dma),
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
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
        rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
        mut config: Config,
    ) -> Result<Self, ConfigError> {
        config.swap_rx_tx = true;
        config.half_duplex = true;

        Self::new_inner(
            peri,
            None,
            None,
            new_pin!(rx, AFType::OutputPushPull),
            None,
            None,
            new_dma!(tx_dma),
            new_dma!(rx_dma),
            config,
        )
    }

    /// Perform an asynchronous write
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    /// Perform an asynchronous read into `buffer`
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }

    /// Perform an an asynchronous read with idle line detection enabled
    pub async fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.rx.read_until_idle(buffer).await
    }
}

impl<'d> Uart<'d, Blocking> {
    /// Create a new blocking bidirectional UART.
    pub fn new_blocking<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            new_pin!(tx, config.tx_af()),
            None,
            None,
            None,
            None,
            None,
            config,
        )
    }

    /// Create a new bidirectional UART with request-to-send and clear-to-send pins
    pub fn new_blocking_with_rtscts<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            new_pin!(tx, config.tx_af()),
            new_pin!(rts, AFType::OutputPushPull),
            new_pin!(cts, AFType::Input),
            None,
            None,
            None,
            config,
        )
    }

    #[cfg(not(any(usart_v1, usart_v2)))]
    /// Create a new bidirectional UART with a driver-enable pin
    pub fn new_blocking_with_de<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        de: impl Peripheral<P = impl DePin<T>> + 'd,
        config: Config,
    ) -> Result<Self, ConfigError> {
        Self::new_inner(
            peri,
            new_pin!(rx, config.rx_af()),
            new_pin!(tx, config.tx_af()),
            None,
            None,
            new_pin!(de, AFType::OutputPushPull),
            None,
            None,
            config,
        )
    }

    /// Create a single-wire half-duplex Uart transceiver on a single Tx pin.
    ///
    /// See [`new_half_duplex_on_rx`][`Self::new_half_duplex_on_rx`] if you would prefer to use an Rx pin
    /// (when it is available for your chip). There is no functional difference between these methods, as both
    /// allow bidirectional communication.
    ///
    /// The pin is always released when no data is transmitted. Thus, it acts as a standard
    /// I/O in idle or in reception.
    /// Apart from this, the communication protocol is similar to normal USART mode. Any conflict
    /// on the line must be managed by software (for instance by using a centralized arbiter).
    #[doc(alias("HDSEL"))]
    pub fn new_blocking_half_duplex<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        mut config: Config,
    ) -> Result<Self, ConfigError> {
        #[cfg(not(any(usart_v1, usart_v2)))]
        {
            config.swap_rx_tx = false;
        }
        config.half_duplex = true;

        Self::new_inner(
            peri,
            None,
            new_pin!(tx, AFType::OutputPushPull),
            None,
            None,
            None,
            None,
            None,
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
    pub fn new_blocking_half_duplex_on_rx<T: Instance>(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        mut config: Config,
    ) -> Result<Self, ConfigError> {
        config.swap_rx_tx = true;
        config.half_duplex = true;

        Self::new_inner(
            peri,
            None,
            None,
            new_pin!(rx, AFType::OutputPushPull),
            None,
            None,
            None,
            None,
            config,
        )
    }
}

impl<'d, M: Mode> Uart<'d, M> {
    fn new_inner<T: Instance>(
        _peri: impl Peripheral<P = T> + 'd,
        rx: Option<PeripheralRef<'d, AnyPin>>,
        tx: Option<PeripheralRef<'d, AnyPin>>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
        de: Option<PeripheralRef<'d, AnyPin>>,
        tx_dma: Option<ChannelAndRequest<'d>>,
        rx_dma: Option<ChannelAndRequest<'d>>,
        config: Config,
    ) -> Result<Self, ConfigError> {
        rcc::enable_and_reset::<T>();

        let info = T::info();
        let state = T::state();
        let kernel_clock = T::frequency();
        let r = info.regs;

        r.cr3().write(|w| {
            w.set_rtse(rts.is_some());
            w.set_ctse(cts.is_some());
            #[cfg(not(any(usart_v1, usart_v2)))]
            w.set_dem(de.is_some());
        });
        configure(info, kernel_clock, &config, true, true)?;

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        state.tx_rx_refcount.store(2, Ordering::Relaxed);

        Ok(Self {
            tx: UartTx {
                _phantom: PhantomData,
                info,
                state,
                kernel_clock,
                tx,
                cts,
                de,
                tx_dma,
            },
            rx: UartRx {
                _phantom: PhantomData,
                info,
                state,
                kernel_clock,
                rx,
                rts,
                rx_dma,
                detect_previous_overrun: config.detect_previous_overrun,
                #[cfg(any(usart_v1, usart_v2))]
                buffered_sr: stm32_metapac::usart::regs::Sr(0),
            },
        })
    }

    /// Perform a blocking write
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    /// Block until transmission complete
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Read a single `u8` or return `WouldBlock`
    pub(crate) fn nb_read(&mut self) -> Result<u8, nb::Error<Error>> {
        self.rx.nb_read()
    }

    /// Perform a blocking read into `buffer`
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Split the Uart into a transmitter and receiver, which is
    /// particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split(self) -> (UartTx<'d, M>, UartRx<'d, M>) {
        (self.tx, self.rx)
    }
}

fn reconfigure(info: &Info, kernel_clock: Hertz, config: &Config) -> Result<(), ConfigError> {
    info.interrupt.disable();
    let r = info.regs;

    let cr = r.cr1().read();
    configure(info, kernel_clock, config, cr.re(), cr.te())?;

    info.interrupt.unpend();
    unsafe { info.interrupt.enable() };

    Ok(())
}

fn configure(
    info: &Info,
    kernel_clock: Hertz,
    config: &Config,
    enable_rx: bool,
    enable_tx: bool,
) -> Result<(), ConfigError> {
    let r = info.regs;
    let kind = info.kind;

    if !enable_rx && !enable_tx {
        return Err(ConfigError::RxOrTxNotEnabled);
    }

    #[cfg(not(usart_v4))]
    static DIVS: [(u16, ()); 1] = [(1, ())];

    #[cfg(usart_v4)]
    static DIVS: [(u16, vals::Presc); 12] = [
        (1, vals::Presc::DIV1),
        (2, vals::Presc::DIV2),
        (4, vals::Presc::DIV4),
        (6, vals::Presc::DIV6),
        (8, vals::Presc::DIV8),
        (10, vals::Presc::DIV10),
        (12, vals::Presc::DIV12),
        (16, vals::Presc::DIV16),
        (32, vals::Presc::DIV32),
        (64, vals::Presc::DIV64),
        (128, vals::Presc::DIV128),
        (256, vals::Presc::DIV256),
    ];

    let (mul, brr_min, brr_max) = match kind {
        #[cfg(any(usart_v3, usart_v4))]
        Kind::Lpuart => {
            trace!("USART: Kind::Lpuart");
            (256, 0x300, 0x10_0000)
        }
        Kind::Uart => {
            trace!("USART: Kind::Uart");
            (1, 0x10, 0x1_0000)
        }
    };

    fn calculate_brr(baud: u32, pclk: u32, presc: u32, mul: u32) -> u32 {
        // The calculation to be done to get the BRR is `mul * pclk / presc / baud`
        // To do this in 32-bit only we can't multiply `mul` and `pclk`
        let clock = pclk / presc;

        // The mul is applied as the last operation to prevent overflow
        let brr = clock / baud * mul;

        // The BRR calculation will be a bit off because of integer rounding.
        // Because we multiplied our inaccuracy with mul, our rounding now needs to be in proportion to mul.
        let rounding = ((clock % baud) * mul + (baud / 2)) / baud;

        brr + rounding
    }

    // UART must be disabled during configuration.
    r.cr1().modify(|w| {
        w.set_ue(false);
    });

    #[cfg(not(usart_v1))]
    let mut over8 = false;
    let mut found_brr = None;
    for &(presc, _presc_val) in &DIVS {
        let brr = calculate_brr(config.baudrate, kernel_clock.0, presc as u32, mul);
        trace!(
            "USART: presc={}, div=0x{:08x} (mantissa = {}, fraction = {})",
            presc,
            brr,
            brr >> 4,
            brr & 0x0F
        );

        if brr < brr_min {
            #[cfg(not(usart_v1))]
            if brr * 2 >= brr_min && kind == Kind::Uart && !cfg!(usart_v1) {
                over8 = true;
                r.brr().write_value(regs::Brr(((brr << 1) & !0xF) | (brr & 0x07)));
                #[cfg(usart_v4)]
                r.presc().write(|w| w.set_prescaler(_presc_val));
                found_brr = Some(brr);
                break;
            }
            return Err(ConfigError::BaudrateTooHigh);
        }

        if brr < brr_max {
            r.brr().write_value(regs::Brr(brr));
            #[cfg(usart_v4)]
            r.presc().write(|w| w.set_prescaler(_presc_val));
            found_brr = Some(brr);
            break;
        }
    }

    let brr = found_brr.ok_or(ConfigError::BaudrateTooLow)?;

    #[cfg(not(usart_v1))]
    let oversampling = if over8 { "8 bit" } else { "16 bit" };
    #[cfg(usart_v1)]
    let oversampling = "default";
    trace!(
        "Using {} oversampling, desired baudrate: {}, actual baudrate: {}",
        oversampling,
        config.baudrate,
        kernel_clock.0 / brr * mul
    );

    r.cr2().write(|w| {
        w.set_stop(match config.stop_bits {
            StopBits::STOP0P5 => vals::Stop::STOP0P5,
            StopBits::STOP1 => vals::Stop::STOP1,
            StopBits::STOP1P5 => vals::Stop::STOP1P5,
            StopBits::STOP2 => vals::Stop::STOP2,
        });

        #[cfg(any(usart_v3, usart_v4))]
        {
            w.set_txinv(config.invert_tx);
            w.set_rxinv(config.invert_rx);
            w.set_swap(config.swap_rx_tx);
        }
    });

    r.cr3().modify(|w| {
        #[cfg(not(usart_v1))]
        w.set_onebit(config.assume_noise_free);
        w.set_hdsel(config.half_duplex);
    });

    r.cr1().write(|w| {
        // enable uart
        w.set_ue(true);
        // enable transceiver
        w.set_te(enable_tx);
        // enable receiver
        w.set_re(enable_rx);
        // configure word size
        // if using odd or even parity it must be configured to 9bits
        w.set_m0(if config.parity != Parity::ParityNone {
            trace!("USART: m0: vals::M0::BIT9");
            vals::M0::BIT9
        } else {
            trace!("USART: m0: vals::M0::BIT8");
            vals::M0::BIT8
        });
        // configure parity
        w.set_pce(config.parity != Parity::ParityNone);
        w.set_ps(match config.parity {
            Parity::ParityOdd => {
                trace!("USART: set_ps: vals::Ps::ODD");
                vals::Ps::ODD
            }
            Parity::ParityEven => {
                trace!("USART: set_ps: vals::Ps::EVEN");
                vals::Ps::EVEN
            }
            _ => {
                trace!("USART: set_ps: vals::Ps::EVEN");
                vals::Ps::EVEN
            }
        });
        #[cfg(not(usart_v1))]
        w.set_over8(vals::Over8::from_bits(over8 as _));
        #[cfg(usart_v4)]
        {
            trace!("USART: set_fifoen: true (usart_v4)");
            w.set_fifoen(true);
        }
    });

    Ok(())
}

impl<'d, M: Mode> embedded_hal_02::serial::Read<u8> for UartRx<'d, M> {
    type Error = Error;
    fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
        self.nb_read()
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for UartTx<'d, M> {
    type Error = Error;
    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }
    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d, M: Mode> embedded_hal_02::serial::Read<u8> for Uart<'d, M> {
    type Error = Error;
    fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
        self.nb_read()
    }
}

impl<'d, M: Mode> embedded_hal_02::blocking::serial::Write<u8> for Uart<'d, M> {
    type Error = Error;
    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(buffer)
    }
    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl embedded_hal_nb::serial::Error for Error {
    fn kind(&self) -> embedded_hal_nb::serial::ErrorKind {
        match *self {
            Self::Framing => embedded_hal_nb::serial::ErrorKind::FrameFormat,
            Self::Noise => embedded_hal_nb::serial::ErrorKind::Noise,
            Self::Overrun => embedded_hal_nb::serial::ErrorKind::Overrun,
            Self::Parity => embedded_hal_nb::serial::ErrorKind::Parity,
            Self::BufferTooLong => embedded_hal_nb::serial::ErrorKind::Other,
        }
    }
}

impl<'d, M: Mode> embedded_hal_nb::serial::ErrorType for Uart<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_hal_nb::serial::ErrorType for UartTx<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_hal_nb::serial::ErrorType for UartRx<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_hal_nb::serial::Read for UartRx<'d, M> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.nb_read()
    }
}

impl<'d, M: Mode> embedded_hal_nb::serial::Write for UartTx<'d, M> {
    fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
        self.blocking_write(&[char]).map_err(nb::Error::Other)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.blocking_flush().map_err(nb::Error::Other)
    }
}

impl<'d, M: Mode> embedded_hal_nb::serial::Read for Uart<'d, M> {
    fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
        self.nb_read()
    }
}

impl<'d, M: Mode> embedded_hal_nb::serial::Write for Uart<'d, M> {
    fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
        self.blocking_write(&[char]).map_err(nb::Error::Other)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.blocking_flush().map_err(nb::Error::Other)
    }
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl<M: Mode> embedded_io::ErrorType for Uart<'_, M> {
    type Error = Error;
}

impl<M: Mode> embedded_io::ErrorType for UartTx<'_, M> {
    type Error = Error;
}

impl<M: Mode> embedded_io::Write for Uart<'_, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<M: Mode> embedded_io::Write for UartTx<'_, M> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl embedded_io_async::Write for Uart<'_, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write(buf).await?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl embedded_io_async::Write for UartTx<'_, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write(buf).await?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

pub use buffered::*;

pub use crate::usart::buffered::InterruptHandler as BufferedInterruptHandler;
mod buffered;

#[cfg(not(gpdma))]
mod ringbuffered;
#[cfg(not(gpdma))]
pub use ringbuffered::RingBufferedUartRx;

#[cfg(any(usart_v1, usart_v2))]
fn tdr(r: crate::pac::usart::Usart) -> *mut u8 {
    r.dr().as_ptr() as _
}

#[cfg(any(usart_v1, usart_v2))]
fn rdr(r: crate::pac::usart::Usart) -> *mut u8 {
    r.dr().as_ptr() as _
}

#[cfg(any(usart_v1, usart_v2))]
fn sr(r: crate::pac::usart::Usart) -> crate::pac::common::Reg<regs::Sr, crate::pac::common::RW> {
    r.sr()
}

#[cfg(any(usart_v1, usart_v2))]
#[allow(unused)]
fn clear_interrupt_flags(_r: Regs, _sr: regs::Sr) {
    // On v1 the flags are cleared implicitly by reads and writes to DR.
}

#[cfg(any(usart_v3, usart_v4))]
fn tdr(r: Regs) -> *mut u8 {
    r.tdr().as_ptr() as _
}

#[cfg(any(usart_v3, usart_v4))]
fn rdr(r: Regs) -> *mut u8 {
    r.rdr().as_ptr() as _
}

#[cfg(any(usart_v3, usart_v4))]
fn sr(r: Regs) -> crate::pac::common::Reg<regs::Isr, crate::pac::common::R> {
    r.isr()
}

#[cfg(any(usart_v3, usart_v4))]
#[allow(unused)]
fn clear_interrupt_flags(r: Regs, sr: regs::Isr) {
    r.icr().write(|w| *w = regs::Icr(sr.0));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Uart,
    #[cfg(any(usart_v3, usart_v4))]
    #[allow(unused)]
    Lpuart,
}

struct State {
    rx_waker: AtomicWaker,
    tx_rx_refcount: AtomicU8,
}

impl State {
    const fn new() -> Self {
        Self {
            rx_waker: AtomicWaker::new(),
            tx_rx_refcount: AtomicU8::new(0),
        }
    }
}

struct Info {
    regs: Regs,
    rcc: RccInfo,
    interrupt: Interrupt,
    kind: Kind,
}

#[allow(private_interfaces)]
pub(crate) trait SealedInstance: crate::rcc::RccPeripheral {
    fn info() -> &'static Info;
    fn state() -> &'static State;
    fn buffered_state() -> &'static buffered::State;
}

/// USART peripheral instance trait.
#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + SealedInstance + 'static + Send {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);
pin_trait!(CtsPin, Instance);
pin_trait!(RtsPin, Instance);
pin_trait!(CkPin, Instance);
pin_trait!(DePin, Instance);

dma_trait!(TxDma, Instance);
dma_trait!(RxDma, Instance);

macro_rules! impl_usart {
    ($inst:ident, $irq:ident, $kind:expr) => {
        #[allow(private_interfaces)]
        impl SealedInstance for crate::peripherals::$inst {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: unsafe { Regs::from_ptr(crate::pac::$inst.as_ptr()) },
                    rcc: crate::peripherals::$inst::RCC_INFO,
                    interrupt: crate::interrupt::typelevel::$irq::IRQ,
                    kind: $kind,
                };
                &INFO
            }

            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }

            fn buffered_state() -> &'static buffered::State {
                static BUFFERED_STATE: buffered::State = buffered::State::new();
                &BUFFERED_STATE
            }
        }

        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

foreach_interrupt!(
    ($inst:ident, usart, LPUART, $signal_name:ident, $irq:ident) => {
        impl_usart!($inst, $irq, Kind::Lpuart);
    };
    ($inst:ident, usart, $block:ident, $signal_name:ident, $irq:ident) => {
        impl_usart!($inst, $irq, Kind::Uart);
    };
);
