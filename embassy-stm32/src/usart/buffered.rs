use core::future::poll_fn;
use core::slice;
use core::task::Poll;

use embassy_hal_internal::atomic_ring_buffer::RingBuffer;
use embassy_sync::waitqueue::AtomicWaker;

use super::*;
use crate::interrupt::typelevel::Interrupt;

/// Interrupt handler.
pub struct InterruptHandler<T: BasicInstance> {
    _phantom: PhantomData<T>,
}

impl<T: BasicInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let r = T::regs();
        let state = T::buffered_state();

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
                buf[0] = dr.unwrap();
                rx_writer.push_done(1);
            } else {
                // FIXME: Should we disable any further RX interrupts when the buffer becomes full.
            }

            if state.rx_buf.is_full() {
                state.rx_waker.wake();
            }
        }

        if sr_val.idle() {
            state.rx_waker.wake();
        }

        // TX
        if sr(r).read().txe() {
            let mut tx_reader = state.tx_buf.reader();
            let buf = tx_reader.pop_slice();
            if !buf.is_empty() {
                r.cr1().modify(|w| {
                    w.set_txeie(true);
                });
                tdr(r).write_volatile(buf[0].into());
                tx_reader.pop_done(1);
                state.tx_waker.wake();
            } else {
                // Disable interrupt until we have something to transmit again
                r.cr1().modify(|w| {
                    w.set_txeie(false);
                });
            }
        }
    }
}

pub struct State {
    rx_waker: AtomicWaker,
    rx_buf: RingBuffer,

    tx_waker: AtomicWaker,
    tx_buf: RingBuffer,
}

impl State {
    pub const fn new() -> Self {
        Self {
            rx_buf: RingBuffer::new(),
            tx_buf: RingBuffer::new(),
            rx_waker: AtomicWaker::new(),
            tx_waker: AtomicWaker::new(),
        }
    }
}

pub struct BufferedUart<'d, T: BasicInstance> {
    rx: BufferedUartRx<'d, T>,
    tx: BufferedUartTx<'d, T>,
}

pub struct BufferedUartTx<'d, T: BasicInstance> {
    phantom: PhantomData<&'d mut T>,
}

pub struct BufferedUartRx<'d, T: BasicInstance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: BasicInstance> BufferedUart<'d, T> {
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> BufferedUart<'d, T> {
        // UartRx and UartTx have one refcount ea.
        T::enable();
        T::enable();
        T::reset();

        Self::new_inner(peri, rx, tx, tx_buffer, rx_buffer, config)
    }

    pub fn new_with_rtscts(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> BufferedUart<'d, T> {
        into_ref!(cts, rts);

        // UartRx and UartTx have one refcount ea.
        T::enable();
        T::enable();
        T::reset();

        rts.set_as_af(rts.af_num(), AFType::OutputPushPull);
        cts.set_as_af(cts.af_num(), AFType::Input);
        T::regs().cr3().write(|w| {
            w.set_rtse(true);
            w.set_ctse(true);
        });

        Self::new_inner(peri, rx, tx, tx_buffer, rx_buffer, config)
    }

    #[cfg(not(any(usart_v1, usart_v2)))]
    pub fn new_with_de(
        peri: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        de: impl Peripheral<P = impl DePin<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> BufferedUart<'d, T> {
        into_ref!(de);

        // UartRx and UartTx have one refcount ea.
        T::enable();
        T::enable();
        T::reset();

        de.set_as_af(de.af_num(), AFType::OutputPushPull);
        T::regs().cr3().write(|w| {
            w.set_dem(true);
        });

        Self::new_inner(peri, rx, tx, tx_buffer, rx_buffer, config)
    }

    fn new_inner(
        _peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> BufferedUart<'d, T> {
        into_ref!(_peri, rx, tx);

        let state = T::buffered_state();
        let len = tx_buffer.len();
        unsafe { state.tx_buf.init(tx_buffer.as_mut_ptr(), len) };
        let len = rx_buffer.len();
        unsafe { state.rx_buf.init(rx_buffer.as_mut_ptr(), len) };

        let r = T::regs();
        rx.set_as_af(rx.af_num(), AFType::Input);
        tx.set_as_af(tx.af_num(), AFType::OutputPushPull);

        configure(r, &config, T::frequency(), T::KIND, true, true);

        r.cr1().modify(|w| {
            #[cfg(lpuart_v2)]
            w.set_fifoen(true);

            w.set_rxneie(true);
            w.set_idleie(true);
        });

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self {
            rx: BufferedUartRx { phantom: PhantomData },
            tx: BufferedUartTx { phantom: PhantomData },
        }
    }

    pub fn split(self) -> (BufferedUartTx<'d, T>, BufferedUartRx<'d, T>) {
        (self.tx, self.rx)
    }
}

