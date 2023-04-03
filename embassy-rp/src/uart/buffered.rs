use core::future::{poll_fn, Future};
use core::slice;
use core::task::Poll;

use embassy_cortex_m::interrupt::{Interrupt, InterruptExt};
use embassy_hal_common::atomic_ring_buffer::RingBuffer;
use embassy_sync::waitqueue::AtomicWaker;

use super::*;
use crate::RegExt;

pub struct State {
    tx_waker: AtomicWaker,
    tx_buf: RingBuffer,
    rx_waker: AtomicWaker,
    rx_buf: RingBuffer,
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

pub struct BufferedUart<'d, T: Instance> {
    pub(crate) rx: BufferedUartRx<'d, T>,
    pub(crate) tx: BufferedUartTx<'d, T>,
}

pub struct BufferedUartRx<'d, T: Instance> {
    pub(crate) phantom: PhantomData<&'d mut T>,
}

pub struct BufferedUartTx<'d, T: Instance> {
    pub(crate) phantom: PhantomData<&'d mut T>,
}

pub(crate) fn init_buffers<'d, T: Instance + 'd>(
    irq: PeripheralRef<'d, T::Interrupt>,
    tx_buffer: &'d mut [u8],
    rx_buffer: &'d mut [u8],
) {
    let state = T::state();
    let len = tx_buffer.len();
    unsafe { state.tx_buf.init(tx_buffer.as_mut_ptr(), len) };
    let len = rx_buffer.len();
    unsafe { state.rx_buf.init(rx_buffer.as_mut_ptr(), len) };

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
    let regs = T::regs();
    unsafe {
        regs.uartimsc().write_set(|w| {
            w.set_rxim(true);
            w.set_rtim(true);
            w.set_txim(true);
        });
    };

    irq.set_handler(on_interrupt::<T>);
    irq.unpend();
    irq.enable();
}

impl<'d, T: Instance> BufferedUart<'d, T> {
    pub fn new(
        _uart: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        into_ref!(irq, tx, rx);

        super::Uart::<'d, T, Async>::init(Some(tx.map_into()), Some(rx.map_into()), None, None, config);
        init_buffers::<T>(irq, tx_buffer, rx_buffer);

        Self {
            rx: BufferedUartRx { phantom: PhantomData },
            tx: BufferedUartTx { phantom: PhantomData },
        }
    }

    pub fn new_with_rtscts(
        _uart: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        into_ref!(irq, tx, rx, cts, rts);

        super::Uart::<'d, T, Async>::init(
            Some(tx.map_into()),
            Some(rx.map_into()),
            Some(rts.map_into()),
            Some(cts.map_into()),
            config,
        );
        init_buffers::<T>(irq, tx_buffer, rx_buffer);

        Self {
            rx: BufferedUartRx { phantom: PhantomData },
            tx: BufferedUartTx { phantom: PhantomData },
        }
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<usize, Error> {
        self.tx.blocking_write(buffer)
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.tx.blocking_flush()
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        self.rx.blocking_read(buffer)
    }

    pub fn split(self) -> (BufferedUartRx<'d, T>, BufferedUartTx<'d, T>) {
        (self.rx, self.tx)
    }
}

impl<'d, T: Instance> BufferedUartRx<'d, T> {
    pub fn new(
        _uart: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        into_ref!(irq, rx);

        super::Uart::<'d, T, Async>::init(None, Some(rx.map_into()), None, None, config);
        init_buffers::<T>(irq, &mut [], rx_buffer);

        Self { phantom: PhantomData }
    }

    pub fn new_with_rts(
        _uart: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        into_ref!(irq, rx, rts);

        super::Uart::<'d, T, Async>::init(None, Some(rx.map_into()), Some(rts.map_into()), None, config);
        init_buffers::<T>(irq, &mut [], rx_buffer);

        Self { phantom: PhantomData }
    }

    fn read<'a>(buf: &'a mut [u8]) -> impl Future<Output = Result<usize, Error>> + 'a {
        poll_fn(move |cx| {
            if buf.is_empty() {
                return Poll::Ready(Ok(0));
            }

            let state = T::state();
            let mut rx_reader = unsafe { state.rx_buf.reader() };
            let n = rx_reader.pop(|data| {
                let n = data.len().min(buf.len());
                buf[..n].copy_from_slice(&data[..n]);
                n
            });
            if n == 0 {
                state.rx_waker.register(cx.waker());
                return Poll::Pending;
            }

            // (Re-)Enable the interrupt to receive more data in case it was
            // disabled because the buffer was full.
            let regs = T::regs();
            unsafe {
                regs.uartimsc().write_set(|w| {
                    w.set_rxim(true);
                    w.set_rtim(true);
                });
            }

            Poll::Ready(Ok(n))
        })
    }

    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        loop {
            let state = T::state();
            let mut rx_reader = unsafe { state.rx_buf.reader() };
            let n = rx_reader.pop(|data| {
                let n = data.len().min(buf.len());
                buf[..n].copy_from_slice(&data[..n]);
                n
            });

            if n > 0 {
                // (Re-)Enable the interrupt to receive more data in case it was
                // disabled because the buffer was full.
                let regs = T::regs();
                unsafe {
                    regs.uartimsc().write_set(|w| {
                        w.set_rxim(true);
                        w.set_rtim(true);
                    });
                }

                return Ok(n);
            }
        }
    }

    fn fill_buf<'a>() -> impl Future<Output = Result<&'a [u8], Error>> {
        poll_fn(move |cx| {
            let state = T::state();
            let mut rx_reader = unsafe { state.rx_buf.reader() };
            let (p, n) = rx_reader.pop_buf();
            if n == 0 {
                state.rx_waker.register(cx.waker());
                return Poll::Pending;
            }

            let buf = unsafe { slice::from_raw_parts(p, n) };
            Poll::Ready(Ok(buf))
        })
    }

    fn consume(amt: usize) {
        let state = T::state();
        let mut rx_reader = unsafe { state.rx_buf.reader() };
        rx_reader.pop_done(amt);

        // (Re-)Enable the interrupt to receive more data in case it was
        // disabled because the buffer was full.
        let regs = T::regs();
        unsafe {
            regs.uartimsc().write_set(|w| {
                w.set_rxim(true);
                w.set_rtim(true);
            });
        }
    }
}

