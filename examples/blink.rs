#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa276 as hal;
use embassy_time::{Duration, Timer};
use hal::gpio::pins::PIO3_18;
use hal::gpio::{Level, Output};

mod common;

use embassy_mcxa276::bind_interrupts;

// Bind only OS_EVENT for timer interrupts
bind_interrupts!(struct Irqs {
    OS_EVENT => hal::ostimer::time_driver::OsEventHandler;
});

#[used]
#[no_mangle]
static KEEP_OS_EVENT: unsafe extern "C" fn() = OS_EVENT;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = hal::init(hal::config::Config::default());

    // Board-style init: enable LED GPIO/PORT clocks used by blink
    unsafe {
        common::init_led(hal::pac());
    }
    // Initialize OSTIMER for async timing
    unsafe {
        common::init_ostimer0(hal::pac());
    }

    // Initialize embassy-time global driver backed by OSTIMER0
    hal::ostimer::time_driver::init(
        hal::config::Config::default().time_interrupt_priority,
        1_000_000,
    );

    // Configure LED pin for GPIO mode
    PIO3_18::set_mux_gpio();

    let mut led = Output::new(PIO3_18::degrade(), Level::High);

    // Complex blinking pattern: SOS in Morse code
    // S: ... (3 short)
    // O: --- (3 long)
    // S: ... (3 short)
    // With pauses between letters and words

    loop {
        // S: three short blinks
        for _ in 0..3 {
            led.set_low();
            Timer::after(Duration::from_millis(150)).await;
            led.set_high();
            Timer::after(Duration::from_millis(150)).await;
        }

        // Pause between letters
        Timer::after(Duration::from_millis(300)).await;

        // O: three long blinks
        for _ in 0..3 {
            led.set_low();
            Timer::after(Duration::from_millis(450)).await;
            led.set_high();
            Timer::after(Duration::from_millis(150)).await;
        }

        // Pause between letters
        Timer::after(Duration::from_millis(300)).await;

        // S: three short blinks
        for _ in 0..3 {
            led.set_low();
            Timer::after(Duration::from_millis(150)).await;
            led.set_high();
            Timer::after(Duration::from_millis(150)).await;
        }

        // Long pause between words (SOS repeats)
        Timer::after(Duration::from_millis(1000)).await;
    }
}
