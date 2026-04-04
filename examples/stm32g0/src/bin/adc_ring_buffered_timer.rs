//! ADC with DMA ring buffer triggered by timer
//!
//! This example demonstrates periodic ADC sampling using a timer trigger and DMA ring buffer.
//! The timer generates regular ADC conversions at a controlled rate, and DMA automatically
//! stores the samples in a circular buffer for efficient data acquisition.
//!
//! Hardware setup:
//! - PA0: ADC input (connect to analog signal)
//! - Internal VrefInt and Temperature sensors also monitored

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel as _, Clock, Presc, RegularAdcTrigger, SampleTime};
use embassy_stm32::pac::adc::vals::Exten;
use embassy_stm32::peripherals::DMA1_CH1;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, Mms2};
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::triggers::TIM1_TRGO2;
use embassy_stm32::{bind_interrupts, dma};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Default::default();
    let mut p = embassy_stm32::init(config);

    info!("ADC Ring Buffer with Timer Trigger example for STM32G0");

    // Configure TIM1 to generate TRGO2 events at 10 Hz
    // This will trigger ADC conversions periodically
    let tim1 = p.TIM1;
    let mut pwm = ComplementaryPwm::new(
        tim1,
        None,          // CH1
        None,          // CH1N
        None,          // CH2
        None,          // CH2N
        None,          // CH3
        None,          // CH3N
        None,          // CH4
        None,          // CH4N
        Hertz::hz(10), // 10 Hz trigger rate
        CountingMode::EdgeAlignedUp,
    );

    // Configure TRGO2 to trigger on update event
    pwm.set_mms2(Mms2::UPDATE);

    // Configure ADC with DMA ring buffer
    let adc = Adc::new_with_clock(p.ADC1, Clock::Async { div: Presc::DIV1 });

    // Setup channels to measure
    let mut vrefint = adc.enable_vrefint();
    let mut temperature = adc.enable_temperature();
    let vrefint_channel = vrefint.degrade_adc();
    let temp_channel = temperature.degrade_adc();
    let pa0 = p.PA0.degrade_adc();

    let sequence = [
        (vrefint_channel, SampleTime::CYCLES79_5),
        (temp_channel, SampleTime::CYCLES79_5),
        (pa0, SampleTime::CYCLES79_5),
    ]
    .into_iter();

    // DMA buffer - using double size allows reading one half while DMA fills the other
    // Buffer holds continuous samples
    let mut dma_buf = [0u16; 12]; // 4 complete samples (3 channels each)

    // Create ring-buffered ADC with timer trigger
    let mut ring_buffered_adc = adc.into_ring_buffered(
        p.DMA1_CH1,
        &mut dma_buf,
        Irqs,
        sequence,
        RegularAdcTrigger::from(TIM1_TRGO2, Exten::RISING_EDGE), // Timer 1 TRGO2 as trigger source and Trigger on rising edge (can also use FALLING_EDGE or BOTH_EDGES)
    );

    // Start ADC conversions and DMA transfer
    ring_buffered_adc.start();

    info!("ADC configured with TIM1 trigger at 10 Hz");
    info!("Reading 3 channels: VrefInt, Temperature, PA0");

    // Buffer to read samples - must be half the size of dma_buf
    let mut data = [0u16; 6]; // 2 complete samples

    loop {
        match ring_buffered_adc.read(&mut data).await {
            Ok(remaining) => {
                // Data contains interleaved samples: [vref0, temp0, pa0_0, vref1, temp1, pa0_1]
                info!("Sample 1: VrefInt={}, Temp={}, PA0={}", data[0], data[1], data[2]);
                info!("Sample 2: VrefInt={}, Temp={}, PA0={}", data[3], data[4], data[5]);
                info!("Remaining samples in buffer: {}", remaining);

                // Convert VrefInt to millivolts (example calculation)
                let vdda_mv = (3300 * 1210) / data[0] as u32; // Assuming VREFINT_CAL = 1210
                info!("Estimated VDDA: {} mV", vdda_mv);
            }
            Err(e) => {
                error!("DMA error: {:?}", e);
                ring_buffered_adc.clear();
            }
        }
    }
}