impl<'d, T: BasicInstance> BufferedUartRx<'d, T> {
    async fn read(&self, buf: &mut [u8]) -> Result<usize, Error> {
        poll_fn(move |cx| {
            let state = T::buffered_state();
            let mut rx_reader = unsafe { state.rx_buf.reader() };
            let data = rx_reader.pop_slice();

            if !data.is_empty() {
                let len = data.len().min(buf.len());
                buf[..len].copy_from_slice(&data[..len]);

                let do_pend = state.rx_buf.is_full();
                rx_reader.pop_done(len);

                if do_pend {
                    T::Interrupt::pend();
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
            let state = T::buffered_state();
            let mut rx_reader = unsafe { state.rx_buf.reader() };
            let data = rx_reader.pop_slice();

            if !data.is_empty() {
                let len = data.len().min(buf.len());
                buf[..len].copy_from_slice(&data[..len]);

                let do_pend = state.rx_buf.is_full();
                rx_reader.pop_done(len);

                if do_pend {
                    T::Interrupt::pend();
                }

                return Ok(len);
            }
        }
    }

    async fn fill_buf(&self) -> Result<&[u8], Error> {
        poll_fn(move |cx| {
            let state = T::buffered_state();
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
        let state = T::buffered_state();
        let mut rx_reader = unsafe { state.rx_buf.reader() };
        let full = state.rx_buf.is_full();
        rx_reader.pop_done(amt);
        if full {
            T::Interrupt::pend();
        }
    }
}

impl<'d, T: BasicInstance> BufferedUartTx<'d, T> {
    async fn write(&self, buf: &[u8]) -> Result<usize, Error> {
        poll_fn(move |cx| {
            let state = T::buffered_state();
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
                T::Interrupt::pend();
            }

            Poll::Ready(Ok(n))
        })
        .await
    }

    async fn flush(&self) -> Result<(), Error> {
        poll_fn(move |cx| {
            let state = T::buffered_state();
            if !state.tx_buf.is_empty() {
                state.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            Poll::Ready(Ok(()))
        })
        .await
    }

    fn blocking_write(&self, buf: &[u8]) -> Result<usize, Error> {
        loop {
            let state = T::buffered_state();
            let empty = state.tx_buf.is_empty();

            let mut tx_writer = unsafe { state.tx_buf.writer() };
            let data = tx_writer.push_slice();
            if !data.is_empty() {
                let n = data.len().min(buf.len());
                data[..n].copy_from_slice(&buf[..n]);
                tx_writer.push_done(n);

                if empty {
                    T::Interrupt::pend();
                }

                return Ok(n);
            }
        }
    }

    fn blocking_flush(&self) -> Result<(), Error> {
        loop {
            let state = T::buffered_state();
            if state.tx_buf.is_empty() {
                return Ok(());
            }
        }
    }
}

impl<'d, T: BasicInstance> Drop for BufferedUartRx<'d, T> {
    fn drop(&mut self) {
        let state = T::buffered_state();
        unsafe {
            state.rx_buf.deinit();

            // TX is inactive if the the buffer is not available.
            // We can now unregister the interrupt handler
            if state.tx_buf.len() == 0 {
                T::Interrupt::disable();
            }
        }

        T::disable();
    }
}

impl<'d, T: BasicInstance> Drop for BufferedUartTx<'d, T> {
    fn drop(&mut self) {
        let state = T::buffered_state();
        unsafe {
            state.tx_buf.deinit();

            // RX is inactive if the the buffer is not available.
            // We can now unregister the interrupt handler
            if state.rx_buf.len() == 0 {
                T::Interrupt::disable();
            }
        }

        T::disable();
    }
}

impl embedded_io_async::Error for Error {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        embedded_io_async::ErrorKind::Other
    }
}

impl<'d, T: BasicInstance> embedded_io_async::ErrorType for BufferedUart<'d, T> {
    type Error = Error;
}

impl<'d, T: BasicInstance> embedded_io_async::ErrorType for BufferedUartRx<'d, T> {
    type Error = Error;
}

impl<'d, T: BasicInstance> embedded_io_async::ErrorType for BufferedUartTx<'d, T> {
    type Error = Error;
}

impl<'d, T: BasicInstance> embedded_io_async::Read for BufferedUart<'d, T> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf).await
    }
}

impl<'d, T: BasicInstance> embedded_io_async::Read for BufferedUartRx<'d, T> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Self::read(self, buf).await
    }
}

impl<'d, T: BasicInstance> embedded_io_async::BufRead for BufferedUart<'d, T> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        self.rx.fill_buf().await
    }

    fn consume(&mut self, amt: usize) {
        self.rx.consume(amt)
    }
}

