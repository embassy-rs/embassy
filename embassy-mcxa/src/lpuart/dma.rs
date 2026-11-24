use embassy_hal_internal::Peri;

use super::*;
use crate::dma::{
    Channel, DMA_MAX_TRANSFER_SIZE, DmaChannel, DmaRequest, InvalidParameters, PeripheralReadOperation,
    PeripheralWriteOperation, RingBuffer, TransferOptions,
};
use crate::gpio::AnyPin;

/// DMA mode.
pub struct Dma<'d> {
    dma: DmaChannel<'d>,
    request: DmaRequest,
}
impl sealed::Sealed for Dma<'_> {}
impl Mode for Dma<'_> {}

/// Buffered DMA receiver using finite, peripheral-paced windows.
///
/// DMA stops at allocation capacity instead of overwriting unread data. Reads
/// briefly pause requests and retire DMA before copying. A full allocation
/// stays stopped until consumed; without flow control, UART FIFO overflow is
/// possible during consumer delays and is reported as a sticky Overrun error.
pub struct RingBufferedLpuartRx<'peri, 'ring> {
    ring: RingBuffer<'peri, 'ring, u8>,
    info: &'static Info,
    error: Option<crate::dma::Error>,
}

impl Drop for RingBufferedLpuartRx<'_, '_> {
    fn drop(&mut self) {
        self.info.regs().baud().modify(|w| w.set_rdmae(false));
        let _ = self.info.regs().baud().read();
        // `ring` drops next and retires accepted requests before releasing buf.
    }
}

struct TxDmaGuard<'a> {
    dma: DmaChannel<'a>,
    info: &'static Info,
}

impl<'a> TxDmaGuard<'a> {
    fn new(dma: DmaChannel<'a>, info: &'static Info) -> Self {
        Self { dma, info }
    }

    /// Complete the transfer normally (don't abort on drop).
    fn complete(self) {
        // Cleanup
        self.info.regs().baud().modify(|w| w.set_tdmae(false));
        let _ = self.info.regs().baud().read();
        self.dma.stop();
        // Don't run drop since we've cleaned up
        core::mem::forget(self);
    }
}

impl Drop for TxDmaGuard<'_> {
    fn drop(&mut self) {
        self.info.regs().baud().modify(|w| w.set_tdmae(false));
        let _ = self.info.regs().baud().read();
        self.dma.stop();
    }
}

/// Guard struct for RX DMA transfers.
struct RxDmaGuard<'a> {
    dma: DmaChannel<'a>,
    info: &'static Info,
}

impl<'a> RxDmaGuard<'a> {
    fn new(dma: DmaChannel<'a>, info: &'static Info) -> Self {
        Self { dma, info }
    }

    /// Complete the transfer normally (don't abort on drop).
    fn complete(self) {
        // Ensure DMA writes are visible to CPU
        cortex_m::asm::dsb();
        // Cleanup
        self.info.regs().baud().modify(|w| w.set_rdmae(false));
        let _ = self.info.regs().baud().read();
        self.dma.stop();
        // Don't run drop since we've cleaned up
        core::mem::forget(self);
    }
}

impl Drop for RxDmaGuard<'_> {
    fn drop(&mut self) {
        self.info.regs().baud().modify(|w| w.set_rdmae(false));
        let _ = self.info.regs().baud().read();
        self.dma.stop();
    }
}

