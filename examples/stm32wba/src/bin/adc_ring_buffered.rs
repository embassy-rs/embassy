//! ADC Ring Buffered Example - True Circular DMA for ADC4 Internal Channels
//!
//! This example demonstrates continuous ADC sampling using circular DMA with
//! GPDMA linked-list mode. The ADC continuously samples internal channels
//! (VREFINT, VCORE, Temperature) at 5000 samples/sec per channel.
//!
//! Temperature is calculated using factory calibration values (TS_CAL1, TS_CAL2)
//! from the DESIG peripheral for accurate readings.
//!
//! Sample rate calculation (48 MHz ADC clock):
//! - CYCLES12_5 sample time + 12.5 conversion = 25 cycles per raw sample
//! - Samples128 averaging: 128 × 25 = 3200 cycles per averaged result
//! - 3 channels: 3200 × 3 = 9600 cycles per complete set
//! - Rate: 48 MHz / 9600 = 5000 sets/sec

#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::adc::adc4::Calibration;
use embassy_stm32::adc::{Adc, AdcChannel, RegularConversionMode, RingBufferedAdc, adc4};
use embassy_stm32::peripherals::{ADC4, GPDMA1_CH1};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::{Config, bind_interrupts, dma};
use {defmt_rtt as _, panic_probe as _};

// DMA buffer size - must be large enough to prevent overruns
// Buffer holds: [vrefint, vcore, temp, vrefint, vcore, temp, ...]
// Size should be a multiple of number of channels (3) and large enough for processing
const DMA_BUF_LEN: usize = 3 * 256; // 256 samples per channel

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL1 => dma::InterruptHandler<GPDMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    // Configure RCC with PLL1 - required for ADC4 clock
    let mut config = Config::default();
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,  // PLLM = 1 → HSI / 1 = 16 MHz
        mul: PllMul::MUL30,       // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
        divr: Some(PllDiv::DIV5), // PLLR = 5 → 96 MHz (Sysclk)
        divq: None,
        divp: Some(PllDiv::DIV30), // PLLP = 30 → 16 MHz (ADC4 clock source)
        frac: Some(0),
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    config.rcc.sys = Sysclk::PLL1_R;

    let p = embassy_stm32::init(config);

    info!("STM32WBA ADC4 Ring Buffered Example - Circular DMA @ 5000 samples/sec with Calibrated Temperature");

    // Read factory calibration values
    let calibration = Calibration::read();
    info!(
        "Calibration values: TS_CAL1={} (30C), TS_CAL2={} (130C), VREFINT_CAL={}",
        calibration.ts_cal1, calibration.ts_cal2, calibration.vrefint_cal
    );

    // Initialize ADC4 with appropriate settings
    // Samples128 averaging with CYCLES12_5 = 5000 samples/sec per channel
    let mut adc = Adc::new_adc4(p.ADC4);
    adc.set_resolution_adc4(adc4::Resolution::BITS12);
    adc.set_averaging_adc4(adc4::Averaging::Samples128);

    let max_count = adc4::resolution_to_max_count(adc4::Resolution::BITS12);

    // Enable internal channels
    let vrefint = adc.enable_vrefint_adc4();
    let temperature = adc.enable_temperature_adc4();
    let vcore = adc.enable_vcore_adc4();

    // Degrade to AnyAdcChannel for use with DMA
    // IMPORTANT: Order matters for ADC4 - must be ascending channel numbers
    // VrefInt: Channel 0, VCORE: Channel 12, Temperature: Channel 13
    let vrefint_ch = vrefint.degrade_adc();
    let vcore_ch = vcore.degrade_adc();
    let temp_ch = temperature.degrade_adc();

    info!("Internal channels enabled, setting up ring buffer...");

    // Create DMA buffer - must be static for ring-buffered operation
    static mut DMA_BUF: [u16; DMA_BUF_LEN] = [0u16; DMA_BUF_LEN];

    // Create the ring-buffered ADC with continuous mode
    // Channels must be in ascending order for ADC4
    // CYCLES12_5 + Samples128 averaging = 5000 samples/sec per channel
    let mut ring_adc: RingBufferedAdc<ADC4> = adc.into_ring_buffered(
        p.GPDMA1_CH1,
        unsafe { &mut *core::ptr::addr_of_mut!(DMA_BUF) },
        Irqs,
        [
            (vrefint_ch, adc4::SampleTime::CYCLES12_5), // Channel 0
            (vcore_ch, adc4::SampleTime::CYCLES12_5),   // Channel 12
            (temp_ch, adc4::SampleTime::CYCLES12_5),    // Channel 13
        ]
        .into_iter(),
        RegularConversionMode::Continuous,
    );

    info!("Ring buffer configured, starting continuous sampling...");

    // Read buffer - must be half the size of DMA buffer
    let mut measurements = [0u16; DMA_BUF_LEN / 2];

    // Track iterations and accumulated values for periodic logging
    let mut iteration_count: u32 = 0;
    let mut accumulated_vrefint: u32 = 0;
    let mut accumulated_vcore: u32 = 0;
    let mut accumulated_temp: u32 = 0;
    let mut total_samples: u32 = 0;

    // Log every N iterations (~1 second at 5000 samples/sec)
    // Half buffer = 128 sample sets, 5000/128 ≈ 39 reads/sec
    const LOG_INTERVAL: u32 = 39;

    loop {
        // Read from ring buffer - this will return when half buffer is filled
        // IMPORTANT: Read continuously without delay to prevent overruns
        match ring_adc.read(&mut measurements).await {
            Ok(_) => {
                // Process measurements - they come in channel order repeated:
                // [vrefint, vcore, temp, vrefint, vcore, temp, ...]
                let num_samples = measurements.len() / 3;

                // Accumulate samples for each channel
                for i in 0..num_samples {
                    accumulated_vrefint += measurements[i * 3] as u32;
                    accumulated_vcore += measurements[i * 3 + 1] as u32;
                    accumulated_temp += measurements[i * 3 + 2] as u32;
                }
                total_samples += num_samples as u32;

                iteration_count += 1;

                // Log periodically to avoid flooding output
                if iteration_count >= LOG_INTERVAL {
                    let vrefint_avg = accumulated_vrefint / total_samples;
                    let vcore_avg = accumulated_vcore / total_samples;
                    let temp_avg = accumulated_temp / total_samples;

                    // Calculate actual VDDA and convert readings to millivolts
                    let vdda_mv = calibration.calculate_vdda_mv(vrefint_avg);
                    let vcore_mv = (vdda_mv * vcore_avg) / max_count;

                    // Convert temperature using factory calibration with VDDA compensation
                    let temp_mc = calibration.convert_to_millicelsius(temp_avg, vrefint_avg);
                    let temp_c = temp_mc / 1000;
                    let temp_frac = (temp_mc % 1000).unsigned_abs();

                    info!(
                        "Averaged {} samples: VDDA={} mV | VCORE={} mV | Temp={}.{:03} C",
                        total_samples, vdda_mv, vcore_mv, temp_c, temp_frac
                    );

                    // Reset accumulators
                    iteration_count = 0;
                    accumulated_vrefint = 0;
                    accumulated_vcore = 0;
                    accumulated_temp = 0;
                    total_samples = 0;
                }
            }
            Err(_e) => {
                warn!("DMA overrun error - consider increasing buffer size or reducing sample rate");
                // Reset accumulators on error
                iteration_count = 0;
                accumulated_vrefint = 0;
                accumulated_vcore = 0;
                accumulated_temp = 0;
                total_samples = 0;
            }
        }
    }
}
