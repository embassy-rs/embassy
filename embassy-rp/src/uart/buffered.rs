use core::future::{poll_fn, Future};
use core::slice;
use core::task::Poll;

use cortex_m::peripheral::NVIC;
use embassy_hal_common::atomic_ring_buffer::RingBuffer;
use embassy_sync::waitqueue::AtomicWaker;

use super::*;
use crate::interrupt::Registration;

pub(crate) struct State {
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

pub struct BufferedUart<'d> {
    info: &'static Info,
    phantom: PhantomData<&'d mut ()>,
}

pub struct BufferedUartRx<'d> {
    info: &'static Info,
    phantom: PhantomData<&'d mut ()>,
}

pub struct BufferedUartTx<'d> {
    info: &'static Info,
    phantom: PhantomData<&'d mut ()>,
}

impl<'d> BufferedUart<'d> {
    pub fn new<T: Instance>(
        _uart: impl Peripheral<P = T> + 'd,
        _irq: impl Registration<T::Interrupt>,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        into_ref!(tx, rx);
        Self::new_inner(
            T::info(),
            tx.map_into(),
            rx.map_into(),
            None,
            None,
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    pub fn new_with_rtscts<T: Instance>(
        _uart: impl Peripheral<P = T> + 'd,
        _irq: impl Registration<T::Interrupt>,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        rts: impl Peripheral<P = impl RtsPin<T>> + 'd,
        cts: impl Peripheral<P = impl CtsPin<T>> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        into_ref!(tx, rx, cts, rts);
        Self::new_inner(
            T::info(),
            tx.map_into(),
            rx.map_into(),
            Some(rts.map_into()),
            Some(cts.map_into()),
            tx_buffer,
            rx_buffer,
            config,
        )
    }

    fn new_inner(
        info: &'static Info,
        mut tx: PeripheralRef<'d, AnyPin>,
        mut rx: PeripheralRef<'d, AnyPin>,
        mut rts: Option<PeripheralRef<'d, AnyPin>>,
        mut cts: Option<PeripheralRef<'d, AnyPin>>,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> Self {
        init(
            info,
            Some(tx.reborrow()),
            Some(rx.reborrow()),
            rts.as_mut().map(|x| x.reborrow()),
            cts.as_mut().map(|x| x.reborrow()),
            config,
        );

        let len = tx_buffer.len();
        unsafe { info.state.tx_buf.init(tx_buffer.as_mut_ptr(), len) };
        let len = rx_buffer.len();
        unsafe { info.state.rx_buf.init(rx_buffer.as_mut_ptr(), len) };

        unsafe {
            info.regs.uartimsc().modify(|w| {
                w.set_rxim(true);
                w.set_rtim(true);
                w.set_txim(true);
            });

            NVIC::unpend(info.irq);
            NVIC::unmask(info.irq);
            NVIC::pend(info.irq);
        }

        Self {
            info,
            phantom: PhantomData,
        }
    }
}

impl<'d> BufferedUartRx<'d> {
    pub fn new<T: Instance>(
        _uart: impl Peripheral<P = T> + 'd,
        irq: impl Registration<T::Interrupt>,
        rx_buffer: &'d mut [u8],
        config: Config,
    ) -> BufferedUartRx<'d> {
        let info = T::info();
        unsafe {
            info.regs.uartimsc().modify(|w| {
                w.set_rxim(true);
                w.set_rtim(true);
            });

            NVIC::unpend(info.irq);
            NVIC::unmask(info.irq);
        }

        Self {
            info,
            phantom: PhantomData,
        }
    }
}

impl<'d> BufferedUartTx<'d> {
    pub fn new<T: Instance>(
        _uart: impl Peripheral<P = T> + 'd,
        irq: impl Registration<T::Interrupt>,
        tx_buffer: &'d mut [u8],
        config: Config,
    ) -> BufferedUartTx<'d> {
        let info = T::info();
        unsafe {
            info.regs.uartimsc().modify(|w| {
                w.set_txim(true);
            });

            NVIC::unpend(info.irq);
            NVIC::unmask(info.irq);
        }

        Self {
            info,
            phantom: PhantomData,
        }
    }
}

impl<'d> Drop for BufferedUart<'d> {
    fn drop(&mut self) {
        NVIC::mask(self.info.irq);
    }
}

impl<'d> Drop for BufferedUartRx<'d> {
    fn drop(&mut self) {
        NVIC::mask(self.info.irq);
    }
}

impl<'d> Drop for BufferedUartTx<'d> {
    fn drop(&mut self) {
        NVIC::mask(self.info.irq);
    }
}

