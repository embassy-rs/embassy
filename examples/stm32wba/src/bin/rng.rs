#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale, mux,
};
use embassy_stm32::rng::Rng;
use embassy_stm32::{Config, bind_interrupts, peripherals, rng};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    // Configure PLL1 (required on WBA)
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,  // PLLM = 1 → HSI / 1 = 16 MHz
        mul: PllMul::MUL30,       // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
        divr: Some(PllDiv::DIV5), // PLLR = 5 → 96 MHz (Sysclk)
        divq: None,
        divp: Some(PllDiv::DIV30), // PLLP = 30 → 16 MHz (required for SAI)
        frac: Some(0),
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    config.rcc.sys = Sysclk::PLL1_R;

    // Configure RNG clock source to HSI (required for WBA)
    config.rcc.mux.rngsel = mux::Rngsel::HSI;

    let p = embassy_stm32::init(config);
    info!("STM32WBA65 RNG Test");
    info!("Initializing RNG...");

    // Initialize RNG - this will trigger the reset() function
    // which previously hung at line 144
    let mut rng = Rng::new(p.RNG, Irqs);
    info!("RNG initialized successfully!");

    // Test 1: Generate random bytes using async method
    info!("\n=== Test 1: Async random bytes ===");
    let mut buf = [0u8; 16];
    match rng.async_fill_bytes(&mut buf).await {
        Ok(_) => info!("Generated 16 random bytes: {:02x}", buf),
        Err(e) => error!("Error generating random bytes: {:?}", e),
    }

    // Test 2: Generate multiple u32 values using blocking method
    info!("\n=== Test 2: Blocking u32 generation ===");
    for i in 0..5 {
        let random = rng.next_u32();
        info!("Random u32 #{}: 0x{:08x} ({})", i + 1, random, random);
        Timer::after_millis(100).await;
    }

    // Test 3: Generate u64 values
    info!("\n=== Test 3: u64 generation ===");
    for i in 0..3 {
        let random = rng.next_u64();
        info!("Random u64 #{}: 0x{:016x}", i + 1, random);
        Timer::after_millis(100).await;
    }

    // Test 4: Fill buffer using blocking method
    info!("\n=== Test 4: Blocking buffer fill ===");
    let mut buf2 = [0u8; 32];
    rng.fill_bytes(&mut buf2);
    info!("Generated 32 random bytes:");
    info!("  {:02x}", &buf2[0..16]);
    info!("  {:02x}", &buf2[16..32]);

    // Test 5: Continuous generation loop
    info!("\n=== Test 5: Continuous generation (10 samples) ===");
    for i in 0..10 {
        let random = rng.next_u32();
        info!("Sample #{}: 0x{:08x}", i + 1, random);
        Timer::after_millis(200).await;
    }

    info!("\nAll RNG tests completed successfully!");
    info!("The RNG is working correctly on STM32WBA65.");

    cortex_m::asm::bkpt();
}
