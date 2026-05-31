use embassy_hal_internal::Peri as DmaPeri;

use crate::adc::{self, Adc, AnyAdcChannel, BasicAdcRegs, ConfiguredTransfer, RegularAdcTrigger, RxDma};
use crate::fmac::{self, Fmac};
use crate::interrupt;

/// A type used to bind ADC to FMAC using DMA
pub struct FromAdc<'d, FMAC: fmac::Instance, ADC: adc::DefaultInstance> {
    #[allow(unused)]
    transfer: ConfiguredTransfer<'d, ADC::Regs>,
    #[allow(unused)]
    fmac: Fmac<'d, FMAC>,
}

impl<'d, ADC: adc::DefaultInstance, FMAC: fmac::Instance> FromAdc<'d, FMAC, ADC> {
    /// Bind ADC to FMAC using DMA and start conversion
    pub fn new<'di: 'd, D: RxDma<ADC>, Irq>(
        fmac: Fmac<'d, FMAC>,
        adc: &'d mut Adc<'d, ADC>,
        adc_ch: &'d mut AnyAdcChannel<ADC>,
        sample_time: <ADC::Regs as BasicAdcRegs>::SampleTime,
        trigger: RegularAdcTrigger<ADC>,
        dma_ch: DmaPeri<'d, D>,
        irq_not_used: impl interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'di,
    ) -> Self {
        let transfer = adc.configured_transfer(
            [(adc_ch, sample_time)].into_iter(),
            trigger,
            dma_ch,
            FMAC::wdata(),
            irq_not_used,
        );

        Self { fmac, transfer }
    }
}
