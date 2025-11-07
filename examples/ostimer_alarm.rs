#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m;
use embassy_executor::Spawner;
use embassy_mcxa276 as hal;
use hal::uart;

mod common;

use {defmt_rtt as _, panic_probe as _};

use embassy_mcxa276::bind_interrupts;

// Bind only OS_EVENT, and retain the symbol explicitly so it can't be GC'ed.
bind_interrupts!(struct Irqs {
    OS_EVENT => hal::ostimer::time_driver::OsEventHandler;
});

#[used]
#[no_mangle]
static KEEP_OS_EVENT: unsafe extern "C" fn() = OS_EVENT;

// Global flag for alarm callback
static ALARM_FLAG: AtomicBool = AtomicBool::new(false);

// Alarm callback function
fn alarm_callback() {
    ALARM_FLAG.store(true, Ordering::Release);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    // Enable/clock OSTIMER0 and UART2 before touching their registers
    unsafe {
        common::init_ostimer0(hal::pac());
    }
    unsafe {
        common::init_uart2(hal::pac());
    }
    let src = unsafe { hal::clocks::uart2_src_hz(hal::pac()) };
    let uart = uart::Uart::<uart::Lpuart2>::new(p.LPUART2, uart::Config::new(src));
    uart.write_str_blocking("OSTIMER Alarm Example\n");

    // Initialize embassy-time global driver backed by OSTIMER0
    hal::ostimer::time_driver::init(
        hal::config::Config::default().time_interrupt_priority,
        1_000_000,
    );

    // Create OSTIMER instance
    let config = hal::ostimer::Config {
        init_match_max: true,
        clock_frequency_hz: 1_000_000, // 1MHz
    };
    let ostimer =
        hal::ostimer::Ostimer::<hal::ostimer::Ostimer0>::new(p.OSTIMER0, config, hal::pac());

    // Create alarm with callback
    let alarm = hal::ostimer::Alarm::new()
        .with_callback(alarm_callback)
        .with_flag(&ALARM_FLAG);

    uart.write_str_blocking("Scheduling alarm for 2 seconds...\n");

    // Schedule alarm to expire in 2 seconds (2,000,000 microseconds)
    let scheduled = ostimer.schedule_alarm_delay(&alarm, 2_000_000);
    if scheduled {
        uart.write_str_blocking("Alarm scheduled successfully\n");
    } else {
        uart.write_str_blocking("Failed to schedule alarm (would exceed timer capacity)\n");
        return;
    }

    // Wait for alarm to expire
    loop {
        // Check if alarm has expired
        if ALARM_FLAG.load(Ordering::Acquire) {
            uart.write_str_blocking("Alarm expired! Callback executed.\n");
            break;
        }

        // Busy wait - don't use Timer::after_millis as it interferes with alarm MATCH
        for _ in 0..100000 {
            cortex_m::asm::nop();
        }
    }

    // Demonstrate canceling an alarm
    uart.write_str_blocking("Scheduling another alarm for 3 seconds...\n");
    ALARM_FLAG.store(false, Ordering::Release); // Reset flag

    let scheduled = ostimer.schedule_alarm_delay(&alarm, 3_000_000);
    if scheduled {
        uart.write_str_blocking("Alarm scheduled. Waiting 1 second then canceling...\n");

        // Wait 1 second
        embassy_time::Timer::after_millis(1000).await;

        // Cancel the alarm
        ostimer.cancel_alarm(&alarm);
        uart.write_str_blocking("Alarm canceled\n");

        // Check immediately if alarm flag is set
        if !ALARM_FLAG.load(Ordering::Acquire) {
            uart.write_str_blocking("Alarm was successfully canceled\n");
        } else {
            uart.write_str_blocking("Alarm fired despite cancellation\n");
        }
    }

    uart.write_str_blocking("Example complete\n");
}
