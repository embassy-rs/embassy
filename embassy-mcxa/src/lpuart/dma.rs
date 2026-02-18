use core::future::Future;

use embassy_hal_internal::Peri;

use super::*;
use crate::dma::{Channel, DMA_MAX_TRANSFER_SIZE, DmaChannel, DmaRequest, EnableInterrupt, RingBuffer};
use crate::gpio::AnyPin;
use crate::pac::lpuart::vals::{Tc, Tdre};

/// DMA mode.
pub struct Dma<'d> {
    dma: DmaChannel<'d>,
    request_number: u8,
}
impl sealed::Sealed for Dma<'_> {}
impl Mode for Dma<'_> {}

/// Lpuart RX driver with ring-buffered DMA support.
pub struct RingBufferedLpuartRx<'peri, 'ring> {
    ring: RingBuffer<'peri, 'ring, u8>,
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
        unsafe {
            self.dma.disable_request();
            self.dma.clear_done();
        }
        // Don't run drop since we've cleaned up
        core::mem::forget(self);
    }
}

impl Drop for TxDmaGuard<'_> {
    fn drop(&mut self) {
        // Abort the DMA transfer if still running
        unsafe {
            self.dma.disable_request();
            self.dma.clear_done();
            self.dma.clear_interrupt();
        }
        // Disable UART TX DMA request
        self.info.regs().baud().modify(|w| w.set_tdmae(false));
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
        unsafe {
            self.dma.disable_request();
            self.dma.clear_done();
        }
        // Don't run drop since we've cleaned up
        core::mem::forget(self);
    }
}

impl Drop for RxDmaGuard<'_> {
    fn drop(&mut self) {
        // Abort the DMA transfer if still running
        unsafe {
            self.dma.disable_request();
            self.dma.clear_done();
            self.dma.clear_interrupt();
        }
        // Disable UART RX DMA request
        self.info.regs().baud().modify(|w| w.set_rdmae(false));
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
    ) -> Result<Self> {
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
                request_number: T::TxDmaRequest::REQUEST_NUMBER,
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
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut total = 0;
        for chunk in buf.chunks(DMA_MAX_TRANSFER_SIZE) {
            total += self.write_dma_inner(chunk).await?;
        }

        Ok(total)
    }

    /// Internal helper to write a single chunk (max 0x7FFF bytes) using DMA.
    async fn write_dma_inner(&mut self, buf: &[u8]) -> Result<usize> {
        let len = buf.len();
        let peri_addr = self.info.regs().data().as_ptr() as *mut u8;

        let dma = &mut self.mode.dma;

        unsafe {
            // Clean up channel state
            dma.disable_request();
            dma.clear_done();
            dma.clear_interrupt();

            // Set DMA request source from instance type (type-safe)
            dma.set_request_source(self.mode.request_number);

            // Configure TCD for memory-to-peripheral transfer
            dma.setup_write_to_peripheral(buf, peri_addr, EnableInterrupt::Yes);

            // Enable UART TX DMA request
            self.info.regs().baud().modify(|w| w.set_tdmae(true));

            // Enable DMA channel request
            dma.enable_request();
        }

        // Create guard that will abort DMA if this future is dropped
        let guard = TxDmaGuard::new(dma.reborrow(), self.info);

        // Wait for completion asynchronously
        core::future::poll_fn(|cx| {
            guard.dma.waker().register(cx.waker());
            if guard.dma.is_done() {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        })
        .await;

        // Transfer completed successfully - clean up without aborting
        guard.complete();

        Ok(len)
    }

    /// Blocking write (fallback when DMA is not needed)
    pub fn blocking_write(&mut self, buf: &[u8]) -> Result<()> {
        for &byte in buf {
            while self.info.regs().stat().read().tdre() == Tdre::TXDATA {}
            self.info.regs().data().write(|w| w.0 = byte as u32);
        }
        Ok(())
    }

    /// Flush TX blocking
    pub fn blocking_flush(&mut self) -> Result<()> {
        while self.info.regs().water().read().txcount() != 0 {}
        while self.info.regs().stat().read().tc() == Tc::ACTIVE {}
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
    ) -> Result<Self> {
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
                request_number: T::RxDmaRequest::REQUEST_NUMBER,
            },
            _wg,
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
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut total = 0;
        for chunk in buf.chunks_mut(DMA_MAX_TRANSFER_SIZE) {
            total += self.read_dma_inner(chunk).await?;
        }

        Ok(total)
    }

    /// Internal helper to read a single chunk (max 0x7FFF bytes) using DMA.
    async fn read_dma_inner(&mut self, buf: &mut [u8]) -> Result<usize> {
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
            rx_dma.set_request_source(self.mode.request_number);

            // Configure TCD for peripheral-to-memory transfer
            rx_dma.setup_read_from_peripheral(peri_addr, buf, EnableInterrupt::Yes);

            // Enable UART RX DMA request
            self.info.regs().baud().modify(|w| w.set_rdmae(true));

            // Enable DMA channel request
            rx_dma.enable_request();
        }

        // Create guard that will abort DMA if this future is dropped
        let guard = RxDmaGuard::new(rx_dma.reborrow(), self.info);

        // Wait for completion asynchronously
        core::future::poll_fn(|cx| {
            guard.dma.waker().register(cx.waker());
            if guard.dma.is_done() {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        })
        .await;

        // Transfer completed successfully - clean up without aborting
        guard.complete();

        Ok(len)
    }

    /// Blocking read (fallback when DMA is not needed)
    pub fn blocking_read(&mut self, buf: &mut [u8]) -> Result<()> {
        for byte in buf.iter_mut() {
            loop {
                if has_data(self.info) {
                    *byte = (self.info.regs().data().read().0 & 0xFF) as u8;
                    break;
                }
                check_and_clear_rx_errors(self.info)?;
            }
        }
        Ok(())
    }

    pub fn into_ring_dma_rx<'buf: 'a>(&mut self, buf: &'buf mut [u8]) -> RingBufferedLpuartRx<'_, 'buf> {
        let ring = self.setup_ring_buffer(buf);
        unsafe { ring.enable_dma_request() };
        RingBufferedLpuartRx { ring }
    }

    /// Set up a ring buffer for continuous DMA reception.
    ///
    /// This configures the DMA channel for circular operation, enabling continuous
    /// reception of data without gaps. The DMA will continuously write received
    /// bytes into the buffer, wrapping around when it reaches the end.
    ///
    /// This method encapsulates all the low-level setup:
    /// - Configures the DMA request source for this LPUART instance
    /// - Enables the RX DMA request in the LPUART peripheral
    /// - Sets up the circular DMA transfer
    /// - Enables the NVIC interrupt for async wakeups
    ///
    /// # Arguments
    ///
    /// * `buf` - Destination buffer for received data (power-of-2 size is ideal for efficiency)
    ///
    /// # Returns
    ///
    /// A [`RingBuffer`] that can be used to asynchronously read received data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// static mut RX_BUF: [u8; 64] = [0; 64];
    ///
    /// let rx = LpuartRxDma::new(p.LPUART2, p.P2_3, p.DMA_CH0, config).unwrap();
    /// let ring_buf = unsafe { rx.setup_ring_buffer(&mut RX_BUF) };
    ///
    /// // Read data as it arrives
    /// let mut buf = [0u8; 16];
    /// let n = ring_buf.read(&mut buf).await.unwrap();
    /// ```
    fn setup_ring_buffer<'buf: 'a>(&mut self, buf: &'buf mut [u8]) -> RingBuffer<'_, 'buf, u8> {
        // Get the peripheral data register address
        let peri_addr = self.info.regs().data().as_ptr() as *const u8;

        // Configure DMA request source for this LPUART instance (type-safe)
        unsafe {
            self.mode.dma.set_request_source(self.mode.request_number);
        }

        // Enable RX DMA request in the LPUART peripheral
        self.info.regs().baud().modify(|w| w.set_rdmae(true));

        // Set up circular DMA transfer (this also enables NVIC interrupt)
        unsafe { self.mode.dma.setup_circular_read(peri_addr, buf) }
    }
}

