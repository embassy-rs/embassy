use core::sync::atomic::{Ordering, compiler_fence};

use embassy_hal_internal::Peri;

use super::AdcRegs;
use crate::adc::{Instance, RxDma, check_dma_len};
use crate::dma::ChannelAndRequest;
use crate::rcc::RccInfo;

/// An ADC with a pre-configured channel sequence for repeated DMA reads.
///
/// Unlike [`Adc::read`], this type programs the ADC channel sequence registers
/// only once at construction. Each call to [`read`](ConfiguredSequence::read)
/// reuses the existing hardware sequence configuration, avoiding the per-call
/// overhead of reprogramming the sequence registers.
///
/// Obtain via [`Adc::configured_sequence`].
#[allow(private_bounds)]
pub struct ConfiguredSequence<'adc, R: AdcRegs> {
    regs: R,
    info: RccInfo,
    len: usize,
    dma: ChannelAndRequest<'adc>,
}

#[allow(private_bounds)]
impl<'adc, R: AdcRegs> ConfiguredSequence<'adc, R> {
    pub(crate) fn new<'d, T: Instance<Regs = R>, D: RxDma<T>>(
        _adc: &'adc mut super::Adc<'d, T>,
        rx_dma: Peri<'adc, D>,
        len: usize,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'adc,
    ) -> Self {
        Self {
            regs: T::regs(),
            info: T::RCC_INFO,
            len,
            dma: new_dma!(rx_dma, irq).unwrap(),
        }
    }

    /// Trigger one DMA conversion of the pre-configured channel sequence and
    /// wait for it to complete.
    ///
    /// Returns a slice over the results in the same channel order as the
    /// sequence passed to [`Adc::configured_sequence`].
    ///
    /// The ADC and DMA are configured once at construction by
    /// [`Adc::configured_sequence`]. The hardware is configured so that
    /// DMA stays armed between calls while the ADC runs only one sequence per
    /// [`start`](AdcRegs::start) call.
    pub async fn read(&mut self, buf: &mut [u16]) {
        let _scoped_wake_guard = self.info.wake_guard();

        check_dma_len(self.len, Some(buf.len()), true);

        let transfer = unsafe { self.dma.read(self.regs.data(), buf, Default::default()) };

        self.regs.start();
        transfer.await;
    }
}

impl<R: AdcRegs> Drop for ConfiguredSequence<'_, R> {
    fn drop(&mut self) {
        self.regs.stop(false);
        compiler_fence(Ordering::SeqCst);
    }
}
