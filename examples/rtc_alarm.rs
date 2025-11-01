#![no_std]
#![no_main]


use embassy_mcxa276 as hal;
use cortex_m;
use embassy_executor::Spawner;
use hal::rtc::{RtcDateTime, RtcInterruptEnable};
use hal::uart;
use hal::InterruptExt;

mod common;

type MyRtc = hal::rtc::Rtc<hal::rtc::Rtc0>;

#[cfg(all(feature = "defmt", feature = "defmt-rtt"))]
use defmt_rtt as _;
#[cfg(feature = "defmt")]
use panic_probe as _;
#[cfg(all(feature = "defmt", feature = "defmt-rtt"))]
use rtt_target as _;

use embassy_mcxa276::bind_interrupts;
bind_interrupts!(struct Irqs {
    RTC => hal::rtc::RtcHandler;
});

#[used]
#[no_mangle]
static KEEP_RTC: unsafe extern "C" fn() = RTC;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    
    let p = hal::init(hal::config::Config::default());

    unsafe {
        common::init_uart2(hal::pac());
    }

    let src = unsafe { hal::clocks::uart2_src_hz(hal::pac()) };
    let uart = uart::Uart::<uart::Lpuart2>::new(p.LPUART2, uart::Config::new(src));

    uart.write_str_blocking("\r\n=== RTC Alarm Example ===\r\n");

    unsafe { hal::clocks::init_fro16k(hal::pac()) };

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
    
    loop {

    }
}

#[cfg(not(feature = "defmt"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
