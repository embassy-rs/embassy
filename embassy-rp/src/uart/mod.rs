//! UART driver.
use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU16, Ordering};
use core::task::Poll;

use embassy_futures::select::{Either, select};
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::{Delay, Timer};
use pac::uart::regs::Uartris;

use crate::clocks::clk_peri_freq;
use crate::dma::{AnyChannel, Channel};
use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt::typelevel::{Binding, Interrupt as _};
use crate::interrupt::{Interrupt, InterruptExt};
use crate::pac::io::vals::{Inover, Outover};
use crate::{RegExt, interrupt, pac, peripherals};

mod buffered;
pub use buffered::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, BufferedUartTx};

/// Word length.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataBits {
    /// 5 bits.
    DataBits5,
    /// 6 bits.
    DataBits6,
    /// 7 bits.
    DataBits7,
    /// 8 bits.
    DataBits8,
}

impl DataBits {
    fn bits(&self) -> u8 {
        match self {
            Self::DataBits5 => 0b00,
            Self::DataBits6 => 0b01,
            Self::DataBits7 => 0b10,
            Self::DataBits8 => 0b11,
        }
    }
}

/// Parity bit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    /// No parity.
    ParityNone,
    /// Even parity.
    ParityEven,
    /// Odd parity.
    ParityOdd,
}

/// Stop bits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopBits {
    #[doc = "1 stop bit"]
    STOP1,
    #[doc = "2 stop bits"]
    STOP2,
}

/// UART config.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Config {
    /// Baud rate.
    pub baudrate: u32,
    /// Word length.
    pub data_bits: DataBits,
    /// Stop bits.
    pub stop_bits: StopBits,
    /// Parity bit.
    pub parity: Parity,
    /// Invert the tx pin output
    pub invert_tx: bool,
    /// Invert the rx pin input
    pub invert_rx: bool,
    /// Invert the rts pin
    pub invert_rts: bool,
    /// Invert the cts pin
    pub invert_cts: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
            invert_rx: false,
            invert_tx: false,
            invert_rts: false,
            invert_cts: false,
        }
    }
}

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Triggered when the FIFO (or shift-register) is overflowed.
    Overrun,
    /// Triggered when a break is received
    Break,
    /// Triggered when there is a parity mismatch between what's received and
    /// our settings.
    Parity,
    /// Triggered when the received character didn't have a valid stop bit.
    Framing,
}

/// Read To Break error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ReadToBreakError {
    /// Read this many bytes, but never received a line break.
    MissingBreak(usize),
    /// Other, standard issue with the serial request
    Other(Error),
}

/// Internal DMA state of UART RX.
pub struct DmaState {
    rx_err_waker: AtomicWaker,
    rx_errs: AtomicU16,
}

/// UART driver.
pub struct Uart<'d, M: Mode> {
    tx: UartTx<'d, M>,
    rx: UartRx<'d, M>,
}

/// UART TX driver.
pub struct UartTx<'d, M: Mode> {
    info: &'static Info,
    tx_dma: Option<Peri<'d, AnyChannel>>,
    phantom: PhantomData<M>,
}

/// UART RX driver.
pub struct UartRx<'d, M: Mode> {
    info: &'static Info,
    dma_state: &'static DmaState,
    rx_dma: Option<Peri<'d, AnyChannel>>,
    phantom: PhantomData<M>,
}

impl<'d, M: Mode> UartTx<'d, M> {
    /// Create a new DMA-enabled UART which can only send data
    pub fn new<T: Instance>(
        _uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        tx_dma: Peri<'d, impl Channel>,
        config: Config,
    ) -> Self {
        Uart::<M>::init(T::info(), Some(tx.into()), None, None, None, config);
        Self::new_inner(T::info(), Some(tx_dma.into()))
    }

    fn new_inner(info: &'static Info, tx_dma: Option<Peri<'d, AnyChannel>>) -> Self {
        Self {
            info,
            tx_dma,
            phantom: PhantomData,
        }
    }

    /// Transmit the provided buffer blocking execution until done.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let r = self.info.regs;
        for &b in buffer {
            while r.uartfr().read().txff() {}
            r.uartdr().write(|w| w.set_data(b));
        }
        Ok(())
    }

    /// Flush UART TX blocking execution until done.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while !self.info.regs.uartfr().read().txfe() {}
        Ok(())
    }

    /// Check if UART is busy transmitting.
    pub fn busy(&self) -> bool {
        self.info.regs.uartfr().read().busy()
    }

    /// Assert a break condition after waiting for the transmit buffers to empty,
    /// for the specified number of bit times. This condition must be asserted
    /// for at least two frame times to be effective, `bits` will adjusted
    /// according to frame size, parity, and stop bit settings to ensure this.
    ///
    /// This method may block for a long amount of time since it has to wait
    /// for the transmit fifo to empty, which may take a while on slow links.
    pub async fn send_break(&mut self, bits: u32) {
        let regs = self.info.regs;
        let bits = bits.max({
            let lcr = regs.uartlcr_h().read();
            let width = lcr.wlen() as u32 + 5;
            let parity = lcr.pen() as u32;
            let stops = 1 + lcr.stp2() as u32;
            2 * (1 + width + parity + stops)
        });
        let divx64 = (((regs.uartibrd().read().baud_divint() as u32) << 6)
            + regs.uartfbrd().read().baud_divfrac() as u32) as u64;
        let div_clk = clk_peri_freq() as u64 * 64;
        let wait_usecs = (1_000_000 * bits as u64 * divx64 * 16 + div_clk - 1) / div_clk;

        self.blocking_flush().unwrap();
        while self.busy() {}
        regs.uartlcr_h().write_set(|w| w.set_brk(true));
        Timer::after_micros(wait_usecs).await;
        regs.uartlcr_h().write_clear(|w| w.set_brk(true));
    }
}

