use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_futures::yield_now;
use embassy_hal_internal::into_ref;

use super::{Adc, RxDma};
use crate::adc::Instance;
use crate::dma::ringbuffer::OverrunError;
use crate::dma::ReadableRingBuffer;
use crate::pac::adc::vals;

fn clear_interrupt_flags(r: crate::pac::adc::Adc) {
    r.sr().modify(|regs| {
        regs.set_eoc(false);
        regs.set_ovr(false);
    });
}

pub struct RingBufferedAdc<'d, T: Instance> {
    _phantom: PhantomData<T>,
    // rx_dma: PeripheralRef<'d, S>,
    ring_buf: ReadableRingBuffer<'d, u16>,
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn into_ring_buffered(self, rx_dma: impl RxDma<T>, dma_buf: &'d mut [u16]) -> RingBufferedAdc<'d, T> {
        assert!(!dma_buf.is_empty() && dma_buf.len() <= 0xFFFF);
        into_ref!(rx_dma);
        let opts = Default::default();

        let rx_src = T::regs().dr().as_ptr() as *mut u16;
        // Safety: we forget the struct before this function returns.
        let request = rx_dma.request();

        let ring_buf = unsafe { ReadableRingBuffer::new(rx_dma, request, rx_src, dma_buf, opts) };

        // Don't disable the clock
        mem::forget(self);

        RingBufferedAdc {
            _phantom: PhantomData,
            ring_buf,
        }
    }
}

impl<'d, T: Instance> RingBufferedAdc<'d, T> {
    fn is_on() -> bool {
        T::regs().cr2().read().adon()
    }

    pub fn start(&mut self) -> Result<(), OverrunError> {
        self.ring_buf.clear();

        self.setup_adc();

        Ok(())
    }

    fn stop(&mut self, err: OverrunError) -> Result<usize, OverrunError> {
        self.teardown_adc();
        Err(err)
    }

    pub fn teardown_adc(&mut self) {
        self.ring_buf.request_stop();

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

    fn setup_adc(&mut self) {
        compiler_fence(Ordering::SeqCst);

        self.ring_buf.start();

        let r = T::regs();

        //Enable ADC
        let was_on = Self::is_on();
        if !was_on {
            r.cr2().modify(|reg| {
                reg.set_adon(true);
                reg.set_swstart(false);
            });
        }

        // Clear all interrupts
        r.sr().modify(|regs| {
            regs.set_eoc(false);
            regs.set_ovr(false);
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
            w.set_cont(vals::Cont::CONTINUOUS);
            // DMA requests are issues as long as DMA=1 and data are converted.
            w.set_dds(vals::Dds::CONTINUOUS);
            // EOC flag is set at the end of each conversion.
            w.set_eocs(vals::Eocs::EACHSEQUENCE);
        });

        //Being ADC conversions
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
            reg.set_swstart(true);
        });
    }

    /// Read bytes that are readily available in the ring buffer.
    /// If no bytes are currently available in the buffer the call waits until the some
    /// bytes are available (at least one byte and at most half the buffer size)
    ///
    /// Background receive is started if `start()` has not been previously called.
    ///
    /// Receive in the background is terminated if an error is returned.
    /// It must then manually be started again by calling `start()` or by re-calling `read()`.
    pub async fn read(&mut self, buf: &mut [u16]) -> Result<usize, OverrunError> {
        let r = T::regs();

        // Start background receive if it was not already started
        if !r.cr2().read().dma() {
            self.start()?;
        }

        // Clear overrun flag if set.
        if r.sr().read().ovr() {
            r.sr().modify(|regs| {
                regs.set_ovr(false);
                regs.set_eoc(false);
            });
            // return self.stop(OverrunError);
        }

        loop {
            match self.ring_buf.read(buf) {
                Ok((0, _)) => {
                    yield_now().await;
                }
                Ok((len, _)) => {
                    return Ok(len);
                }
                Err(_) => {
                    return self.stop(OverrunError);
                }
            }
        }
    }
}

impl<T: Instance> Drop for RingBufferedAdc<'_, T> {
    fn drop(&mut self) {
        self.teardown_adc();
        T::disable();
    }
}
