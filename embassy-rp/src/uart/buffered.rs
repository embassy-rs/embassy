//! Buffered UART driver.
use core::marker::PhantomData;
use core::slice;
use core::sync::atomic::{AtomicU8, Ordering};

use embassy_hal_internal::atomic_ring_buffer::RingBuffer;

use super::*;

pub struct State {
    tx_waker: AtomicWaker,
    tx_buf: RingBuffer,
    rx_waker: AtomicWaker,
    rx_buf: RingBuffer,
    rx_error: AtomicU8,
}

// these must match bits 8..11 in UARTDR
const RXE_OVERRUN: u8 = 8;
const RXE_BREAK: u8 = 4;
const RXE_PARITY: u8 = 2;
const RXE_FRAMING: u8 = 1;

impl State {
    pub const fn new() -> Self {
        Self {
            rx_buf: RingBuffer::new(),
            tx_buf: RingBuffer::new(),
            rx_waker: AtomicWaker::new(),
            tx_waker: AtomicWaker::new(),
            rx_error: AtomicU8::new(0),
        }
    }
}

/// Buffered UART driver.
pub struct BufferedUart<'d> {
    pub(super) rx: BufferedUartRx<'d>,
    pub(super) tx: BufferedUartTx<'d>,
}

/// Buffered UART RX handle.
pub struct BufferedUartRx<'d> {
    pub(super) info: &'static Info,
    pub(super) state: &'static State,
    /// True for a handle produced by `split_ref`, which borrows the buffers rather than owning
    /// them and so must not tear the peripheral down when it goes out of scope.
    pub(super) is_borrowed: bool,
    pub(super) _phantom: PhantomData<&'d mut [u8]>,
}

/// Buffered UART TX handle.
pub struct BufferedUartTx<'d> {
    pub(super) info: &'static Info,
    pub(super) state: &'static State,
    /// See [`BufferedUartRx::is_borrowed`].
    pub(super) is_borrowed: bool,
    pub(super) _phantom: PhantomData<&'d mut [u8]>,
}

pub(super) fn init_buffers<'d>(
    info: &Info,
    state: &State,
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

    // From the datasheet:
    // "The transmit interrupt is based on a transition through a level, rather
    // than on the level itself. When the interrupt and the UART is enabled
    // before any data is written to the transmit FIFO the interrupt is not set.
    // The interrupt is only set, after written data leaves the single location
    // of the transmit FIFO and it becomes empty."
    //
    // This means we can leave the interrupt enabled the whole time as long as
    // we clear it after it happens. The downside is that the we manually have
    // to pend the ISR when we want data transmission to start.
    info.regs.uartimsc().write(|w| {
        w.set_rxim(true);
        w.set_rtim(true);
        w.set_txim(true);
    });

    info.interrupt.unpend();
    unsafe { info.interrupt.enable() };
}

impl<'d> BufferedUart<'d> {
    /// Create a buffered UART instance.
    pub fn new<T: Instance>(
        _uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        super::Uart::<'d, Async>::init(T::info(), Some(tx.into()), Some(rx.into()), None, None, config);
        init_buffers(T::info(), T::buffered_state(), Some(tx_buffer), Some(rx_buffer));

        Self {
            rx: BufferedUartRx {
                info: T::info(),
                state: T::buffered_state(),
                is_borrowed: false,
                _phantom: PhantomData,
            },
            tx: BufferedUartTx {
                info: T::info(),
                state: T::buffered_state(),
                is_borrowed: false,
                _phantom: PhantomData,
            },
        }
    }

    /// Create a buffered UART instance with flow control.
    pub fn new_with_rtscts<T: Instance>(
        _uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        rx: Peri<'d, impl RxPin<T>>,
        rts: Peri<'d, impl RtsPin<T>>,
        cts: Peri<'d, impl CtsPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        super::Uart::<'d, Async>::init(
            T::info(),
            Some(tx.into()),
            Some(rx.into()),
            Some(rts.into()),
            Some(cts.into()),
            config,
        );
        init_buffers(T::info(), T::buffered_state(), Some(tx_buffer), Some(rx_buffer));

        Self {
            rx: BufferedUartRx {
                info: T::info(),
                state: T::buffered_state(),
                is_borrowed: false,
                _phantom: PhantomData,
            },
            tx: BufferedUartTx {
                info: T::info(),
                state: T::buffered_state(),
                is_borrowed: false,
                _phantom: PhantomData,
            },
        }
    }

