#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use defmt::panic;
use embassy::executor::{task, Executor};
use embassy::time::{Duration, Timer};
use embassy::traits::uart::Uart;
use embassy::util::Forever;
use embassy_nrf::{interrupt, pac, rtc, uarte};
use futures::future::{select, Either};
use nrf52840_hal::clocks;
use nrf52840_hal::gpio;

#[task]
async fn run(uart: pac::UARTE0, port: pac::P0) {
    // Init UART
    let port0 = gpio::p0::Parts::new(port);

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
    let mut uart = unsafe {
        uarte::Uarte::new(
            uart,
            interrupt::take!(UARTE0_UART0),
            pins,
            uarte::Parity::EXCLUDED,
            uarte::Baudrate::BAUD115200,
        )
    };

    info!("uarte initialized!");

    // Message must be in SRAM
    let mut buf = [0; 8];
    buf.copy_from_slice(b"Hello!\r\n");

    unwrap!(uart.send(&buf).await);
    info!("wrote hello in uart!");

    loop {
        let buf_len = buf.len();
        info!("reading...");

        // `receive()` doesn't return until the buffer has been completely filled with
        // incoming data, which in this case is 8 bytes.
        //
        // This example shows how to use `select` to run an uart receive concurrently with a
        // 1 second timer, effectively adding a timeout to the receive operation.
        let recv_fut = uart.receive(&mut buf);
        let timer_fut = Timer::after(Duration::from_millis(1000));
        let received_len = match select(recv_fut, timer_fut).await {
            // recv_fut completed first, so we've received `buf_len` bytes.
            Either::Left(_) => buf_len,
            // timer_fut completed first. `select` gives us back the future that didn't complete, which
            // is `recv_fut` in this case, so we can do further stuff with it.
            //
            // The recv_fut would stop the uart read automatically when dropped. However, we want to know how
            // many bytes have been received, so we have to "gracefully stop" it with `.stop()`.
            Either::Right((_, recv_fut)) => recv_fut.stop().await,
        };
        let received = &mut buf[..received_len];

        if !received.is_empty() {
            info!("read done, got {}", received);

            // Echo back received data
            unwrap!(uart.send(received).await);
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
    let executor = EXECUTOR.put(Executor::new());
    executor.set_alarm(alarm);

    let uarte0 = p.UARTE0;
    let p0 = p.P0;
    executor.run(|spawner| {
        unwrap!(spawner.spawn(run(uarte0, p0)));
    });
}
