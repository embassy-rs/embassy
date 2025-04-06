use core::future::poll_fn;
use core::mem;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embedded_io_async::ReadReady;
use futures_util::future::{select, Either};

use super::{rdr, reconfigure, set_baudrate, sr, Config, ConfigError, Error, Info, State, UartRx};
use crate::dma::ReadableRingBuffer;
use crate::gpio::{AnyPin, SealedPin as _};
use crate::mode::Async;
use crate::time::Hertz;
use crate::usart::Regs;
use crate::Peri;

/// Rx-only Ring-buffered UART Driver
///
/// Created with [UartRx::into_ring_buffered]
pub struct RingBufferedUartRx<'d> {
    info: &'static Info,
    state: &'static State,
    kernel_clock: Hertz,
    rx: Option<Peri<'d, AnyPin>>,
    rts: Option<Peri<'d, AnyPin>>,
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
        let rx = unsafe { self.rx.as_ref().map(|x| x.clone_unchecked()) };
        let rts = unsafe { self.rts.as_ref().map(|x| x.clone_unchecked()) };

        // Don't disable the clock
        mem::forget(self);

        RingBufferedUartRx {
            info,
            state,
            kernel_clock,
            rx,
            rts,
            ring_buf,
        }
    }
}

impl<'d> RingBufferedUartRx<'d> {
    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        reconfigure(self.info, self.kernel_clock, config)
    }

    /// Configure and start the DMA backed UART receiver
    ///
    /// Note: This is also done automatically by [`read()`] if required.
    pub fn start_uart(&mut self) {
        // Clear the buffer so that it is ready to receive data
        compiler_fence(Ordering::SeqCst);
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

    /// Stop DMA backed UART receiver
    fn stop_uart(&mut self) {
        self.ring_buf.request_pause();

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

    /// (Re-)start DMA and Uart if it is not running (has not been started yet or has failed), and
    /// check for errors in status register. Error flags are checked/cleared first.
    fn start_dma_or_check_errors(&mut self) -> Result<(), Error> {
        let r = self.info.regs;

        check_idle_and_errors(r)?;
        if !r.cr3().read().dmar() {
            self.start_uart();
        }
        Ok(())
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
        self.start_dma_or_check_errors()?;

        // In half-duplex mode, we need to disable the Transmitter and enable the Receiver
        // since they can't operate simultaneously on the shared line
        let r = self.info.regs;
        if r.cr3().read().hdsel() && r.cr1().read().te() {
            r.cr1().modify(|reg| {
                reg.set_re(true);
                reg.set_te(false);
            });
        }

        loop {
            match self.ring_buf.read(buf) {
                Ok((0, _)) => {}
                Ok((len, _)) => {
                    return Ok(len);
                }
                Err(_) => {
                    self.stop_uart();
                    return Err(Error::Overrun);
                }
            }

            match self.wait_for_data_or_idle().await {
                Ok(_) => {}
                Err(err) => {
                    self.stop_uart();
                    return Err(err);
                }
            }
        }
    }

    /// Wait for uart idle or dma half-full or full
    async fn wait_for_data_or_idle(&mut self) -> Result<(), Error> {
        compiler_fence(Ordering::SeqCst);

        // Future which completes when idle line is detected
        let s = self.state;
        let uart = poll_fn(|cx| {
            s.rx_waker.register(cx.waker());

            compiler_fence(Ordering::SeqCst);

            if check_idle_and_errors(self.info.regs)? {
                // Idle line is detected
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        });

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

        match select(uart, dma).await {
            Either::Left((result, _)) => result,
            Either::Right(((), _)) => Ok(()),
        }
    }

    /// Set baudrate
    pub fn set_baudrate(&self, baudrate: u32) -> Result<(), ConfigError> {
        set_baudrate(self.info, self.kernel_clock, baudrate)
    }
}

impl Drop for RingBufferedUartRx<'_> {
    fn drop(&mut self) {
        self.stop_uart();
        self.rx.as_ref().map(|x| x.set_as_disconnected());
        self.rts.as_ref().map(|x| x.set_as_disconnected());
        super::drop_tx_rx(self.info, self.state);
    }
}

/// Check and clear idle and error interrupts, return true if idle, Err(e) on error
///
/// All flags are read and cleared in a single step, respectively. When more than one flag is set
/// at the same time, all flags will be cleared but only one flag will be reported. So the other
/// flag(s) will gone missing unnoticed. The error flags are checked first, the idle flag last.
///
/// For usart_v1 and usart_v2, all status flags must be handled together anyway because all flags
/// are cleared by a single read to the RDR register.
fn check_idle_and_errors(r: Regs) -> Result<bool, Error> {
    // Critical section is required so that the flags aren't set after read and before clear
    let sr = critical_section::with(|_| {
        // SAFETY: read only and we only use Rx related flags
        let sr = sr(r).read();

        #[cfg(any(usart_v3, usart_v4))]
        r.icr().write(|w| {
            w.set_idle(true);
            w.set_pe(true);
            w.set_fe(true);
            w.set_ne(true);
            w.set_ore(true);
        });
        #[cfg(not(any(usart_v3, usart_v4)))]
        unsafe {
            // This read also clears the error and idle interrupt flags on v1 (TODO and v2?)
            rdr(r).read_volatile()
        };
        sr
    });
    if sr.pe() {
        Err(Error::Parity)
    } else if sr.fe() {
        Err(Error::Framing)
    } else if sr.ne() {
        Err(Error::Noise)
    } else if sr.ore() {
        Err(Error::Overrun)
    } else {
        r.cr1().modify(|w| w.set_idleie(true));
        Ok(sr.idle())
    }
}

impl embedded_io_async::ErrorType for RingBufferedUartRx<'_> {
    type Error = Error;
}

impl embedded_io_async::Read for RingBufferedUartRx<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read(buf).await
    }
}

impl embedded_hal_nb::serial::Read for RingBufferedUartRx<'_> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.start_dma_or_check_errors()?;

        let mut buf = [0u8; 1];
        match self.ring_buf.read(&mut buf) {
            Ok((0, _)) => Err(nb::Error::WouldBlock),
            Ok((len, _)) => {
                assert!(len == 1);
                Ok(buf[0])
            }
            Err(_) => {
                self.stop_uart();
                Err(nb::Error::Other(Error::Overrun))
            }
        }
    }
}

impl embedded_hal_nb::serial::ErrorType for RingBufferedUartRx<'_> {
    type Error = Error;
}

impl ReadReady for RingBufferedUartRx<'_> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        let len = self.ring_buf.len().map_err(|e| match e {
            crate::dma::ringbuffer::Error::Overrun => Self::Error::Overrun,
            crate::dma::ringbuffer::Error::DmaUnsynced => {
                error!(
                    "Ringbuffer error: DmaUNsynced, driver implementation is 
                    probably bugged please open an issue"
                );
                // we report this as overrun since its recoverable in the same way
                Self::Error::Overrun
            }
        })?;
        Ok(len > 0)
    }
}