    /// Read from UART RX buffer.
    ///
    /// Waits until at least one byte is available, then reads as many bytes as
    /// are available (up to `buf.len()`) and returns the number of bytes read.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.rx.read(buf).await
    }

    /// Wait until data is available in the RX buffer, and return a slice to it.
    ///
    /// Call [`consume`](Self::consume) afterwards to mark bytes as read.
    pub async fn fill_buf(&mut self) -> Result<&[u8], Error> {
        self.rx.fill_buf().await
    }

    /// Mark `amt` bytes returned by [`fill_buf`](Self::fill_buf) as read.
    pub fn consume(&mut self, amt: usize) {
        self.rx.consume(amt)
    }

    /// Check whether data is available in the RX buffer, i.e. whether a read would not block.
    pub fn read_ready(&mut self) -> Result<bool, Error> {
        self.rx.read_ready()
    }

    /// Write to UART TX buffer.
    ///
    /// Waits until there is space in the TX buffer, then writes as many bytes as
    /// fit (up to `buf.len()`) and returns the number of bytes written.
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.tx.write(buf).await
    }

    /// Wait until all written bytes have been fully transmitted on the wire.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.tx.flush().await
    }

    /// Write to UART TX buffer blocking execution until done.
    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        self.tx.blocking_write(buffer)
    }

    /// Flush UART TX blocking execution until done.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    /// Read from UART RX buffer blocking execution until done.
    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
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

    /// sets baudrate on runtime
    pub fn set_baudrate(&mut self, baudrate: u32) {
        self.tx.set_baudrate(baudrate);
    }

    /// Set the configuration at runtime (ignores pin inversions)
    pub fn set_config(&mut self, config: Config) {
        self.tx.set_config(config);
    }

    /// Split into separate RX and TX handles.
    pub fn split(self) -> (BufferedUartTx<'d>, BufferedUartRx<'d>) {
        (self.tx, self.rx)
    }

    /// Split the Uart into a transmitter and receiver by mutable reference,
    /// which is particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    pub fn split_ref(&mut self) -> (BufferedUartTx<'_>, BufferedUartRx<'_>) {
        (self.tx.reborrow(), self.rx.reborrow())
    }
}

impl<'d> BufferedUartRx<'d> {
    /// Create a new buffered UART RX.
    pub fn new<T: Instance>(
        _uart: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        super::Uart::<'d, Async>::init(T::info(), None, Some(rx.into()), None, None, config);
        init_buffers(T::info(), T::buffered_state(), None, Some(rx_buffer));

        Self {
            info: T::info(),
            state: T::buffered_state(),
            is_borrowed: false,
            _phantom: PhantomData,
        }
    }

    /// Create a new buffered UART RX with flow control.
    pub fn new_with_rts<T: Instance>(
        _uart: Peri<'d, T>,
        rx: Peri<'d, impl RxPin<T>>,
        rts: Peri<'d, impl RtsPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        super::Uart::<'d, Async>::init(T::info(), None, Some(rx.into()), Some(rts.into()), None, config);
        init_buffers(T::info(), T::buffered_state(), None, Some(rx_buffer));

        Self {
            info: T::info(),
            state: T::buffered_state(),
            is_borrowed: false,
            _phantom: PhantomData,
        }
    }

