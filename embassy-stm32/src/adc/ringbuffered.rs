use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};

#[allow(unused_imports)]
use embassy_hal_internal::Peri;

use super::AdcRegs;
#[allow(unused_imports)]
use crate::adc::{Instance, RxDma};
use crate::dma::Channel;
#[allow(unused_imports)]
use crate::dma::{ReadableRingBuffer, TransferOptions};
use crate::rcc;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OverrunError;

pub struct RingBufferedAdc<'d, T: Instance> {
    _phantom: PhantomData<T>,
    ring_buf: ReadableRingBuffer<'d, u16>,
}

impl<'d, T: Instance> RingBufferedAdc<'d, T> {
    pub(crate) fn new<D: RxDma<T>>(
        dma: Peri<'d, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'd,
        dma_buf: &'d mut [u16],
    ) -> Self {
        // DMA side setup - configuration differs between DMA/BDMA and GPDMA
        // For DMA/BDMA: use circular mode via TransferOptions
        // For GPDMA: circular mode is achieved via linked-list ping-pong
        #[cfg(not(gpdma))]
        let opts = TransferOptions {
            half_transfer_ir: true,
            circular: true,
            ..Default::default()
        };

        #[cfg(gpdma)]
        let opts = TransferOptions {
            half_transfer_ir: true,
            ..Default::default()
        };

        // Safety: we forget the struct before this function returns.
        let request = dma.request();

        let ring_buf =
            unsafe { ReadableRingBuffer::new(Channel::new(dma, irq), request, T::regs().data(), dma_buf, opts) };

        Self {
            _phantom: PhantomData,
            ring_buf,
        }
    }

    /// Turns on ADC if it is not already turned on and starts continuous DMA transfer.
    pub fn start(&mut self) {
        compiler_fence(Ordering::SeqCst);
        self.ring_buf.start();

        T::regs().start();
    }

    pub fn stop(&mut self) {
        self.ring_buf.request_pause();

        compiler_fence(Ordering::SeqCst);
    }

    pub fn clear(&mut self) {
        self.ring_buf.clear();
    }

    /// Reads measurements from the DMA ring buffer.
    ///
    /// This method fills the provided `measurements` array with ADC readings from the DMA buffer.
    /// The length of the `measurements` array should be exactly half of the DMA buffer length.
    /// Because interrupts are only generated if half or full DMA transfer completes.
    ///
    /// Each call to `read` will populate the `measurements` array in the same order as the channels
    /// defined with `sequence`. There will be many sequences worth of measurements in this array
    /// because it only returns if at least half of the DMA buffer is filled. For example if 2
    /// channels are sampled `measurements` contain: `[sq0 sq1 sq0 sq1 sq0 sq1 ..]`.
    ///
    /// Note that the ADC Datarate can be very fast, it is suggested to use DMA mode inside tightly
    /// running tasks. Otherwise, you'll see constant Overrun errors occurring, this means that
    /// you're sampling too quickly for the task to handle, and you may need to increase the buffer size.
    /// Example:
    /// ```rust,ignore
    /// const DMA_BUF_LEN: usize = 120;
    /// use embassy_stm32::adc::{Adc, AdcChannel}
    ///
    /// let mut adc = Adc::new(p.ADC1);
    /// let mut adc_pin0 = p.PA0.degrade_adc();
    /// let mut adc_pin1 = p.PA1.degrade_adc();
    /// let adc_dma_buf = [0u16; DMA_BUF_LEN];
    ///
    /// let mut ring_buffered_adc: RingBufferedAdc<embassy_stm32::peripherals::ADC1> = adc.into_ring_buffered(
    ///     p.DMA2_CH0,
    ///      adc_dma_buf, [
    ///         (&mut *adc_pin0, SampleTime::CYCLES160_5),
    ///         (&mut *adc_pin1, SampleTime::CYCLES160_5),
    ///     ].into_iter());
    ///
    ///
    /// let mut measurements = [0u16; DMA_BUF_LEN / 2];
    /// loop {
    ///     match ring_buffered_adc.read(&mut measurements).await {
    ///         Ok(_) => {
    ///             defmt::info!("adc1: {}", measurements);
    ///         }
    ///         Err(e) => {
    ///             defmt::warn!("Error: {:?}", e);
    ///         }
    ///     }
    /// }
    /// ```
    ///
    ///
    /// [`teardown_adc`]: #method.teardown_adc
    /// [`start_continuous_sampling`]: #method.start_continuous_sampling
    pub async fn read(&mut self, measurements: &mut [u16]) -> Result<usize, OverrunError> {
        assert_eq!(
            self.ring_buf.capacity() / 2,
            measurements.len(),
            "Buffer size must be half the size of the ring buffer"
        );

        if !self.ring_buf.is_running() {
            self.start();
        }

        //        #[cfg(adc_v2)]
        //        {
        //            // Clear overrun flag if set.
        //            if T::regs().sr().read().ovr() {
        //                self.stop();
        //
        //                return Err(OverrunError);
        //            }
        //        }

        self.ring_buf.read_exact(measurements).await.map_err(|_| OverrunError)
    }

    /// Read bytes that are readily available in the ring buffer.
    /// If no bytes are currently available in the buffer the call waits until the some
    /// bytes are available (at least one byte and at most half the buffer size)
    ///
    /// Background receive is started if `start_continuous_sampling()` has not been previously called.
    ///
    /// Receive in the background is terminated if an error is returned.
    /// It must then manually be started again by calling `start_continuous_sampling()` or by re-calling `blocking_read()`.
    pub fn blocking_read(&mut self, buf: &mut [u16]) -> Result<usize, OverrunError> {
        if !self.ring_buf.is_running() {
            self.start();
        }

        //        #[cfg(adc_v2)]
        //        {
        //            // Clear overrun flag if set.
        //            if T::regs().sr().read().ovr() {
        //                self.stop();
        //
        //                return Err(OverrunError);
        //            }
        //        }

        loop {
            match self.ring_buf.read(buf) {
                Ok((0, _)) => {}
                Ok((len, _)) => {
                    return Ok(len);
                }
                Err(_) => {
                    self.ring_buf.request_pause();

                    return Err(OverrunError);
                }
            }
        }
    }
}

impl<T: Instance> Drop for RingBufferedAdc<'_, T> {
    fn drop(&mut self) {
        T::regs().stop();

        compiler_fence(Ordering::SeqCst);

        self.ring_buf.request_pause();
        rcc::disable::<T>();
    }
}
