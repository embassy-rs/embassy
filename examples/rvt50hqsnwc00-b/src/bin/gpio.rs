#![no_std]
#![no_main]

//! GPIO polling demo for the Riverdi RVT50 user button and LED.
//!
//! Press the user button (`PH3` / `BOOT0`) to increment a counter and flash
//! the user LED (`PE5`). Pin level is logged on RTT for debugging BOOT0/PH3.

use defmt::info;
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board;
use embassy_stm32::Peripherals;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

const DEBOUNCE_MS: u64 = 30;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = rvt50_board::init_clocks();

    let Peripherals { PH3, PE5, .. } = p;

    let button = rvt50_board::init_user_button_input(PH3);
    let mut led = rvt50_board::init_user_led(PE5);

    let mut count = 0u32;
    let mut was_pressed = button.is_high();

    info!(
        "RVT50 GPIO poll: {} (initial={}), {} flashes on press",
        rvt50_board::pins::USER_BUTTON,
        was_pressed,
        rvt50_board::pins::USER_LED,
    );

    loop {
        let pressed = button.is_high();

        if pressed != was_pressed {
            Timer::after_millis(DEBOUNCE_MS).await;
            let pressed = button.is_high();
            if pressed != was_pressed {
                was_pressed = pressed;
                if pressed {
                    count = count.wrapping_add(1);
                    info!("Button pressed, count={}", count);

                    led.set_high();
                    Timer::after_millis(100).await;
                    led.set_low();
                } else {
                    info!("Button released");
                }
            }
        }

        Timer::after_millis(10).await;
    }
}
