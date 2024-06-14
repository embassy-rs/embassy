#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::lcd::{Bias, Config, Duty, Lcd, VoltageSource};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // The RTC clock = the LCD clock and must be running
    {
        use embassy_stm32::rcc::*;
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.hsi = true;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI, // 16 MHz
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL7, // 16 * 7 = 112 MHz
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2), // 112 / 2 = 56 MHz
        });
        config.rcc.ls = LsConfig::default();
    }

    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut config = Config::default();
    config.bias = Bias::Third;
    config.duty = Duty::Quarter;

    let mut lcd = Lcd::new(
        p.LCD,
        config,
        [
            p.PC4.into(),
            p.PC5.into(),
            p.PB1.into(),
            p.PE7.into(),
            p.PE8.into(),
            p.PE9.into(),
            p.PB11.into(),
            p.PB14.into(),
            p.PB15.into(),
            p.PD8.into(),
            p.PD9.into(),
            p.PD12.into(),
            p.PB9.into(),
            p.PA10.into(),
            p.PA9.into(),
            p.PA8.into(),
            p.PD13.into(),
            p.PC6.into(),
            p.PC8.into(),
            p.PC9.into(),
            p.PC10.into(),
            p.PD0.into(),
            p.PD1.into(),
            p.PD3.into(),
            p.PD4.into(),
            p.PD5.into(),
            p.PD6.into(),
            p.PC11.into(),
        ],
    );

    loop {
        defmt::info!("Writing frame");
        lcd.write_frame(&[0xAAAAAAAA; 16]);
        defmt::info!("Writing frame");
        lcd.write_frame(&[!0xAAAAAAAA; 16]);
    }
}