impl<'a> LpuartTx<'a, Dma<'a>> {
    /// Create a new LPUART TX driver with DMA support.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_async_with_dma<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        tx_dma_ch: Peri<'a, impl Channel>,
        config: Config,
    ) -> Result<Self, Error> {
        tx_pin.as_tx();
        let tx_pin: Peri<'a, AnyPin> = tx_pin.into();

        // Initialize LPUART with TX enabled, RX disabled, no flow control
        let wg = Lpuart::<Dma<'_>>::init::<T>(true, false, false, false, config)?;

        // Enable interrupt
        let dma = DmaChannel::new(tx_dma_ch);
        dma.enable_interrupt();

        Ok(Self::new_inner::<T>(
            tx_pin,
            None,
            Dma {
                dma,
                request: T::TX_DMA_REQUEST,
            },
            wg,
        ))
    }

    /// Create a new LPUART TX driver with DMA support and CTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_async_with_dma_cts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        tx_dma_ch: Peri<'a, impl Channel>,
        config: Config,
    ) -> Result<Self, Error> {
        tx_pin.as_tx();
        cts_pin.as_cts();
        let tx_pin: Peri<'a, AnyPin> = tx_pin.into();
        let cts_pin: Peri<'a, AnyPin> = cts_pin.into();

        // Initialize LPUART with TX enabled, RX disabled, CTS flow control enabled
        let wg = Lpuart::<Dma<'_>>::init::<T>(true, false, true, false, config)?;

        let dma = DmaChannel::new(tx_dma_ch);
        dma.enable_interrupt();

        Ok(Self::new_inner::<T>(
            tx_pin,
            Some(cts_pin),
            Dma {
                dma,
                request: T::TX_DMA_REQUEST,
            },
            wg,
        ))
    }

    /// Write data using DMA.
    ///
    /// This configures the DMA channel for a memory-to-peripheral transfer
    /// and waits for completion asynchronously. Large buffers are automatically
    /// split into chunks that fit within the DMA transfer limit.
    ///
    /// The DMA request source is automatically derived from the LPUART instance type.
    ///
    /// # Safety
    ///
    /// If the returned future is dropped before completion (e.g., due to a timeout),
    /// the DMA transfer is automatically aborted to prevent use-after-free.
    ///
    /// # Arguments
    /// * `buf` - Data buffer to transmit
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut total = 0;
        for chunk in buf.chunks(DMA_MAX_TRANSFER_SIZE) {
            total += self.write_dma_inner(chunk).await?;
        }

        Ok(total)
    }

    /// Write multiple buffers using one scatter-gather DMA transfer.
    pub async fn write_scatter_gather<const N: usize>(&mut self, bufs: [&[u8]; N]) -> Result<usize, Error> {
        let len = bufs.iter().map(|buf| buf.len()).sum();
        let peri_addr = self.info.regs().data().as_ptr() as *mut u8;
        let mut sequence = PeripheralWriteOperation::new_sequence(bufs.map(|buf| (buf, peri_addr)))?;
        let dma = &mut self.mode.dma;

        unsafe {
            // Clean up channel state
            dma.disable_request();
            dma.clear_done();
            dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            dma.set_request_source(self.mode.request);

            // Configure TCD for memory-to-peripheral scatter-gather transfer
            dma.setup_write_to_peripheral_scatter_gather(&mut sequence)?;

            // Enable UART TX DMA request
            self.info.regs().baud().modify(|w| w.set_tdmae(true));

            // Enable DMA channel request
            dma.enable_request();
        }

        // Create guard that will abort DMA if this future is dropped
        let guard = TxDmaGuard::new(dma.reborrow(), self.info);

        // Wait for completion asynchronously
        core::future::poll_fn(|cx| guard.dma.poll_complete(cx))
            .await
            .map_err(|_| Error::Other)?;

        // Transfer completed successfully - clean up without aborting
        guard.complete();

        Ok(len)
    }

    /// Internal helper to write a single chunk (max 0x7FFF bytes) using DMA.
    async fn write_dma_inner(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let len = buf.len();
        let peri_addr = self.info.regs().data().as_ptr() as *mut u8;

        let dma = &mut self.mode.dma;

        unsafe {
            // Clean up channel state
            dma.disable_request();
            dma.clear_done();
            dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            dma.set_request_source(self.mode.request);

            // Configure TCD for memory-to-peripheral transfer
            dma.setup_write_to_peripheral(buf, peri_addr, false, TransferOptions::COMPLETE_INTERRUPT)?;

            // Enable UART TX DMA request
            self.info.regs().baud().modify(|w| w.set_tdmae(true));

            // Enable DMA channel request
            dma.enable_request();
        }

        // Create guard that will abort DMA if this future is dropped
        let guard = TxDmaGuard::new(dma.reborrow(), self.info);

        // Wait for completion asynchronously
        core::future::poll_fn(|cx| guard.dma.poll_complete(cx))
            .await
            .map_err(|_| Error::Other)?;

        // Transfer completed successfully - clean up without aborting
        guard.complete();

        Ok(len)
    }

    /// Blocking write (fallback when DMA is not needed)
    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<(), Error> {
        for &byte in buf {
            while !self.info.regs().stat().read().tdre() {}
            self.info.regs().data().write(|w| w.0 = byte as u32);
        }
        Ok(())
    }

    /// Flush TX blocking
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        while self.info.regs().water().read().txcount() != 0 {}
        while !self.info.regs().stat().read().tc() {}
        Ok(())
    }
    /// Wait until all written bytes have been fully transmitted on the wire.
    pub async fn flush(&mut self) -> Result<(), Error> {
        // DMA mode doesn't bind the LPUART interrupt, so there's no waker for TC. Poll instead.
        while self.info.regs().water().read().txcount() != 0 || !self.info.regs().stat().read().tc() {
            embassy_futures::yield_now().await;
        }
        Ok(())
    }
}

