#![no_std]
#![no_main]

use defmt::*;
use embassy_stm32::adc::{Adc, AdcChannel, SampleTime, adc4};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::{Config, bind_interrupts, dma, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL1 => dma::InterruptHandler<peripherals::GPDMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
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

    let mut p = embassy_stm32::init(config);

    // **** ADC4 init ****
    let mut adc4 = Adc::new_adc4(p.ADC4);
    let mut adc4_pin1 = p.PA0; // A4
    let mut adc4_pin2 = p.PA1; // A5
    adc4.set_resolution_adc4(adc4::Resolution::BITS12);
    adc4.set_averaging_adc4(adc4::Averaging::Samples256);
    let max4 = adc4::resolution_to_max_count(adc4::Resolution::BITS12);

    // **** ADC4 blocking read ****
    let raw: u16 = adc4.blocking_read(&mut adc4_pin1, adc4::SampleTime::CYCLES1_5);
    let volt: f32 = 3.0 * raw as f32 / max4 as f32;
    info!("Read adc4 pin 1 {}", volt);

    let raw: u16 = adc4.blocking_read(&mut adc4_pin2, adc4::SampleTime::CYCLES1_5);
    let volt: f32 = 3.3 * raw as f32 / max4 as f32;
    info!("Read adc4 pin 2 {}", volt);

    // **** ADC4 async read ****
    let mut degraded41 = adc4_pin1.degrade_adc();
    let mut degraded42 = adc4_pin2.degrade_adc();
    let mut measurements = [0u16; 2];

    // The channels must be in ascending order and can't repeat for ADC4
    adc4.read(
        p.GPDMA1_CH1.reborrow(),
        Irqs,
        [
            (&mut degraded42, SampleTime::CYCLES12_5),
            (&mut degraded41, SampleTime::CYCLES12_5),
        ]
        .into_iter(),
        &mut measurements,
    )
    .await;
    let volt2: f32 = 3.3 * measurements[0] as f32 / max4 as f32;
    let volt1: f32 = 3.0 * measurements[1] as f32 / max4 as f32;
    info!("Async read 4 pin 1 {}", volt1);
    info!("Async read 4 pin 2 {}", volt2);
}