impl<'d, T: Instance> BufferedUartTx<'d, T> {
    pub fn new(
        _uart: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        into_ref!(irq, tx);

        super::Uart::<'d, T, Async>::init(Some(tx.map_into()), None, None, None, config);
        init_buffers::<T>(irq, tx_buffer, &mut []);

        Self { phantom: PhantomData }
    }

    pub fn new_with_cts(
        _uart: impl Peripheral<P = T> + 'd,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        into_ref!(irq, tx, cts);

        super::Uart::<'d, T, Async>::init(Some(tx.map_into()), None, None, Some(cts.map_into()), config);
        init_buffers::<T>(irq, tx_buffer, &mut []);

        Self { phantom: PhantomData }
    }

    fn write<'a>(buf: &'a [u8]) -> impl Future<Output = Result<usize, Error>> + 'a {
        poll_fn(move |cx| {
            if buf.is_empty() {
                return Poll::Ready(Ok(0));
            }

            let state = T::state();
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
            unsafe { T::Interrupt::steal() }.pend();
            Poll::Ready(Ok(n))
        })
    }

    fn flush() -> impl Future<Output = Result<(), Error>> {
        poll_fn(move |cx| {
            let state = T::state();
            if !state.tx_buf.is_empty() {
                state.tx_waker.register(cx.waker());
                return Poll::Pending;
            }

            Poll::Ready(Ok(()))
        })
    }

    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        loop {
            let state = T::state();
            let mut tx_writer = unsafe { state.tx_buf.writer() };
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
                unsafe { T::Interrupt::steal() }.pend();
                return Ok(n);
            }
        }
    }

    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        loop {
            let state = T::state();
            if state.tx_buf.is_empty() {
                return Ok(());
            }
        }
    }
}

impl<'d, T: Instance> Drop for BufferedUartRx<'d, T> {
    fn drop(&mut self) {
        let state = T::state();
        unsafe {
            state.rx_buf.deinit();

            // TX is inactive if the the buffer is not available.
            // We can now unregister the interrupt handler
            if state.tx_buf.len() == 0 {
                T::Interrupt::steal().disable();
            }
        }
    }
}

impl<'d, T: Instance> Drop for BufferedUartTx<'d, T> {
    fn drop(&mut self) {
        let state = T::state();
        unsafe {
            state.tx_buf.deinit();

            // RX is inactive if the the buffer is not available.
            // We can now unregister the interrupt handler
            if state.rx_buf.len() == 0 {
                T::Interrupt::steal().disable();
            }
        }
    }
}

pub(crate) unsafe fn on_interrupt<T: Instance>(_: *mut ()) {
    let r = T::regs();
    let s = T::state();

    unsafe {
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

        trace!("on_interrupt ris={:#X}", ris.0);

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
        let mut rx_writer = s.rx_buf.writer();
        let rx_buf = rx_writer.push_slice();
        let mut n_read = 0;
        for rx_byte in rx_buf {
            if r.uartfr().read().rxfe() {
                break;
            }
            *rx_byte = r.uartdr().read().data();
            n_read += 1;
        }
        if n_read > 0 {
            rx_writer.push_done(n_read);
            s.rx_waker.wake();
        }
        // Disable any further RX interrupts when the buffer becomes full.
        if s.rx_buf.is_full() {
            r.uartimsc().write_clear(|w| {
                w.set_rxim(true);
                w.set_rtim(true);
            });
        }

        // TX
        let mut tx_reader = s.tx_buf.reader();
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

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl<'d, T: Instance> embedded_io::Io for BufferedUart<'d, T> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::Io for BufferedUartRx<'d, T> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::Io for BufferedUartTx<'d, T> {
    type Error = Error;
}

impl<'d, T: Instance + 'd> embedded_io::asynch::Read for BufferedUart<'d, T> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        BufferedUartRx::<'d, T>::read(buf).await
    }
}