impl<'a> LpuartRx<'a, Dma<'a>> {
    /// Create a new LPUART RX driver with DMA support.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_async_with_dma<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rx_dma_ch: Peri<'a, impl Channel>,
        config: Config,
    ) -> Result<Self, Error> {
        rx_pin.as_rx();
        let rx_pin: Peri<'a, AnyPin> = rx_pin.into();

        // Initialize LPUART with TX disabled, RX enabled, no flow control
        let _wg = Lpuart::<Dma<'_>>::init::<T>(false, true, false, false, config)?;

        // Enable dma interrupt
        let dma = DmaChannel::new(rx_dma_ch);
        dma.enable_interrupt();

        Ok(Self::new_inner::<T>(
            rx_pin,
            None,
            Dma {
                dma,
                request: T::RX_DMA_REQUEST,
            },
            _wg,
        ))
    }

    /// Create a new LPUART RX driver with DMA support and RTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_async_with_dma_rts<T: Instance>(
        _inner: Peri<'a, T>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        rx_dma_ch: Peri<'a, impl Channel>,
        config: Config,
    ) -> Result<Self, Error> {
        rx_pin.as_rx();
        rts_pin.as_rts();
        let rx_pin: Peri<'a, AnyPin> = rx_pin.into();
        let rts_pin: Peri<'a, AnyPin> = rts_pin.into();

        // Initialize LPUART with TX disabled, RX enabled, RTS flow control enabled
        let wg = Lpuart::<Dma<'_>>::init::<T>(false, true, false, true, config)?;

        let dma = DmaChannel::new(rx_dma_ch);
        dma.enable_interrupt();

        Ok(Self::new_inner::<T>(
            rx_pin,
            Some(rts_pin),
            Dma {
                dma,
                request: T::RX_DMA_REQUEST,
            },
            wg,
        ))
    }

    /// Read data using DMA.
    ///
    /// This configures the DMA channel for a peripheral-to-memory transfer
    /// and waits for completion asynchronously. Large buffers are automatically
    /// split into chunks that fit within the DMA transfer limit.
    ///
    /// The DMA request source is automatically derived from the LPUART instance type.
    ///
    /// # Safety
    ///
    /// If the returned future is dropped before completion (e.g., due to a timeout),
    /// the DMA transfer is automatically aborted to prevent use-after-free.
    ///
    /// # Arguments
    /// * `buf` - Buffer to receive data into
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut total = 0;
        for chunk in buf.chunks_mut(DMA_MAX_TRANSFER_SIZE) {
            total += self.read_dma_inner(chunk).await?;
        }

        Ok(total)
    }

    /// Read into multiple buffers using one scatter-gather DMA transfer.
    pub async fn read_scatter_gather<const N: usize>(&mut self, bufs: [&mut [u8]; N]) -> Result<usize, Error> {
        // First check if there are any RX errors
        check_and_clear_rx_errors(self.info)?;

        let len = bufs.iter().map(|buf| buf.len()).sum();
        let peri_addr = self.info.regs().data().as_ptr() as *const u8;
        let mut sequence = PeripheralReadOperation::new_sequence(bufs.map(|buf| (peri_addr, buf)))?;
        let rx_dma = &mut self.mode.dma;

        unsafe {
            // Clean up channel state
            rx_dma.disable_request();
            rx_dma.clear_done();
            rx_dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            rx_dma.set_request_source(self.mode.request);

            rx_dma.setup_read_from_peripheral_scatter_gather(&mut sequence)?;

            // Enable UART RX DMA request
            self.info.regs().baud().modify(|w| w.set_rdmae(true));

            // Enable DMA channel request
            rx_dma.enable_request();
        }

        // Create guard that will abort DMA if this future is dropped
        let guard = RxDmaGuard::new(rx_dma.reborrow(), self.info);

        // Wait for completion asynchronously
        core::future::poll_fn(|cx| guard.dma.poll_complete(cx))
            .await
            .map_err(|_| Error::Other)?;

        // Transfer completed successfully - clean up without aborting
        guard.complete();

        Ok(len)
    }

    /// Internal helper to read a single chunk (max 0x7FFF bytes) using DMA.
    async fn read_dma_inner(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        // First check if there are any RX errors
        check_and_clear_rx_errors(self.info)?;

        // We already check in the public function that the length is
        // 0 < buf.len <= DMA_MAX_TRANSFER_SIZE
        let len = buf.len();
        let peri_addr = self.info.regs().data().as_ptr() as *const u8;
        let rx_dma = &mut self.mode.dma;

        unsafe {
            // Clean up channel state
            rx_dma.disable_request();
            rx_dma.clear_done();
            rx_dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            rx_dma.set_request_source(self.mode.request);

            // Configure TCD for peripheral-to-memory transfer
            rx_dma.setup_read_from_peripheral(peri_addr, buf, false, TransferOptions::COMPLETE_INTERRUPT)?;

            // Enable UART RX DMA request
            self.info.regs().baud().modify(|w| w.set_rdmae(true));

            // Enable DMA channel request
            rx_dma.enable_request();
        }

        // Create guard that will abort DMA if this future is dropped
        let guard = RxDmaGuard::new(rx_dma.reborrow(), self.info);

        // Wait for completion asynchronously
        core::future::poll_fn(|cx| guard.dma.poll_complete(cx))
            .await
            .map_err(|_| Error::Other)?;

        // Transfer completed successfully - clean up without aborting
        guard.complete();

        Ok(len)
    }

    /// Blocking read (fallback when DMA is not needed)
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        for byte in buf.iter_mut() {
            loop {
                if has_rx_data_pending(self.info) {
                    *byte = (self.info.regs().data().read().0 & 0xFF) as u8;
                    break;
                }
                check_and_clear_rx_errors(self.info)?;
            }
        }
        Ok(())
    }

    /// Discard all bytes currently buffered in the hardware RX FIFO.
    pub fn blocking_flush(&mut self) -> Result<(), Error> {
        self.info.regs().fifo().modify(|w| w.set_rxflush(true));

        // Flushing is a recovery operation, so clear stale receive errors
        // without reporting them to the next transfer.
        let _ = check_and_clear_rx_errors(self.info);
        Ok(())
    }

    /// Start a borrowed, buffered DMA receiver.
    ///
    /// # Safety
    ///
    /// `buf` must be aligned DMA-accessible, coherent storage that remains live
    /// and exclusive to this receiver until DMA actually retires, even if the
    /// receiver is forgotten. Drop normally disables the UART request and drains
    /// DMA, but forgetting does neither. CPU reads retire DMA before copying;
    /// callers need not race a continuously wrapping producer. Forgetting the
    /// receiver still does not retain its backing allocation.
    pub unsafe fn into_ring_dma_rx<'buf: 'a>(
        &mut self,
        buf: &'buf mut [u8],
    ) -> Result<RingBufferedLpuartRx<'_, 'buf>, InvalidParameters> {
        let info = self.info;
        let mut ring = self.setup_ring_buffer(buf)?;
        // SAFETY: setup_ring_buffer disabled/read back RDMAE and stopped the
        // channel. The caller supplies the storage lifetime/exclusivity contract.
        unsafe { ring.resume()? };
        info.regs().baud().modify(|w| w.set_rdmae(true));
        Ok(RingBufferedLpuartRx {
            ring,
            info,
            error: None,
        })
    }

    /// Set up a finite DMA receive allocation without enabling its request source.
    ///
    /// This method encapsulates all the low-level setup:
    /// - Configures the DMA request source for this LPUART instance
    /// - Disables/read backs RX DMA requests and retires the previous transfer
    /// - Prepares storage for finite receive windows
    ///
    /// # Arguments
    ///
    /// * `buf` - Nonempty destination buffer, no larger than DMA_MAX_TRANSFER_SIZE
    ///
    /// # Returns
    ///
    /// A [`RingBuffer`] that can be used to asynchronously read received data.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// #
    /// # extern crate panic_halt;
    /// # extern crate embassy_mcxa;
    /// # extern crate embassy_executor;
    /// # use panic_halt as _;
    /// # use embassy_executor::Spawner;
    /// # use embassy_mcxa::clocks::config::Div8;
    /// # use embassy_mcxa::config::Config;
    /// # use embassy_mcxa::lpuart;
    /// # use static_cell::ConstStaticCell;
    /// #
    /// static RX_RING_BUFFER: ConstStaticCell<[u8; 64]> = ConstStaticCell::new([0; 64]);
    /// # #[embassy_executor::main]
    /// # async fn main(_spawner: Spawner) {
    /// #     let mut cfg = Config::default();
    /// #     cfg.clock_cfg.sirc.fro_12m_enabled = true;
    /// #     cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    /// #
    /// #     let p = embassy_mcxa::init(cfg);
    /// #
    /// #     // Create UART configuration
    /// #     let config = lpuart::Config {
    /// #     baudrate_bps: 115_200,
    /// #     ..Default::default()
    /// #     };
    ///
    ///      // Create UART instance with DMA channels
    ///      let mut lpuart = lpuart::Lpuart::new_async_with_dma(
    ///          p.LPUART2, // Instance
    ///          p.P2_2,    // TX pin
    ///          p.P2_3,    // RX pin
    ///          p.DMA_CH0, // TX DMA channel
    ///          p.DMA_CH1, // RX DMA channel
    ///          config,
    ///      )
    ///      .unwrap();
    ///
    ///      let (_, mut rx) = lpuart.split();
    ///
    ///      let buf = RX_RING_BUFFER.take();
    ///      // SAFETY: static exclusive DMA-accessible SRAM; never forget the
    ///      // receiver while its backing allocation could be reused.
    ///      let mut ring_buf = unsafe { rx.into_ring_dma_rx(buf) }.unwrap();
    ///      let mut read_buf = [0u8; 16];
    ///      let n = ring_buf.read(&mut read_buf).await.unwrap();
    /// # }
    /// ```
    fn setup_ring_buffer<'buf: 'a>(
        &mut self,
        buf: &'buf mut [u8],
    ) -> Result<RingBuffer<'_, 'buf, u8>, InvalidParameters> {
        // Get the peripheral data register address
        let peri_addr = self.info.regs().data().as_ptr() as *const u8;

        // Configure before enabling the request source. Invalid parameters must
        // not leave the UART issuing requests to a previous descriptor.
        self.info.regs().baud().modify(|w| w.set_rdmae(false));
        let _ = self.info.regs().baud().read();
        self.mode.dma.stop();

        unsafe {
            self.mode.dma.set_request_source(self.mode.request);
            self.mode.dma.setup_buffered_read(peri_addr, buf)
        }
    }
}

