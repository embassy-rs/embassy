use atomic_polyfill::{compiler_fence, Ordering};
use core::future::Future;
use core::task::Poll;
use embassy::waitqueue::WakerRegistration;
use embassy_hal_common::peripheral::{PeripheralMutex, PeripheralState, StateStorage};
use embassy_hal_common::ring_buffer::RingBuffer;
use futures::future::poll_fn;

use super::*;

pub struct State<'d, T: Instance>(StateStorage<StateInner<'d, T>>);
impl<'d, T: Instance> State<'d, T> {
    pub fn new() -> Self {
        Self(StateStorage::new())
    }
}

struct StateInner<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,

    rx_waker: WakerRegistration,
    rx: RingBuffer<'d>,

    tx_waker: WakerRegistration,
    tx: RingBuffer<'d>,
}

unsafe impl<'d, T: Instance> Send for StateInner<'d, T> {}
unsafe impl<'d, T: Instance> Sync for StateInner<'d, T> {}

pub struct BufferedUart<'d, T: Instance> {
    inner: PeripheralMutex<'d, StateInner<'d, T>>,
}

impl<'d, T: Instance> Unpin for BufferedUart<'d, T> {}

impl<'d, T: Instance> BufferedUart<'d, T> {
    pub fn new(
        state: &'d mut State<'d, T>,
        _uart: Uart<'d, T, NoDma, NoDma>,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
    ) -> BufferedUart<'d, T> {
        unborrow!(irq);

        let r = T::regs();
        unsafe {
            r.cr1().modify(|w| {
                w.set_rxneie(true);
                w.set_idleie(true);
            });
        }

        Self {
            inner: PeripheralMutex::new(irq, &mut state.0, move || StateInner {
                phantom: PhantomData,
                tx: RingBuffer::new(tx_buffer),
                tx_waker: WakerRegistration::new(),

                rx: RingBuffer::new(rx_buffer),
                rx_waker: WakerRegistration::new(),
            }),
        }
    }
}

impl<'d, T: Instance> StateInner<'d, T>
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

impl<'d, T: Instance> PeripheralState for StateInner<'d, T>
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

impl<'d, T: Instance> embedded_io::Io for BufferedUart<'d, T> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::asynch::Read for BufferedUart<'d, T> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        poll_fn(move |cx| {
            let mut do_pend = false;
            let res = self.inner.with(|state| {
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
                self.inner.pend();
            }

            res
        })
    }
}

impl<'d, T: Instance> embedded_io::asynch::BufRead for BufferedUart<'d, T> {
    type FillBufFuture<'a> = impl Future<Output = Result<&'a [u8], Self::Error>>
    where
        Self: 'a;

    fn fill_buf<'a>(&'a mut self) -> Self::FillBufFuture<'a> {
        poll_fn(move |cx| {
            self.inner.with(|state| {
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
                Poll::<Result<&[u8], Self::Error>>::Pending
            })
        })
    }

    fn consume(&mut self, amt: usize) {
        let signal = self.inner.with(|state| {
            let full = state.rx.is_full();
            state.rx.pop(amt);
            full
        });
        if signal {
            self.inner.pend();
        }
    }
}

impl<'d, T: Instance> embedded_io::asynch::Write for BufferedUart<'d, T> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        poll_fn(move |cx| {
            let (poll, empty) = self.inner.with(|state| {
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
                self.inner.pend();
            }
            poll
        })
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        poll_fn(move |cx| {
            self.inner.with(|state| {
                if !state.tx.is_empty() {
                    state.tx_waker.register(cx.waker());
                    return Poll::Pending;
                }

                Poll::Ready(Ok(()))
            })
        })
    }
}
