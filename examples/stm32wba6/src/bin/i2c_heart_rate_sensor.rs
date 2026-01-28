#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_time::Instant;
use embassy_stm32::{Config, dma};
use embassy_stm32::rcc::{
    AHBPrescaler, AHB5Prescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::i2c;
use embassy_stm32::peripherals::{I2C1, GPDMA1_CH4, GPDMA1_CH5};
use embassy_stm32::bind_interrupts;

use max30102_driver::{Max30102, PpgFilter, HrDetector};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    // Fine-tune PLL1 dividers/multipliers
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,  // PLLM = 1 → HSI / 1 = 16 MHz
        mul: PllMul::MUL30,       // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
        divr: Some(PllDiv::DIV5), // PLLR = 5 → 96 MHz (Sysclk)
        // divq: Some(PllDiv::DIV10), // PLLQ = 10 → 48 MHz (NOT USED)
        divq: None,
        divp: Some(PllDiv::DIV30), // PLLP = 30 → 16 MHz (USBOTG)
        frac: Some(0),             // Fractional part (enabled)
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;

    // voltage scale for max performance
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    // route PLL1_P into the USB‐OTG‐HS block
    config.rcc.sys = Sysclk::PLL1_R;
    
    let p = embassy_stm32::init(config);

    bind_interrupts!(struct Irqs {
        I2C1_EV => i2c::EventInterruptHandler<I2C1>;
        I2C1_ER => i2c::ErrorInterruptHandler<I2C1>;
        GPDMA1_CHANNEL4 => dma::InterruptHandler<GPDMA1_CH4>;
        GPDMA1_CHANNEL5 => dma::InterruptHandler<GPDMA1_CH5>;
    });

    let mut i2c_myconfig = i2c::Config::default();
    i2c_myconfig.scl_pullup = true;
    i2c_myconfig.sda_pullup = true;
    let i2c_bus = i2c::I2c::new(p.I2C1, p.PB2, p.PB1, p.GPDMA1_CH4, p.GPDMA1_CH5, Irqs, i2c_myconfig);
    
    // Initialize MAX30102 driver
    let mut max30102 = Max30102::new(i2c_bus);
    let part_id = max30102.init().await.unwrap();
    defmt::info!("Part ID: {}", part_id);

    let mut filter = PpgFilter::new(100.0); // 100 Hz sampling
    let mut hr_detector = HrDetector::new();
    let mut no_finger_count: u32 = 0;
    let mut finger_present = false;
    let mut no_finger_display_count: u32 = 0;

    // Let filter stabilize (high-pass needs time to remove DC)
    defmt::info!("Stabilizing filter...");
    for _ in 0..100 {
        if let Ok(sample) = max30102.read_sample().await {
            filter.process(sample);
        }
        Timer::after_millis(10).await;
    }
    defmt::info!("Filter stabilized, starting HR detection...");

    let mut sample_count: u32 = 0;
    
    loop {
        // Check for FIFO overflow
        if max30102.check_overflow().await.unwrap_or(false) {
            defmt::warn!("FIFO overflow detected! Clearing...");
        }

        let samples_available = max30102.samples_available().await.unwrap_or(0);
        
        if !finger_present && samples_available > 0 {
            // Finger was removed - check if finger returned
            if let Ok(check_ir) = max30102.read_sample().await {
                if max30102.is_finger_present(check_ir) {
                    // Finger returned - drain remaining stale samples
                    defmt::info!("Finger detected (ir={=u32}), clearing stale samples", check_ir);
                    let _ = max30102.drain_fifo(31).await;
                    hr_detector.reset();
                    finger_present = true;
                    no_finger_count = 0;
                    no_finger_display_count = 0;
                } else {
                    // Still no finger - display 0 BPM periodically
                    no_finger_display_count += 1;
                    if no_finger_display_count % 50 == 0 {
                        defmt::info!("HR: 0.0 BPM (no finger)");
                    }
                    Timer::after_millis(10).await;
                    continue;
                }
            }
        }
        
        // Read one sample if available and finger is present
        if samples_available > 0 && finger_present {
            if let Ok(ir_value) = max30102.read_sample().await {
                // Check if finger is still present
                if !max30102.is_finger_present(ir_value) {
                    no_finger_count += 1;
                    
                    // Only reset after sustained low readings (debounce)
                    if no_finger_count >= max30102.no_finger_threshold_count() {
                        defmt::warn!("Finger removed (ir={=u32}), resetting detector", ir_value);
                        hr_detector.reset();
                        finger_present = false;
                    }
                    
                    // Don't process this sample - continue to next iteration
                    Timer::after_millis(10).await;
                    continue;
                } else {
                    // Finger is present - process normally
                    no_finger_count = 0;
                    
                    // Process this sample normally
                    let filtered = filter.process(ir_value);
                    let now_ms = Instant::now().as_millis() as u32;
                    sample_count += 1;
                                               
                    // Update HR detector (always process, even if no heartbeat detected)
                    hr_detector.update(filtered, now_ms);
                    
                    // Always display current BPM (even if 0) - update every 50 samples (~0.5 seconds at 100Hz)
                    if sample_count % 50 == 0 {
                        let current_bpm = hr_detector.current_bpm();
                        defmt::info!("HR: {=f32} BPM", current_bpm);
                    }
                }
            }
        } else {
            // No samples available, wait a bit
            Timer::after_millis(1).await;
        }

        // Small delay to prevent tight loop
        Timer::after_millis(1).await;
    }
}
