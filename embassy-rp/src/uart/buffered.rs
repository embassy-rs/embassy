use core::future::Future;
use core::task::{Poll, Waker};

use atomic_polyfill::{compiler_fence, Ordering};
use embassy_cortex_m::peripheral::{PeripheralMutex, PeripheralState, StateStorage};
use embassy_hal_common::ring_buffer::RingBuffer;
use embassy_sync::waitqueue::WakerRegistration;
use futures::future::poll_fn;

use super::*;

pub struct State<'d, T: Instance>(StateStorage<FullStateInner<'d, T>>);
impl<'d, T: Instance> State<'d, T> {
    pub const fn new() -> Self {
        Self(StateStorage::new())
    }
}

pub struct RxState<'d, T: Instance>(StateStorage<RxStateInner<'d, T>>);
impl<'d, T: Instance> RxState<'d, T> {
    pub const fn new() -> Self {
        Self(StateStorage::new())
    }
}

pub struct TxState<'d, T: Instance>(StateStorage<TxStateInner<'d, T>>);
impl<'d, T: Instance> TxState<'d, T> {
    pub const fn new() -> Self {
        Self(StateStorage::new())
    }
}

struct RxStateInner<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,

    waker: WakerRegistration,
    buf: RingBuffer<'d>,
}

struct TxStateInner<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,

    waker: WakerRegistration,
    buf: RingBuffer<'d>,
}

struct FullStateInner<'d, T: Instance> {
    rx: RxStateInner<'d, T>,
    tx: TxStateInner<'d, T>,
}

unsafe impl<'d, T: Instance> Send for RxStateInner<'d, T> {}
unsafe impl<'d, T: Instance> Sync for RxStateInner<'d, T> {}

unsafe impl<'d, T: Instance> Send for TxStateInner<'d, T> {}
unsafe impl<'d, T: Instance> Sync for TxStateInner<'d, T> {}

unsafe impl<'d, T: Instance> Send for FullStateInner<'d, T> {}
unsafe impl<'d, T: Instance> Sync for FullStateInner<'d, T> {}

pub struct BufferedUart<'d, T: Instance> {
    inner: PeripheralMutex<'d, FullStateInner<'d, T>>,
}

pub struct RxBufferedUart<'d, T: Instance> {
    inner: PeripheralMutex<'d, RxStateInner<'d, T>>,
}

pub struct TxBufferedUart<'d, T: Instance> {
    inner: PeripheralMutex<'d, TxStateInner<'d, T>>,
}

impl<'d, T: Instance> Unpin for BufferedUart<'d, T> {}
impl<'d, T: Instance> Unpin for RxBufferedUart<'d, T> {}
impl<'d, T: Instance> Unpin for TxBufferedUart<'d, T> {}

impl<'d, T: Instance> BufferedUart<'d, T> {
    pub fn new<M: Mode>(
        state: &'d mut State<'d, T>,
        _uart: Uart<'d, T, M>,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
    ) -> BufferedUart<'d, T> {
        into_ref!(irq);

        let r = T::regs();
        unsafe {
            r.uartimsc().modify(|w| {
                w.set_rxim(true);
                w.set_rtim(true);
                w.set_txim(true);
            });
        }

        Self {
            inner: PeripheralMutex::new(irq, &mut state.0, move || FullStateInner {
                tx: TxStateInner {
                    phantom: PhantomData,
                    waker: WakerRegistration::new(),
                    buf: RingBuffer::new(tx_buffer),
                },
                rx: RxStateInner {
                    phantom: PhantomData,
                    waker: WakerRegistration::new(),
                    buf: RingBuffer::new(rx_buffer),
                },
            }),
        }
    }
}

impl<'d, T: Instance> RxBufferedUart<'d, T> {
    pub fn new<M: Mode>(
        state: &'d mut RxState<'d, T>,
        _uart: UartRx<'d, T, M>,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        rx_buffer: &'d mut [u8],
    ) -> RxBufferedUart<'d, T> {
        into_ref!(irq);

        let r = T::regs();
        unsafe {
            r.uartimsc().modify(|w| {
                w.set_rxim(true);
                w.set_rtim(true);
            });
        }

        Self {
            inner: PeripheralMutex::new(irq, &mut state.0, move || RxStateInner {
                phantom: PhantomData,

                buf: RingBuffer::new(rx_buffer),
                waker: WakerRegistration::new(),
            }),
        }
    }
}

