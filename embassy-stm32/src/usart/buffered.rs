use core::cell::RefCell;
use core::future::{poll_fn, Future};
use core::task::Poll;

use atomic_polyfill::{compiler_fence, Ordering};
use embassy_cortex_m::peripheral::{PeripheralMutex, PeripheralState, StateStorage};
use embassy_hal_common::ring_buffer::RingBuffer;
use embassy_sync::waitqueue::WakerRegistration;

use super::*;

pub struct State<'d, T: BasicInstance>(StateStorage<StateInner<'d, T>>);
impl<'d, T: BasicInstance> State<'d, T> {
    pub const fn new() -> Self {
        Self(StateStorage::new())
    }
}

struct StateInner<'d, T: BasicInstance> {
    phantom: PhantomData<&'d mut T>,

    rx_waker: WakerRegistration,
    rx: RingBuffer<'d>,

    tx_waker: WakerRegistration,
    tx: RingBuffer<'d>,
}

unsafe impl<'d, T: BasicInstance> Send for StateInner<'d, T> {}
unsafe impl<'d, T: BasicInstance> Sync for StateInner<'d, T> {}

pub struct BufferedUart<'d, T: BasicInstance> {
    inner: RefCell<PeripheralMutex<'d, StateInner<'d, T>>>,
}

pub struct BufferedUartTx<'u, 'd, T: BasicInstance> {
    inner: &'u BufferedUart<'d, T>,
}

pub struct BufferedUartRx<'u, 'd, T: BasicInstance> {
    inner: &'u BufferedUart<'d, T>,
}

impl<'d, T: BasicInstance> Unpin for BufferedUart<'d, T> {}

impl<'d, T: BasicInstance> BufferedUart<'d, T> {
    pub fn new(
        state: &'d mut State<'d, T>,
        _uart: Uart<'d, T, NoDma, NoDma>,
        irq: impl Peripheral<P = T::Interrupt> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
    ) -> BufferedUart<'d, T> {
        into_ref!(irq);

        let r = T::regs();
        unsafe {
            r.cr1().modify(|w| {
                w.set_rxneie(true);
                w.set_idleie(true);
            });
        }

        Self {
            inner: RefCell::new(PeripheralMutex::new(irq, &mut state.0, move || StateInner {
                phantom: PhantomData,
                tx: RingBuffer::new(tx_buffer),
                tx_waker: WakerRegistration::new(),

                rx: RingBuffer::new(rx_buffer),
                rx_waker: WakerRegistration::new(),
            })),
        }
    }

    pub fn split<'u>(&'u mut self) -> (BufferedUartRx<'u, 'd, T>, BufferedUartTx<'u, 'd, T>) {
        (BufferedUartRx { inner: self }, BufferedUartTx { inner: self })
    }

    async fn inner_read<'a>(&'a self, buf: &'a mut [u8]) -> Result<usize, Error> {
        poll_fn(move |cx| {
            let mut do_pend = false;
            let mut inner = self.inner.borrow_mut();
            let res = inner.with(|state| {
                compiler_fence(Ordering::SeqCst);

                // We have data ready in buffer? Return it.
                let data = state.rx.pop_buf();
                if !data.is_empty() {
                    let len = data.len().min(buf.len());
                    buf[..len].copy_from_slice(&data[..len]);

                    if state.rx.is_full() {
                        do_pend = true;
                    }
                    state.rx.pop(len);

                    return Poll::Ready(Ok(len));
                }

                state.rx_waker.register(cx.waker());
                Poll::Pending
            });

            if do_pend {
                inner.pend();
            }

            res
        })
        .await
    }

    async fn inner_write<'a>(&'a self, buf: &'a [u8]) -> Result<usize, Error> {
        poll_fn(move |cx| {
            let mut inner = self.inner.borrow_mut();
            let (poll, empty) = inner.with(|state| {
                let empty = state.tx.is_empty();
                let tx_buf = state.tx.push_buf();
                if tx_buf.is_empty() {
                    state.tx_waker.register(cx.waker());
                    return (Poll::Pending, empty);
                }

                
                let n = core::cmp::min(tx_buf.len(), buf.len());
                tx_buf[..n].copy_from_slice(&buf[..n]);
                state.tx.push(n);

                (Poll::Ready(Ok(n)), empty)
            });
            if empty {
                inner.pend();
            }
            poll
        })
        .await
    }

    async fn inner_flush<'a>(&'a self) -> Result<(), Error> {
        poll_fn(move |cx| {
            self.inner.borrow_mut().with(|state| {
                if !state.tx.is_empty() {
                    state.tx_waker.register(cx.waker());
                    return Poll::Pending;
                }

                Poll::Ready(Ok(()))
            })
        })
        .await
    }

    async fn inner_fill_buf<'a>(&'a self) -> Result<&'a [u8], Error> {
        poll_fn(move |cx| {
            self.inner.borrow_mut().with(|state| {
                compiler_fence(Ordering::SeqCst);

                // We have data ready in buffer? Return it.
                let buf = state.rx.pop_buf();
                if !buf.is_empty() {
                    let buf: &[u8] = buf;
                    // Safety: buffer lives as long as uart
                    let buf: &[u8] = unsafe { core::mem::transmute(buf) };
                    return Poll::Ready(Ok(buf));
                }

                state.rx_waker.register(cx.waker());
                Poll::<Result<&[u8], Error>>::Pending
            })
        })
        .await
    }

    fn inner_consume(&self, amt: usize) {
        let mut inner = self.inner.borrow_mut();
        let signal = inner.with(|state| {
            let full = state.rx.is_full();
            state.rx.pop(amt);
            full
        });
        if signal {
            inner.pend();
        }
    }
}

