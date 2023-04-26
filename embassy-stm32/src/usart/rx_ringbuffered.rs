use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::PeripheralRef;

use super::{rdr, sr, BasicInstance, Error, UartRx};
use crate::dma::ringbuffer::OverrunError;
use crate::dma::RingBuffer;

pub struct RingBufferedUartRx<'d, T: BasicInstance, RxDma: super::RxDma<T>> {
    _peri: PeripheralRef<'d, T>,
    ring_buf: RingBuffer<'d, RxDma, u8>,
}

impl<'d, T: BasicInstance, RxDma: super::RxDma<T>> UartRx<'d, T, RxDma> {
    /// Turn the `UartRx` into a buffered uart which can continously receive in the background
    /// without the possibility of loosing bytes. The `dma_buf` is a buffer registered to the
    /// DMA controller, and must be sufficiently large, such that it will not overflow.
    pub fn into_ring_buffered(self, dma_buf: &'d mut [u8]) -> RingBufferedUartRx<'d, T, RxDma> {
        assert!(dma_buf.len() > 0 && dma_buf.len() <= 0xFFFF);

        let request = self.rx_dma.request();
        let opts = Default::default();
        let ring_buf = unsafe { RingBuffer::new_read(self.rx_dma, request, rdr(T::regs()), dma_buf, opts) };
        RingBufferedUartRx {
            _peri: self._peri,
            ring_buf,
        }
    }
}

impl<'d, T: BasicInstance, RxDma: super::RxDma<T>> RingBufferedUartRx<'d, T, RxDma> {
    pub fn start(&mut self) -> Result<(), Error> {
        // Clear the ring buffer so that it is ready to receive data
        self.ring_buf.clear();

        self.setup_uart();

        Ok(())
    }

    /// Start uart background receive
    fn setup_uart(&mut self) {
        // fence before starting DMA.
        compiler_fence(Ordering::SeqCst);

        self.ring_buf.start();

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
    fn teardown_uart(&mut self) {
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

        compiler_fence(Ordering::SeqCst);

        self.ring_buf.request_stop();
        while self.ring_buf.is_running() {}
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
        let s = unsafe { sr(r).read() };
        let has_errors = s.pe() || s.fe() || s.ne() || s.ore();
        if has_errors {
            self.teardown_uart();

            if s.pe() {
                return Err(Error::Parity);
            } else if s.fe() {
                return Err(Error::Framing);
            } else if s.ne() {
                return Err(Error::Noise);
            } else {
                return Err(Error::Overrun);
            }
        }

        let ndtr = self.ring_buf.get_remaining_transfers();
        self.ring_buf.set_ndtr(ndtr);
        match self.ring_buf.read(buf) {
            Ok(len) if len == 0 => {}
            Ok(len) => {
                assert!(len > 0);
                return Ok(len);
            }
            Err(OverrunError) => {
                // Stop any transfer from now on
                // The user must re-start to receive any more data
                self.teardown_uart();
                return Err(Error::Overrun);
            }
        }

        // Wait for any data since `ndtr`
        self.wait_for_data(ndtr).await?;

        // ndtr is now different than the value provided to `wait_for_data()`
        // Re-sample ndtr now when it has changed.
        self.ring_buf.set_ndtr(self.ring_buf.get_remaining_transfers());
        let len = self.ring_buf.read(buf).map_err(|_err| Error::Overrun)?;
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

        // future which completes when RX "not empty" is detected,
        // i.e. when there is data in uart rx register
        let rxne = poll_fn(|cx| {
            let s = T::state();

            // Register waker to be awaken when RXNE interrupt is received
            s.rx_waker.register(cx.waker());

            compiler_fence(Ordering::SeqCst);

            // SAFETY: read only and we only use Rx related flags
            let s = unsafe { sr(r).read() };
            let has_errors = s.pe() || s.fe() || s.ne() || s.ore();
            if has_errors {
                if s.pe() {
                    return Poll::Ready(Err(Error::Parity));
                } else if s.fe() {
                    return Poll::Ready(Err(Error::Framing));
                } else if s.ne() {
                    return Poll::Ready(Err(Error::Noise));
                } else {
                    return Poll::Ready(Err(Error::Overrun));
                }
            }

            // Re-sample ndtr and determine if it has changed since we started
            // waiting for data.
            let new_ndtr = self.ring_buf.get_remaining_transfers();
            if new_ndtr != old_ndtr {
                // Some data was received as NDTR has changed
                Poll::Ready(Ok(()))
            } else {
                // It may be that the DMA controller is currently busy consuming the
                // RX data register. We therefore wait register to become empty.
                while unsafe { sr(r).read().rxne() } {}

                compiler_fence(Ordering::SeqCst);

                // Re-get again: This time we know that the DMA controller has consumed
                // the current read register if it was busy doing so
                let new_ndtr = self.ring_buf.get_remaining_transfers();
                if new_ndtr != old_ndtr {
                    // Some data was received as NDTR has changed
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            }
        });

        compiler_fence(Ordering::SeqCst);

        let new_ndtr = self.ring_buf.get_remaining_transfers();
        if new_ndtr != old_ndtr {
            // Fast path - NDTR has already changed, no reason to poll
            Ok(())
        } else {
            // NDTR has not changed since we first read from the ring buffer
            // Wait for RXNE interrupt...
            match rxne.await {
                Ok(()) => Ok(()),
                Err(e) => {
                    self.teardown_uart();
                    Err(e)
                }
            }
        }
    }
}

impl<T: BasicInstance, RxDma: super::RxDma<T>> Drop for RingBufferedUartRx<'_, T, RxDma> {
    fn drop(&mut self) {
        self.teardown_uart();
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
