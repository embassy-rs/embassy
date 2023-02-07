use core::cmp;
use core::task::{RawWaker, RawWakerVTable, Waker};

use atomic_polyfill::{AtomicUsize, Ordering};
use embassy_hal_common::atomic_ring_buffer::RingBuffer;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

use super::*;
use crate::dma;

pub struct BufferedDmaTx<'a, T: BasicInstance, Dma: TxDma<T>> {
    uart: UartTx<'a, T, Dma>,
    buf: RingBuffer,
    /* length of current DMA transaction */
    dma_len: AtomicUsize,
    signal: Signal<CriticalSectionRawMutex, ()>,
}

impl<'a, T: BasicInstance, Dma: TxDma<T>> UartTx<'a, T, Dma> {
    /**
     * Setup DMA transmit buffer.
     *
     * Writes are buffered in the transmit buffer. If there is not enough space
     * in the buffer the caller will await until space becomes available as
     * data drains from the buffer. With an appropriate buffer size this allows
     * the caller to return immediately unless the transmitter is saturated.
     *
     * Also implements core::fmt::Write so can be used for printing without an
     * extra layer of buffering.
     *
     * # Example
     * ```
     * let config = usart::Config::default();
     * let irq = interrupt::take!(USART6);
     * let (tx, _) = usart::Uart::new(p.USART6, p.PG9, p.PG14, irq, p.DMA2_CH6, p.DMA2_CH1, config).split();
     * static mut TX_BUF: [u8; 256] = [0; 256];
     * let tx = debug_tx.new_buffered_dma(unsafe { &mut TX_BUF });
     * writeln!(tx, "hello {}", "world");
     * ```
     */
    pub fn new_buffered_dma(self, buf: &'a mut [u8]) -> BufferedDmaTx<'a, T, Dma> {
        assert!(buf.len() > 0);

        let state = BufferedDmaTx {
            uart: self,
            buf: RingBuffer::new(),
            dma_len: AtomicUsize::new(0),
            signal: Signal::new(),
        };
        unsafe {
            state.buf.init(buf.as_mut_ptr(), buf.len());
        }
        state
    }
}

impl<'a, T: BasicInstance, Dma: TxDma<T>> BufferedDmaTx<'a, T, Dma> {
    pub async fn write(&mut self, mut d: &[u8]) {
        while !d.is_empty() {
            while self.buf.is_full() {
                self.signal.wait().await;
                continue;
            }
            let n = unsafe { self.buf.writer() }.push(|b| {
                let n = cmp::min(d.len(), b.len());
                b[..n].copy_from_slice(&d[..n]);
                /* TODO: cache flush b[..n] */
                n
            });
            d = &d[n..];
            self.start_tx();
        }
    }

    pub async fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> core::fmt::Result {
        if let Some(s) = fmt.as_str() {
            self.write(s.as_bytes()).await;
            return Ok(());
        }

        /* TODO: async formatting -
         * currently this will Err if the formatted string won't fit in the tx
         * buffer because core::fmt::write can't async wait for more space */
        let mut bw = unsafe { self.buf.writer() };
        loop {
            let was_empty = self.buf.is_empty();
            let ps = bw.push_slices();
            let mut w = BufWriter::new(ps);
            if core::fmt::write(&mut w, fmt).is_ok() {
                let l = w.off();
                /* TODO: cache flush ps[..l] */
                bw.push_done(l);
                break;
            }
            if was_empty {
                /* output too big for buffer */
                return Err(core::fmt::Error);
            }

            /* wait for more space in buffer */
            self.signal.wait().await;
        }

        self.start_tx();
        Ok(())
    }

    fn tx_dma_isr(&mut self) {
        /* disable dma request from peripheral while reconfiguring */
        unsafe {
            T::regs().cr3().modify(|v| {
                v.set_dmat(false);
            });
        }
        unsafe { self.buf.reader() }.pop_done(self.dma_len.swap(0, Ordering::Relaxed));
        self.signal.signal(());
        /* we may have more data to transmit */
        self.start_tx();
    }

    fn start_tx(&mut self) {
        if self.uart.tx_dma.is_running() || self.buf.is_empty() {
            return;
        }
        let data = self as *const Self as *const ();
        self.uart
            .tx_dma
            .set_waker(unsafe { &Waker::from_raw(RawWaker::new(data, &Self::TX_DMA_VTABLE)) });
        let request = self.uart.tx_dma.request();
        let mut tr = unsafe { self.buf.reader() };
        let tb = tr.pop_slice();
        /* limit the transfer size to half the buffer size to allow the
         * buffer to be filled by a writer while the it's DMA'ing out */
        let tb = &tb[..cmp::min(tb.len(), self.buf.len() / 2)];
        self.dma_len.fetch_add(tb.len(), Ordering::Relaxed);
        unsafe {
            /* FIXME: hard coded to highest priority! */
            self.uart
                .tx_dma
                .start_write(request, tb, tdr(T::regs()), dma::TransferOptions::default());
            T::regs().cr3().modify(|v| {
                v.set_dmat(true);
            });
        }
    }

    const TX_DMA_VTABLE: RawWakerVTable =
        RawWakerVTable::new(Self::waker_clone, Self::waker_wake, Self::waker_wake, Self::waker_drop);

    unsafe fn waker_clone(ptr: *const ()) -> RawWaker {
        RawWaker::new(ptr, &Self::TX_DMA_VTABLE)
    }

    unsafe fn waker_drop(_: *const ()) {}

    unsafe fn waker_wake(ptr: *const ()) {
        (*(ptr as *mut Self)).tx_dma_isr();
    }
}

/* BufWriter implements core::fmt::Write and writes to an array of two byte slices */
struct BufWriter<'a> {
    bufs: [&'a mut [u8]; 2],
    off: usize,
}

impl<'a> BufWriter<'a> {
    fn new(bufs: [&'a mut [u8]; 2]) -> Self {
        Self { bufs, off: 0 }
    }

    fn off(&self) -> usize {
        self.off
    }
}

impl<'a> core::fmt::Write for BufWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut s = s.as_bytes();
        let mut o = self.off;
        for b in self.bufs.iter_mut() {
            if o >= b.len() {
                o -= b.len();
                continue;
            }
            let b = &mut b[o..];
            let l = cmp::min(b.len(), s.len());
            b[..l].copy_from_slice(&s[..l]);
            s = &s[l..];
            self.off += l;
            if s.len() == 0 {
                return Ok(());
            }
            o = 0;
        }
        Err(core::fmt::Error)
    }
}
