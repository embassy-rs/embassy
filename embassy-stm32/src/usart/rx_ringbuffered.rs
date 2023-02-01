use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_common::drop::OnDrop;

use super::dma_ringbuffer::{DmaCtrl, DmaRingBuffer};
use super::{rdr, sr, BasicInstance, Error, UartRx};
use crate::dma::TransferOptions;
use crate::usart::dma_ringbuffer::OverrunError;

pub struct RingBufferedUartRx<'d, T: BasicInstance, RxDma: super::RxDma<T>> {
    uart: UartRx<'d, T, RxDma>,
    ring_buf: DmaRingBuffer<'d>,
}

impl<'d, T: BasicInstance, RxDma: super::RxDma<T>> DmaCtrl for UartRx<'d, T, RxDma> {
    fn ndtr(&self) -> usize {
        self.rx_dma.remaining_transfers() as usize
    }

    fn tcif(&self) -> bool {
        self.rx_dma.get_tcif()
    }

    fn clear_tcif(&mut self) {
        self.rx_dma.clear_tcif()
    }
}

impl<'d, T: BasicInstance, RxDma: super::RxDma<T>> UartRx<'d, T, RxDma> {
    /// Turn the `UartRx` into a buffered uart which can continously receive in the background
    /// without the possibility of loosing bytes. The `dma_buf` is a buffer registered to the
    /// DMA controller, and must be sufficiently large, such that it will not overflow.
    pub fn into_ring_buffered(self, dma_buf: &'d mut [u8]) -> RingBufferedUartRx<'d, T, RxDma> {
        assert!(dma_buf.len() > 0 && dma_buf.len() <= 0xFFFF);

        RingBufferedUartRx {
            uart: self,
            ring_buf: DmaRingBuffer::new(dma_buf),
        }
    }
}

impl<'d, T: BasicInstance, RxDma: super::RxDma<T>> RingBufferedUartRx<'d, T, RxDma> {
    /// Start receiving in the background into the previously provided `dma_buf` buffer.
    pub fn start(&mut self) -> Result<(), Error> {
        let ch = &mut self.uart.rx_dma;
        let request = ch.request();

        // Clear the ring buffer so that it is ready to receive data
        self.ring_buf.clear();

        unsafe {
            // Start dma read
            // The memory address cannot be changed once the transfer is started
            let mut options = TransferOptions::default();
            options.circ = true; // Enable circular buffer mode
            options.tcie = false; // Do not enable transfer completed interrupt
            ch.start_read(request, rdr(T::regs()), self.ring_buf.dma_buf, options);
        }

        compiler_fence(Ordering::SeqCst);

        Self::setup_uart();

        Ok(())
    }

    /// Start uart background receive
    fn setup_uart() {
        let r = T::regs();
        // clear all interrupts and DMA Rx Request
        // SAFETY: only clears Rx related flags
        unsafe {
            r.cr1().modify(|w| {
                // disable RXNE interrupt
                w.set_rxneie(false);
                // enable parity interrupt if not ParityNone
                w.set_peie(w.pce());
                // disable idle line interrupt
                w.set_idleie(false);
            });
            r.cr3().modify(|w| {
                // enable Error Interrupt: (Frame error, Noise error, Overrun error)
                w.set_eie(true);
                // enable DMA Rx Request
                w.set_dmar(true);
            });
        }
    }

    /// Stop uart background receive
    fn teardown_uart() {
        let r = T::regs();
        // clear all interrupts and DMA Rx Request
        // SAFETY: only clears Rx related flags
        unsafe {
            r.cr1().modify(|w| {
                // disable RXNE interrupt
                w.set_rxneie(false);
                // disable parity interrupt
                w.set_peie(false);
                // disable idle line interrupt
                w.set_idleie(false);
            });
            r.cr3().modify(|w| {
                // disable Error Interrupt: (Frame error, Noise error, Overrun error)
                w.set_eie(false);
                // disable DMA Rx Request
                w.set_dmar(false);
            });
        }
    }

    /// Read bytes that are readily available in the ring buffer.
    /// If no bytes are currently available in the buffer the call waits until data are received.
    ///
    /// Background receive is started if `start()` has not been previously called.
    ///
    /// Receive in the background is terminated if an error is returned.
    /// It must then manually be started again by calling `start()` or by re-calling `read()`.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let r = T::regs();

        // SAFETY: read only
        let is_started = unsafe { r.cr3().read().dmar() };

        // Start background receive if it was not already started
        if !is_started {
            self.start()?;
        }