impl<'d, T: Instance> TxBufferedUart<'d, T> {
    pub fn new<M: Mode>(
        state: &'d mut TxState<'d, T>,
        _uart: UartTx<'d, T, M>,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        tx_buffer: &'d mut [u8],
    ) -> TxBufferedUart<'d, T> {
        into_ref!(irq);

        let r = T::regs();
        unsafe {
            r.uartimsc().modify(|w| {
                w.set_txim(true);
            });
        }

        Self {
            inner: PeripheralMutex::new(irq, &mut state.0, move || TxStateInner {
                phantom: PhantomData,

                buf: RingBuffer::new(tx_buffer),
                waker: WakerRegistration::new(),
            }),
        }
    }
}

impl<'d, T: Instance> PeripheralState for FullStateInner<'d, T>
where
    Self: 'd,
{
    type Interrupt = T::Interrupt;
    fn on_interrupt(&mut self) {
        self.rx.on_interrupt();
        self.tx.on_interrupt();
    }
}

impl<'d, T: Instance> RxStateInner<'d, T>
where
    Self: 'd,
{
    fn read(&mut self, buf: &mut [u8], waker: &Waker) -> (Poll<Result<usize, Error>>, bool) {
        // We have data ready in buffer? Return it.
        let mut do_pend = false;
        let data = self.buf.pop_buf();
        if !data.is_empty() {
            let len = data.len().min(buf.len());
            buf[..len].copy_from_slice(&data[..len]);

            if self.buf.is_full() {
                do_pend = true;
            }
            self.buf.pop(len);

            return (Poll::Ready(Ok(len)), do_pend);
        }

        self.waker.register(waker);
        (Poll::Pending, do_pend)
    }

    fn fill_buf<'a>(&mut self, waker: &Waker) -> Poll<Result<&'a [u8], Error>> {
        // We have data ready in buffer? Return it.
        let buf = self.buf.pop_buf();
        if !buf.is_empty() {
            let buf: &[u8] = buf;
            // Safety: buffer lives as long as uart
            let buf: &[u8] = unsafe { core::mem::transmute(buf) };
            return Poll::Ready(Ok(buf));
        }

        self.waker.register(waker);
        Poll::Pending
    }

    fn consume(&mut self, amt: usize) -> bool {
        let full = self.buf.is_full();
        self.buf.pop(amt);
        full
    }
}

impl<'d, T: Instance> PeripheralState for RxStateInner<'d, T>
where
    Self: 'd,
{
    type Interrupt = T::Interrupt;
    fn on_interrupt(&mut self) {
        let r = T::regs();
        unsafe {
            let ris = r.uartmis().read();
            // Clear interrupt flags
            r.uarticr().modify(|w| {
                w.set_rxic(true);
                w.set_rtic(true);
            });

            if ris.rxmis() {
                if ris.pemis() {
                    warn!("Parity error");
                    r.uarticr().modify(|w| {
                        w.set_peic(true);
                    });
                }
                if ris.femis() {
                    warn!("Framing error");
                    r.uarticr().modify(|w| {
                        w.set_feic(true);
                    });
                }
                if ris.bemis() {
                    warn!("Break error");
                    r.uarticr().modify(|w| {
                        w.set_beic(true);
                    });
                }
                if ris.oemis() {
                    warn!("Overrun error");
                    r.uarticr().modify(|w| {
                        w.set_oeic(true);
                    });
                }

                let buf = self.buf.push_buf();
                if !buf.is_empty() {
                    buf[0] = r.uartdr().read().data();
                    self.buf.push(1);
                } else {
                    warn!("RX buffer full, discard received byte");
                }

                if self.buf.is_full() {
                    self.waker.wake();
                }
            }

            if ris.rtmis() {
                self.waker.wake();
            };
        }
    }
}

impl<'d, T: Instance> TxStateInner<'d, T>
where
    Self: 'd,
{
    fn write(&mut self, buf: &[u8], waker: &Waker) -> (Poll<Result<usize, Error>>, bool) {
        let empty = self.buf.is_empty();
        let tx_buf = self.buf.push_buf();
        if tx_buf.is_empty() {
            self.waker.register(waker);
            return (Poll::Pending, empty);
        }

        let n = core::cmp::min(tx_buf.len(), buf.len());
        tx_buf[..n].copy_from_slice(&buf[..n]);
        self.buf.push(n);

        (Poll::Ready(Ok(n)), empty)
    }

    fn flush(&mut self, waker: &Waker) -> Poll<Result<(), Error>> {
        if !self.buf.is_empty() {
            self.waker.register(waker);
            return Poll::Pending;
        }

        Poll::Ready(Ok(()))
    }
}