impl<'d> UartTx<'d, Blocking> {
    /// Create a new UART TX instance for blocking mode operations.
    pub fn new_blocking<T: Instance>(_uart: Peri<'d, T>, tx: Peri<'d, impl TxPin<T>>, config: Config) -> Self {
        Uart::<Blocking>::init(T::info(), Some(tx.into()), None, None, None, config);
        Self::new_inner(T::info(), None)
    }

    /// Convert this uart TX instance into a buffered uart using the provided
    /// irq and transmit buffer.
    pub fn into_buffered<T: Instance>(
        self,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
    ) -> BufferedUartTx {
        buffered::init_buffers(T::info(), T::buffered_state(), Some(tx_buffer), None);

        BufferedUartTx {
            info: T::info(),
            state: T::buffered_state(),
        }
    }
}

impl<'d> UartTx<'d, Async> {
    /// Write to UART TX from the provided buffer using DMA.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        let ch = self.tx_dma.as_mut().unwrap().reborrow();
        let transfer = unsafe {
            self.info.regs.uartdmacr().write_set(|reg| {
                reg.set_txdmae(true);
            });
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::write(
                ch,
                buffer,
                self.info.regs.uartdr().as_ptr() as *mut _,
                self.info.tx_dreq.into(),
            )
        };
        transfer.await;
        Ok(())
    }
}

impl<'d, M: Mode> UartRx<'d, M> {
    /// Create a new DMA-enabled UART which can only receive data
    pub fn new<T: Instance>(
        _uart: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        rx_dma: Peri<'d, impl Channel>,
        config: Config,
    ) -> Self {
        Uart::<M>::init(T::info(), None, Some(rx.into()), None, None, config);
        Self::new_inner(T::info(), T::dma_state(), true, Some(rx_dma.into()))
    }

    fn new_inner(
        info: &'static Info,
        dma_state: &'static DmaState,
        has_irq: bool,
        rx_dma: Option<Peri<'d, AnyChannel>>,
    ) -> Self {
        debug_assert_eq!(has_irq, rx_dma.is_some());
        if has_irq {
            // disable all error interrupts initially
            info.regs.uartimsc().write(|w| w.0 = 0);
            info.interrupt.unpend();
            unsafe { info.interrupt.enable() };
        }
        Self {
            info,
            dma_state,
            rx_dma,
            phantom: PhantomData,
        }
    }

    /// Read from UART RX blocking execution until done.
    pub fn blocking_read(&mut self, mut buffer: &mut [u8]) -> Result<(), Error> {
        while !buffer.is_empty() {
            let received = self.drain_fifo(buffer).map_err(|(_i, e)| e)?;
            buffer = &mut buffer[received..];
        }
        Ok(())
    }

    /// Returns Ok(len) if no errors occurred. Returns Err((len, err)) if an error was
    /// encountered. In both cases, `len` is the number of *good* bytes copied into
    /// `buffer`.
    fn drain_fifo(&mut self, buffer: &mut [u8]) -> Result<usize, (usize, Error)> {
        let r = self.info.regs;
        for (i, b) in buffer.iter_mut().enumerate() {
            if r.uartfr().read().rxfe() {
                return Ok(i);
            }

            let dr = r.uartdr().read();

            if dr.oe() {
                return Err((i, Error::Overrun));
            } else if dr.be() {
                return Err((i, Error::Break));
            } else if dr.pe() {
                return Err((i, Error::Parity));
            } else if dr.fe() {
                return Err((i, Error::Framing));
            } else {
                *b = dr.data();
            }
        }
        Ok(buffer.len())
    }
}

impl<'d, M: Mode> Drop for UartRx<'d, M> {
    fn drop(&mut self) {
        if self.rx_dma.is_some() {
            self.info.interrupt.disable();
            // clear dma flags. irq handlers use these to disambiguate among themselves.
            self.info.regs.uartdmacr().write_clear(|reg| {
                reg.set_rxdmae(true);
                reg.set_txdmae(true);
                reg.set_dmaonerr(true);
            });
        }
    }
}

impl<'d> UartRx<'d, Blocking> {
    /// Create a new UART RX instance for blocking mode operations.
    pub fn new_blocking<T: Instance>(_uart: Peri<'d, T>, rx: Peri<'d, impl RxPin<T>>, config: Config) -> Self {
        Uart::<Blocking>::init(T::info(), None, Some(rx.into()), None, None, config);
        Self::new_inner(T::info(), T::dma_state(), false, None)
    }

    /// Convert this uart RX instance into a buffered uart using the provided
    /// irq and receive buffer.
    pub fn into_buffered<T: Instance>(
        self,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        rx_buffer: &'d mut [u8],
    ) -> BufferedUartRx {
        buffered::init_buffers(T::info(), T::buffered_state(), None, Some(rx_buffer));

        BufferedUartRx {
            info: T::info(),
            state: T::buffered_state(),
        }
    }
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _uart: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let uart = T::info().regs;
        if !uart.uartdmacr().read().rxdmae() {
            return;
        }

        let state = T::dma_state();
        let errs = uart.uartris().read();
        state.rx_errs.store(errs.0 as u16, Ordering::Relaxed);
        state.rx_err_waker.wake();
        // disable the error interrupts instead of clearing the flags. clearing the
        // flags would allow the dma transfer to continue, potentially signaling
        // completion before we can check for errors that happened *during* the transfer.
        uart.uartimsc().write_clear(|w| w.0 = errs.0);
    }
}

