#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa as hal;
use hal::lpuart::{Config, Lpuart};
use hal::rtc::{RtcDateTime, RtcInterruptEnable};
use hal::InterruptExt;

type MyRtc = hal::rtc::Rtc<'static, hal::rtc::Rtc0>;

use embassy_mcxa::bind_interrupts;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RTC => hal::rtc::RtcHandler;
});

#[used]
#[no_mangle]
static KEEP_RTC: unsafe extern "C" fn() = RTC;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    // Create UART configuration
    let config = Config {
        baudrate_bps: 115_200,
        enable_tx: true,
        enable_rx: true,
        ..Default::default()
    };

    // Create UART instance using LPUART2 with PIO2_2 as TX and PIO2_3 as RX
    unsafe {
        embassy_mcxa_examples::init_uart2_pins(hal::pac());
    }
    let mut uart = Lpuart::new_blocking(
        p.LPUART2, // Peripheral
        p.PIO2_2,  // TX pin
        p.PIO2_3,  // RX pin
        config,
    )
    .unwrap();

    uart.write_str_blocking("\r\n=== RTC Alarm Example ===\r\n");

    let rtc_config = hal::rtc::get_default_config();

    let rtc = MyRtc::new(p.RTC0, rtc_config);

    let now = RtcDateTime {
        year: 2025,
        month: 10,
        day: 15,
        hour: 14,
        minute: 30,
        second: 0,
    };

    rtc.stop();

    uart.write_str_blocking("Time set to: 2025-10-15 14:30:00\r\n");
    rtc.set_datetime(now);

    let mut alarm = now;
    alarm.second += 10;

    rtc.set_alarm(alarm);
    uart.write_str_blocking("Alarm set for: 2025-10-15 14:30:10 (+10 seconds)\r\n");

    rtc.set_interrupt(RtcInterruptEnable::RTC_ALARM_INTERRUPT_ENABLE);

    unsafe {
        hal::interrupt::RTC.enable();
    }

    unsafe {
        cortex_m::interrupt::enable();
    }

    rtc.start();

    uart.write_str_blocking("RTC started, waiting for alarm...\r\n");

    loop {
        if rtc.is_alarm_triggered() {
            uart.write_str_blocking("\r\n*** ALARM TRIGGERED! ***\r\n");
            break;
        }
    }

    uart.write_str_blocking("Example complete - Test PASSED!\r\n");
}
