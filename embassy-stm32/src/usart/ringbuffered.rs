use core::future::poll_fn;
use core::mem;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use futures_util::future::{select, Either};

use super::{clear_interrupt_flags, rdr, reconfigure, sr, Config, ConfigError, Error, Info, State, UartRx};
use crate::dma::ReadableRingBuffer;
use crate::mode::Async;
use crate::time::Hertz;
use crate::usart::{Regs, Sr};

/// Rx-only Ring-buffered UART Driver
///
/// Created with [UartRx::into_ring_buffered]
pub struct RingBufferedUartRx<'d> {
    info: &'static Info,
    state: &'static State,
    kernel_clock: Hertz,
    ring_buf: ReadableRingBuffer<'d, u8>,
}

impl<'d> SetConfig for RingBufferedUartRx<'d> {
    type Config = Config;
    type ConfigError = ConfigError;

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d> UartRx<'d, Async> {
    /// Turn the `UartRx` into a buffered uart which can continously receive in the background
    /// without the possibility of losing bytes. The `dma_buf` is a buffer registered to the
    /// DMA controller, and must be large enough to prevent overflows.
    pub fn into_ring_buffered(mut self, dma_buf: &'d mut [u8]) -> RingBufferedUartRx<'d> {
        assert!(!dma_buf.is_empty() && dma_buf.len() <= 0xFFFF);

        let opts = Default::default();

        // Safety: we forget the struct before this function returns.
        let rx_dma = self.rx_dma.as_mut().unwrap();
        let request = rx_dma.request;
        let rx_dma = unsafe { rx_dma.channel.clone_unchecked() };

        let info = self.info;
        let state = self.state;
        let kernel_clock = self.kernel_clock;
        let ring_buf = unsafe { ReadableRingBuffer::new(rx_dma, request, rdr(info.regs), dma_buf, opts) };

        // Don't disable the clock
        mem::forget(self);

        RingBufferedUartRx {
            info,
            state,
            kernel_clock,
            ring_buf,
        }
    }
}

impl<'d> RingBufferedUartRx<'d> {
    /// Clear the ring buffer and start receiving in the background
    pub fn start(&mut self) -> Result<(), Error> {
        // Clear the ring buffer so that it is ready to receive data
        self.ring_buf.clear();

        self.setup_uart();

        Ok(())
    }

    fn stop(&mut self, err: Error) -> Result<usize, Error> {
        self.teardown_uart();

        Err(err)
    }

    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        reconfigure(self.info, self.kernel_clock, config)
    }

    /// Start uart background receive
    fn setup_uart(&mut self) {
        // fence before starting DMA.
        compiler_fence(Ordering::SeqCst);

        // start the dma controller
        self.ring_buf.start();

        let r = self.info.regs;
        // clear all interrupts and DMA Rx Request
        r.cr1().modify(|w| {
            // disable RXNE interrupt
            w.set_rxneie(false);
            // enable parity interrupt if not ParityNone
            w.set_peie(w.pce());
            // enable idle line interrupt
            w.set_idleie(true);
        });
        r.cr3().modify(|w| {
            // enable Error Interrupt: (Frame error, Noise error, Overrun error)
            w.set_eie(true);
            // enable DMA Rx Request
            w.set_dmar(true);
        });
    }

    /// Stop uart background receive
    fn teardown_uart(&mut self) {
        self.ring_buf.request_stop();

        let r = self.info.regs;
        // clear all interrupts and DMA Rx Request
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

        compiler_fence(Ordering::SeqCst);
    }

    /// Read bytes that are readily available in the ring buffer.
    /// If no bytes are currently available in the buffer the call waits until the some
    /// bytes are available (at least one byte and at most half the buffer size)
    ///
    /// Background receive is started if `start()` has not been previously called.
    ///
    /// Receive in the background is terminated if an error is returned.
    /// It must then manually be started again by calling `start()` or by re-calling `read()`.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let r = self.info.regs;

        // Start background receive if it was not already started
        if !r.cr3().read().dmar() {
            self.start()?;
        }

        check_for_errors(clear_idle_flag(r))?;

        loop {
            match self.ring_buf.read(buf) {
                Ok((0, _)) => {}
                Ok((len, _)) => {
                    return Ok(len);
                }
                Err(_) => {
                    return self.stop(Error::Overrun);
                }
            }

            match self.wait_for_data_or_idle().await {
                Ok(_) => {}
                Err(err) => {
                    return self.stop(err);
                }
            }
        }
    }

    /// Wait for uart idle or dma half-full or full
    async fn wait_for_data_or_idle(&mut self) -> Result<(), Error> {
        compiler_fence(Ordering::SeqCst);

        let mut dma_init = false;
        // Future which completes when there is dma is half full or full
        let dma = poll_fn(|cx| {
            self.ring_buf.set_waker(cx.waker());

            let status = match dma_init {
                false => Poll::Pending,
                true => Poll::Ready(()),
            };

            dma_init = true;
            status
        });

        // Future which completes when idle line is detected
        let s = self.state;
        let uart = poll_fn(|cx| {
            s.rx_waker.register(cx.waker());

            compiler_fence(Ordering::SeqCst);

            // Critical section is needed so that IDLE isn't set after
            // our read but before we clear it.
            let sr = critical_section::with(|_| clear_idle_flag(self.info.regs));

            check_for_errors(sr)?;

            if sr.idle() {
                // Idle line is detected
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        });

        match select(dma, uart).await {
            Either::Left(((), _)) => Ok(()),
            Either::Right((result, _)) => result,
        }
    }
}

impl Drop for RingBufferedUartRx<'_> {
    fn drop(&mut self) {
        self.teardown_uart();
        super::drop_tx_rx(self.info, self.state);
    }
}

/// Return an error result if the Sr register has errors
fn check_for_errors(s: Sr) -> Result<(), Error> {
    if s.pe() {
        Err(Error::Parity)
    } else if s.fe() {
        Err(Error::Framing)
    } else if s.ne() {
        Err(Error::Noise)
    } else if s.ore() {
        Err(Error::Overrun)
    } else {
        Ok(())
    }
}

/// Clear IDLE and return the Sr register
fn clear_idle_flag(r: Regs) -> Sr {
    // SAFETY: read only and we only use Rx related flags

    let sr = sr(r).read();

    // This read also clears the error and idle interrupt flags on v1.
    unsafe { rdr(r).read_volatile() };
    clear_interrupt_flags(r, sr);

    r.cr1().modify(|w| w.set_idleie(true));

    sr
}

impl embedded_io_async::ErrorType for RingBufferedUartRx<'_> {
    type Error = Error;
}

impl embedded_io_async::Read for RingBufferedUartRx<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read(buf).await
    }
}