impl<'peri, 'buf> RingBufferedLpuartRx<'peri, 'buf> {
    /// Read a batch, pausing and retiring DMA before copying from its allocation.
    ///
    /// Notification remains DMA half/full/error driven. A short burst smaller
    /// than the remaining window's half threshold may wait for more traffic;
    /// this API does not introduce a UART idle-interrupt binding.
    /// Cancellation leaves the long-lived receiver armed. Overrun and DMA errors
    /// remain sticky until `clear`, which explicitly discards buffered data.
    pub async fn read(&mut self, dst: &mut [u8]) -> Result<usize, crate::dma::Error> {
        use core::task::Poll;
        if dst.is_empty() {
            return Ok(0);
        }
        if let Some(error) = self.error {
            return Err(error);
        }
        core::future::poll_fn(|cx| {
            loop {
                let registered = self.ring.poll_wait(cx);
                if let Err(error) = self.pause() {
                    return Poll::Ready(Err(error));
                }
                let n = self.ring.read_quiescent(dst);
                // FIFO overflow during the CPU copy is a data gap too. Do not clear
                // its evidence or report unconditional success before rearming.
                if self.info.regs().stat().read().or() {
                    self.error = Some(crate::dma::Error::Overrun);
                    return Poll::Ready(Err(crate::dma::Error::Overrun));
                }
                if let Err(error) = self.rearm() {
                    return Poll::Ready(Err(error));
                }
                if n > 0 {
                    return Poll::Ready(Ok(n));
                }
                if registered {
                    return Poll::Pending;
                }
            }
        })
        .await
    }

