use embassy_hal_internal::Peri as DmaPeri;

use crate::adc::{self, Adc, AdcChannel, AdcRegs, BasicAdcRegs, ConversionMode, RegularAdcTrigger, RxDma};
use crate::dma::TransferOptions;
use crate::fmac::{self, Fmac};
use crate::{dma, interrupt};

pub struct FromAdc<'df, 'da, 'dd, FMAC: fmac::Instance, ADC: adc::Instance> {
    fmac: Fmac<'df, FMAC>,
    adc: Adc<'da, ADC>,
    dma_ch: dma::Channel<'dd>,
    transfer: dma::Transfer<'dd>,
}

impl<'df, 'da, 'dd, ADC: adc::Instance, FMAC: fmac::Instance> FromAdc<'df, 'da, 'dd, FMAC, ADC> {
    /// Bind ADC to FMAC using DMA and start conversion
    pub fn new<'di: 'dd, D: RxDma<ADC>, Irq>(
        fmac: Fmac<'df, FMAC>,
        adc: Adc<'da, ADC>,
        adc_ch: impl AdcChannel<ADC>,
        sample_time: <ADC::Regs as BasicAdcRegs>::SampleTime,
        trigger: RegularAdcTrigger<ADC>,
        dma_ch: DmaPeri<'dd, D>,
        irq_not_used: impl interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>> + 'di,
    ) -> Self {
        {
            // Ensure no conversions are ongoing
            ADC::regs().stop(false);
            ADC::regs().configure_sequence([((adc_ch.channel(), adc_ch.is_differential()), sample_time)].into_iter());

            ADC::regs().enable();

            // Configure DMA once, reused across all subsequent read() calls.
            ADC::regs().configure_dma(ConversionMode::Repeated(Some((trigger._trigger, trigger._edge))));
        }

        let dma_request = dma_ch.request();
        let mut dma_channel = dma::Channel::new(dma_ch, irq_not_used);
        let transfer = unsafe {
            dma_channel.read_repeated(
                dma_request,
                ADC::regs().data(),
                FMAC::wdata(),
                TransferOptions {
                    priority: dma::Priority::VeryHigh,
                    circular: true,
                    half_transfer_ir: false,
                    complete_transfer_ir: false,
                },
            )
        };

        ADC::regs().start();

        Self {
            fmac,
            adc,
            dma_ch: dma_channel,
            transfer, // <--- this borrows `dma_channel` which is moved :/
        }
    }
}
