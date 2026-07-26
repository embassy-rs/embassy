use core::future::poll_fn;
use core::mem;
use core::pin::pin;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_hal_internal::Peri;
use embedded_io_async::ReadReady;
use futures_util::future::select;

use super::mode::Slave;
use super::{Config, Error, Info, RegsExt, Spi, Word, check_error_flags, reconfigure, set_rxdmaen};
use crate::dma::ReadableRingBuffer;
use crate::exti::{Channel, ExtiInput, InterruptHandler};
use crate::gpio::{Flex, Pin};
use crate::interrupt::typelevel::Binding;
use crate::mode::Async;
use crate::rcc::WakeGuard;
#[cfg(any(spi_v4, spi_v5, spi_v6))]
use crate::spi::SlaveSelectPolarity;
use crate::time::Hertz;

/// Rx-only Ring-buffered SPI Driver
///
/// Created with [Spi::into_ring_buffered]
///
/// ### Notes on 'waiting for bytes'
///
/// The `read(buf)` (but not `read()`) function
/// may need to wait for bytes to arrive, if the ring buffer does not
/// contain enough bytes to fill the buffer passed by the caller of
/// the function, or is empty.
///
/// Waiting for bytes operates in one of two modes, depending on
/// the behavior of the sender, the size of the buffer passed
/// to the function:
///
/// - If the slave is deselected (either rising or falling edge on NSS pin
/// depending on SPI configuration), then any bytes in the ring buffer
/// will be returned. If there are no bytes in the buffer, the check will
/// be repeated each time the NSS deselect edge is detected, so if the
/// sender sends just a single byte, it will be returned once the NSS
/// deselect edge is detected.
///
/// - If the sender sends continuously, the call will wait until
/// the DMA controller indicates that it has written to either the
/// middle byte or last byte of the ring buffer ('half transfer'
/// or 'transfer complete', respectively). This does not indicate
/// the buffer is half-full or full, though, because the DMA
/// controller does not detect those conditions; it sends an
/// interrupt when those specific buffer addresses have been written.
pub struct RingBufferedSpiRx<'d, W: Word> {
    info: &'static Info,
    kernel_clock: Hertz,
    _wake_guard: WakeGuard,
    _sck: Option<Flex<'d>>,
    _mosi: Option<Flex<'d>>,
    _miso: Option<Flex<'d>>,
    nss: ExtiInput<'d, Async>,
    #[cfg(any(spi_v4, spi_v5, spi_v6))]
    nss_polarity: SlaveSelectPolarity,
    ring_buf: ReadableRingBuffer<'d, W>,
}

impl<'d, W: Word> SetConfig for RingBufferedSpiRx<'d, W> {
    type Config = Config;
    type ConfigError = ();

    fn set_config(&mut self, config: &Self::Config) -> Result<(), Self::ConfigError> {
        self.set_config(config)
    }
}

impl<'d> Spi<'d, Async, Slave> {
    /// Turn the `Spi` into a buffered spi which can continuously receive in the background
    /// without the possibility of losing bytes. The `dma_buf` is a buffer registered to the
    /// DMA controller, and must be large enough to prevent overflows.
    pub fn into_ring_buffered<W: Word, C: Channel>(
        mut self,
        dma_buf: &'d mut [W],
        ch: Peri<'d, C>,
        irq: impl Binding<C::IRQ, InterruptHandler<C::IRQ>>,
    ) -> RingBufferedSpiRx<'d, W> {
        assert!(!dma_buf.is_empty() && dma_buf.len() <= 0xFFFF);

        self.set_word_size(W::CONFIG);

        let opts = Default::default();

        // Safety: we forget the struct before this function returns.
        let rx_dma = self.rx_dma.as_mut().unwrap();
        let request = rx_dma.request;
        let rx_dma = unsafe { rx_dma.channel.clone_unchecked() };

        let tx_dma = unsafe { self.tx_dma.as_ref().map(|x| x.clone_unchecked()) };

        let info = self.info;
        let kernel_clock = self.kernel_clock;
        let ring_buf = unsafe { ReadableRingBuffer::new(rx_dma, request, info.regs.rx_ptr::<W>(), dma_buf, opts) };
        let sck = unsafe { self._sck.as_ref().map(|x| x.clone_unchecked()) };
        let mosi = unsafe { self._mosi.as_ref().map(|x| x.clone_unchecked()) };
        let miso = unsafe { self.miso.as_ref().map(|x| x.clone_unchecked()) };
        let nss = unsafe { self.nss.as_ref().unwrap().clone_unchecked() };

        // verify at runtime whether given EXTI channel is associated with NSS pin
        assert_eq!(nss.pin.pin(), ch.number());
        // EXTI can be used on alternate function pins
        // this feature seems to be undocumented though
        let nss = unsafe { ExtiInput::from_flex(nss, ch, irq) };

        let wake_guard = self.info.rcc.wake_guard();

        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        let nss_polarity = if self.info.regs.cfg2().read().ssiop() == super::vals::Ssiop::ActiveLow {
            SlaveSelectPolarity::ActiveLow
        } else {
            SlaveSelectPolarity::ActiveHigh
        };

        // Don't disable the clock
        mem::forget(self);

        // Drop (cleanup) peripherals that are no longer needed.
        // Miso pin is kept to maintain its state set by user.
        drop(tx_dma);

        RingBufferedSpiRx {
            info,
            kernel_clock,
            _wake_guard: wake_guard,
            _sck: sck,
            _mosi: mosi,
            _miso: miso,
            nss,
            #[cfg(any(spi_v4, spi_v5, spi_v6))]
            nss_polarity,
            ring_buf,
        }
    }
}

