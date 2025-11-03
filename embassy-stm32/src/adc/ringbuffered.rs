use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};

#[allow(unused_imports)]
use embassy_hal_internal::Peri;

#[allow(unused_imports)]
use crate::adc::{Instance, RxDma};
#[allow(unused_imports)]
use crate::dma::{ReadableRingBuffer, TransferOptions};
use crate::rcc;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OverrunError;

#[cfg(adc_v2)]
fn clear_interrupt_flags(r: crate::pac::adc::Adc) {
    r.sr().modify(|regs| {
        regs.set_eoc(false);
        regs.set_ovr(false);
    });
}

pub struct RingBufferedAdc<'d, T: Instance> {
    pub _phantom: PhantomData<T>,
    pub ring_buf: ReadableRingBuffer<'d, u16>,
}

impl<'d, T: Instance> RingBufferedAdc<'d, T> {
    #[cfg(not(adc_v2))]
    pub(crate) fn new(dma: Peri<'d, impl RxDma<T>>, dma_buf: &'d mut [u16]) -> Self {
        //dma side setup
        let opts = TransferOptions {
            half_transfer_ir: true,
            circular: true,
            ..Default::default()
        };

        // Safety: we forget the struct before this function returns.
        let request = dma.request();

        let ring_buf =
            unsafe { ReadableRingBuffer::new(dma, request, T::regs().dr().as_ptr() as *mut u16, dma_buf, opts) };

        Self {
            _phantom: PhantomData,
            ring_buf,
        }
    }

    #[cfg(adc_v2)]
    fn is_on() -> bool {
        T::regs().cr2().read().adon()
    }

    #[cfg(adc_v2)]
    /// Turns on ADC if it is not already turned on and starts continuous DMA transfer.
    pub fn start(&mut self) -> Result<(), OverrunError> {
        self.setup_adc();
        self.ring_buf.clear();

        Ok(())
    }

    #[cfg(adc_v2)]
    fn stop(&mut self, err: OverrunError) -> Result<usize, OverrunError> {
        self.teardown_adc();
        Err(err)
    }

    #[cfg(adc_v2)]
    /// Stops DMA transfer.
    /// It does not turn off ADC.
    /// Calling `start` restarts continuous DMA transfer.
    ///
    /// [`start`]: #method.start
    pub fn teardown_adc(&mut self) {
        // Stop the DMA transfer
        self.ring_buf.request_pause();

        let r = T::regs();

        // Stop ADC
        r.cr2().modify(|reg| {
            // Stop ADC
            reg.set_swstart(false);
            // Stop DMA
            reg.set_dma(false);
        });

        r.cr1().modify(|w| {
            // Disable interrupt for end of conversion
            w.set_eocie(false);
            // Disable interrupt for overrun
            w.set_ovrie(false);
        });

        clear_interrupt_flags(r);

        compiler_fence(Ordering::SeqCst);
    }