    /// Read from UART RX buffer.
    ///
    /// Waits until at least one byte is available, then reads as many bytes as
    /// are available (up to `buf.len()`) and returns the number of bytes read.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let info = self.info;
        let state = self.state;
        poll_fn(move |cx| {
            // Register before `try_read`, which may (re-)enable the rx
            // interrupt on its way to returning pending. Doing it the other way
            // round leaves a window where the irq handler fires, finds no waker
            // registered, and the wakeup is lost with data already in the ring.
            state.rx_waker.register(cx.waker());
            Self::try_read(info, state, buf)
        })
        .await
    }

    fn get_rx_error(state: &State) -> Option<Error> {
        let errs = critical_section::with(|_| {
            let val = state.rx_error.load(Ordering::Relaxed);
            state.rx_error.store(0, Ordering::Relaxed);
            val
        });
        if errs & RXE_OVERRUN != 0 {
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

    fn try_read(info: &Info, state: &State, buf: &mut [u8]) -> Poll<Result<usize, Error>> {
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
                None => None,
                Some(e) => Some(Err(e)),
            }
        } else {
            Some(Ok(n))
        };

        // (Re-)Enable the interrupt to receive more data in case it was
        // disabled because the buffer was full or errors were detected.
        //
        // This has to happen on the pending path as well, not just when we
        // have something to report. The irq handler disables the interrupt on
        // error, and the reader that consumes the error flag may well be gone
        // by the time anyone looks again -- `embassy-net-ppp`, for instance,
        // abandons the transport on a read error. A fresh reader then arrives
        // to an empty buffer with no error left to observe, and without
        // re-enabling here it would wait on an interrupt that never comes.
        info.regs.uartimsc().write_set(|w| {
            w.set_rxim(true);
            w.set_rtim(true);
        });

        match result {
            Some(result) => Poll::Ready(result),
            None => Poll::Pending,
        }
    }

    /// Read from UART RX buffer blocking execution until done.
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        loop {
            match Self::try_read(self.info, self.state, buf) {
                Poll::Ready(res) => return res,
                Poll::Pending => continue,
            }
        }
    }

    /// Wait until data is available in the RX buffer, and return a slice to it.
    ///
    /// Call [`consume`](Self::consume) afterwards to mark bytes as read.
    pub async fn fill_buf(&mut self) -> Result<&[u8], Error> {
        let info = self.info;
        let state = self.state;
        poll_fn(move |cx| {
            let mut rx_reader = unsafe { state.rx_buf.reader() };
            let (p, n) = rx_reader.pop_buf();
            let result = if n == 0 {
                match Self::get_rx_error(state) {
                    None => {
                        state.rx_waker.register(cx.waker());
                        None
                    }
                    Some(e) => Some(Err(e)),
                }
            } else {
                let buf = unsafe { slice::from_raw_parts(p, n) };
                Some(Ok(buf))
            };

            // (Re-)Enable the interrupt to receive more data in case it was
            // disabled because the buffer was full or errors were detected.
            //
            // `consume` also does this, but it is only reached after a
            // successful fill. Returning an error here without re-arming would
            // leave the interrupt masked with no way back: the error flag has
            // just been consumed, so no later reader can observe it and
            // re-enable on our behalf. `embassy-net-ppp`, for example, gives up
            // on a read error without ever calling `consume`.
            info.regs.uartimsc().write_set(|w| {
                w.set_rxim(true);
                w.set_rtim(true);
            });

            match result {
                Some(result) => Poll::Ready(result),
                None => Poll::Pending,
            }
        })
        .await
    }

    /// Mark `amt` bytes returned by [`fill_buf`](Self::fill_buf) as read.
    pub fn consume(&mut self, amt: usize) {
        let info = self.info;
        let state = self.state;
        let mut rx_reader = unsafe { state.rx_buf.reader() };
        rx_reader.pop_done(amt);

        // (Re-)Enable the interrupt to receive more data in case it was
        // disabled because the buffer was full or errors were detected.
        info.regs.uartimsc().write_set(|w| {
            w.set_rxim(true);
            w.set_rtim(true);
        });
    }

    /// Check whether data is available in the RX buffer, i.e. whether a read would not block.
    pub fn read_ready(&mut self) -> Result<bool, Error> {
        Ok(!self.state.rx_buf.is_empty())
    }
}

impl<'d> BufferedUartTx<'d> {
    /// Create a new buffered UART TX.
    pub fn new<T: Instance>(
        _uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        super::Uart::<'d, Async>::init(T::info(), Some(tx.into()), None, None, None, config);
        init_buffers(T::info(), T::buffered_state(), Some(tx_buffer), None);

        Self {
            info: T::info(),
            state: T::buffered_state(),
            is_borrowed: false,
            _phantom: PhantomData,
        }
    }

    /// Create a new buffered UART TX with flow control.
    pub fn new_with_cts<T: Instance>(
        _uart: Peri<'d, T>,
        tx: Peri<'d, impl TxPin<T>>,
        cts: Peri<'d, impl CtsPin<T>>,
        _irq: impl Binding<T::Interrupt, BufferedInterruptHandler<T>>,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        super::Uart::<'d, Async>::init(T::info(), Some(tx.into()), None, None, Some(cts.into()), config);
        init_buffers(T::info(), T::buffered_state(), Some(tx_buffer), None);

        Self {
            info: T::info(),
            state: T::buffered_state(),
            is_borrowed: false,
            _phantom: PhantomData,
        }
    }