impl<'d, T: BasicInstance> StateInner<'d, T>
where
    Self: 'd,
{
    fn on_rx(&mut self) {
        let r = T::regs();
        unsafe {
            let sr = sr(r).read();
            clear_interrupt_flags(r, sr);

            // This read also clears the error and idle interrupt flags on v1.
            let b = rdr(r).read_volatile();

            if sr.rxne() {
                if sr.pe() {
                    warn!("Parity error");
                }
                if sr.fe() {
                    warn!("Framing error");
                }
                if sr.ne() {
                    warn!("Noise error");
                }
                if sr.ore() {
                    warn!("Overrun error");
                }

                let buf = self.rx.push_buf();
                if !buf.is_empty() {
                    buf[0] = b;
                    self.rx.push(1);
                } else {
                    warn!("RX buffer full, discard received byte");
                }

                if self.rx.is_full() {
                    self.rx_waker.wake();
                }
            }

            if sr.idle() {
                self.rx_waker.wake();
            };
        }
    }

    fn on_tx(&mut self) {
        let r = T::regs();
        unsafe {
            if sr(r).read().txe() {
                let buf = self.tx.pop_buf();
                if !buf.is_empty() {
                    r.cr1().modify(|w| {
                        w.set_txeie(true);
                    });
                    tdr(r).write_volatile(buf[0].into());
                    self.tx.pop(1);
                    self.tx_waker.wake();
                } else {
                    // Disable interrupt until we have something to transmit again
                    r.cr1().modify(|w| {
                        w.set_txeie(false);
                    });
                }
            }
        }
    }
}

impl<'d, T: BasicInstance> PeripheralState for StateInner<'d, T>
where
    Self: 'd,
{
    type Interrupt = T::Interrupt;
    fn on_interrupt(&mut self) {
        self.on_rx();
        self.on_tx();
    }
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl<'d, T: BasicInstance> embedded_io::Io for BufferedUart<'d, T> {
    type Error = Error;
}

impl<'u, 'd, T: BasicInstance> embedded_io::Io for BufferedUartRx<'u, 'd, T> {
    type Error = Error;
}

impl<'u, 'd, T: BasicInstance> embedded_io::Io for BufferedUartTx<'u, 'd, T> {
    type Error = Error;
}

impl<'d, T: BasicInstance> embedded_io::asynch::Read for BufferedUart<'d, T> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        self.inner_read(buf)
    }
}

impl<'u, 'd, T: BasicInstance> embedded_io::asynch::Read for BufferedUartRx<'u, 'd, T> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        self.inner.inner_read(buf)
    }
}

impl<'d, T: BasicInstance> embedded_io::asynch::BufRead for BufferedUart<'d, T> {
    type FillBufFuture<'a> = impl Future<Output = Result<&'a [u8], Self::Error>>
    where
        Self: 'a;

    fn fill_buf<'a>(&'a mut self) -> Self::FillBufFuture<'a> {
        self.inner_fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner_consume(amt)
    }
}

impl<'u, 'd, T: BasicInstance> embedded_io::asynch::BufRead for BufferedUartRx<'u, 'd, T> {
    type FillBufFuture<'a> = impl Future<Output = Result<&'a [u8], Self::Error>>
    where
        Self: 'a;

    fn fill_buf<'a>(&'a mut self) -> Self::FillBufFuture<'a> {
        self.inner.inner_fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.inner_consume(amt)
    }
}

impl<'d, T: BasicInstance> embedded_io::asynch::Write for BufferedUart<'d, T> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        self.inner_write(buf)
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        self.inner_flush()
    }
}

impl<'u, 'd, T: BasicInstance> embedded_io::asynch::Write for BufferedUartTx<'u, 'd, T> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        self.inner.inner_write(buf)
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        self.inner.inner_flush()
    }
}