impl<'d> UartRx<'d, Async> {
    /// Read from UART RX into the provided buffer.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        // clear error flags before we drain the fifo. errors that have accumulated
        // in the flags will also be present in the fifo.
        self.dma_state.rx_errs.store(0, Ordering::Relaxed);
        self.info.regs.uarticr().write(|w| {
            w.set_oeic(true);
            w.set_beic(true);
            w.set_peic(true);
            w.set_feic(true);
        });

        // then drain the fifo. we need to read at most 32 bytes. errors that apply
        // to fifo bytes will be reported directly.
        let buffer = match {
            let limit = buffer.len().min(32);
            self.drain_fifo(&mut buffer[0..limit])
        } {
            Ok(len) if len < buffer.len() => &mut buffer[len..],
            Ok(_) => return Ok(()),
            Err((_i, e)) => return Err(e),
        };

        // start a dma transfer. if errors have happened in the interim some error
        // interrupt flags will have been raised, and those will be picked up immediately
        // by the interrupt handler.
        let ch = self.rx_dma.as_mut().unwrap().reborrow();
        self.info.regs.uartimsc().write_set(|w| {
            w.set_oeim(true);
            w.set_beim(true);
            w.set_peim(true);
            w.set_feim(true);
        });
        self.info.regs.uartdmacr().write_set(|reg| {
            reg.set_rxdmae(true);
            reg.set_dmaonerr(true);
        });
        let transfer = unsafe {
            // If we don't assign future to a variable, the data register pointer
            // is held across an await and makes the future non-Send.
            crate::dma::read(
                ch,
                self.info.regs.uartdr().as_ptr() as *const _,
                buffer,
                self.info.rx_dreq.into(),
            )
        };

        // wait for either the transfer to complete or an error to happen.
        let transfer_result = select(
            transfer,
            poll_fn(|cx| {
                self.dma_state.rx_err_waker.register(cx.waker());
                let rx_errs = critical_section::with(|_| {
                    let val = self.dma_state.rx_errs.load(Ordering::Relaxed);
                    self.dma_state.rx_errs.store(0, Ordering::Relaxed);
                    val
                });
                match rx_errs {
                    0 => Poll::Pending,
                    e => Poll::Ready(Uartris(e as u32)),
                }
            }),
        )
        .await;

        let errors = match transfer_result {
            Either::First(()) => {
                // We're here because the DMA finished, BUT if an error occurred on the LAST
                // byte, then we may still need to grab the error state!
                Uartris(critical_section::with(|_| {
                    let val = self.dma_state.rx_errs.load(Ordering::Relaxed);
                    self.dma_state.rx_errs.store(0, Ordering::Relaxed);
                    val
                }) as u32)
            }
            Either::Second(e) => {
                // We're here because we errored, which means this is the error that
                // was problematic.
                e
            }
        };

        // If we got no error, just return at this point
        if errors.0 == 0 {
            return Ok(());
        }

