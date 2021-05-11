use crate::low_power_wait_until;
use crate::ring_buffer::RingBuffer;
use core::cmp::min;
use core::mem;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};
use embassy::interrupt::InterruptExt;
use embassy::io::{AsyncBufRead, AsyncWrite, Result};
use embassy::util::WakerRegistration;

use crate::fmt::*;

#[derive(Copy, Clone, Debug, PartialEq)]
enum RxState {
    Idle,
    Receiving,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum TxState {
    Idle,
    Transmitting(usize),
}

pub trait UartPeripheral {
    // RX methods
    fn start_rx(&self, buf: &mut [u8]);
    fn stop_rx(&self);
    fn clear_rx(&self) -> usize;
    fn rx_done(&self) -> bool;

    // TX methods
    fn start_tx(&self, buf: &[u8]);
    fn stop_tx(&self);
    fn clear_tx(&self);
    fn tx_done(&self) -> bool;
}

struct BufferedUart<'d, U: UartPeripheral> {
    uart: U,

    rx: RingBuffer<'d>,
    rx_state: RxState,
    rx_waker: WakerRegistration,

    tx: RingBuffer<'d>,
    tx_state: TxState,
    tx_waker: WakerRegistration,
}

impl<'d, U: UartPeripheral> BufferedUart<'d, U> {
    /// unsafe: may not leak self or futures
    pub unsafe fn new(uart: U, rx_buffer: &'d mut [u8], tx_buffer: &'d mut [u8]) -> Self {
        Self {
            uart,
            rx: RingBuffer::new(rx_buffer),
            rx_state: RxState::Idle,
            rx_waker: WakerRegistration::new(),

            tx: RingBuffer::new(tx_buffer),
            tx_state: TxState::Idle,
            tx_waker: WakerRegistration::new(),
        }
    }

    pub fn with_peripheral<F: FnOnce(&mut U)>(&mut self, f: F) {
        f(&mut self.uart);
    }

    /// Invoke when IRQ for the underlying peripheral is raised
    pub fn on_interrupt(&mut self) {
        trace!("irq: start");
        loop {
            match self.rx_state {
                RxState::Idle => {
                    trace!("  irq_rx: in state idle");

                    let buf = self.rx.push_buf();
                    if !buf.is_empty() {
                        trace!("  irq_rx: starting {:?}", buf.len());
                        self.rx_state = RxState::Receiving;
                        self.uart.start_rx(buf);
                    }
                    break;
                }
                RxState::Receiving => {
                    trace!("  irq_rx: in state receiving");
                    if self.uart.rx_done() {
                        let n = self.uart.clear_rx();
                        self.rx.push(n);
                        self.rx_waker.wake();
                        self.rx_state = RxState::Idle;
                    } else {
                        break;
                    }
                }
            }
        }

        loop {
            match self.tx_state {
                TxState::Idle => {
                    trace!("  irq_tx: in state Idle");
                    let buf = self.tx.pop_buf();
                    if !buf.is_empty() {
                        self.tx_state = TxState::Transmitting(buf.len());
                        self.uart.start_tx(buf);
                    }
                    break;
                }
                TxState::Transmitting(n) => {
                    trace!("  irq_tx: in state Transmitting");
                    if self.uart.tx_done() {
                        self.uart.clear_tx();
                        self.tx.pop(n);
                        self.tx_waker.wake();
                        self.tx_state = TxState::Idle;
                    } else {
                        break;
                    }
                }
            }
        }
        trace!("irq: end");
    }
}

impl<'d, U: UartPeripheral> AsyncBufRead for BufferedUart<'d, U> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        compiler_fence(Ordering::SeqCst);
        trace!("poll_read");

        // We have data ready in buffer? Return it.
        let buf = self.rx.pop_buf();
        if !buf.is_empty() {
            trace!("  got {:?} {:?}", buf.as_ptr() as u32, buf.len());
            let buf: &[u8] = buf;
            let buf: &[u8] = unsafe { mem::transmute(buf) };
            return Poll::Ready(Ok(buf));
        }

        trace!("  empty");
        self.rx_waker.register(cx.waker());
        Poll::<Result<&[u8]>>::Pending
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        trace!("consume {:?}", amt);
        self.rx.pop(amt);
        //    irq.pend();
    }
}

impl<'d, U: UartPeripheral> AsyncWrite for BufferedUart<'d, U> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        trace!("poll_write: {:?}", buf.len());

        let tx_buf = self.tx.push_buf();
        if tx_buf.is_empty() {
            trace!("poll_write: pending");
            self.tx_waker.register(cx.waker());
            return Poll::Pending;
        }

        let n = min(tx_buf.len(), buf.len());
        tx_buf[..n].copy_from_slice(&buf[..n]);
        self.tx.push(n);

        trace!("poll_write: queued {:?}", n);

        // Conservative compiler fence to prevent optimizations that do not
        // take in to account actions by DMA. The fence has been placed here,
        // before any DMA action has started
        //    compiler_fence(Ordering::SeqCst);

        // irq.pend();

        Poll::Ready(Ok(n))
    }
}

impl<'d, U: UartPeripheral> Drop for BufferedUart<'d, U> {
    fn drop(&mut self) {
        if let RxState::Receiving = self.rx_state {
            self.uart.stop_rx();
        }
        if let TxState::Transmitting(_) = self.tx_state {
            self.uart.stop_tx();
        }
        if let RxState::Receiving = self.rx_state {
            low_power_wait_until(|| self.uart.rx_done());
        }
        if let TxState::Transmitting(_) = self.tx_state {
            low_power_wait_until(|| self.uart.tx_done());
        }
    }
}

pub struct BufferedReader<'d, U: UartPeripheral> {
    inner: &'d mut BufferedUart<'d, U>,
}

impl<'d, U: UartPeripheral> BufferedReader<'d, U> {
    pub fn new(inner: &'d mut BufferedUart<'d, U>) -> Self {
        Self { inner }
    }

    fn inner(self: Pin<&mut Self>) -> Pin<&mut BufferedUart<'d, U>> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner) }
    }
}

impl<'d, U: UartPeripheral> AsyncBufRead for BufferedReader<'d, U> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        self.inner().poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.inner().consume(amt)
    }
}

pub struct BufferedWriter<'d, U: UartPeripheral> {
    inner: &'d mut BufferedUart<'d, U>,
}

impl<'d, U: UartPeripheral> BufferedWriter<'d, U> {
    pub fn new(inner: &'d mut BufferedUart<'d, U>) -> Self {
        Self { inner }
    }

    fn inner(self: Pin<&mut Self>) -> Pin<&mut BufferedUart<'d, U>> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner) }
    }
}

impl<'d, U: UartPeripheral> AsyncWrite for BufferedWriter<'d, U> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        self.inner().poll_write(cx, buf)
    }
}
