#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use defmt::panic;
use embassy::executor::{task, Executor};
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_nrf::{interrupt, pac, rtc, uarte};
use futures::future::{select, Either};
use nrf52840_hal::clocks;
use nrf52840_hal::gpio;

#[task]
async fn run(mut uart: uarte::Uarte<pac::UARTE0>) {
    info!("uarte initialized!");

    // Message must be in SRAM
    let mut buf = [0; 8];
    buf.copy_from_slice(b"Hello!\r\n");

    uart.send(&buf).await;
    info!("wrote hello in uart!");

    info!("reading...");
    loop {
        let received = match select(
            uart.receive(&mut buf),
            Timer::after(Duration::from_millis(10)),
        )
        .await
        {
            Either::Left((buf, _)) => buf,
            Either::Right((_, read)) => {
                let (buf, n) = read.stop().await;
                &buf[..n]
            }
        };

        if received.len() > 0 {
            info!("read done, got {:[u8]}", received);

            // Echo back received data
            uart.send(received).await;
        }
    }
}

static RTC: Forever<rtc::RTC<pac::RTC1>> = Forever::new();
static ALARM: Forever<rtc::Alarm<pac::RTC1>> = Forever::new();
static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = unwrap!(embassy_nrf::pac::Peripherals::take());

    clocks::Clocks::new(p.CLOCK)
        .enable_ext_hfosc()
        .set_lfclk_src_external(clocks::LfOscConfiguration::NoExternalNoBypass)
        .start_lfclk();

    let rtc = RTC.put(rtc::RTC::new(p.RTC1, interrupt::take!(RTC1)));
    rtc.start();

    unsafe { embassy::time::set_clock(rtc) };

    let alarm = ALARM.put(rtc.alarm0());
    let executor = EXECUTOR.put(Executor::new_with_alarm(alarm, cortex_m::asm::sev));

    // Init UART
    let port0 = gpio::p0::Parts::new(p.P0);

    let pins = uarte::Pins {
        rxd: port0.p0_08.into_floating_input().degrade(),
        txd: port0
            .p0_06
            .into_push_pull_output(gpio::Level::Low)
            .degrade(),
        cts: None,
        rts: None,
    };

    // NOTE(unsafe): Safe becasue we do not use `mem::forget` anywhere.
    let uart = unsafe {
        uarte::Uarte::new(
            p.UARTE0,
            interrupt::take!(UARTE0_UART0),
            pins,
            uarte::Parity::EXCLUDED,
            uarte::Baudrate::BAUD115200,
        )
    };

    unwrap!(executor.spawn(run(uart)));

    loop {
        executor.run();
        cortex_m::asm::wfe();
    }
}