        // If we DID get an error, we need to figure out which one it was.
        if errors.oeris() {
            return Err(Error::Overrun);
        } else if errors.beris() {
            return Err(Error::Break);
        } else if errors.peris() {
            return Err(Error::Parity);
        } else if errors.feris() {
            return Err(Error::Framing);
        }
        unreachable!("unrecognized rx error");
    }

    /// Read from the UART, waiting for a break.
    ///
    /// We read until one of the following occurs:
    ///
    /// * We read `buffer.len()` bytes without a break
    ///     * returns `Err(ReadToBreakError::MissingBreak(buffer.len()))`
    /// * We read `n` bytes then a break occurs
    ///     * returns `Ok(n)`
    /// * We encounter some error OTHER than a break
    ///     * returns `Err(ReadToBreakError::Other(error))`
    ///
    /// **NOTE**: you MUST provide a buffer one byte larger than your largest expected
    /// message to reliably detect the framing on one single call to `read_to_break()`.
    ///
    /// * If you expect a message of 20 bytes + break, and provide a 20-byte buffer:
    ///     * The first call to `read_to_break()` will return `Err(ReadToBreakError::MissingBreak(20))`
    ///     * The next call to `read_to_break()` will immediately return `Ok(0)`, from the "stale" break
    /// * If you expect a message of 20 bytes + break, and provide a 21-byte buffer:
    ///     * The first call to `read_to_break()` will return `Ok(20)`.
    ///     * The next call to `read_to_break()` will work as expected
    ///
    /// **NOTE**: In the UART context, a break refers to a break condition (the line being held low for
    /// for longer than a single character), not an ASCII line break.
    pub async fn read_to_break(&mut self, buffer: &mut [u8]) -> Result<usize, ReadToBreakError> {
        self.read_to_break_with_count(buffer, 0).await
    }

    /// Read from the UART, waiting for a break as soon as at least `min_count` bytes have been read.
    ///
    /// We read until one of the following occurs:
    ///
    /// * We read `buffer.len()` bytes without a break
    ///     * returns `Err(ReadToBreakError::MissingBreak(buffer.len()))`
    /// * We read `n > min_count` bytes then a break occurs
    ///     * returns `Ok(n)`
    /// * We encounter some error OTHER than a break
    ///     * returns `Err(ReadToBreakError::Other(error))`
    ///
    /// If a break occurs before `min_count` bytes have been read, the break will be ignored and the read will continue
    ///
    /// **NOTE**: you MUST provide a buffer one byte larger than your largest expected
    /// message to reliably detect the framing on one single call to `read_to_break()`.
    ///
    /// * If you expect a message of 20 bytes + break, and provide a 20-byte buffer:
    ///     * The first call to `read_to_break()` will return `Err(ReadToBreakError::MissingBreak(20))`
    ///     * The next call to `read_to_break()` will immediately return `Ok(0)`, from the "stale" line break
    /// * If you expect a message of 20 bytes + break, and provide a 21-byte buffer:
    ///     * The first call to `read_to_break()` will return `Ok(20)`.
    ///     * The next call to `read_to_break()` will work as expected
    ///
    /// **NOTE**: In the UART context, a break refers to a break condition (the line being held low for
    /// for longer than a single character), not an ASCII line break.
    pub async fn read_to_break_with_count(
        &mut self,
        buffer: &mut [u8],
        min_count: usize,
    ) -> Result<usize, ReadToBreakError> {
        // clear error flags before we drain the fifo. errors that have accumulated
        // in the flags will also be present in the fifo.
        self.dma_state.rx_errs.store(0, Ordering::Relaxed);
        self.info.regs.uarticr().write(|w| {
            w.set_oeic(true);
            w.set_beic(true);
            w.set_peic(true);
            w.set_feic(true);
        });

        // then drain the fifo. we need to read at most 32 bytes. errors that apply
        // to fifo bytes will be reported directly.
        let mut sbuffer = match {
            let limit = buffer.len().min(32);
            self.drain_fifo(&mut buffer[0..limit])
        } {
            // Drained fifo, still some room left!
            Ok(len) if len < buffer.len() => &mut buffer[len..],
            // Drained (some/all of the fifo), no room left
            Ok(len) => return Err(ReadToBreakError::MissingBreak(len)),
            // We got a break WHILE draining the FIFO, return what we did get before the break
            Err((len, Error::Break)) => {
                if len < min_count && len < buffer.len() {
                    &mut buffer[len..]
                } else {
                    return Ok(len);
                }
            }
            // Some other error, just return the error
            Err((_i, e)) => return Err(ReadToBreakError::Other(e)),
        };

        // start a dma transfer. if errors have happened in the interim some error
        // interrupt flags will have been raised, and those will be picked up immediately
        // by the interrupt handler.
        let ch = self.rx_dma.as_mut().unwrap();
        self.info.regs.uartimsc().write_set(|w| {
            w.set_oeim(true);
            w.set_beim(true);
            w.set_peim(true);
            w.set_feim(true);
        });
        self.info.regs.uartdmacr().write_set(|reg| {
            reg.set_rxdmae(true);
            reg.set_dmaonerr(true);
        });

        loop {
            let transfer = unsafe {
                // If we don't assign future to a variable, the data register pointer
                // is held across an await and makes the future non-Send.
                crate::dma::read(
                    ch.reborrow(),
                    self.info.regs.uartdr().as_ptr() as *const _,
                    sbuffer,
                    self.info.rx_dreq.into(),
                )
            };

            // wait for either the transfer to complete or an error to happen.
            let transfer_result = select(
                transfer,
                poll_fn(|cx| {
                    self.dma_state.rx_err_waker.register(cx.waker());
                    let rx_errs = critical_section::with(|_| {
                        let val = self.dma_state.rx_errs.load(Ordering::Relaxed);
                        self.dma_state.rx_errs.store(0, Ordering::Relaxed);
                        val
                    });
                    match rx_errs {
                        0 => Poll::Pending,
                        e => Poll::Ready(Uartris(e as u32)),
                    }
                }),
            )
            .await;

            // Figure out our error state
            let errors = match transfer_result {
                Either::First(()) => {
                    // We're here because the DMA finished, BUT if an error occurred on the LAST
                    // byte, then we may still need to grab the error state!
                    Uartris(critical_section::with(|_| {
                        let val = self.dma_state.rx_errs.load(Ordering::Relaxed);
                        self.dma_state.rx_errs.store(0, Ordering::Relaxed);
                        val
                    }) as u32)
                }
                Either::Second(e) => {
                    // We're here because we errored, which means this is the error that
                    // was problematic.
                    e
                }
            };

            if errors.0 == 0 {
                // No errors? That means we filled the buffer without a line break.
                // For THIS function, that's a problem.
                return Err(ReadToBreakError::MissingBreak(buffer.len()));
            } else if errors.beris() {
                // We got a Line Break! By this point, we've finished/aborted the DMA
                // transaction, which means that we need to figure out where it left off
                // by looking at the write_addr.
                //
                // First, we do a sanity check to make sure the write value is within the
                // range of DMA we just did.
                let sval = buffer.as_ptr() as usize;
                let eval = sval + buffer.len();

                // This is the address where the DMA would write to next
                let next_addr = ch.regs().write_addr().read() as usize;

                // If we DON'T end up inside the range, something has gone really wrong.
                // Note that it's okay that `eval` is one past the end of the slice, as
                // this is where the write pointer will end up at the end of a full
                // transfer.
                if (next_addr < sval) || (next_addr > eval) {
                    unreachable!("UART DMA reported invalid `write_addr`");
                }

                if (next_addr - sval) < min_count {
                    sbuffer = &mut buffer[(next_addr - sval)..];
                    continue;
                }

                let regs = self.info.regs;
                let all_full = next_addr == eval;

                // NOTE: This is off label usage of RSR! See the issue below for
                // why I am not checking if there is an "extra" FIFO byte, and why
                // I am checking RSR directly (it seems to report the status of the LAST
                // POPPED value, rather than the NEXT TO POP value like the datasheet
                // suggests!)
                //
                // issue: https://github.com/raspberrypi/pico-feedback/issues/367
                let last_was_break = regs.uartrsr().read().be();

                return match (all_full, last_was_break) {
                    (true, true) | (false, _) => {
                        // We got less than the full amount + a break, or the full amount
                        // and the last byte was a break. Subtract the break off by adding one to sval.
                        Ok(next_addr.saturating_sub(1 + sval))
                    }
                    (true, false) => {
                        // We finished the whole DMA, and the last DMA'd byte was NOT a break
                        // character. This is an error.
                        //
                        // NOTE: we COULD potentially return Ok(buffer.len()) here, since we
                        // know a line break occured at SOME POINT after the DMA completed.
                        //
                        // However, we have no way of knowing if there was extra data BEFORE
                        // that line break, so instead return an Err to signal to the caller
                        // that there are "leftovers", and they'll catch the actual line break
                        // on the next call.
                        //
                        // Doing it like this also avoids racyness: now whether you finished
                        // the full read BEFORE the line break occurred or AFTER the line break
                        // occurs, you still get `MissingBreak(buffer.len())` instead of sometimes
                        // getting `Ok(buffer.len())` if you were "late enough" to observe the
                        // line break.
                        Err(ReadToBreakError::MissingBreak(buffer.len()))
                    }
                };
            } else if errors.oeris() {
                return Err(ReadToBreakError::Other(Error::Overrun));
            } else if errors.peris() {
                return Err(ReadToBreakError::Other(Error::Parity));
            } else if errors.feris() {
                return Err(ReadToBreakError::Other(Error::Framing));
            }
            unreachable!("unrecognized rx error");
        }
    }
}

