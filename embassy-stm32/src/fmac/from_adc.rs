use dsp_fixedpoint::Q16;
use stm32_metapac::adc::vals::SampleTime;

use crate::adc::{self, Adc, BorrowedAdcChannel, ConfiguredTransfer, RegularAdcTrigger, RxDma};
use crate::fmac::{self, Fmac};

/// A type used to bind ADC to FMAC using DMA
pub struct FromAdc<'d, FMAC: fmac::Instance, ADC: adc::DefaultInstance> {
    #[allow(unused)]
    transfer: ConfiguredTransfer<'d, ADC::Regs>,
    #[allow(unused)]
    fmac: Fmac<'d, FMAC>,
}

impl<'d, ADC: adc::DefaultInstance, FMAC: fmac::Instance> FromAdc<'d, FMAC, ADC> {
    #[allow(unused)]
    /// Bind ADC to FMAC using DMA and start conversion
    pub fn new<'ch, 'a, D: RxDma<ADC>>(
        fmac: Fmac<'d, FMAC>,
        adc: &'d mut Adc<'a, ADC>,
        sequence: impl ExactSizeIterator<Item = (BorrowedAdcChannel<'ch, ADC>, SampleTime)>,
        trigger: RegularAdcTrigger<ADC>,
        dma_ch: embassy_hal_internal::Peri<'d, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
    ) -> Self
    where
        'ch: 'd,
    {
        Self {
            fmac,
            transfer: adc.configured_transfer(dma_ch, irq, sequence, trigger, FMAC::wdata()),
        }
    }

    /// Read output value
    pub fn read(&mut self) -> Option<Q16<15>> {
        self.fmac.read()
    }
}
