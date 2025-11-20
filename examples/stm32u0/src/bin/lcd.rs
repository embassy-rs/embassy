#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    lcd::{Bias, Config, Duty, Lcd, LcdPin},
    time::Hertz,
};
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
        config.rcc.ls = LsConfig::default_lsi();
    }

    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut config = Config::default();
    config.bias = Bias::Third;
    config.duty = Duty::Quarter;
    config.target_fps = Hertz(100);

    let mut lcd = Lcd::new(
        p.LCD,
        config,
        p.PC3,
        [
            LcdPin::from(p.PA8),
            LcdPin::from(p.PA9),
            LcdPin::from(p.PA10),
            LcdPin::from(p.PB1),
            LcdPin::from(p.PB9),
            LcdPin::from(p.PB11),
            LcdPin::from(p.PB14),
            LcdPin::from(p.PB15),
            LcdPin::from(p.PC4),
            LcdPin::from(p.PC5),
            LcdPin::from(p.PC6),
            LcdPin::from(p.PC8),
            LcdPin::from(p.PC9),
            LcdPin::from(p.PC10),
            LcdPin::from(p.PC11),
            LcdPin::from(p.PD8),
            LcdPin::from(p.PD9),
            LcdPin::from(p.PD12),
            LcdPin::from(p.PD13),
            LcdPin::from(p.PD0),
            LcdPin::from(p.PD1),
            LcdPin::from(p.PD3),
            LcdPin::from(p.PD4),
            LcdPin::from(p.PD5),
            LcdPin::from(p.PD6),
            LcdPin::from(p.PE7),
            LcdPin::from(p.PE8),
            LcdPin::from(p.PE9),
        ],
    );

    loop {
        defmt::info!("Writing frame");
        lcd.write_frame(&[0xAAAAAAAA; 16]);

        embassy_time::Timer::after_secs(1).await;

        defmt::info!("Writing frame");
        lcd.write_frame(&[!0xAAAAAAAA; 16]);

        embassy_time::Timer::after_secs(1).await;
    }
}
