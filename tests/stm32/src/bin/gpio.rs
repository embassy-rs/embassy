#![no_std]
#![no_main]
#[path = "../common.rs"]
mod common;

use common::*;
use defmt::assert;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Flex, Input, Level, Output, OutputOpenDrain, Pull, Speed};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init();
    info!("Hello World!");

    // Arduino pins D0 and D1
    // They're connected together with a 1K resistor.
    let mut a = peri!(p, UART_RX);
    let mut b = peri!(p, UART_TX);

    // Test initial output
    {
        let b = Input::new(b.reborrow(), Pull::None);

        {
            let a = Output::new(a.reborrow(), Level::Low, Speed::Low);
            delay();
            assert!(b.is_low());
            assert!(!b.is_high());
            assert!(a.is_set_low());
            assert!(!a.is_set_high());
        }
        {
            let mut a = Output::new(a.reborrow(), Level::High, Speed::Low);
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
        let b = Input::new(b.reborrow(), Pull::None);
        // no pull, the status is undefined

        let mut a = Output::new(a.reborrow(), Level::Low, Speed::Low);
        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    // Test input pulldown
    {
        let b = Input::new(b.reborrow(), Pull::Down);
        delay();
        assert!(b.is_low());

        let mut a = Output::new(a.reborrow(), Level::Low, Speed::Low);
        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    // Test input pullup
    {
        let b = Input::new(b.reborrow(), Pull::Up);
        delay();
        assert!(b.is_high());

        let mut a = Output::new(a.reborrow(), Level::Low, Speed::Low);
        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    // Test output open drain
    {
        let b = Input::new(b.reborrow(), Pull::Down);
        // no pull, the status is undefined

        let mut a = OutputOpenDrain::new(a.reborrow(), Level::Low, Speed::Low);
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
        let mut b = Flex::new(b.reborrow());
        b.set_as_input(Pull::None);

        {
            //Flex pin configured as output
            let mut a = Flex::new(a.reborrow()); //Flex pin configured as output
            a.set_low(); // Pin state must be set before configuring the pin, thus we avoid unknown state
            a.set_as_output(Speed::Low);
            delay();
            assert!(b.is_low());
        }
        {
            //Flex pin configured as output
            let mut a = Flex::new(a.reborrow());
            a.set_high();
            a.set_as_output(Speed::Low);

            delay();
            assert!(b.is_high());
        }
    }

    // Test input no pull
    {
        let mut b = Flex::new(b.reborrow());
        b.set_as_input(Pull::None); // no pull, the status is undefined

        let mut a = Flex::new(a.reborrow());
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
        let mut b = Flex::new(b.reborrow());
        b.set_as_input(Pull::Down);
        delay();
        assert!(b.is_low());

        let mut a = Flex::new(a.reborrow());
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
        let mut b = Flex::new(b.reborrow());
        b.set_as_input(Pull::Up);
        delay();
        assert!(b.is_high());

        let mut a = Flex::new(a.reborrow());
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
        let mut b = Flex::new(b.reborrow());
        b.set_as_input(Pull::Down);

        let mut a = Flex::new(a.reborrow());
        a.set_low();
        a.set_as_input_output(Speed::Low);
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
    #[cfg(any(
        feature = "stm32h755zi",
        feature = "stm32h753zi",
        feature = "stm32h7a3zi",
        feature = "stm32h7s3l8"
    ))]
    cortex_m::asm::delay(9000);
    cortex_m::asm::delay(1000);
}
