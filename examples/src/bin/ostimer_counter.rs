//! # OSTIMER Counter Reading and Reset Example
//!
//! This example demonstrates the new timer counter reading and reset functionality
//! of the OSTIMER driver.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::periph_helpers::OstimerClockSel;
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::lpuart::{Blocking, Config, Lpuart};
use embassy_time::{Duration, Timer};
use hal::bind_interrupts;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    OS_EVENT => hal::ostimer::time_driver::OsEventHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());

    // Create UART configuration
    let config = Config {
        baudrate_bps: 115_200,
        enable_tx: true,
        enable_rx: true,
        ..Default::default()
    };

    // Create UART instance using LPUART2 with P2_2 as TX and P2_3 as RX
    unsafe {
        embassy_mcxa_examples::init_uart2_pins(hal::pac());
    }
    let mut uart = Lpuart::new_blocking(
        p.LPUART2, // Peripheral
        p.P2_2,    // TX pin
        p.P2_3,    // RX pin
        config,
    )
    .unwrap();

    uart.write_str_blocking("OSTIMER Counter Reading and Reset Example\n");

    // Initialize the OSTIMER time driver
    hal::ostimer::time_driver::init(
        hal::interrupt::Priority::from(3),
        1_000_000, // 1MHz clock
    );

    // Create OSTIMER instance
    let ostimer = hal::ostimer::Ostimer::<hal::ostimer::Ostimer0>::new(
        p.OSTIMER0,
        hal::ostimer::Config {
            init_match_max: true,
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: OstimerClockSel::Clk1M,
        },
    );

    // Read initial counter value
    let initial_counter = ostimer.now();
    uart.write_str_blocking("Initial counter value: ");
    write_u64(&mut uart, initial_counter);
    uart.write_str_blocking("\n");

    // Wait a bit to let counter increment
    Timer::after(Duration::from_millis(100)).await;

    // Read counter again
    let counter_after_wait = ostimer.now();
    uart.write_str_blocking("Counter after 100ms wait: ");
    write_u64(&mut uart, counter_after_wait);
    uart.write_str_blocking("\n");
    uart.write_str_blocking("Difference: ");
    write_u64(&mut uart, counter_after_wait - initial_counter);
    uart.write_str_blocking(" ticks\n");

    // Reset the timer
    uart.write_str_blocking("Resetting timer...\n");
    ostimer.reset(hal::pac());

    // Read counter after reset
    let counter_after_reset = ostimer.now();
    uart.write_str_blocking("Counter after reset: ");
    write_u64(&mut uart, counter_after_reset);
    uart.write_str_blocking("\n");

    // Wait again to verify timer is working
    Timer::after(Duration::from_millis(50)).await;

    let final_counter = ostimer.now();
    uart.write_str_blocking("Counter after another 50ms: ");
    write_u64(&mut uart, final_counter);
    uart.write_str_blocking("\n");
    uart.write_str_blocking("Difference after reset: ");
    write_u64(&mut uart, final_counter - counter_after_reset);
    uart.write_str_blocking(" ticks\n");

    uart.write_str_blocking("Example complete\n");
}

// Helper function to write a u64 value as decimal string
fn write_u64(uart: &mut Lpuart<'_, Blocking>, value: u64) {
    if value == 0 {
        uart.write_str_blocking("0");
        return;
    }

    let mut buffer = [0u8; 20]; // Enough for max u64
    let mut i = 0;
    let mut v = value;

    while v > 0 {
        buffer[i] = b'0' + (v % 10) as u8;
        v /= 10;
        i += 1;
    }

    // Write digits in reverse order
    while i > 0 {
        i -= 1;
        match buffer[i] {
            b'0' => uart.write_str_blocking("0"),
            b'1' => uart.write_str_blocking("1"),
            b'2' => uart.write_str_blocking("2"),
            b'3' => uart.write_str_blocking("3"),
            b'4' => uart.write_str_blocking("4"),
            b'5' => uart.write_str_blocking("5"),
            b'6' => uart.write_str_blocking("6"),
            b'7' => uart.write_str_blocking("7"),
            b'8' => uart.write_str_blocking("8"),
            b'9' => uart.write_str_blocking("9"),
            _ => uart.write_str_blocking("?"),
        }
    }
}
