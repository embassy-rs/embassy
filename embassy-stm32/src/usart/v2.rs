use atomic_polyfill::{compiler_fence, Ordering};
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use embassy::util::Unborrow;
use embassy::waitqueue::WakerRegistration;
use embassy_hal_common::peripheral::{PeripheralMutex, PeripheralState, StateStorage};
use embassy_hal_common::ring_buffer::RingBuffer;
use embassy_hal_common::unborrow;
use futures::TryFutureExt;

use super::*;
use crate::dma::NoDma;
use crate::gpio::sealed::OutputType::{OpenDrain, PushPull};
use crate::pac::usart::{regs, vals};

pub struct Uart<'d, T: Instance, TxDma = NoDma, RxDma = NoDma> {
    inner: T,
    phantom: PhantomData<&'d mut T>,
    tx_dma: TxDma,
    rx_dma: RxDma,
}

impl<'d, T: Instance, TxDma, RxDma> Uart<'d, T, TxDma, RxDma> {
    pub fn new(
        inner: impl Unborrow<Target = T>,
        rx: impl Unborrow<Target = impl RxPin<T>>,
        tx: impl Unborrow<Target = impl TxPin<T>>,
        tx_dma: impl Unborrow<Target = TxDma>,
        rx_dma: impl Unborrow<Target = RxDma>,
        config: Config,
    ) -> Self {
        unborrow!(inner, rx, tx, tx_dma, rx_dma);

        T::enable();
        let pclk_freq = T::frequency();

        // TODO: better calculation, including error checking and OVER8 if possible.
        let div = (pclk_freq.0 + (config.baudrate / 2)) / config.baudrate;

        let r = inner.regs();

        unsafe {
            rx.set_as_af(rx.af_num(), OpenDrain);
            tx.set_as_af(tx.af_num(), PushPull);

            r.cr2().write(|_w| {});
            r.cr3().write(|_w| {});

            r.brr().write(|w| w.set_brr(div as u16));
            r.cr1().write(|w| {
                w.set_ue(true);
                w.set_te(true);
                w.set_re(true);
                w.set_m0(vals::M0::BIT8);
                w.set_m1(vals::M1::M0);
                w.set_pce(config.parity != Parity::ParityNone);
                w.set_ps(match config.parity {
                    Parity::ParityOdd => vals::Ps::ODD,
                    Parity::ParityEven => vals::Ps::EVEN,
                    _ => vals::Ps::EVEN,
                });
            });
            r.cr2().write(|_w| {});
            r.cr3().write(|_w| {});
        }

        Self {
            inner,
            phantom: PhantomData,
            tx_dma,
            rx_dma,
        }
    }

    async fn write_dma(&mut self, buffer: &[u8]) -> Result<(), Error>
    where
        TxDma: crate::usart::TxDma<T>,
    {
        let ch = &mut self.tx_dma;
        unsafe {
            self.inner.regs().cr3().modify(|reg| {
                reg.set_dmat(true);
            });
        }
        let r = self.inner.regs();
        let dst = r.tdr().ptr() as *mut u8;
        ch.write(ch.request(), buffer, dst).await;
        Ok(())
    }

    async fn read_dma(&mut self, buffer: &mut [u8]) -> Result<(), Error>
    where
        RxDma: crate::usart::RxDma<T>,
    {
        let ch = &mut self.rx_dma;
        unsafe {
            self.inner.regs().cr3().modify(|reg| {
                reg.set_dmar(true);
            });
        }
        let r = self.inner.regs();
        let src = r.rdr().ptr() as *mut u8;
        ch.read(ch.request(), src, buffer).await;
        Ok(())
    }

    pub fn read_blocking(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        unsafe {
            let r = self.inner.regs();
            for b in buffer {
                loop {
                    let sr = r.isr().read();
                    if sr.pe() {
                        r.rdr().read();
                        return Err(Error::Parity);
                    } else if sr.fe() {
                        r.rdr().read();
                        return Err(Error::Framing);
                    } else if sr.nf() {
                        r.rdr().read();
                        return Err(Error::Noise);
                    } else if sr.ore() {
                        r.rdr().read();
                        return Err(Error::Overrun);
                    } else if sr.rxne() {
                        break;
                    }
                }
                *b = r.rdr().read().0 as u8;
            }
        }
        Ok(())
    }
}

