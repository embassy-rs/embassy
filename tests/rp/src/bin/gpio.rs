#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert, *};
use embassy_executor::Spawner;
#[cfg(feature = "rp2040")]
use embassy_rp::gpio::OutputOpenDrain;
use embassy_rp::gpio::{Flex, Input, Level, Output, Pull};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Hello World!");

    let (mut a, mut b) = (p.PIN_0, p.PIN_1);

    // Test initial output
    {
        let b = Input::new(b.reborrow(), Pull::None);

        {
            let a = Output::new(a.reborrow(), Level::Low);
            delay();
            assert!(b.is_low());
            assert!(!b.is_high());
            assert!(a.is_set_low());
            assert!(!a.is_set_high());
        }
        {
            let mut a = Output::new(a.reborrow(), Level::High);
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

        let mut a = Output::new(a.reborrow(), Level::Low);
        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    // Test input pulldown
    #[cfg(feature = "rp2040")]
    {
        let b = Input::new(b.reborrow(), Pull::Down);
        delay();
        assert!(b.is_low());

        let mut a = Output::new(a.reborrow(), Level::Low);
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

        let mut a = Output::new(a.reborrow(), Level::Low);
        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    // OUTPUT OPEN DRAIN
    #[cfg(feature = "rp2040")]
    {
        let mut b = OutputOpenDrain::new(b.reborrow(), Level::High);
        let mut a = Flex::new(a.reborrow());
        a.set_as_input();

        // When an OutputOpenDrain is high, it doesn't drive the pin.
        b.set_high();
        a.set_pull(Pull::Up);
        delay();
        assert!(a.is_high());
        a.set_pull(Pull::Down);
        delay();
        assert!(a.is_low());

        // When an OutputOpenDrain is low, it drives the pin low.
        b.set_low();
        a.set_pull(Pull::Up);
        delay();
        assert!(a.is_low());
        a.set_pull(Pull::Down);
        delay();
        assert!(a.is_low());

        // Check high again
        b.set_high();
        a.set_pull(Pull::Up);
        delay();
        assert!(a.is_high());
        a.set_pull(Pull::Down);
        delay();
        assert!(a.is_low());

        // When an OutputOpenDrain is high, it reads the input value in the pin.
        b.set_high();
        a.set_as_input();
        a.set_pull(Pull::Up);
        delay();
        assert!(b.is_high());
        a.set_as_output();
        a.set_low();
        delay();
        assert!(b.is_low());

        // When an OutputOpenDrain is low, it always reads low.
        b.set_low();
        a.set_as_input();
        a.set_pull(Pull::Up);
        delay();
        assert!(b.is_low());
        a.set_as_output();
        a.set_low();
        delay();
        assert!(b.is_low());
    }

    // FLEX
    // Test initial output
    {
        //Flex pin configured as input
        let mut b = Flex::new(b.reborrow());
        b.set_as_input();

        {
            //Flex pin configured as output
            let mut a = Flex::new(a.reborrow()); //Flex pin configured as output
            a.set_low(); // Pin state must be set before configuring the pin, thus we avoid unknown state
            a.set_as_output();
            delay();
            assert!(b.is_low());
        }
        {
            //Flex pin configured as output
            let mut a = Flex::new(a.reborrow());
            a.set_high();
            a.set_as_output();

            delay();
            assert!(b.is_high());
        }
    }

    // Test input no pull
    {
        let mut b = Flex::new(b.reborrow());
        b.set_as_input(); // no pull by default.

        let mut a = Flex::new(a.reborrow());
        a.set_low();
        a.set_as_output();

        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    // Test input pulldown
    #[cfg(feature = "rp2040")]
    {
        let mut b = Flex::new(b.reborrow());
        b.set_as_input();
        b.set_pull(Pull::Down);
        delay();
        assert!(b.is_low());

        let mut a = Flex::new(a.reborrow());
        a.set_low();
        a.set_as_output();
        delay();
        assert!(b.is_low());
        a.set_high();
        delay();
        assert!(b.is_high());
    }

    // Test input pullup
    {
        let mut b = Flex::new(b.reborrow());
        b.set_as_input();
        b.set_pull(Pull::Up);
        delay();
        assert!(b.is_high());

        let mut a = Flex::new(a.reborrow());
        a.set_high();
        a.set_as_output();
        delay();
        assert!(b.is_high());
        a.set_low();
        delay();
        assert!(b.is_low());
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

fn delay() {
    cortex_m::asm::delay(10000);
}