impl<'d> Uart<'d, Blocking> {
    /// Create a new UART without hardware flow control
    pub fn new_blocking<T: Instance>(
        uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(uart, tx.into(), rx.into(), None, None, false, None, None, config)
    }

    /// Create a new UART with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts_blocking<T: Instance>(
        uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        rts: Peri<'d, impl RtsPin<T>>,
        cts: Peri<'d, impl CtsPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            uart,
            tx.into(),
            rx.into(),
            Some(rts.into()),
            Some(cts.into()),
            false,
            None,
            None,
            config,
        )
    }

    /// Convert this uart instance into a buffered uart using the provided
    /// irq, transmit and receive buffers.
    pub fn into_buffered<T: Instance>(
        self,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
    ) -> BufferedUart {
        buffered::init_buffers(T::info(), T::buffered_state(), Some(tx_buffer), Some(rx_buffer));

        BufferedUart {
            rx: BufferedUartRx {
                info: T::info(),
                state: T::buffered_state(),
            },
            tx: BufferedUartTx {
                info: T::info(),
                state: T::buffered_state(),
            },
        }
    }
}

impl<'d> Uart<'d, Async> {
    /// Create a new DMA enabled UART without hardware flow control
    pub fn new<T: Instance>(
        uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            uart,
            tx.into(),
            rx.into(),
            None,
            None,
            true,
            Some(tx_dma.into()),
            Some(rx_dma.into()),
            config,
        )
    }

    /// Create a new DMA enabled UART with hardware flow control (RTS/CTS)
    pub fn new_with_rtscts<T: Instance>(
        uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        rts: Peri<'d, impl RtsPin<T>>,
        cts: Peri<'d, impl CtsPin<T>>,
        _irq: impl Binding<T::Interrupt, InterruptHandler<T>>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'d, impl Channel>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            uart,
            tx.into(),
            rx.into(),
            Some(rts.into()),
            Some(cts.into()),
            true,
            Some(tx_dma.into()),
            Some(rx_dma.into()),
            config,
        )
    }
}

impl<'d, M: Mode> Uart<'d, M> {
    fn new_inner<T: Instance>(
        _uart: Peri<'d, T>,
        mut tx: Peri<'d, AnyPin>,
        mut rx: Peri<'d, AnyPin>,
        mut rts: Option<Peri<'d, AnyPin>>,
        mut cts: Option<Peri<'d, AnyPin>>,
        has_irq: bool,
        tx_dma: Option<Peri<'d, AnyChannel>>,
        rx_dma: Option<Peri<'d, AnyChannel>>,
        config: Config,
    ) -> Self {
        Self::init(
            T::info(),
            Some(tx.reborrow()),
            Some(rx.reborrow()),
            rts.as_mut().map(|x| x.reborrow()),
            cts.as_mut().map(|x| x.reborrow()),
            config,
        );

        Self {
            tx: UartTx::new_inner(T::info(), tx_dma),
            rx: UartRx::new_inner(T::info(), T::dma_state(), has_irq, rx_dma),
        }
    }