impl<'d, T: BasicInstance> embedded_io_async::BufRead for BufferedUartRx<'d, T> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        Self::fill_buf(self).await
    }

    fn consume(&mut self, amt: usize) {
        Self::consume(self, amt)
    }
}

impl<'d, T: BasicInstance> embedded_io_async::Write for BufferedUart<'d, T> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush().await
    }
}

impl<'d, T: BasicInstance> embedded_io_async::Write for BufferedUartTx<'d, T> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Self::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Self::flush(self).await
    }
}

impl<'d, T: BasicInstance> embedded_io::Read for BufferedUart<'d, T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.blocking_read(buf)
    }
}

impl<'d, T: BasicInstance> embedded_io::Read for BufferedUartRx<'d, T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf)
    }
}

impl<'d, T: BasicInstance> embedded_io::Write for BufferedUart<'d, T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.blocking_write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.blocking_flush()
    }
}

impl<'d, T: BasicInstance> embedded_io::Write for BufferedUartTx<'d, T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Self::blocking_write(self, buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Self::blocking_flush(self)
    }
}

mod eh02 {
    use super::*;

    impl<'d, T: BasicInstance> embedded_hal_02::serial::Read<u8> for BufferedUartRx<'d, T> {
        type Error = Error;

        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            let r = T::regs();
            unsafe {
                let sr = sr(r).read();
                if sr.pe() {
                    rdr(r).read_volatile();
                    Err(nb::Error::Other(Error::Parity))
                } else if sr.fe() {
                    rdr(r).read_volatile();
                    Err(nb::Error::Other(Error::Framing))
                } else if sr.ne() {
                    rdr(r).read_volatile();
                    Err(nb::Error::Other(Error::Noise))
                } else if sr.ore() {
                    rdr(r).read_volatile();
                    Err(nb::Error::Other(Error::Overrun))
                } else if sr.rxne() {
                    Ok(rdr(r).read_volatile())
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
        }
    }

    impl<'d, T: BasicInstance> embedded_hal_02::blocking::serial::Write<u8> for BufferedUartTx<'d, T> {
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

    impl<'d, T: BasicInstance> embedded_hal_02::serial::Read<u8> for BufferedUart<'d, T> {
        type Error = Error;

        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            embedded_hal_02::serial::Read::read(&mut self.rx)
        }
    }

    impl<'d, T: BasicInstance> embedded_hal_02::blocking::serial::Write<u8> for BufferedUart<'d, T> {
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
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl<'d, T: BasicInstance> embedded_hal_nb::serial::ErrorType for BufferedUart<'d, T> {
        type Error = Error;
    }

    impl<'d, T: BasicInstance> embedded_hal_nb::serial::ErrorType for BufferedUartTx<'d, T> {
        type Error = Error;
    }

    impl<'d, T: BasicInstance> embedded_hal_nb::serial::ErrorType for BufferedUartRx<'d, T> {
        type Error = Error;
    }

    impl<'d, T: BasicInstance> embedded_hal_nb::serial::Read for BufferedUartRx<'d, T> {
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            embedded_hal_02::serial::Read::read(self)
        }
    }

    impl<'d, T: BasicInstance> embedded_hal_nb::serial::Write for BufferedUartTx<'d, T> {
        fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
            self.blocking_write(&[char]).map(drop).map_err(nb::Error::Other)
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.blocking_flush().map_err(nb::Error::Other)
        }
    }

    impl<'d, T: BasicInstance> embedded_hal_nb::serial::Read for BufferedUart<'d, T> {
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            embedded_hal_02::serial::Read::read(&mut self.rx)
        }
    }

    impl<'d, T: BasicInstance> embedded_hal_nb::serial::Write for BufferedUart<'d, T> {
        fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
            self.tx.blocking_write(&[char]).map(drop).map_err(nb::Error::Other)
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.tx.blocking_flush().map_err(nb::Error::Other)
        }
    }
}

#[cfg(all(
    feature = "unstable-traits",
    feature = "nightly",
    feature = "_todo_embedded_hal_serial"
))]
mod eha {
    use core::future::Future;

    use super::*;

    impl<'d, T: BasicInstance> embedded_hal_async::serial::Write for BufferedUartTx<'d, T> {
        async fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
            Self::write(buf)
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            Self::flush()
        }
    }

    impl<'d, T: BasicInstance> embedded_hal_async::serial::Read for BufferedUartRx<'d, T> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
            Self::read(buf)
        }
    }

    impl<'d, T: BasicInstance> embedded_hal_async::serial::Write for BufferedUart<'d, T> {
        async fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
            self.tx.write(buf)
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.tx.flush()
        }
    }

    impl<'d, T: BasicInstance> embedded_hal_async::serial::Read for BufferedUart<'d, T> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
            self.rx.read(buf)
        }
    }
}