    /// Write to UART TX buffer.
    ///
    /// Waits until there is space in the TX buffer, then writes as many bytes as
    /// fit (up to `buf.len()`) and returns the number of bytes written.
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let info = self.info;
        let state = self.state;
        poll_fn(move |cx| {
            if buf.is_empty() {
                return Poll::Ready(Ok(0));
            }

            // Register before pushing, mirroring `read`. With the old
            // push-then-register order the irq handler could drain the whole
            // buffer and wake in the window between a failed push and the
            // register; its final drain pops an empty buffer and does not wake
            // again, so the writer parked forever on an empty buffer with the
            // FIFO idle. Registering first closes the window: any drain after
            // this wakes us for a re-poll.
            state.tx_waker.register(cx.waker());

            let mut tx_writer = unsafe { state.tx_buf.writer() };
            let n = tx_writer.push(|data| {
                let n = data.len().min(buf.len());
                data[..n].copy_from_slice(&buf[..n]);
                n
            });
            // The TX interrupt only fires on a transition through the FIFO
            // trigger level, so an empty FIFO never raises it again. Kick the
            // drain by hand; a full ring needs it just as much as a short write.
            info.interrupt.pend();

            if n == 0 {
                return Poll::Pending;
            }

            Poll::Ready(Ok(n))
        })
        .await
    }

    /// Wait until all written bytes have been fully transmitted on the wire.
    pub async fn flush(&mut self) -> Result<(), Error> {
        let info = self.info;
        let state = self.state;
        poll_fn(move |cx| {
            // Register before checking, for the same lost-wakeup window as in
            // `write` above.
            state.tx_waker.register(cx.waker());

            if !state.tx_buf.is_empty() {
                // Same one-shot TX interrupt hazard as in `write`: bytes are
                // still queued, so make sure something will shovel them out.
                info.interrupt.pend();
                return Poll::Pending;
            }

            Poll::Ready(())
        })
        .await;

        // The ring buffer is empty, but the hardware FIFO and shift register may not be.
        // There's no interrupt for that, so poll.
        while self.busy() {
            embassy_futures::yield_now().await;
        }
        Ok(())
    }

    /// Write to UART TX buffer blocking execution until done.
    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        loop {
            let mut tx_writer = unsafe { self.state.tx_buf.writer() };
            let n = tx_writer.push(|data| {
                let n = data.len().min(buf.len());
                data[..n].copy_from_slice(&buf[..n]);
                n
            });

            if n != 0 {
                // The TX interrupt only triggers when the there was data in the
                // FIFO and the number of bytes drops below a threshold. When the
                // FIFO was empty we have to manually pend the interrupt to shovel
                // TX data from the buffer into the FIFO.
                self.info.interrupt.pend();
                return Ok(n);
            }
        }
    }

    /// Flush UART TX blocking execution until done.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while !self.state.tx_buf.is_empty() {}
        while self.busy() {}
        Ok(())
    }

    /// Check if UART is busy.
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

        self.flush().await.unwrap();
        regs.uartlcr_h().write_set(|w| w.set_brk(true));
        Timer::after_micros(wait_usecs).await;
        regs.uartlcr_h().write_clear(|w| w.set_brk(true));
    }

    /// sets baudrate on runtime
    pub fn set_baudrate(&mut self, baudrate: u32) {
        super::Uart::<'d, Async>::set_baudrate_inner(self.info, baudrate);
    }

    /// Set the configuration at runtime (ignores pin inversions)
    pub fn set_config(&mut self, config: Config) {
        super::Uart::<'d, Async>::set_config_inner(self.info, config);
    }
}

impl<'d> BufferedUartRx<'d> {
    fn reborrow(&mut self) -> BufferedUartRx<'_> {
        BufferedUartRx {
            info: self.info,
            state: self.state,
            is_borrowed: true,
            _phantom: PhantomData,
        }
    }
}

impl<'d> BufferedUartTx<'d> {
    fn reborrow(&mut self) -> BufferedUartTx<'_> {
        BufferedUartTx {
            info: self.info,
            state: self.state,
            is_borrowed: true,
            _phantom: PhantomData,
        }
    }
}

impl<'d> Drop for BufferedUartRx<'d> {
    fn drop(&mut self) {
        if self.is_borrowed {
            return;
        }

        unsafe { self.state.rx_buf.deinit() }

        // TX is inactive if the buffer is not available.
        // We can now unregister the interrupt handler
        if !self.state.tx_buf.is_available() {
            self.info.interrupt.disable();
        }
    }
}

impl<'d> Drop for BufferedUartTx<'d> {
    fn drop(&mut self) {
        if self.is_borrowed {
            return;
        }

        unsafe { self.state.tx_buf.deinit() }

        // RX is inactive if the buffer is not available.
        // We can now unregister the interrupt handler
        if !self.state.rx_buf.is_available() {
            self.info.interrupt.disable();
        }
    }
}

/// Interrupt handler.
pub struct BufferedInterruptHandler<T: Instance> {
    _uart: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for BufferedInterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::info().regs;
        if r.uartdmacr().read().rxdmae() {
            return;
        }

        let s = T::buffered_state();