    fn init(
        info: &Info,
        tx: Option<Peri<'_, AnyPin>>,
        rx: Option<Peri<'_, AnyPin>>,
        rts: Option<Peri<'_, AnyPin>>,
        cts: Option<Peri<'_, AnyPin>>,
        config: Config,
    ) {
        let r = info.regs;
        if let Some(pin) = &tx {
            let funcsel = {
                let pin_number = ((pin.gpio().as_ptr() as u32) & 0x1FF) / 8;
                if (pin_number % 4) == 0 { 2 } else { 11 }
            };
            pin.gpio().ctrl().write(|w| {
                w.set_funcsel(funcsel);
                w.set_outover(if config.invert_tx {
                    Outover::INVERT
                } else {
                    Outover::NORMAL
                });
            });
            pin.pad_ctrl().write(|w| {
                #[cfg(feature = "_rp235x")]
                w.set_iso(false);
                w.set_ie(true);
            });
        }
        if let Some(pin) = &rx {
            let funcsel = {
                let pin_number = ((pin.gpio().as_ptr() as u32) & 0x1FF) / 8;
                if ((pin_number - 1) % 4) == 0 { 2 } else { 11 }
            };
            pin.gpio().ctrl().write(|w| {
                w.set_funcsel(funcsel);
                w.set_inover(if config.invert_rx {
                    Inover::INVERT
                } else {
                    Inover::NORMAL
                });
            });
            pin.pad_ctrl().write(|w| {
                #[cfg(feature = "_rp235x")]
                w.set_iso(false);
                w.set_ie(true);
            });
        }
        if let Some(pin) = &cts {
            pin.gpio().ctrl().write(|w| {
                w.set_funcsel(2);
                w.set_inover(if config.invert_cts {
                    Inover::INVERT
                } else {
                    Inover::NORMAL
                });
            });
            pin.pad_ctrl().write(|w| {
                #[cfg(feature = "_rp235x")]
                w.set_iso(false);
                w.set_ie(true);
            });
        }
        if let Some(pin) = &rts {
            pin.gpio().ctrl().write(|w| {
                w.set_funcsel(2);
                w.set_outover(if config.invert_rts {
                    Outover::INVERT
                } else {
                    Outover::NORMAL
                });
            });
            pin.pad_ctrl().write(|w| {
                #[cfg(feature = "_rp235x")]
                w.set_iso(false);
                w.set_ie(true);
            });
        }

        Self::set_baudrate_inner(info, config.baudrate);

        let (pen, eps) = match config.parity {
            Parity::ParityNone => (false, false),
            Parity::ParityOdd => (true, false),
            Parity::ParityEven => (true, true),
        };

        r.uartlcr_h().write(|w| {
            w.set_wlen(config.data_bits.bits());
            w.set_stp2(config.stop_bits == StopBits::STOP2);
            w.set_pen(pen);
            w.set_eps(eps);
            w.set_fen(true);
        });

        r.uartifls().write(|w| {
            w.set_rxiflsel(0b100);
            w.set_txiflsel(0b000);
        });

        r.uartcr().write(|w| {
            w.set_uarten(true);
            w.set_rxe(true);
            w.set_txe(true);
            w.set_ctsen(cts.is_some());
            w.set_rtsen(rts.is_some());
        });
    }

    fn lcr_modify<R>(info: &Info, f: impl FnOnce(&mut crate::pac::uart::regs::UartlcrH) -> R) -> R {
        let r = info.regs;

        // Notes from PL011 reference manual:
        //
        // - Before writing the LCR, if the UART is enabled it needs to be
        //   disabled and any current TX + RX activity has to be completed
        //
        // - There is a BUSY flag which waits for the current TX char, but this is
        //   OR'd with TX FIFO !FULL, so not usable when FIFOs are enabled and
        //   potentially nonempty
        //
        // - FIFOs can't be set to disabled whilst a character is in progress
        //   (else "FIFO integrity is not guaranteed")
        //
        // Combination of these means there is no general way to halt and poll for
        // end of TX character, if FIFOs may be enabled. Either way, there is no
        // way to poll for end of RX character.
        //
        // So, insert a 15 Baud period delay before changing the settings.
        // 15 Baud is comfortably higher than start + max data + parity + stop.
        // Anything else would require API changes to permit a non-enabled UART
        // state after init() where settings can be changed safely.
        let clk_base = crate::clocks::clk_peri_freq();

        let cr = r.uartcr().read();
        if cr.uarten() {
            r.uartcr().modify(|w| {
                w.set_uarten(false);
                w.set_txe(false);
                w.set_rxe(false);
            });

            // Note: Maximise precision here. Show working, the compiler will mop this up.
            // Create a 16.6 fixed-point fractional division ratio; then scale to 32-bits.
            let mut brdiv_ratio = 64 * r.uartibrd().read().0 + r.uartfbrd().read().0;
            brdiv_ratio <<= 10;
            // 3662 is ~(15 * 244.14) where 244.14 is 16e6 / 2^16
            let scaled_freq = clk_base / 3662;
            let wait_time_us = brdiv_ratio / scaled_freq;
            embedded_hal_1::delay::DelayNs::delay_us(&mut Delay, wait_time_us);
        }

        let res = r.uartlcr_h().modify(f);

        r.uartcr().write_value(cr);

        res
    }

    /// sets baudrate on runtime
    pub fn set_baudrate(&mut self, baudrate: u32) {
        Self::set_baudrate_inner(self.tx.info, baudrate);
    }

    fn set_baudrate_inner(info: &Info, baudrate: u32) {
        let r = info.regs;

        let clk_base = crate::clocks::clk_peri_freq();

        let baud_rate_div = (8 * clk_base) / baudrate;
        let mut baud_ibrd = baud_rate_div >> 7;
        let mut baud_fbrd = ((baud_rate_div & 0x7f) + 1) / 2;

        if baud_ibrd == 0 {
            baud_ibrd = 1;
            baud_fbrd = 0;
        } else if baud_ibrd >= 65535 {
            baud_ibrd = 65535;
            baud_fbrd = 0;
        }

        // Load PL011's baud divisor registers
        r.uartibrd().write_value(pac::uart::regs::Uartibrd(baud_ibrd));
        r.uartfbrd().write_value(pac::uart::regs::Uartfbrd(baud_fbrd));

        Self::lcr_modify(info, |_| {});
    }
}