    fn pause(&mut self) -> Result<(), crate::dma::Error> {
        critical_section::with(|_| {
            self.info.regs().baud().modify(|w| w.set_rdmae(false));
            let _ = self.info.regs().baud().read();
        });
        let result = self.ring.pause();
        let error = result.err().or_else(|| {
            self.info
                .regs()
                .stat()
                .read()
                .or()
                .then_some(crate::dma::Error::Overrun)
        });
        if let Some(error) = error {
            self.error = Some(error);
            Err(error)
        } else {
            Ok(())
        }
    }

    fn rearm(&mut self) -> Result<(), crate::dma::Error> {
        // SAFETY: pause disabled/read back the source and retired DMA. No CPU
        // slice survives read_quiescent. Only consumed/unused storage is armed.
        match unsafe { self.ring.resume() } {
            Ok(true) => {
                critical_section::with(|_| self.info.regs().baud().modify(|w| w.set_rdmae(true)));
                Ok(())
            }
            Ok(false) => Ok(()), // Full allocation still contains unread data.
            Err(_) => {
                self.error = Some(crate::dma::Error::Configuration);
                Err(crate::dma::Error::Configuration)
            }
        }
    }

    /// Discard buffered data and FIFO contents, acknowledge errors, and rearm.
    /// This is explicit data-gap recovery, not a lossless operation.
    pub fn clear(&mut self) {
        let _ = self.pause();
        self.info.regs().fifo().modify(|w| w.set_rxflush(true));
        let _ = check_and_clear_rx_errors(self.info);
        self.ring.clear_quiescent();
        self.error = None;
        let _ = self.rearm();
    }
}

