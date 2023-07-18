#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Flex, Input, Level, Output, OutputOpenDrain, Pull, Speed};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(config());
    info!("Hello World!");

    // Arduino pins D0 and D1
    // They're connected together with a 1K resistor.
    #[cfg(feature = "stm32f103c8")]
    let (mut a, mut b) = (p.PA9, p.PA10);
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
    #[cfg(feature = "stm32u585ai")]
    let (mut a, mut b) = (p.PD9, p.PD8);
    #[cfg(feature = "stm32h563zi")]
    let (mut a, mut b) = (p.PB6, p.PB7);
    #[cfg(feature = "stm32c031c6")]
    let (mut a, mut b) = (p.PB6, p.PB7);

    // Test initial output
    {
        let b = Input::new(&mut b, Pull::None);

        {
            let a = Output::new(&mut a, Level::Low, Speed::Low);
            delay();
            assert!(b.is_low());
            assert!(!b.is_high());
            assert!(a.is_set_low());
            assert!(!a.is_set_high());
        }
        {
            let mut a = Output::new(&mut a, Level::High, Speed::Low);
            delay();
            assert!(!b.is_low());
            assert!(b.is_high());
            assert!(!a.is_set_low());
            assert!(a.is_set_high());

            // Test is_set_low / is_set_high
            a.set_low();
            delay();
            assert!(b.is_low());
            assert!(a.is_set_low());
            assert!(!a.is_set_high());

            a.set_high();
            delay();
            assert!(b.is_high());
            assert!(!a.is_set_low());
            assert!(a.is_set_high());

            // Test toggle
            a.toggle();
            delay();
            assert!(b.is_low());
            assert!(a.is_set_low());
            assert!(!a.is_set_high());

            a.toggle();
            delay();
            assert!(b.is_high());
            assert!(!a.is_set_low());
            assert!(a.is_set_high());
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

    // Test output open drain
    {
        let b = Input::new(&mut b, Pull::Down);
        // no pull, the status is undefined

        let mut a = OutputOpenDrain::new(&mut a, Level::Low, Speed::Low, Pull::None);
        delay();
        assert!(b.is_low());
        a.set_high(); // High-Z output
        delay();
        assert!(b.is_low());
    }

    // FLEX
    // Test initial output
    {
        //Flex pin configured as input
        let mut b = Flex::new(&mut b);
        b.set_as_input(Pull::None);

        {
            //Flex pin configured as output
            let mut a = Flex::new(&mut a); //Flex pin configured as output
            a.set_low(); // Pin state must be set before configuring the pin, thus we avoid unknown state
            a.set_as_output(Speed::Low);
            delay();
            assert!(b.is_low());
        }
        {
            //Flex pin configured as output
            let mut a = Flex::new(&mut a);
            a.set_high();
            a.set_as_output(Speed::Low);

            delay();
            assert!(b.is_high());
        }
    }

    // Test input no pull
    {
        let mut b = Flex::new(&mut b);
        b.set_as_input(Pull::None); // no pull, the status is undefined

        let mut a = Flex::new(&mut a);
        a.set_low();
        a.set_as_output(Speed::Low);

        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    // Test input pulldown
    {
        let mut b = Flex::new(&mut b);
        b.set_as_input(Pull::Down);
        delay();
        assert!(b.is_low());

        let mut a = Flex::new(&mut a);
        a.set_low();
        a.set_as_output(Speed::Low);
        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    // Test input pullup
    {
        let mut b = Flex::new(&mut b);
        b.set_as_input(Pull::Up);
        delay();
        assert!(b.is_high());

        let mut a = Flex::new(&mut a);
        a.set_high();
        a.set_as_output(Speed::Low);
        delay();
        assert!(b.is_high());
        a.set_low();
        delay();
        assert!(b.is_low());
    }

    // Test output open drain
    {
        let mut b = Flex::new(&mut b);
        b.set_as_input(Pull::Down);

        let mut a = Flex::new(&mut a);
        a.set_low();
        a.set_as_input_output(Speed::Low, Pull::None);
        delay();
        assert!(b.is_low());
        a.set_high(); // High-Z output
        delay();
        assert!(b.is_low());
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
