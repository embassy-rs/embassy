#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Config, InterruptHandler, Pin};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::Pull;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_rp::init(Default::default());
    let mut adc = Adc::new(p.ADC, Irqs, Config::default());

    {
        {
            let mut p = Pin::new(&mut p.PIN_26, Pull::Down);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() < 0b01_0000_0000);
            defmt::assert!(adc.read(&mut p).await.unwrap() < 0b01_0000_0000);
        }
        {
            let mut p = Pin::new(&mut p.PIN_26, Pull::Up);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() > 0b11_0000_0000);
            defmt::assert!(adc.read(&mut p).await.unwrap() > 0b11_0000_0000);
        }
    }
    // not bothering with async reads from now on
    {
        {
            let mut p = Pin::new(&mut p.PIN_27, Pull::Down);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() < 0b01_0000_0000);
        }
        {
            let mut p = Pin::new(&mut p.PIN_27, Pull::Up);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() > 0b11_0000_0000);
        }
    }
    {
        {
            let mut p = Pin::new(&mut p.PIN_28, Pull::Down);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() < 0b01_0000_0000);
        }
        {
            let mut p = Pin::new(&mut p.PIN_28, Pull::Up);
            defmt::assert!(adc.blocking_read(&mut p).unwrap() > 0b11_0000_0000);
        }
    }
    {
        // gp29 is connected to vsys through a 200k/100k divider,
        // adding pulls should change the value
        let low = {
            let mut p = Pin::new(&mut p.PIN_29, Pull::Down);
            adc.blocking_read(&mut p).unwrap()
        };
        let none = {
            let mut p = Pin::new(&mut p.PIN_29, Pull::None);
            adc.blocking_read(&mut p).unwrap()
        };
        let up = {
            let mut p = Pin::new(&mut p.PIN_29, Pull::Up);
            adc.blocking_read(&mut p).unwrap()
        };
        defmt::assert!(low < none);
        defmt::assert!(none < up);
    }

    let temp = convert_to_celsius(adc.read_temperature().await.unwrap());
    defmt::assert!(temp > 0.0);
    defmt::assert!(temp < 60.0);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

fn convert_to_celsius(raw_temp: u16) -> f32 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721 as f32
}