impl<'a> Lpuart<'a, Dma<'a>> {
    /// Create a new LPUART driver with DMA support for both TX and RX.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_async_with_dma<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        tx_dma_ch: Peri<'a, impl Channel>,
        rx_dma_ch: Peri<'a, impl Channel>,
        config: Config,
    ) -> Result<Self, Error> {
        tx_pin.as_tx();
        rx_pin.as_rx();

        let tx_pin: Peri<'a, AnyPin> = tx_pin.into();
        let rx_pin: Peri<'a, AnyPin> = rx_pin.into();

        // Initialize LPUART with both TX and RX enabled, no flow control
        let wg = Lpuart::<Dma<'a>>::init::<T>(true, true, false, false, config)?;

        // Enable DMA interrupts
        let tx_dma = DmaChannel::new(tx_dma_ch);
        let rx_dma = DmaChannel::new(rx_dma_ch);
        tx_dma.enable_interrupt();
        rx_dma.enable_interrupt();

        Ok(Self {
            tx: LpuartTx::new_inner::<T>(
                tx_pin,
                None,
                Dma {
                    dma: tx_dma,
                    request: T::TX_DMA_REQUEST,
                },
                wg.clone(),
            ),
            rx: LpuartRx::new_inner::<T>(
                rx_pin,
                None,
                Dma {
                    dma: rx_dma,
                    request: T::RX_DMA_REQUEST,
                },
                wg,
            ),
        })
    }

    /// Create a new full-duplex LPUART DMA driver with RTS/CTS flow control.
    ///
    /// Any external pin will be placed into Disabled state upon Drop.
    pub fn new_async_with_dma_rtscts<T: Instance>(
        _inner: Peri<'a, T>,
        tx_pin: Peri<'a, impl TxPin<T>>,
        rx_pin: Peri<'a, impl RxPin<T>>,
        cts_pin: Peri<'a, impl CtsPin<T>>,
        rts_pin: Peri<'a, impl RtsPin<T>>,
        tx_dma_ch: Peri<'a, impl Channel>,
        rx_dma_ch: Peri<'a, impl Channel>,
        config: Config,
    ) -> Result<Self, Error> {
        tx_pin.as_tx();
        rx_pin.as_rx();
        cts_pin.as_cts();
        rts_pin.as_rts();

        let tx_pin: Peri<'a, AnyPin> = tx_pin.into();
        let rx_pin: Peri<'a, AnyPin> = rx_pin.into();
        let cts_pin: Peri<'a, AnyPin> = cts_pin.into();
        let rts_pin: Peri<'a, AnyPin> = rts_pin.into();

        // Initialize LPUART with both TX and RX enabled, and both flow controls enabled
        let wg = Lpuart::<Dma<'a>>::init::<T>(true, true, true, true, config)?;

        let tx_dma = DmaChannel::new(tx_dma_ch);
        let rx_dma = DmaChannel::new(rx_dma_ch);
        tx_dma.enable_interrupt();
        rx_dma.enable_interrupt();

        Ok(Self {
            tx: LpuartTx::new_inner::<T>(
                tx_pin,
                Some(cts_pin),
                Dma {
                    dma: tx_dma,
                    request: T::TX_DMA_REQUEST,
                },
                wg.clone(),
            ),
            rx: LpuartRx::new_inner::<T>(
                rx_pin,
                Some(rts_pin),
                Dma {
                    dma: rx_dma,
                    request: T::RX_DMA_REQUEST,
                },
                wg,
            ),
        })
    }

    /// Write data using DMA
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.tx.write(buf).await
    }

    /// Read data using DMA
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.rx.read(buf).await
    }
    /// Flush TX.
    ///
    /// See [`LpuartTx::flush`] for more information.
    pub async fn flush(&mut self) -> Result<(), Error> {
        self.tx.flush().await
    }
}

impl<'a> embedded_io_async::Read for LpuartRx<'a, Dma<'a>> {
    async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        self.read(buf).await
    }
}

impl<'a> embedded_io_async::Write for LpuartTx<'a, Dma<'a>> {
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        self.write(buf).await
    }

    async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        self.flush().await
    }
}

impl<'a> embedded_io_async::Read for Lpuart<'a, Dma<'a>> {
    async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        self.read(buf).await
    }
}

impl<'a> embedded_io_async::Write for Lpuart<'a, Dma<'a>> {
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        self.write(buf).await
    }

    async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        self.flush().await
    }
}
