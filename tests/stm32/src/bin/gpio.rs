#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::assert;
use embassy::executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::Peripherals;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use example_common::*;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    // Arduino pins D0 and D1
    // They're connected together with a 1K resistor.
    #[cfg(feature = "stm32g491re")]
    let (mut a, mut b) = (p.PC4, p.PC5);
    #[cfg(feature = "stm32g071rb")]
    let (mut a, mut b) = (p.PC4, p.PC5);
    #[cfg(feature = "stm32f429zi")]
    let (mut a, mut b) = (p.PG14, p.PG9);

    // Test initial output
    {
        let b = Input::new(&mut b, Pull::None);

        {
            let _a = Output::new(&mut a, Level::Low, Speed::Low);
            cortex_m::asm::delay(1000);
            assert!(b.is_low().unwrap());
        }
        {
            let _a = Output::new(&mut a, Level::High, Speed::Low);
            cortex_m::asm::delay(1000);
            assert!(b.is_high().unwrap());
        }
    }

    // Test input no pull
    {
        let b = Input::new(&mut b, Pull::None);
        // no pull, the status is undefined

        let mut a = Output::new(&mut a, Level::Low, Speed::Low);
        cortex_m::asm::delay(1000);
        assert!(b.is_low().unwrap());
        a.set_high().unwrap();
        cortex_m::asm::delay(1000);
        assert!(b.is_high().unwrap());
    }

    // Test input pulldown
    {
        let b = Input::new(&mut b, Pull::Down);
        cortex_m::asm::delay(1000);
        assert!(b.is_low().unwrap());

        let mut a = Output::new(&mut a, Level::Low, Speed::Low);
        cortex_m::asm::delay(1000);
        assert!(b.is_low().unwrap());
        a.set_high().unwrap();
        cortex_m::asm::delay(1000);
        assert!(b.is_high().unwrap());
    }

    // Test input pullup
    {
        let b = Input::new(&mut b, Pull::Up);
        cortex_m::asm::delay(1000);
        assert!(b.is_high().unwrap());

        let mut a = Output::new(&mut a, Level::Low, Speed::Low);
        cortex_m::asm::delay(1000);
        assert!(b.is_low().unwrap());
        a.set_high().unwrap();
        cortex_m::asm::delay(1000);
        assert!(b.is_high().unwrap());
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
