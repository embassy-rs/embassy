//! # OSTIMER Race Condition Test
//!
//! This example tests for race conditions in the OSTIMER driver by:
//! - Scheduling alarms sequentially (hardware limitation: only one at a time)
//! - Reading the counter during interrupt-heavy periods
//! - Testing concurrent timer operations
//! - Stress testing interrupt handling

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};

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

#[used]
#[no_mangle]
static KEEP_OS_EVENT: unsafe extern "C" fn() = OS_EVENT;

// Global counters for race condition detection
static ALARM_CALLBACK_COUNT: AtomicU32 = AtomicU32::new(0);
static INTERRUPT_COUNT: AtomicU32 = AtomicU32::new(0);
static RACE_DETECTED: AtomicU32 = AtomicU32::new(0);

// Alarm callback function
fn alarm_callback() {
    let _count = ALARM_CALLBACK_COUNT.fetch_add(1, Ordering::SeqCst);
    INTERRUPT_COUNT.fetch_add(1, Ordering::SeqCst);

    // Simulate some work in the callback to increase chance of races
    for _ in 0..10 {
        cortex_m::asm::nop();
    }
}

fn report_default_handler(uart: &mut Lpuart<'_, Blocking>) {
    let snapshot = hal::interrupt::default_handler_snapshot();
    if snapshot.count == 0 {
        return;
    }

    uart.write_str_blocking("WARNING: DefaultHandler executed ");
    write_u32(uart, snapshot.count);
    uart.write_str_blocking(" time(s). Vector=");
    write_u32(uart, snapshot.vector as u32);
    uart.write_str_blocking(" CFSR=0x");
    write_hex32(uart, snapshot.cfsr);
    uart.write_str_blocking(" HFSR=0x");
    write_hex32(uart, snapshot.hfsr);
    uart.write_str_blocking(" PC=0x");
    write_hex32(uart, snapshot.stacked_pc);
    uart.write_str_blocking(" LR=0x");
    write_hex32(uart, snapshot.stacked_lr);
    uart.write_str_blocking(" SP=0x");
    write_hex32(uart, snapshot.stacked_sp);
    uart.write_str_blocking("\n");

    hal::interrupt::clear_default_handler_snapshot();
}

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

    // Create UART instance using LPUART2 with PIO2_2 as TX and PIO2_3 as RX
    unsafe {
        embassy_mcxa_examples::init_uart2(hal::pac());
    }
    let mut uart = Lpuart::new_blocking(
        p.LPUART2, // Peripheral
        p.PIO2_2,  // TX pin
        p.PIO2_3,  // RX pin
        config,
    )
    .unwrap();

    uart.write_str_blocking("OSTIMER Race Condition Test Starting...\n");

    // The bind_interrupts! macro handles handler binding automatically

    // Initialize the OSTIMER time driver FIRST
    hal::ostimer::time_driver::init(
        hal::interrupt::Priority::from(3),
        1_000_000, // 1MHz clock
    );

    uart.write_str_blocking("Time driver initialized\n");

    // Create OSTIMER instance
    let ostimer = hal::ostimer::Ostimer::<hal::ostimer::Ostimer0>::new(
        p.OSTIMER0,
        hal::ostimer::Config {
            init_match_max: true,
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: OstimerClockSel::Clk1M,
        },
    );

    uart.write_str_blocking("OSTIMER instance created\n");

    // Test 1: Sequential alarm scheduling (OSTIMER only supports one alarm at a time)
    uart.write_str_blocking("Test 1: Sequential alarm scheduling...\n");
    test_rapid_alarms(&ostimer, &mut uart).await;
    report_default_handler(&mut uart);

    // Test 2: Counter reading during interrupts
    uart.write_str_blocking("Test 2: Counter reading during interrupts...\n");
    test_counter_reading_during_interrupts(&ostimer, &mut uart).await;
    report_default_handler(&mut uart);

    // Test 3: Concurrent timer operations
    uart.write_str_blocking("Test 3: Concurrent timer operations...\n");
    test_concurrent_operations(&ostimer, &mut uart).await;
    report_default_handler(&mut uart);

    // Test 4: Timer reset during operation
    uart.write_str_blocking("Test 4: Timer reset during operation...\n");
    test_reset_during_operation(&ostimer, &mut uart, hal::pac()).await;
    report_default_handler(&mut uart);

    // Report results
    uart.write_str_blocking("Race condition test complete\n");
    uart.write_str_blocking("Callback count: ");
    write_u32(&mut uart, ALARM_CALLBACK_COUNT.load(Ordering::SeqCst));
    uart.write_str_blocking("\nInterrupt count: ");
    write_u32(&mut uart, INTERRUPT_COUNT.load(Ordering::SeqCst));
    uart.write_str_blocking("\nRaces detected: ");
    write_u32(&mut uart, RACE_DETECTED.load(Ordering::SeqCst));
    uart.write_str_blocking("\n");
}