impl<'d, M: Mode> Uart<'d, M> {
    /// Transmit the provided buffer blocking execution until done.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.blocking_write(buffer)
    }

    /// Flush UART TX blocking execution until done.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Read from UART RX blocking execution until done.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.blocking_read(buffer)
    }

    /// Check if UART is busy transmitting.
    pub fn busy(&self) -> bool {
        self.tx.busy()
    }

    /// Wait until TX is empty and send break condition.
    pub async fn send_break(&mut self, bits: u32) {
        self.tx.send_break(bits).await
    }

    /// Split the Uart into a transmitter and receiver, which is particularly
    /// useful when having two tasks correlating to transmitting and receiving.
    pub fn split(self) -> (UartTx<'d, M>, UartRx<'d, M>) {
        (self.tx, self.rx)
    }

    /// Split the Uart into a transmitter and receiver by mutable reference,
    /// which is particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split_ref(&mut self) -> (&mut UartTx<'d, M>, &mut UartRx<'d, M>) {
        (&mut self.tx, &mut self.rx)
    }
}

impl<'d> Uart<'d, Async> {
    /// Write to UART TX from the provided buffer.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.tx.write(buffer).await
    }

    /// Read from UART RX into the provided buffer.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.rx.read(buffer).await
    }

    /// Read until the buffer is full or a line break occurs.
    ///
    /// See [`UartRx::read_to_break()`] for more details
    pub async fn read_to_break<'a>(&mut self, buf: &'a mut [u8]) -> Result<usize, ReadToBreakError> {
        self.rx.read_to_break(buf).await
    }

    /// Read until the buffer is full or a line break occurs after at least `min_count` bytes have been read.
    ///
    /// See [`UartRx::read_to_break_with_count()`] for more details
    pub async fn read_to_break_with_count<'a>(
        &mut self,
        buf: &'a mut [u8],
        min_count: usize,
    ) -> Result<usize, ReadToBreakError> {
        self.rx.read_to_break_with_count(buf, min_count).await
    }
}

impl<'d, M: Mode> embedded_hal_02::serial::Read<u8> for UartRx<'d, M> {
    type Error = Error;
    fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
        let r = self.info.regs;
        if r.uartfr().read().rxfe() {
            return Err(nb::Error::WouldBlock);
        }

        let dr = r.uartdr().read();

        if dr.oe() {
            Err(nb::Error::Other(Error::Overrun))
        } else if dr.be() {
            Err(nb::Error::Other(Error::Break))
        } else if dr.pe() {
            Err(nb::Error::Other(Error::Parity))
        } else if dr.fe() {
            Err(nb::Error::Other(Error::Framing))
        } else {
            Ok(dr.data())
        }
    }
}

impl<'d, M: Mode> embedded_hal_02::serial::Write<u8> for UartTx<'d, M> {
    type Error = Error;

    fn write(&mut self, word: u8) -> Result<(), nb::Error<Self::Error>> {
        let r = self.info.regs;
        if r.uartfr().read().txff() {
            return Err(nb::Error::WouldBlock);
        }

        r.uartdr().write(|w| w.set_data(word));
        Ok(())
    }

    fn flush(&mut self) -> Result<(), nb::Error<Self::Error>> {
        let r = self.info.regs;
        if !r.uartfr().read().txfe() {
            return Err(nb::Error::WouldBlock);
        }
        Ok(())
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
        embedded_hal_02::serial::Read::read(&mut self.rx)
    }
}

impl<'d, M: Mode> embedded_hal_02::serial::Write<u8> for Uart<'d, M> {
    type Error = Error;

    fn write(&mut self, word: u8) -> Result<(), nb::Error<Self::Error>> {
        embedded_hal_02::serial::Write::write(&mut self.tx, word)
    }

    fn flush(&mut self) -> Result<(), nb::Error<Self::Error>> {
        embedded_hal_02::serial::Write::flush(&mut self.tx)
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
            Self::Break => embedded_hal_nb::serial::ErrorKind::Other,
            Self::Overrun => embedded_hal_nb::serial::ErrorKind::Overrun,
            Self::Parity => embedded_hal_nb::serial::ErrorKind::Parity,
        }
    }
}

impl<'d, M: Mode> embedded_hal_nb::serial::ErrorType for UartRx<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_hal_nb::serial::ErrorType for UartTx<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_hal_nb::serial::ErrorType for Uart<'d, M> {
    type Error = Error;
}

impl<'d, M: Mode> embedded_hal_nb::serial::Read for UartRx<'d, M> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let r = self.info.regs;
        if r.uartfr().read().rxfe() {
            return Err(nb::Error::WouldBlock);
        }

        let dr = r.uartdr().read();

        if dr.oe() {
            Err(nb::Error::Other(Error::Overrun))
        } else if dr.be() {
            Err(nb::Error::Other(Error::Break))
        } else if dr.pe() {
            Err(nb::Error::Other(Error::Parity))
        } else if dr.fe() {
            Err(nb::Error::Other(Error::Framing))
        } else {
            Ok(dr.data())
        }
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

impl<'d> embedded_io::ErrorType for UartTx<'d, Blocking> {
    type Error = Error;
}

