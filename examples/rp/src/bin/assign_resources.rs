//! This example demonstrates how to assign resources to multiple tasks by splitting up the peripherals.
//! It is not about sharing the same resources between tasks, see sharing.rs for that or head to https://embassy.dev/book/#_sharing_peripherals_between_tasks)
//! Of course splitting up resources and sharing resources can be combined, yet this example is only about splitting up resources.
//!
//! There are basically two ways we demonstrate here:
//! 1) Assigning resources to a task by passing parts of the peripherals
//! 2) Assigning resources to a task by passing a struct with the split up peripherals, using the assign-resources macro
//!
//! using four LEDs on Pins 10, 11, 20 and 21

#![no_std]
#![no_main]

use assign_resources::assign_resources;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{self, PIN_20, PIN_21};
use embassy_rp::Peri;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // initialize the peripherals
    let p = embassy_rp::init(Default::default());

    // 1) Assigning a resource to a task by passing parts of the peripherals.
    spawner
        .spawn(double_blinky_manually_assigned(spawner, p.PIN_20, p.PIN_21))
        .unwrap();

    // 2) Using the assign-resources macro to assign resources to a task.
    // we perform the split, see further below for the definition of the resources struct
    let r = split_resources!(p);
    // and then we can use them
    spawner.spawn(double_blinky_macro_assigned(spawner, r.leds)).unwrap();
}

// 1) Assigning a resource to a task by passing parts of the peripherals.
#[embassy_executor::task]
async fn double_blinky_manually_assigned(
    _spawner: Spawner,
    pin_20: Peri<'static, PIN_20>,
    pin_21: Peri<'static, PIN_21>,
) {
    let mut led_20 = Output::new(pin_20, Level::Low);
    let mut led_21 = Output::new(pin_21, Level::High);

    loop {
        info!("toggling leds");
        led_20.toggle();
        led_21.toggle();
        Timer::after_secs(1).await;
    }
}

// 2) Using the assign-resources macro to assign resources to a task.
// first we define the resources we want to assign to the task using the assign_resources! macro
// basically this will split up the peripherals struct into smaller structs, that we define here
// naming is up to you, make sure your future self understands what you did here
assign_resources! {
    leds: Leds{
        led_10: PIN_10,
        led_11: PIN_11,
    }
    // add more resources to more structs if needed, for example defining one struct for each task
}
// this could be done in another file and imported here, but for the sake of simplicity we do it here
// see https://github.com/adamgreig/assign-resources for more information

// 2) Using the split resources in a task
#[embassy_executor::task]
async fn double_blinky_macro_assigned(_spawner: Spawner, r: Leds) {
    let mut led_10 = Output::new(r.led_10, Level::Low);
    let mut led_11 = Output::new(r.led_11, Level::High);

    loop {
        info!("toggling leds");
        led_10.toggle();
        led_11.toggle();
        Timer::after_secs(1).await;
    }
}
