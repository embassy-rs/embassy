#![no_std]
#![no_main]
teleprobe_meta::target!(b"rpi-pico");

use defmt::{assert, *};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("embassy-rp gpio_async test");

    // On the CI device the following pins are connected with each other.
    let (mut output_pin, mut input_pin) = (p.PIN_0, p.PIN_1);

    {
        info!("test wait_for_high");
        let mut output = Output::new(&mut output_pin, Level::Low);
        let mut input = Input::new(&mut input_pin, Pull::None);

        assert!(input.is_low(), "input was expected to be low");

        let set_high_future = async {
            // Allow time for wait_for_high_future to await wait_for_high().
            Timer::after_millis(10).await;
            output.set_high();
        };
        let wait_for_high_future = async {
            let start = Instant::now();
            input.wait_for_high().await;
            assert_duration(start);
        };
        join(set_high_future, wait_for_high_future).await;
        info!("test wait_for_high: OK\n");
    }

    {
        info!("test wait_for_low");
        let mut output = Output::new(&mut output_pin, Level::High);
        let mut input = Input::new(&mut input_pin, Pull::None);

        assert!(input.is_high(), "input was expected to be high");

        let set_low_future = async {
            Timer::after_millis(10).await;
            output.set_low();
        };
        let wait_for_low_future = async {
            let start = Instant::now();
            input.wait_for_low().await;
            assert_duration(start);
        };
        join(set_low_future, wait_for_low_future).await;
        info!("test wait_for_low: OK\n");
    }

    {
        info!("test wait_for_rising_edge");
        let mut output = Output::new(&mut output_pin, Level::Low);
        let mut input = Input::new(&mut input_pin, Pull::None);

        assert!(input.is_low(), "input was expected to be low");

        let set_high_future = async {
            Timer::after_millis(10).await;
            output.set_high();
        };
        let wait_for_rising_edge_future = async {
            let start = Instant::now();
            input.wait_for_rising_edge().await;
            assert_duration(start);
        };
        join(set_high_future, wait_for_rising_edge_future).await;
        info!("test wait_for_rising_edge: OK\n");
    }

    {
        info!("test wait_for_falling_edge");
        let mut output = Output::new(&mut output_pin, Level::High);
        let mut input = Input::new(&mut input_pin, Pull::None);

        assert!(input.is_high(), "input was expected to be high");

        let set_low_future = async {
            Timer::after_millis(10).await;
            output.set_low();
        };
        let wait_for_falling_edge_future = async {
            let start = Instant::now();
            input.wait_for_falling_edge().await;
            assert_duration(start);
        };
        join(set_low_future, wait_for_falling_edge_future).await;
        info!("test wait_for_falling_edge: OK\n");
    }

    {
        info!("test wait_for_any_edge (falling)");
        let mut output = Output::new(&mut output_pin, Level::High);
        let mut input = Input::new(&mut input_pin, Pull::None);

        assert!(input.is_high(), "input was expected to be high");

        let set_low_future = async {
            Timer::after_millis(10).await;
            output.set_low();
        };
        let wait_for_any_edge_future = async {
            let start = Instant::now();
            input.wait_for_any_edge().await;
            assert_duration(start);
        };
        join(set_low_future, wait_for_any_edge_future).await;
        info!("test wait_for_any_edge (falling): OK\n");
    }

    {
        info!("test wait_for_any_edge (rising)");
        let mut output = Output::new(&mut output_pin, Level::Low);
        let mut input = Input::new(&mut input_pin, Pull::None);

        assert!(input.is_low(), "input was expected to be low");

        let set_high_future = async {
            Timer::after_millis(10).await;
            output.set_high();
        };
        let wait_for_any_edge_future = async {
            let start = Instant::now();
            input.wait_for_any_edge().await;
            assert_duration(start);
        };
        join(set_high_future, wait_for_any_edge_future).await;
        info!("test wait_for_any_edge (rising): OK\n");
    }

    info!("Test OK");
    cortex_m::asm::bkpt();

    fn assert_duration(start: Instant) {
        let dur = Instant::now() - start;
        assert!(dur >= Duration::from_millis(10) && dur < Duration::from_millis(11));
    }
}