// Test rapid alarm scheduling to stress interrupt handling
async fn test_rapid_alarms(
    ostimer: &hal::ostimer::Ostimer<'_, hal::ostimer::Ostimer0>,
    uart: &mut Lpuart<'_, Blocking>,
) {
    let initial_count = ALARM_CALLBACK_COUNT.load(Ordering::SeqCst);

    // Schedule 10 alarms sequentially (OSTIMER only supports one alarm at a time)
    for _i in 0..10 {
        let alarm = hal::ostimer::Alarm::new().with_callback(alarm_callback);
        let delay_us = 1000; // 1ms delay for each alarm
        if ostimer.schedule_alarm_delay(&alarm, delay_us) {
            // Wait for this alarm to complete before scheduling the next
            Timer::after(Duration::from_micros(delay_us + 100)).await;
            report_default_handler(uart);
        } else {
            RACE_DETECTED.fetch_add(1, Ordering::SeqCst);
            uart.write_str_blocking("ERROR: Failed to program OSTIMER alarm (match not ready)\n");
        }
    }

    // All alarms should have completed by now
    let final_count = ALARM_CALLBACK_COUNT.load(Ordering::SeqCst);
    let expected_count = initial_count + 10;

    if final_count != expected_count {
        RACE_DETECTED.fetch_add(1, Ordering::SeqCst);
        uart.write_str_blocking("ERROR: Expected ");
        write_u32(uart, expected_count);
        uart.write_str_blocking(" callbacks, got ");
        write_u32(uart, final_count);
        uart.write_str_blocking("\n");
    } else {
        uart.write_str_blocking("PASS: All rapid alarms executed\n");
    }
}

// Test reading counter while interrupts are firing
async fn test_counter_reading_during_interrupts(
    ostimer: &hal::ostimer::Ostimer<'_, hal::ostimer::Ostimer0>,
    uart: &mut Lpuart<'_, Blocking>,
) {
    let initial_interrupt_count = INTERRUPT_COUNT.load(Ordering::SeqCst);

    // Schedule an alarm that will fire soon
    let alarm = hal::ostimer::Alarm::new().with_callback(alarm_callback);
    if !ostimer.schedule_alarm_delay(&alarm, 500) {
        RACE_DETECTED.fetch_add(1, Ordering::SeqCst);
        uart.write_str_blocking("ERROR: Failed to program OSTIMER alarm before counter stress\n");
    }

    // While alarm is pending, read the counter many times rapidly
    // This tests if counter reading is atomic and doesn't get corrupted by interrupts
    let mut last_counter = ostimer.now();
    let mut consistent_reads = 0;
    let mut total_reads = 0;

    for _ in 0..1000 {
        let current_counter = ostimer.now();
        total_reads += 1;

        // Check if counter is monotonically increasing (basic sanity check)
        if current_counter >= last_counter {
            consistent_reads += 1;
        }
        last_counter = current_counter;

        // Small delay between reads
        for _ in 0..10 {
            cortex_m::asm::nop();
        }

        report_default_handler(uart);
    }

    // Wait for alarm to complete
    Timer::after(Duration::from_millis(1)).await;

    let final_interrupt_count = INTERRUPT_COUNT.load(Ordering::SeqCst);

    if consistent_reads == total_reads {
        uart.write_str_blocking("PASS: Counter reading consistent during interrupts\n");
    } else {
        RACE_DETECTED.fetch_add(1, Ordering::SeqCst);
        uart.write_str_blocking("ERROR: Counter reading inconsistent: ");
        write_u32(uart, consistent_reads);
        uart.write_str_blocking("/");
        write_u32(uart, total_reads);
        uart.write_str_blocking(" consistent\n");
    }

    if final_interrupt_count > initial_interrupt_count {
        uart.write_str_blocking("PASS: Interrupt fired during counter reading test\n");
    } else {
        uart.write_str_blocking("WARNING: No interrupt fired during counter reading test\n");
    }
}