impl<'d> embedded_io::Write for UartTx<'d, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf).map(|_| buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d, M: Mode> embedded_hal_nb::serial::Read for Uart<'d, M> {
    fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
        embedded_hal_02::serial::Read::read(&mut self.rx)
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

impl<'d> embedded_io::ErrorType for Uart<'d, Blocking> {
    type Error = Error;
}

impl<'d> embedded_io::Write for Uart<'d, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf).map(|_| buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

struct Info {
    regs: pac::uart::Uart,
    tx_dreq: pac::dma::vals::TreqSel,
    rx_dreq: pac::dma::vals::TreqSel,
    interrupt: Interrupt,
}

trait SealedMode {}

trait SealedInstance {
    fn info() -> &'static Info;

    fn buffered_state() -> &'static buffered::State;

    fn dma_state() -> &'static DmaState;
}

/// UART mode.
#[allow(private_bounds)]
pub trait Mode: SealedMode {}

macro_rules! impl_mode {
    ($name:ident) => {
        impl SealedMode for $name {}
        impl Mode for $name {}
    };
}

/// Blocking mode.
pub struct Blocking;
/// Async mode.
pub struct Async;

impl_mode!(Blocking);
impl_mode!(Async);

/// UART instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {
    /// Interrupt for this instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_instance {
    ($inst:ident, $irq:ident, $tx_dreq:expr, $rx_dreq:expr) => {
        impl SealedInstance for peripherals::$inst {
            fn info() -> &'static Info {
                static INFO: Info = Info {
                    regs: pac::$inst,
                    tx_dreq: $tx_dreq,
                    rx_dreq: $rx_dreq,
                    interrupt: crate::interrupt::typelevel::$irq::IRQ,
                };
                &INFO
            }

            fn buffered_state() -> &'static buffered::State {
                static STATE: buffered::State = buffered::State::new();
                &STATE
            }

            fn dma_state() -> &'static DmaState {
                static STATE: DmaState = DmaState {
                    rx_err_waker: AtomicWaker::new(),
                    rx_errs: AtomicU16::new(0),
                };
                &STATE
            }
        }
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}

impl_instance!(
    UART0,
    UART0_IRQ,
    pac::dma::vals::TreqSel::UART0_TX,
    pac::dma::vals::TreqSel::UART0_RX
);
impl_instance!(
    UART1,
    UART1_IRQ,
    pac::dma::vals::TreqSel::UART1_TX,
    pac::dma::vals::TreqSel::UART1_RX
);

/// Trait for TX pins.
pub trait TxPin<T: Instance>: crate::gpio::Pin {}
/// Trait for RX pins.
pub trait RxPin<T: Instance>: crate::gpio::Pin {}
/// Trait for Clear To Send (CTS) pins.
pub trait CtsPin<T: Instance>: crate::gpio::Pin {}
/// Trait for Request To Send (RTS) pins.
pub trait RtsPin<T: Instance>: crate::gpio::Pin {}

macro_rules! impl_pin {
    ($pin:ident, $instance:ident, $function:ident) => {
        impl $function<peripherals::$instance> for peripherals::$pin {}
    };
}

impl_pin!(PIN_0, UART0, TxPin);
impl_pin!(PIN_1, UART0, RxPin);
impl_pin!(PIN_2, UART0, CtsPin);
impl_pin!(PIN_3, UART0, RtsPin);
impl_pin!(PIN_4, UART1, TxPin);
impl_pin!(PIN_5, UART1, RxPin);
impl_pin!(PIN_6, UART1, CtsPin);
impl_pin!(PIN_7, UART1, RtsPin);
impl_pin!(PIN_8, UART1, TxPin);
impl_pin!(PIN_9, UART1, RxPin);
impl_pin!(PIN_10, UART1, CtsPin);
impl_pin!(PIN_11, UART1, RtsPin);
impl_pin!(PIN_12, UART0, TxPin);
impl_pin!(PIN_13, UART0, RxPin);
impl_pin!(PIN_14, UART0, CtsPin);
impl_pin!(PIN_15, UART0, RtsPin);
impl_pin!(PIN_16, UART0, TxPin);
impl_pin!(PIN_17, UART0, RxPin);
impl_pin!(PIN_18, UART0, CtsPin);
impl_pin!(PIN_19, UART0, RtsPin);
impl_pin!(PIN_20, UART1, TxPin);
impl_pin!(PIN_21, UART1, RxPin);
impl_pin!(PIN_22, UART1, CtsPin);
impl_pin!(PIN_23, UART1, RtsPin);
impl_pin!(PIN_24, UART1, TxPin);
impl_pin!(PIN_25, UART1, RxPin);
impl_pin!(PIN_26, UART1, CtsPin);
impl_pin!(PIN_27, UART1, RtsPin);
impl_pin!(PIN_28, UART0, TxPin);
impl_pin!(PIN_29, UART0, RxPin);

// Additional functions added by all 2350s
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_2, UART0, TxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_3, UART0, RxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_6, UART1, TxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_7, UART1, RxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_10, UART1, TxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_11, UART1, RxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_14, UART0, TxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_15, UART0, RxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_18, UART0, TxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_19, UART0, RxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_22, UART1, TxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_23, UART1, RxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_26, UART1, TxPin);
#[cfg(feature = "_rp235x")]
impl_pin!(PIN_27, UART1, RxPin);

// Additional pins added by larger 2350 packages.
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_30, UART0, CtsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_31, UART0, RtsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_32, UART0, TxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_33, UART0, RxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_34, UART0, CtsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_35, UART0, RtsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_36, UART1, TxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_37, UART1, RxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_38, UART1, CtsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_39, UART1, RtsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_40, UART1, TxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_41, UART1, RxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_42, UART1, CtsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_43, UART1, RtsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_44, UART0, TxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_45, UART0, RxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_46, UART0, CtsPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_47, UART0, RtsPin);

#[cfg(feature = "rp235xb")]
impl_pin!(PIN_30, UART0, TxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_31, UART0, RxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_34, UART0, TxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_35, UART0, RxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_38, UART1, TxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_39, UART1, RxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_42, UART1, TxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_43, UART1, RxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_46, UART0, TxPin);
#[cfg(feature = "rp235xb")]
impl_pin!(PIN_47, UART0, RxPin);