impl<'peri, 'buf> RingBufferedLpuartRx<'peri, 'buf> {
    /// Read from the ring buffer
    pub fn read<'d>(
        &mut self,
        dst: &'d mut [u8],
    ) -> impl Future<Output = core::result::Result<usize, crate::dma::Error>> + use<'_, 'buf, 'd> {
        self.ring.read(dst)
    }

    /// Clear the current contents of the ring buffer
    pub fn clear(&mut self) {
        self.ring.clear();
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
    ) -> Result<Self> {
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
                    request_number: T::TxDmaRequest::REQUEST_NUMBER,
                },
                wg.clone(),
            ),
            rx: LpuartRx::new_inner::<T>(
                rx_pin,
                None,
                Dma {
                    dma: rx_dma,
                    request_number: T::RxDmaRequest::REQUEST_NUMBER,
                },
                wg,
            ),
        })
    }

    /// Write data using DMA
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.tx.write(buf).await
    }

    /// Read data using DMA
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.rx.read(buf).await
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
        // No-op, when DMA transfer is completed, it is also flushed.
        Ok(())
    }
}

impl<'a> embedded_io_async::Read for Lpuart<'a, Dma<'a>> {
    async fn read(&mut self, buf: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        self.rx.read(buf).await
    }
}

impl<'a> embedded_io_async::Write for Lpuart<'a, Dma<'a>> {
    async fn write(&mut self, buf: &[u8]) -> core::result::Result<usize, Self::Error> {
        self.tx.write(buf).await
    }

    async fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        // No-op, when DMA transfer is completed, it is also flushed.
        self.tx.flush().await
    }
}