// Test concurrent timer operations (embassy-time + alarms)
async fn test_concurrent_operations(
    ostimer: &hal::ostimer::Ostimer<'_, hal::ostimer::Ostimer0>,
    uart: &mut Lpuart<'_, Blocking>,
) {
    let initial_interrupt_count = INTERRUPT_COUNT.load(Ordering::SeqCst);

    // Start an embassy-time timer
    let timer_future = Timer::after(Duration::from_millis(2));

    // Schedule an alarm that should fire before the timer
    let alarm = hal::ostimer::Alarm::new().with_callback(alarm_callback);
    if !ostimer.schedule_alarm_delay(&alarm, 1000) {
        RACE_DETECTED.fetch_add(1, Ordering::SeqCst);
        uart.write_str_blocking("ERROR: Failed to program OSTIMER alarm before concurrent operations\n");
    }

    // Wait for both to complete
    timer_future.await;

    let final_interrupt_count = INTERRUPT_COUNT.load(Ordering::SeqCst);

    if final_interrupt_count > initial_interrupt_count {
        uart.write_str_blocking("PASS: Concurrent operations completed\n");
    } else {
        uart.write_str_blocking("WARNING: No interrupts during concurrent operations\n");
    }
}

// Test timer reset during active operations
async fn test_reset_during_operation(
    ostimer: &hal::ostimer::Ostimer<'_, hal::ostimer::Ostimer0>,
    uart: &mut Lpuart<'_, Blocking>,
    peripherals: &hal::pac::Peripherals,
) {
    let initial_counter = ostimer.now();

    // Schedule an alarm
    let alarm = hal::ostimer::Alarm::new().with_callback(alarm_callback);
    if !ostimer.schedule_alarm_delay(&alarm, 2000) {
        RACE_DETECTED.fetch_add(1, Ordering::SeqCst);
        uart.write_str_blocking("ERROR: Failed to program OSTIMER alarm before reset test\n");
    }

    // Wait a bit then reset the timer
    Timer::after(Duration::from_millis(1)).await;
    ostimer.reset(peripherals);

    // Check counter after reset
    let counter_after_reset = ostimer.now();

    // Wait to see if the alarm still fires (it shouldn't after reset)
    Timer::after(Duration::from_millis(2)).await;

    let final_counter = ostimer.now();

    if counter_after_reset < initial_counter {
        uart.write_str_blocking("PASS: Timer reset successful\n");
    } else {
        RACE_DETECTED.fetch_add(1, Ordering::SeqCst);
        uart.write_str_blocking("ERROR: Timer reset may have failed\n");
    }

    uart.write_str_blocking("Counter progression after reset: ");
    write_u64(uart, initial_counter);
    uart.write_str_blocking(" -> ");
    write_u64(uart, counter_after_reset);
    uart.write_str_blocking(" -> ");
    write_u64(uart, final_counter);
    uart.write_str_blocking("\n");
}

// Helper function to write a u32 value as decimal string
fn write_u32(uart: &mut Lpuart<'_, Blocking>, value: u32) {
    if value == 0 {
        uart.write_str_blocking("0");
        return;
    }

    let mut buffer = [0u8; 10]; // Enough for max u32
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

fn write_hex32(uart: &mut Lpuart<'_, Blocking>, value: u32) {
    let mut buf = [b'0'; 8];
    let mut tmp = value;
    for i in (0..8).rev() {
        let digit = (tmp & 0xF) as u8;
        buf[i] = match digit {
            0..=9 => b'0' + digit,
            10..=15 => b'A' + (digit - 10),
            _ => b'?',
        };
        tmp >>= 4;
    }
    uart.blocking_write(&buf).unwrap();
}

// Helper function to write a u64 value as decimal string
fn write_u64(uart: &mut Lpuart<'_, Blocking>, value: u64) {
    if value == 0 {
        uart.blocking_write(b"0").unwrap();
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
            b'0' => uart.blocking_write(b"0").unwrap(),
            b'1' => uart.blocking_write(b"1").unwrap(),
            b'2' => uart.blocking_write(b"2").unwrap(),
            b'3' => uart.blocking_write(b"3").unwrap(),
            b'4' => uart.blocking_write(b"4").unwrap(),
            b'5' => uart.blocking_write(b"5").unwrap(),
            b'6' => uart.blocking_write(b"6").unwrap(),
            b'7' => uart.blocking_write(b"7").unwrap(),
            b'8' => uart.blocking_write(b"8").unwrap(),
            b'9' => uart.blocking_write(b"9").unwrap(),
            _ => uart.blocking_write(b"?").unwrap(),
        }
    }
}
