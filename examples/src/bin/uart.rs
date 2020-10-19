#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use embassy::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};
use embassy_nrf::uarte;
use futures::pin_mut;
use nrf52840_hal::gpio;

use embassy::executor::{task, Executor};
static EXECUTOR: Executor = Executor::new(|| cortex_m::asm::sev());

#[task]
async fn run() {
    let p = embassy_nrf::pac::Peripherals::take().dewrap();

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

    let u = uarte::Uarte::new(
        p.UARTE0,
        pins,
        uarte::Parity::EXCLUDED,
        uarte::Baudrate::BAUD115200,
    );
    pin_mut!(u);

    info!("uarte initialized!");

    u.write_all(b"Hello!\r\n").await.dewrap();
    info!("wrote hello in uart!");

    // Simple demo, reading 8-char chunks and echoing them back reversed.
    loop {
        info!("reading...");
        let mut buf = [0u8; 8];
        u.read_exact(&mut buf).await.dewrap();
        info!("read done, got {:[u8]}", buf);

        // Reverse buf
        for i in 0..4 {
            let tmp = buf[i];
            buf[i] = buf[7 - i];
            buf[7 - i] = tmp;
        }

        info!("writing...");
        u.write_all(&buf).await.dewrap();
        info!("write done");
    }
}

#[entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        EXECUTOR.spawn(run()).dewrap();

        loop {
            EXECUTOR.run();
            cortex_m::asm::wfe();
        }
    }
}
