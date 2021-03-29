#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use defmt::panic;
use embassy::executor::Spawner;
use embassy::traits::uart::{Read, Write};
use embassy::util::Steal;
use embassy_nrf::gpio::NoPin;
use embassy_nrf::{interrupt, uarte, Peripherals};
use futures::pin_mut;

#[embassy::main]
async fn main(spawner: Spawner) {
    let p = unsafe { Peripherals::steal() };

    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;

    let irq = interrupt::take!(UARTE0_UART0);
    let uart = unsafe { uarte::Uarte::new(p.UARTE0, irq, p.P0_08, p.P0_06, NoPin, NoPin, config) };
    pin_mut!(uart);

    info!("uarte initialized!");

    // Message must be in SRAM
    let mut buf = [0; 8];
    buf.copy_from_slice(b"Hello!\r\n");

    unwrap!(uart.as_mut().write(&buf).await);
    info!("wrote hello in uart!");

    loop {
        info!("reading...");
        unwrap!(uart.as_mut().read(&mut buf).await);
        info!("writing...");
        unwrap!(uart.as_mut().write(&buf).await);

        /*
        // `receive()` doesn't return until the buffer has been completely filled with
        // incoming data, which in this case is 8 bytes.
        //
        // This example shows how to use `select` to run an uart receive concurrently with a
        // 1 second timer, effectively adding a timeout to the receive operation.
        let recv_fut = uart.read(&mut buf);
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
            unwrap!(uart.write(received).await);
        }
         */
    }
}