impl<'d, W: Word> RingBufferedSpiRx<'d, W> {
    /// Reconfigure the driver
    pub fn set_config(&mut self, config: &Config) -> Result<(), ()> {
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        {
            self.nss_polarity = config.nss_polarity;
        }
        #[cfg(gpio_v2)]
        super::set_speed(&self._sck, &self._mosi, config.gpio_speed);
        reconfigure(self.info, self.kernel_clock, config)
    }

    /// Configure and start the DMA backed SPI receiver
    ///
    /// Note: This is also done automatically by the read functions if
    /// required.
    pub fn start(&mut self) {
        compiler_fence(Ordering::SeqCst);
        self.ring_buf.start();

        set_rxdmaen(self.info.regs, true);

        self.info.regs.cr1().modify(|w| {
            w.set_spe(true);
        });
    }

    /// Stop DMA backed SPI receiver
    fn stop(&mut self) {
        self.ring_buf.request_pause();

        set_rxdmaen(self.info.regs, false);

        self.info.regs.cr1().modify(|w| {
            w.set_spe(false);
        });

        compiler_fence(Ordering::SeqCst);
    }

    /// (Re-)start DMA and SPI if it is not running (has not been started yet or has failed), and
    /// check for errors in status register. Error flags are checked/cleared first.
    fn start_or_check_errors(&mut self) -> Result<(), Error> {
        let r = self.info.regs;

        check_error_flags(r.sr().read(), true)?;

        if !self.ring_buf.is_running() {
            self.start();
        }
        Ok(())
    }

    /// Read bytes that are available in the ring buffer.
    ///
    /// Background reception is started if necessary (if `start()` had
    /// not previously been called, or if an error was detected which
    /// caused background reception to be stopped).
    ///
    /// Background reception is terminated when an error is returned.
    /// It must be started again by calling `start()` or by
    /// calling a read function again.
    pub async fn read(&mut self, buf: &mut [W]) -> Result<usize, Error> {
        self.start_or_check_errors()?;

        loop {
            match self.ring_buf.read(buf) {
                Ok((0, _)) => {}
                Ok((len, _)) => {
                    return Ok(len);
                }
                Err(_) => {
                    self.stop();
                    return Err(Error::Overrun);
                }
            }

            self.wait_for_data_or_nss_deselect_edge().await;
        }
    }

    /// Wait for NSS deselect edge or dma half-full or full.
    ///
    /// NSS deselect edge is:
    /// spi_v1, spi_v2, spi_v3 - not configurable, always rising edge
    /// spi_v4, spi_v5, spi_v6 - configurable, matches slave select polarity
    async fn wait_for_data_or_nss_deselect_edge(&mut self) {
        compiler_fence(Ordering::SeqCst);

        let mut dma_init = false;
        // Future which completes when the DMA controller indicates it
        // has written to the ring buffer's middle byte, or last byte
        let dma = poll_fn(|cx| {
            self.ring_buf.set_waker(cx.waker());

            let status = match dma_init {
                false => Poll::Pending,
                true => Poll::Ready(()),
            };

            dma_init = true;
            status
        });

        // Future which completes when NSS deselect edge is detected
        #[cfg(any(spi_v4, spi_v5, spi_v6))]
        match self.nss_polarity {
            SlaveSelectPolarity::ActiveHigh => {
                let exti = self.nss.wait_for_falling_edge();
                let exti = pin!(exti);

                select(exti, dma).await;
            }
            SlaveSelectPolarity::ActiveLow => {
                let exti = self.nss.wait_for_rising_edge();
                let exti = pin!(exti);

                select(exti, dma).await;
            }
        };
        #[cfg(not(any(spi_v4, spi_v5, spi_v6)))]
        {
            let exti = self.nss.wait_for_rising_edge();
            let exti = pin!(exti);

            select(exti, dma).await;
        }
    }

    /// Read bytes that are readily available in the ring buffer.
    /// If no bytes are currently available in the buffer the call waits until the some
    /// bytes are available (at least one byte and at most half the buffer size)
    ///
    /// Background receive is started if `start()` has not been previously called.
    ///
    /// Receive in the background is terminated if an error is returned.
    /// It must then manually be started again by calling `start()` or by re-calling `blocking_read()`.
    pub fn blocking_read(&mut self, buf: &mut [W]) -> Result<usize, Error> {
        self.start_or_check_errors()?;

        loop {
            match self.ring_buf.read(buf) {
                Ok((0, _)) => {}
                Ok((len, _)) => {
                    return Ok(len);
                }
                Err(_) => {
                    self.stop();

                    return Err(Error::Overrun);
                }
            }
        }
    }
}

impl<W: Word> Drop for RingBufferedSpiRx<'_, W> {
    fn drop(&mut self) {
        self.stop();
        self.info.rcc.disable();
    }
}

impl<W: Word> embedded_io_async::ErrorType for RingBufferedSpiRx<'_, W> {
    type Error = Error;
}

impl embedded_io_async::Read for RingBufferedSpiRx<'_, u8> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read(buf).await
    }
}

impl<W: Word> ReadReady for RingBufferedSpiRx<'_, W> {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        let len = self.ring_buf.len().map_err(|e| match e {
            crate::dma::ringbuffer::Error::Overrun => Error::Overrun,
            crate::dma::ringbuffer::Error::DmaUnsynced => {
                error!(
                    "Ringbuffer error: DmaUNsynced, driver implementation is
                    probably bugged please open an issue"
                );
                // we report this as overrun since its recoverable in the same way
                Error::Overrun
            }
        })?;
        Ok(len > 0)
    }
}
