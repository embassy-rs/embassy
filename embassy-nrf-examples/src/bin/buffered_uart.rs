#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use defmt::panic;
use nrf52840_hal as hal;
use nrf52840_hal::gpio;

use embassy::executor::{task, Executor};
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy::util::Forever;
use embassy_nrf::buffered_uarte;
use embassy_nrf::interrupt;

static mut TX_BUFFER: [u8; 4096] = [0; 4096];
static mut RX_BUFFER: [u8; 4096] = [0; 4096];

#[task]
async fn run() {
    let p = unwrap!(embassy_nrf::pac::Peripherals::take());

    let port0 = gpio::p0::Parts::new(p.P0);

    let pins = buffered_uarte::Pins {
        rxd: port0.p0_08.into_floating_input().degrade(),
        txd: port0
            .p0_06
            .into_push_pull_output(gpio::Level::Low)
            .degrade(),
        cts: None,
        rts: None,
    };

    let ppi = hal::ppi::Parts::new(p.PPI);

    let irq = interrupt::take!(UARTE0_UART0);
    let mut u = buffered_uarte::BufferedUarte::new(
        p.UARTE0,
        p.TIMER0,
        ppi.ppi0,
        ppi.ppi1,
        irq,
        unsafe { &mut RX_BUFFER },
        unsafe { &mut TX_BUFFER },
        pins,
        buffered_uarte::Parity::EXCLUDED,
        buffered_uarte::Baudrate::BAUD115200,
    );

    info!("uarte initialized!");

    unwrap!(u.write_all(b"Hello!\r\n").await);
    info!("wrote hello in uart!");

    // Simple demo, reading 8-char chunks and echoing them back reversed.
    loop {
        info!("reading...");
        let mut buf = [0u8; 8];
        unwrap!(u.read_exact(&mut buf).await);
        info!("read done, got {:[u8]}", buf);

        // Reverse buf
        for i in 0..4 {
            let tmp = buf[i];
            buf[i] = buf[7 - i];
            buf[7 - i] = tmp;
        }

        info!("writing...");
        unwrap!(u.write_all(&buf).await);
        info!("write done");
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let executor = EXECUTOR.put(Executor::new(cortex_m::asm::sev));
    unwrap!(executor.spawn(run()));

    loop {
        executor.run();
        cortex_m::asm::wfe();
    }
}
