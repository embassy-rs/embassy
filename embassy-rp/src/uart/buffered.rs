use core::future::{poll_fn, Future};
use core::slice;
use core::task::Poll;

use embassy_cortex_m::interrupt::{Interrupt, InterruptExt};
use embassy_hal_common::atomic_ring_buffer::RingBuffer;
use embassy_sync::waitqueue::AtomicWaker;

use super::*;

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
    rx: BufferedUartRx<'d, T>,
    tx: BufferedUartTx<'d, T>,
}

pub struct BufferedUartRx<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

pub struct BufferedUartTx<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

fn init<'d, T: Instance + 'd>(
    irq: PeripheralRef<'d, T::Interrupt>,
    tx: Option<PeripheralRef<'d, AnyPin>>,
    rx: Option<PeripheralRef<'d, AnyPin>>,
    rts: Option<PeripheralRef<'d, AnyPin>>,
    cts: Option<PeripheralRef<'d, AnyPin>>,
    tx_buffer: &'d mut [u8],
    rx_buffer: &'d mut [u8],
    config: Config,
) {
    let regs = T::regs();
    unsafe {
        regs.uartimsc().modify(|w| {
            w.set_rxim(true);
            w.set_rtim(true);
            w.set_txim(true);
        });
    }

    super::Uart::<'d, T, Async>::init(tx, rx, rts, cts, config);

    let state = T::state();
    let len = tx_buffer.len();
    unsafe { state.tx_buf.init(tx_buffer.as_mut_ptr(), len) };
    let len = rx_buffer.len();
    unsafe { state.rx_buf.init(rx_buffer.as_mut_ptr(), len) };

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
        init::<T>(
            irq,
            Some(tx.map_into()),
            Some(rx.map_into()),
            None,
            None,
            tx_buffer,
            rx_buffer,
            config,
        );
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
        init::<T>(
            irq,
            Some(tx.map_into()),
            Some(rx.map_into()),
            Some(rts.map_into()),
            Some(cts.map_into()),
            tx_buffer,
            rx_buffer,
            config,
        );
        Self {
            rx: BufferedUartRx { phantom: PhantomData },
            tx: BufferedUartTx { phantom: PhantomData },
        }
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
        init::<T>(irq, None, Some(rx.map_into()), None, None, &mut [], rx_buffer, config);
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
        init::<T>(
            irq,
            None,
            Some(rx.map_into()),
            Some(rts.map_into()),
            None,
            &mut [],
            rx_buffer,
            config,
        );
        Self { phantom: PhantomData }
    }

    fn read<'a>(buf: &'a mut [u8]) -> impl Future<Output = Result<usize, Error>> + 'a {
        poll_fn(move |cx| {
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

            Poll::Ready(Ok(n))
        })
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
        init::<T>(irq, Some(tx.map_into()), None, None, None, tx_buffer, &mut [], config);
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
        init::<T>(
            irq,
            Some(tx.map_into()),
            None,
            None,
            Some(cts.map_into()),
            tx_buffer,
            &mut [],
            config,
        );
        Self { phantom: PhantomData }
    }

    fn write<'a>(buf: &'a [u8]) -> impl Future<Output = Result<usize, Error>> + 'a {
        poll_fn(move |cx| {
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
    trace!("on_interrupt");

    let r = T::regs();
    let s = T::state();

    unsafe {
        // RX

        let ris = r.uartris().read();
        // Clear interrupt flags
        r.uarticr().write(|w| {
            w.set_rxic(true);
            w.set_rtic(true);
        });

        if ris.peris() {
            warn!("Parity error");
            r.uarticr().write(|w| {
                w.set_peic(true);
            });
        }
        if ris.feris() {
            warn!("Framing error");
            r.uarticr().write(|w| {
                w.set_feic(true);
            });
        }
        if ris.beris() {
            warn!("Break error");
            r.uarticr().write(|w| {
                w.set_beic(true);
            });
        }
        if ris.oeris() {
            warn!("Overrun error");
            r.uarticr().write(|w| {
                w.set_oeic(true);
            });
        }

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

        // TX
        let mut tx_reader = s.tx_buf.reader();
        let tx_buf = tx_reader.pop_slice();
        if tx_buf.len() == 0 {
            // Disable interrupt until we have something to transmit again
            r.uartimsc().modify(|w| {
                w.set_txim(false);
            });
        } else {
            r.uartimsc().modify(|w| {
                w.set_txim(true);
            });

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
        }
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