impl<'d, T: Instance> PeripheralState for TxStateInner<'d, T>
where
    Self: 'd,
{
    type Interrupt = T::Interrupt;
    fn on_interrupt(&mut self) {
        let r = T::regs();
        unsafe {
            let ris = r.uartris().read();
            // Clear interrupt flags
            r.uarticr().write(|w| {
                w.set_rtic(true);
            });

            if ris.txris() {
                let buf = self.buf.pop_buf();
                if !buf.is_empty() {
                    r.uartimsc().modify(|w| {
                        w.set_txim(true);
                    });
                    r.uartdr().write(|w| w.set_data(buf[0].into()));
                    self.buf.pop(1);
                    self.waker.wake();
                } else {
                    // Disable interrupt until we have something to transmit again
                    r.uartimsc().modify(|w| {
                        w.set_txim(false);
                    });
                }
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

impl<'d, T: Instance> embedded_io::Io for RxBufferedUart<'d, T> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::Io for TxBufferedUart<'d, T> {
    type Error = Error;
}

impl<'d, T: Instance + 'd> embedded_io::asynch::Read for BufferedUart<'d, T> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        poll_fn(move |cx| {
            let (res, do_pend) = self.inner.with(|state| {
                compiler_fence(Ordering::SeqCst);
                state.rx.read(buf, cx.waker())
            });

            if do_pend {
                self.inner.pend();
            }

            res
        })
    }
}

impl<'d, T: Instance + 'd> embedded_io::asynch::Read for RxBufferedUart<'d, T> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        poll_fn(move |cx| {
            let (res, do_pend) = self.inner.with(|state| {
                compiler_fence(Ordering::SeqCst);
                state.read(buf, cx.waker())
            });

            if do_pend {
                self.inner.pend();
            }

            res
        })
    }
}

impl<'d, T: Instance + 'd> embedded_io::asynch::BufRead for BufferedUart<'d, T> {
    type FillBufFuture<'a> = impl Future<Output = Result<&'a [u8], Self::Error>>
    where
        Self: 'a;

    fn fill_buf<'a>(&'a mut self) -> Self::FillBufFuture<'a> {
        poll_fn(move |cx| {
            self.inner.with(|state| {
                compiler_fence(Ordering::SeqCst);
                state.rx.fill_buf(cx.waker())
            })
        })
    }

    fn consume(&mut self, amt: usize) {
        let signal = self.inner.with(|state| state.rx.consume(amt));
        if signal {
            self.inner.pend();
        }
    }
}

impl<'d, T: Instance + 'd> embedded_io::asynch::BufRead for RxBufferedUart<'d, T> {
    type FillBufFuture<'a> = impl Future<Output = Result<&'a [u8], Self::Error>>
    where
        Self: 'a;

    fn fill_buf<'a>(&'a mut self) -> Self::FillBufFuture<'a> {
        poll_fn(move |cx| {
            self.inner.with(|state| {
                compiler_fence(Ordering::SeqCst);
                state.fill_buf(cx.waker())
            })
        })
    }

    fn consume(&mut self, amt: usize) {
        let signal = self.inner.with(|state| state.consume(amt));
        if signal {
            self.inner.pend();
        }
    }
}

impl<'d, T: Instance + 'd> embedded_io::asynch::Write for BufferedUart<'d, T> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        poll_fn(move |cx| {
            let (poll, empty) = self.inner.with(|state| state.tx.write(buf, cx.waker()));
            if empty {
                self.inner.pend();
            }
            poll
        })
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        poll_fn(move |cx| self.inner.with(|state| state.tx.flush(cx.waker())))
    }
}

impl<'d, T: Instance + 'd> embedded_io::asynch::Write for TxBufferedUart<'d, T> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        poll_fn(move |cx| {
            let (poll, empty) = self.inner.with(|state| state.write(buf, cx.waker()));
            if empty {
                self.inner.pend();
            }
            poll
        })
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        poll_fn(move |cx| self.inner.with(|state| state.flush(cx.waker())))
    }
}
