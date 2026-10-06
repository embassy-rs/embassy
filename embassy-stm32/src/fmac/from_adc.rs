use dsp_fixedpoint::Q16;

use crate::adc::{self, Adc, BorrowedAdcChannel, ConfiguredTransfer, RegularAdcTrigger, RxDma, SampleTimeOf};
use crate::fmac::{self, Fmac};
use crate::mode::Mode;

/// A type used to bind ADC to FMAC using DMA
pub struct FromAdc<'d, FMAC: fmac::Instance, ADC: adc::Instance> {
    #[allow(unused)]
    transfer: ConfiguredTransfer<'d, ADC::Regs>,
    #[allow(unused)]
    fmac: &'d mut Fmac<'d, FMAC>,
}

impl<'d, ADC: adc::Instance, FMAC: fmac::Instance> FromAdc<'d, FMAC, ADC> {
    #[allow(unused)]
    /// Bind ADC to FMAC using DMA and start conversion
    pub fn new<'ch, 'a, D: RxDma<ADC>, M: Mode>(
        fmac: &'d mut Fmac<'d, FMAC>,
        adc: &'d mut Adc<'a, ADC, M>,
        sequence: impl ExactSizeIterator<Item = (BorrowedAdcChannel<'ch, ADC>, SampleTimeOf<ADC>)>,
        trigger: RegularAdcTrigger<ADC>,
        dma_ch: embassy_hal_internal::Peri<'d, D>,
        irq: impl crate::interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'a,
    ) -> Self
    where
        'ch: 'd,
    {
        Self {
            fmac,
            transfer: unsafe { adc.configure_transfer(dma_ch, irq, sequence, trigger, FMAC::wdata()) },
        }
    }

    /// Read output value
    pub fn read(&mut self) -> Option<Q16<15>> {
        self.fmac.read()
    }
}
