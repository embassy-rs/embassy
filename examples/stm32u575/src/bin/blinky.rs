#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::i2c::{self, Config as I2C_Config, I2c};
use embassy_stm32::peripherals;
use embassy_time::Timer;
use panic_probe as _;
use pca9535::{GPIOBank, Pca9535Immediate, StandardExpanderInterface};

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let dp = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let i2c_bus = I2c::new(
        dp.I2C1,
        dp.PB6,
        dp.PB7,
        Irqs,
        dp.GPDMA1_CH13,
        dp.GPDMA1_CH12,
        I2C_Config::default(),
    );

    // Setup LEDs
    let mut expander = Pca9535Immediate::new(i2c_bus, 0x21);
    expander.pin_into_output(GPIOBank::Bank1, 2).unwrap();

    loop {
        defmt::info!("on!");
        expander.pin_set_low(GPIOBank::Bank1, 2).unwrap();
        Timer::after_millis(200).await;

        defmt::info!("off!");
        expander.pin_set_high(GPIOBank::Bank1, 2).unwrap();
        Timer::after_millis(200).await;
    }
}
