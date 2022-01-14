#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use defmt::assert;
use embassy::executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::Peripherals;
use example_common::*;

#[embassy::main(config = "config()")]
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
    #[cfg(feature = "stm32wb55rg")]
    let (mut a, mut b) = (p.PA3, p.PA2);
    #[cfg(feature = "stm32h755zi")]
    let (mut a, mut b) = (p.PB6, p.PB7);

    // Test initial output
    {
        let b = Input::new(&mut b, Pull::None);

        {
            let _a = Output::new(&mut a, Level::Low, Speed::Low);
            delay();
            assert!(b.is_low());
        }
        {
            let _a = Output::new(&mut a, Level::High, Speed::Low);
            delay();
            assert!(b.is_high());
        }
    }

    // Test input no pull
    {
        let b = Input::new(&mut b, Pull::None);
        // no pull, the status is undefined

        let mut a = Output::new(&mut a, Level::Low, Speed::Low);
        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    // Test input pulldown
    {
        let b = Input::new(&mut b, Pull::Down);
        delay();
        assert!(b.is_low());

        let mut a = Output::new(&mut a, Level::Low, Speed::Low);
        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    // Test input pullup
    {
        let b = Input::new(&mut b, Pull::Up);
        delay();
        assert!(b.is_high());

        let mut a = Output::new(&mut a, Level::Low, Speed::Low);
        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

fn delay() {
    #[cfg(feature = "stm32h755zi")]
    cortex_m::asm::delay(10000);
    #[cfg(not(feature = "stm32h755zi"))]
    cortex_m::asm::delay(1000);
}