pub(crate) fn on_interrupt(info: &'static Info) {
    trace!("on_interrupt");

    let r = info.regs;
    let s = info.state;

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

        let rx_writer = s.rx_buf.writer();
        if !r.uartfr().read().rxfe() {
            let val = r.uartdr().read().data();
            if !rx_writer.push_one(val) {
                warn!("RX buffer full, discard received byte");
            }
            s.rx_waker.wake();
        }

        // TX
        let tx_reader = s.tx_buf.reader();
        if let Some(val) = tx_reader.pop_one() {
            r.uartimsc().modify(|w| {
                w.set_txim(true);
            });
            r.uartdr().write(|w| w.set_data(val));
            s.tx_waker.wake();
        } else {
            // Disable interrupt until we have something to transmit again
            r.uartimsc().modify(|w| {
                w.set_txim(false);
            });
        }
    }
}

fn read<'a>(info: &'static Info, buf: &'a mut [u8]) -> impl Future<Output = Result<usize, Error>> + 'a {
    poll_fn(move |cx| {
        let rx_reader = unsafe { info.state.rx_buf.reader() };
        let n = rx_reader.pop(|data| {
            let n = data.len().min(buf.len());
            buf[..n].copy_from_slice(&data[..n]);
            n
        });
        if n == 0 {
            info.state.rx_waker.register(cx.waker());
            return Poll::Pending;
        }

        Poll::Ready(Ok(n))
    })
}

fn fill_buf<'a>(info: &'static Info) -> impl Future<Output = Result<&'a [u8], Error>> {
    poll_fn(move |cx| {
        let rx_reader = unsafe { info.state.rx_buf.reader() };
        let (p, n) = rx_reader.pop_buf();
        if n == 0 {
            info.state.rx_waker.register(cx.waker());
            return Poll::Pending;
        }

        let buf = unsafe { slice::from_raw_parts(p, n) };
        Poll::Ready(Ok(buf))
    })
}

fn consume(info: &'static Info, amt: usize) {
    let rx_reader = unsafe { info.state.rx_buf.reader() };
    rx_reader.pop_done(amt)
}

fn write<'a>(info: &'static Info, buf: &'a [u8]) -> impl Future<Output = Result<usize, Error>> + 'a {
    poll_fn(move |cx| {
        let tx_writer = unsafe { info.state.tx_buf.writer() };
        let n = tx_writer.push(|data| {
            let n = data.len().min(buf.len());
            data[..n].copy_from_slice(&buf[..n]);
            n
        });
        if n == 0 {
            info.state.tx_waker.register(cx.waker());
            return Poll::Pending;
        } else {
            NVIC::pend(info.irq);
        }

        Poll::Ready(Ok(n))
    })
}

fn flush(info: &'static Info) -> impl Future<Output = Result<(), Error>> {
    poll_fn(move |cx| {
        if !info.state.tx_buf.is_empty() {
            info.state.tx_waker.register(cx.waker());
            return Poll::Pending;
        }

        Poll::Ready(Ok(()))
    })
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl<'d> embedded_io::Io for BufferedUart<'d> {
    type Error = Error;
}

impl<'d> embedded_io::Io for BufferedUartRx<'d> {
    type Error = Error;
}

impl<'d> embedded_io::Io for BufferedUartTx<'d> {
    type Error = Error;
}

impl<'d> embedded_io::asynch::Read for BufferedUart<'d> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        read(self.info, buf)
    }
}

impl<'d> embedded_io::asynch::Read for BufferedUartRx<'d> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        read(self.info, buf)
    }
}

impl<'d> embedded_io::asynch::BufRead for BufferedUart<'d> {
    type FillBufFuture<'a> = impl Future<Output = Result<&'a [u8], Self::Error>> + 'a
    where
        Self: 'a;

    fn fill_buf<'a>(&'a mut self) -> Self::FillBufFuture<'a> {
        fill_buf(self.info)
    }

    fn consume(&mut self, amt: usize) {
        consume(self.info, amt)
    }
}

impl<'d> embedded_io::asynch::BufRead for BufferedUartRx<'d> {
    type FillBufFuture<'a> = impl Future<Output = Result<&'a [u8], Self::Error>> + 'a
    where
        Self: 'a;

    fn fill_buf<'a>(&'a mut self) -> Self::FillBufFuture<'a> {
        fill_buf(self.info)
    }

    fn consume(&mut self, amt: usize) {
        consume(self.info, amt)
    }
}

impl<'d> embedded_io::asynch::Write for BufferedUart<'d> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        write(self.info, buf)
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        flush(self.info)
    }
}

impl<'d> embedded_io::asynch::Write for BufferedUartTx<'d> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        write(self.info, buf)
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        flush(self.info)
    }
}