        // Clear TX and error interrupt flags
        // RX interrupt flags are cleared by reading from the FIFO.
        let ris = r.uartris().read();
        r.uarticr().write(|w| {
            w.set_txic(ris.txris());
            w.set_feic(ris.feris());
            w.set_peic(ris.peris());
            w.set_beic(ris.beris());
            w.set_oeic(ris.oeris());
        });

        // Errors
        if ris.feris() {
            warn!("Framing error");
        }
        if ris.peris() {
            warn!("Parity error");
        }
        if ris.beris() {
            warn!("Break error");
        }
        if ris.oeris() {
            warn!("Overrun error");
        }

        // RX
        if s.rx_buf.is_available() {
            let mut rx_writer = unsafe { s.rx_buf.writer() };
            let rx_buf = rx_writer.push_slice();
            let mut n_read = 0;
            let mut error = false;
            for rx_byte in rx_buf {
                if r.uartfr().read().rxfe() {
                    break;
                }
                let dr = r.uartdr().read();
                if (dr.0 >> 8) != 0 {
                    critical_section::with(|_| {
                        let val = s.rx_error.load(Ordering::Relaxed);
                        s.rx_error.store(val | ((dr.0 >> 8) as u8), Ordering::Relaxed);
                    });
                    error = true;
                    // only fill the buffer with valid characters. the current character is fine
                    // if the error is an overrun, but if we add it to the buffer we'll report
                    // the overrun one character too late. drop it instead and pretend we were
                    // a bit slower at draining the rx fifo than we actually were.
                    // this is consistent with blocking uart error reporting.
                    break;
                }
                *rx_byte = dr.data();
                n_read += 1;
            }
            if n_read > 0 {
                rx_writer.push_done(n_read);
                s.rx_waker.wake();
            } else if error {
                s.rx_waker.wake();
            }
            // Disable any further RX interrupts when the buffer becomes full or
            // errors have occurred. This lets us buffer additional errors in the
            // fifo without needing more error storage locations, and most applications
            // will want to do a full reset of their uart state anyway once an error
            // has happened.
            if s.rx_buf.is_full() || error {
                r.uartimsc().write_clear(|w| {
                    w.set_rxim(true);
                    w.set_rtim(true);
                });
            }
        }

        // TX
        if s.tx_buf.is_available() {
            let mut tx_reader = unsafe { s.tx_buf.reader() };
            let tx_buf = tx_reader.pop_slice();
            let mut n_written = 0;
            for tx_byte in tx_buf.iter_mut() {
                if r.uartfr().read().txff() {
                    break;
                }
                r.uartdr().write(|w| w.set_data(*tx_byte));
                n_written += 1;
            }
            if n_written > 0 {
                tx_reader.pop_done(n_written);
                s.tx_waker.wake();
            }
            // The TX interrupt only triggers once when the FIFO threshold is
            // crossed. No need to disable it when the buffer becomes empty
            // as it does re-trigger anymore once we have cleared it.
        }
    }
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
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
        BufferedUart::read(self, buf).await
    }
}

impl<'d> embedded_io_async::Read for BufferedUartRx<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        BufferedUartRx::read(self, buf).await
    }
}

impl<'d> embedded_io_async::ReadReady for BufferedUart<'d> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        BufferedUart::read_ready(self)
    }
}

impl<'d> embedded_io_async::ReadReady for BufferedUartRx<'d> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        BufferedUartRx::read_ready(self)
    }
}

impl<'d> embedded_io_async::BufRead for BufferedUart<'d> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        BufferedUart::fill_buf(self).await
    }

    fn consume(&mut self, amt: usize) {
        BufferedUart::consume(self, amt)
    }
}

impl<'d> embedded_io_async::BufRead for BufferedUartRx<'d> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        BufferedUartRx::fill_buf(self).await
    }

    fn consume(&mut self, amt: usize) {
        BufferedUartRx::consume(self, amt)
    }
}

impl<'d> embedded_io_async::Write for BufferedUart<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        BufferedUart::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        BufferedUart::flush(self).await
    }
}

impl<'d> embedded_io_async::Write for BufferedUartTx<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        BufferedUartTx::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        BufferedUartTx::flush(self).await
    }
}

impl<'d> embedded_io::Read for BufferedUart<'d> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf)
    }
}

impl<'d> embedded_io::Read for BufferedUartRx<'d> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf)
    }
}

impl<'d> embedded_io::Write for BufferedUart<'d> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

impl<'d> embedded_io::Write for BufferedUartTx<'d> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
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

impl<'d> embedded_hal_02::blocking::serial::Write<u8> for BufferedUart<'d> {
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
