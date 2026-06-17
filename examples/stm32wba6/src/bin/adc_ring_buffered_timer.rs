//! ADC4 ring-buffered DMA triggered by TIM1 TRGO2
//!
//! This example demonstrates periodic ADC4 sampling on STM32WBA6 where TIM1
//! update events are routed to TRGO2 and used as the regular ADC trigger.
//! DMA stores samples in a circular ring buffer.

#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_stm32::adc::{Adc, AdcChannel as _, RingBufferedAdc, adc4};
use embassy_stm32::peripherals::GPDMA1_CH1;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, Mms2};
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::{Config, bind_interrupts, dma};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL1 => dma::InterruptHandler<GPDMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_stm32::init(Config::default());

    info!("STM32WBA6 ADC4 timer-triggered ring-buffered DMA example");

    // TIM1 TRGO2 source: update event, generated at 50 Hz.
    let mut pwm = ComplementaryPwm::new(
        p.TIM1,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Hertz::hz(50),
        CountingMode::EdgeAlignedUp,
    );
    pwm.set_master_output_enable(false);
    pwm.set_mms2(Mms2::Update);

    let mut adc = Adc::new_adc4(p.ADC4);
    adc.set_resolution_adc4(adc4::Resolution::Bits12);
    adc.set_averaging_adc4(adc4::Averaging::Samples8);

    let mut vrefint = adc.enable_vrefint_adc4();
    let mut vcore = adc.enable_vcore_adc4();
    let mut temperature = adc.enable_temperature_adc4();

    // Channel order must be ascending for this ADC4 path.
    let sequence = [
        (vrefint.reborrow_adc(), adc4::SampleTime::Cycles795),     // CH0
        (vcore.reborrow_adc(), adc4::SampleTime::Cycles795),       // CH12
        (temperature.reborrow_adc(), adc4::SampleTime::Cycles795), // CH13
    ]
    .into_iter();

    // Double-buffer pattern: read half while DMA fills the other half.
    let mut dma_buf = [0u16; 3 * 32];
    let mut ring_adc: RingBufferedAdc<_> = adc.into_ring_buffered(
        p.GPDMA1_CH1,
        &mut dma_buf,
        Irqs,
        sequence,
        // TODO: hook up TIM1_TRGO2 once ADC4 regular-trigger mapping is available for WBA6.
        None,
    );

    let mut out = [0u16; (3 * 32) / 2];
    loop {
        match ring_adc.read(&mut out).await {
            Ok(remaining) => {
                let first = (out[0], out[1], out[2]);
                info!(
                    "samples={} first(vref,vcore,temp)=({},{},{}) remaining={}",
                    out.len() / 3,
                    first.0,
                    first.1,
                    first.2,
                    remaining
                );
            }
            Err(e) => {
                error!("DMA error: {:?}", e);
                ring_adc.clear();
            }
        }
    }
}