impl<'d, T: Instance, RxDma> embedded_hal::blocking::serial::Write<u8>
    for Uart<'d, T, NoDma, RxDma>
{
    type Error = Error;
    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        unsafe {
            let r = self.inner.regs();
            for &b in buffer {
                while !r.isr().read().txe() {}
                r.tdr().write_value(regs::Dr(b as u32))
            }
        }
        Ok(())
    }
    fn bflush(&mut self) -> Result<(), Self::Error> {
        unsafe {
            let r = self.inner.regs();
            while !r.isr().read().tc() {}
        }
        Ok(())
    }
}

// rustfmt::skip because intellij removes the 'where' claus on the associated type.
impl<'d, T: Instance, TxDma, RxDma> embassy_traits::uart::Write for Uart<'d, T, TxDma, RxDma>
where
    TxDma: crate::usart::TxDma<T>,
{
    // rustfmt::skip because rustfmt removes the 'where' claus on the associated type.
    #[rustfmt::skip]
    type WriteFuture<'a> where Self: 'a = impl Future<Output = Result<(), embassy_traits::uart::Error>> +'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        self.write_dma(buf)
            .map_err(|_| embassy_traits::uart::Error::Other)
    }
}

impl<'d, T: Instance, TxDma, RxDma> embassy_traits::uart::Read for Uart<'d, T, TxDma, RxDma>
where
    RxDma: crate::usart::RxDma<T>,
{
    // rustfmt::skip because rustfmt removes the 'where' claus on the associated type.
    #[rustfmt::skip]
    type ReadFuture<'a> where Self: 'a = impl Future<Output = Result<(), embassy_traits::uart::Error>> + 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        self.read_dma(buf)
            .map_err(|_| embassy_traits::uart::Error::Other)
    }
}

pub struct State<'d, T: Instance>(StateStorage<StateInner<'d, T>>);
impl<'d, T: Instance> State<'d, T> {
    pub fn new() -> Self {
        Self(StateStorage::new())
    }
}

pub struct StateInner<'d, T: Instance> {
    uart: Uart<'d, T, NoDma, NoDma>,
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
    pub unsafe fn new(
        state: &'d mut State<'d, T>,
        uart: Uart<'d, T, NoDma, NoDma>,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        tx_buffer: &'d mut [u8],
        rx_buffer: &'d mut [u8],
    ) -> BufferedUart<'d, T> {
        unborrow!(irq);

        let r = uart.inner.regs();
        r.cr1().modify(|w| {
            w.set_rxneie(true);
            w.set_idleie(true);
        });

        Self {
            inner: PeripheralMutex::new_unchecked(irq, &mut state.0, move || StateInner {
                uart,
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
        let r = self.uart.inner.regs();
        unsafe {
            let sr = r.isr().read();
            if sr.pe() {
                r.icr().write(|w| {
                    w.set_pe(true);
                });
                trace!("Parity error");
            } else if sr.fe() {
                r.icr().write(|w| {
                    w.set_fe(true);
                });
                trace!("Framing error");
            } else if sr.nf() {
                r.icr().write(|w| {
                    w.set_nf(true);
                });
                trace!("Noise error");
            } else if sr.ore() {
                r.icr().write(|w| {
                    w.set_ore(true);
                });
                trace!("Overrun error");
            } else if sr.rxne() {
                let buf = self.rx.push_buf();
                if buf.is_empty() {
                    self.rx_waker.wake();
                } else {
                    buf[0] = r.rdr().read().0 as u8;
                    self.rx.push(1);
                }
            } else if sr.idle() {
                r.icr().write(|w| {
                    w.set_idle(true);
                });
                self.rx_waker.wake();
            };
        }
    }

    fn on_tx(&mut self) {
        let r = self.uart.inner.regs();
        unsafe {
            if r.isr().read().txe() {
                let buf = self.tx.pop_buf();
                if !buf.is_empty() {
                    r.cr1().modify(|w| {
                        w.set_txeie(true);
                    });
                    r.tdr().write_value(regs::Dr(buf[0].into()));
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

impl<'d, T: Instance> embassy::io::AsyncBufRead for BufferedUart<'d, T> {
    fn poll_fill_buf(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<&[u8], embassy::io::Error>> {
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
            Poll::<Result<&[u8], embassy::io::Error>>::Pending
        })
    }
    fn consume(mut self: Pin<&mut Self>, amt: usize) {
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

impl<'d, T: Instance> embassy::io::AsyncWrite for BufferedUart<'d, T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, embassy::io::Error>> {
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
    }
}