        // SAFETY: read only and we only use Rx related flags
        let sr = unsafe { sr(r).read() };

        let has_errors = sr.pe() || sr.fe() || sr.ne() || sr.ore();
        if has_errors {
            Self::teardown_uart();

            if sr.pe() {
                return Err(Error::Parity);
            }
            if sr.fe() {
                return Err(Error::Framing);
            }
            if sr.ne() {
                return Err(Error::Noise);
            }
            if sr.ore() {
                return Err(Error::Overrun);
            }
        }

        let ndtr = self.uart.ndtr();
        self.ring_buf.ndtr = ndtr;
        match self.ring_buf.read(&mut self.uart, buf) {
            Ok(len) if len == 0 => {}
            Ok(len) => {
                assert!(len > 0);
                return Ok(len);
            }
            Err(OverrunError) => {
                // Stop any transfer from now on
                // The user must re-start to receive any more data
                Self::teardown_uart();
                return Err(Error::Overrun);
            }
        }

        // Wait for any data since `ndtr`
        self.wait_for_data(ndtr).await?;

        // ndtr is now different than the value provided to `wait_for_data()`
        // Re-sample ndtr now when it has changed.
        self.ring_buf.ndtr = self.uart.ndtr();
        let len = self.ring_buf.read(&mut self.uart, buf).map_err(|_err| Error::Overrun)?;
        assert!(len > 0);
        Ok(len)
    }

    /// Wait for uart data
    async fn wait_for_data(&mut self, old_ndtr: usize) -> Result<(), Error> {
        let r = T::regs();

        // make sure USART state is restored to neutral state when this future is dropped
        let _drop = OnDrop::new(move || {
            // SAFETY: only clears Rx related flags
            unsafe {
                r.cr1().modify(|w| {
                    // disable RXNE interrupt
                    w.set_rxneie(false);
                });
            }
        });

        // SAFETY: only sets Rx related flags
        unsafe {
            r.cr1().modify(|w| {
                // enable RXNE interrupt
                w.set_rxneie(true);
            });
        }

        // future which completes when RX "not empty" is detected
        let rxne = poll_fn(|cx| {
            let s = T::state();

            // Register waker to be awaken when RXNE interrupt is received
            s.rx_waker.register(cx.waker());

            compiler_fence(Ordering::SeqCst);

            // SAFETY: read only and we only use Rx related flags
            let sr = unsafe { sr(r).read() };

            let has_errors = sr.pe() || sr.fe() || sr.ne() || sr.ore();
            if has_errors {
                Self::teardown_uart();

                if sr.pe() {
                    return Poll::Ready(Err(Error::Parity));
                }
                if sr.fe() {
                    return Poll::Ready(Err(Error::Framing));
                }
                if sr.ne() {
                    return Poll::Ready(Err(Error::Noise));
                }
                if sr.ore() {
                    return Poll::Ready(Err(Error::Overrun));
                }
            }

            let new_ndtr = self.uart.ndtr();
            if new_ndtr != old_ndtr {
                // Some data was received as NDTR has changed: disable RXNEIE
                // SAFETY: only clears Rx related flags
                unsafe {
                    r.cr1().modify(|w| {
                        // disable RXNE detection
                        w.set_rxneie(false);
                    });
                }

                return Poll::Ready(Ok(()));
            }

            Poll::Pending
        });

        compiler_fence(Ordering::SeqCst);

        let new_ndtr = self.uart.ndtr();
        if new_ndtr == old_ndtr {
            // NDTR has not changed since we first read from the ring buffer
            // Wait for RXNE interrupt...
            rxne.await?;
        }

        Ok(())
    }
}

impl<T: BasicInstance, RxDma: super::RxDma<T>> Drop for RingBufferedUartRx<'_, T, RxDma> {
    fn drop(&mut self) {
        Self::teardown_uart();
    }
}

#[cfg(all(feature = "unstable-traits", feature = "nightly"))]
mod eio {
    use embedded_io::asynch::Read;
    use embedded_io::Io;

    use super::RingBufferedUartRx;
    use crate::usart::{BasicInstance, Error, RxDma};

    impl<T, Rx> Io for RingBufferedUartRx<'_, T, Rx>
    where
        T: BasicInstance,
        Rx: RxDma<T>,
    {
        type Error = Error;
    }

    impl<T, Rx> Read for RingBufferedUartRx<'_, T, Rx>
    where
        T: BasicInstance,
        Rx: RxDma<T>,
    {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.read(buf).await
        }
    }
}