    #[cfg(adc_v2)]
    fn setup_adc(&mut self) {
        use crate::pac::adc::vals;

        compiler_fence(Ordering::SeqCst);

        self.ring_buf.start();

        let r = T::regs();

        // Enable ADC
        let was_on = Self::is_on();
        if !was_on {
            r.cr2().modify(|reg| {
                reg.set_adon(false);
                reg.set_swstart(false);
            });
        }

        // Clear all interrupts
        r.sr().modify(|regs| {
            regs.set_eoc(false);
            regs.set_ovr(false);
            regs.set_strt(false);
        });

        r.cr1().modify(|w| {
            // Enable interrupt for end of conversion
            w.set_eocie(true);
            // Enable interrupt for overrun
            w.set_ovrie(true);
            // Scanning converisons of multiple channels
            w.set_scan(true);
            // Continuous conversion mode
            w.set_discen(false);
        });

        r.cr2().modify(|w| {
            // Enable DMA mode
            w.set_dma(true);
            // Enable continuous conversions
            w.set_cont(true);
            // DMA requests are issues as long as DMA=1 and data are converted.
            w.set_dds(vals::Dds::CONTINUOUS);
            // EOC flag is set at the end of each conversion.
            w.set_eocs(vals::Eocs::EACH_CONVERSION);
        });

        // Begin ADC conversions
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
            reg.set_swstart(true);
        });

        super::blocking_delay_us(3);
    }

    #[cfg(any(adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0))]
    #[inline]
    fn start_continuous_sampling(&mut self) {
        // Start adc conversion
        T::regs().cr().modify(|reg| {
            reg.set_adstart(true);
        });
        self.ring_buf.start();
    }

    #[cfg(any(adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0))]
    #[inline]
    pub fn stop_continuous_sampling(&mut self) {
        // Stop adc conversion
        if T::regs().cr().read().adstart() && !T::regs().cr().read().addis() {
            T::regs().cr().modify(|reg| {
                reg.set_adstp(true);
            });
            while T::regs().cr().read().adstart() {}
        }
    }

    #[cfg(any(adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0))]
    pub fn disable_adc(&mut self) {
        self.stop_continuous_sampling();
        self.ring_buf.clear();
        self.ring_buf.request_pause();
    }

    #[cfg(any(adc_v3, adc_g0, adc_h5, adc_h7rs, adc_u0))]
    pub fn teardown_adc(&mut self) {
        self.disable_adc();

        //disable dma control
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().cfgr().modify(|reg| {
            reg.set_dmaen(false);
        });
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().cfgr1().modify(|reg| {
            reg.set_dmaen(false);
        });

        //TODO: do we need to cleanup the DMA request here?

        compiler_fence(Ordering::SeqCst);
    }

    /// Reads measurements from the DMA ring buffer.
    ///
    /// This method fills the provided `measurements` array with ADC readings from the DMA buffer.
    /// The length of the `measurements` array should be exactly half of the DMA buffer length. Because interrupts are only generated if half or full DMA transfer completes.
    ///
    /// Each call to `read` will populate the `measurements` array in the same order as the channels defined with `sequence`.
    /// There will be many sequences worth of measurements in this array because it only returns if at least half of the DMA buffer is filled.
    /// For example if 2 channels are sampled `measurements` contain: `[sq0 sq1 sq0 sq1 sq0 sq1 ..]`.
    ///
    /// Note that the ADC Datarate can be very fast, it is suggested to use DMA mode inside tightly running tasks
    /// Otherwise, you'll see constant Overrun errors occuring, this means that you're sampling too quickly for the task to handle, and you may need to increase the buffer size.
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

        let r = T::regs();

        #[cfg(adc_v2)]
        {
            // Start background receive if it was not already started
            if !r.cr2().read().dma() {
                self.start()?;
            }
        }

        #[cfg(not(adc_v2))]
        {
            // Clear overrun flag if set.
            if r.sr().read().ovr() {
                return self.stop(OverrunError);
            }

            // Start background receive if it was not already started
            if !r.cr().read().adstart() {
                self.start_continuous_sampling();
            }
        }

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
        let r = T::regs();

        #[cfg(adc_v2)]
        {
            // Start background receive if it was not already started
            if !r.cr2().read().dma() {
                self.start()?;
            }

            // Clear overrun flag if set.
            if r.sr().read().ovr() {
                return self.stop(OverrunError);
            }

            loop {
                match self.ring_buf.read(buf) {
                    Ok((0, _)) => {}
                    Ok((len, _)) => {
                        return Ok(len);
                    }
                    Err(_) => {
                        return self.stop(OverrunError);
                    }
                }
            }
        }

        #[cfg(not(adc_v2))]
        {
            // Start background receive if it was not already started
            if !r.cr().read().adstart() {
                self.start_continuous_sampling();
            }

            loop {
                match self.ring_buf.read(buf) {
                    Ok((0, _)) => {}
                    Ok((len, _)) => {
                        return Ok(len);
                    }
                    Err(_) => {
                        self.stop_continuous_sampling();
                        return Err(OverrunError);
                    }
                }
            }
        }
    }
}

impl<T: Instance> Drop for RingBufferedAdc<'_, T> {
    fn drop(&mut self) {
        self.teardown_adc();
        rcc::disable::<T>();
    }
}
