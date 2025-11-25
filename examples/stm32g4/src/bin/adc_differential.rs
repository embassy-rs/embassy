//! adc differential mode example
//!
//! This example uses adc1 in differential mode
//! p:pa0 n:pa1

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL85,
            divp: None,
            divq: None,
            // Main system clock at 170 MHz
            divr: Some(PllRDiv::DIV2),
        });
        config.rcc.mux.adc12sel = mux::Adcsel::SYS;
        config.rcc.sys = Sysclk::PLL1_R;
    }
    let p = embassy_stm32::init(config);

    let mut adc = Adc::new(p.ADC1, Default::default());
    let mut differential_channel = (p.PA0, p.PA1);

    // can also use
    // adc.set_differential_channel(1, true);
    info!("adc initialized");
    loop {
        let measured = adc.blocking_read(&mut differential_channel, SampleTime::CYCLES247_5);
        info!("data: {}", measured);
        Timer::after_millis(500).await;
    }
}