impl<'d, T: Instance + 'd> embedded_io::asynch::Read for BufferedUartRx<'d, T> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Self::read(buf).await
    }
}

impl<'d, T: Instance + 'd> embedded_io::asynch::BufRead for BufferedUart<'d, T> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        BufferedUartRx::<'d, T>::fill_buf().await
    }

    fn consume(&mut self, amt: usize) {
        BufferedUartRx::<'d, T>::consume(amt)
    }
}

impl<'d, T: Instance + 'd> embedded_io::asynch::BufRead for BufferedUartRx<'d, T> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        Self::fill_buf().await
    }

    fn consume(&mut self, amt: usize) {
        Self::consume(amt)
    }
}

impl<'d, T: Instance + 'd> embedded_io::asynch::Write for BufferedUart<'d, T> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        BufferedUartTx::<'d, T>::write(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        BufferedUartTx::<'d, T>::flush().await
    }
}

impl<'d, T: Instance + 'd> embedded_io::asynch::Write for BufferedUartTx<'d, T> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Self::write(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Self::flush().await
    }
}

impl<'d, T: Instance + 'd> embedded_io::blocking::Read for BufferedUart<'d, T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.blocking_read(buf)
    }
}

impl<'d, T: Instance + 'd> embedded_io::blocking::Read for BufferedUartRx<'d, T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.blocking_read(buf)
    }
}

impl<'d, T: Instance + 'd> embedded_io::blocking::Write for BufferedUart<'d, T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.blocking_write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.blocking_flush()
    }
}

impl<'d, T: Instance + 'd> embedded_io::blocking::Write for BufferedUartTx<'d, T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.blocking_write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.blocking_flush()
    }
}

mod eh02 {
    use super::*;

    impl<'d, T: Instance> embedded_hal_02::serial::Read<u8> for BufferedUartRx<'d, T> {
        type Error = Error;

        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            let r = T::regs();
            unsafe {
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
    }

    impl<'d, T: Instance> embedded_hal_02::blocking::serial::Write<u8> for BufferedUartTx<'d, T> {
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

    impl<'d, T: Instance> embedded_hal_02::serial::Read<u8> for BufferedUart<'d, T> {
        type Error = Error;

        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            embedded_hal_02::serial::Read::read(&mut self.rx)
        }
    }

    impl<'d, T: Instance> embedded_hal_02::blocking::serial::Write<u8> for BufferedUart<'d, T> {
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
}

#[cfg(feature = "unstable-traits")]
mod eh1 {
    use super::*;

    impl<'d, T: Instance> embedded_hal_1::serial::ErrorType for BufferedUart<'d, T> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_1::serial::ErrorType for BufferedUartTx<'d, T> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_1::serial::ErrorType for BufferedUartRx<'d, T> {
        type Error = Error;
    }

    impl<'d, T: Instance> embedded_hal_nb::serial::Read for BufferedUartRx<'d, T> {
        fn read(&mut self) -> nb::Result<u8, Self::Error> {
            embedded_hal_02::serial::Read::read(self)
        }
    }

    impl<'d, T: Instance> embedded_hal_1::serial::Write for BufferedUartTx<'d, T> {
        fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer).map(drop)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<'d, T: Instance> embedded_hal_nb::serial::Write for BufferedUartTx<'d, T> {
        fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
            self.blocking_write(&[char]).map(drop).map_err(nb::Error::Other)
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.blocking_flush().map_err(nb::Error::Other)
        }
    }

    impl<'d, T: Instance> embedded_hal_nb::serial::Read for BufferedUart<'d, T> {
        fn read(&mut self) -> Result<u8, nb::Error<Self::Error>> {
            embedded_hal_02::serial::Read::read(&mut self.rx)
        }
    }

    impl<'d, T: Instance> embedded_hal_1::serial::Write for BufferedUart<'d, T> {
        fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            self.blocking_write(buffer).map(drop)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            self.blocking_flush()
        }
    }

    impl<'d, T: Instance> embedded_hal_nb::serial::Write for BufferedUart<'d, T> {
        fn write(&mut self, char: u8) -> nb::Result<(), Self::Error> {
            self.blocking_write(&[char]).map(drop).map_err(nb::Error::Other)
        }

        fn flush(&mut self) -> nb::Result<(), Self::Error> {
            self.blocking_flush().map_err(nb::Error::Other)
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

    impl<'d, T: Instance> embedded_hal_async::serial::Write for BufferedUartTx<'d, T> {
        type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            Self::write(buf)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            Self::flush()
        }
    }

    impl<'d, T: Instance> embedded_hal_async::serial::Read for BufferedUartRx<'d, T> {
        type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            Self::read(buf)
        }
    }

    impl<'d, T: Instance> embedded_hal_async::serial::Write for BufferedUart<'d, T> {
        type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            BufferedUartTx::<'d, T>::write(buf)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            BufferedUartTx::<'d, T>::flush()
        }
    }

    impl<'d, T: Instance> embedded_hal_async::serial::Read for BufferedUart<'d, T> {
        type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            BufferedUartRx::<'d, T>::read(buf)
        }
    }
}
