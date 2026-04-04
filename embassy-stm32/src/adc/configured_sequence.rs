use core::sync::atomic::{Ordering, compiler_fence};

use embassy_hal_internal::Peri;

use super::AdcRegs;
use crate::adc::{Instance, RxDma};

/// An ADC with a pre-configured channel sequence for repeated DMA reads.
///
/// Unlike [`Adc::read`], this type programs the ADC channel sequence registers
/// only once at construction. Each call to [`read`](ConfiguredSequence::read)
/// reuses the existing hardware sequence configuration, avoiding the per-call
/// overhead of reprogramming the sequence registers.
///
/// Obtain via [`Adc::configured_sequence`].
pub struct ConfiguredSequence<'adc, 'd, T: Instance, D: RxDma<T>> {
    _adc: &'adc mut super::Adc<'d, T>,
    rx_dma: Peri<'adc, D>,
    buf: &'adc mut [u16],
}

impl<'adc, 'd, T: Instance, D: RxDma<T>> ConfiguredSequence<'adc, 'd, T, D> {
    pub(crate) fn new(adc: &'adc mut super::Adc<'d, T>, rx_dma: Peri<'adc, D>, buf: &'adc mut [u16]) -> Self {
        Self { _adc: adc, rx_dma, buf }
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
    pub async fn read(
        &mut self,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>,
    ) -> &[u16] {
        let _scoped_wake_guard = <T as crate::rcc::SealedRccPeripheral>::RCC_INFO.wake_guard();

        let request = self.rx_dma.request();
        let mut dma_channel = crate::dma::Channel::new(self.rx_dma.reborrow(), irq);
        let transfer = unsafe { dma_channel.read(request, T::regs().data(), self.buf, Default::default()) };

        T::regs().start();
        transfer.await;

        &self.buf[..]
    }
}

impl<T: Instance, D: RxDma<T>> Drop for ConfiguredSequence<'_, '_, T, D> {
    fn drop(&mut self) {
        T::regs().stop();
        compiler_fence(Ordering::SeqCst);
    }
}
