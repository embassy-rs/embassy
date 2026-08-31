//! ADC with DMA ring buffer triggered by a timer.
//!
//! The ADC scans a sequence of channels on every timer event and DMA writes the
//! results into a circular buffer, so sampling runs at a hardware-defined rate
//! with no CPU involvement -- the CPU only drains the buffer.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{ADC_MAX, Adc, AdcChannel as _, Exten, RegularAdcTrigger, SampleTime, VREF_INT};
use embassy_stm32::peripherals::DMA1_CH1;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{MasterMode, RoundTo, Timer as LowLevelTimer};
use embassy_stm32::triggers::TIM3_TRGO;
use embassy_stm32::{bind_interrupts, dma};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<DMA1_CH1>;
});

/// Channels per scan: VrefInt, temperature, PA0.
const SEQUENCE_LEN: usize = 3;
/// Scans per half-buffer. `read` returns once the DMA has filled a half.
const SCANS_PER_HALF: usize = 4;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_stm32::init(Default::default());

    info!("ADC ring buffer with timer trigger, STM32F1");

    // Configure TIM3 to generate TRGO events at 4 Hz
    // This will trigger ADC conversions periodically
    // Configured now, but not started yet -- it must not fire before the DMA
    // ring buffer is armed, or the first captured sample won't be the first
    // channel of the sequence and every slot will stay rotated from then on.
    let timer = LowLevelTimer::new(p.TIM3);
    timer.set_frequency(Hertz::hz(4), RoundTo::Slower);
    timer.set_master_mode(MasterMode::Update);

    let mut adc = Adc::new(p.ADC1);
    let mut vrefint = adc.enable_vref();
    let mut temperature = adc.enable_temperature();

    // The temperature sensor and VrefInt need a long sample time (>17.1 us on F1).
    let sequence = [
        (vrefint.reborrow_adc(), SampleTime::Cycles715),
        (temperature.reborrow_adc(), SampleTime::Cycles715),
        (p.PA0.reborrow_adc(), SampleTime::Cycles135),
    ]
    .into_iter();

    // Double-sized so the CPU can drain one half while DMA fills the other.
    let mut dma_buf = [0u16; SEQUENCE_LEN * SCANS_PER_HALF * 2];

    let mut ring_buffered_adc = adc.into_ring_buffered(
        p.DMA1_CH1,
        &mut dma_buf,
        Irqs,
        sequence,
        RegularAdcTrigger::from(TIM3_TRGO, Exten),
    );

    // Arm the DMA before the first trigger, so the scan lands at buffer index 0
    // and each channel keeps a fixed position within the sequence.
    ring_buffered_adc.start();
    timer.start();

    // Buffer to read samples - must be half the size of dma_buf
    let mut data = [0u16; SEQUENCE_LEN * SCANS_PER_HALF];

    loop {
        match ring_buffered_adc.read(&mut data).await {
            Ok(_) => {
                info!("RAW data = {}", data);
                // Samples are interleaved: [vref, temp, pa0, vref, temp, pa0, ...]
                for (i, scan) in data.chunks_exact(SEQUENCE_LEN).enumerate() {
                    let (vrefint_sample, temperature_sample, pa0) = (scan[0], scan[1], scan[2]);

                    // VrefInt is a known 1.20 V, so it calibrates VDDA and lets the
                    // other channels be converted without assuming a 3.3 V rail.
                    let vdda_mv = VREF_INT * ADC_MAX / vrefint_sample.max(1) as u32;
                    let pa0_mv = pa0 as u32 * vdda_mv / ADC_MAX;

                    info!(
                        "scan {}: vrefint={} temp={} pa0={} ({} mV, VDDA {} mV)",
                        i, vrefint_sample, temperature_sample, pa0, pa0_mv, vdda_mv
                    );
                }
            }
            Err(e) => {
                error!("ADC ring buffer overrun: {:?}", e);
                ring_buffered_adc.clear();
            }
        }
    }
}
