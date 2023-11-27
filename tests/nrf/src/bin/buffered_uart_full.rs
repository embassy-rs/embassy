#![no_std]
#![no_main]
teleprobe_meta::target!(b"nrf52840-dk");

use defmt::{assert_eq, *};
use embassy_executor::Spawner;
use embassy_nrf::buffered_uarte::{self, BufferedUarte};
use embassy_nrf::{bind_interrupts, peripherals, uarte};
use embedded_io_async::{Read, Write};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UARTE0_UART0 => buffered_uarte::InterruptHandler<peripherals::UARTE0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD1M;

    let mut tx_buffer = [0u8; 1024];
    let mut rx_buffer = [0u8; 1024];

    let mut u = BufferedUarte::new(
        p.UARTE0,
        p.TIMER0,
        p.PPI_CH0,
        p.PPI_CH1,
        p.PPI_GROUP0,
        Irqs,
        p.P1_03,
        p.P1_02,
        config.clone(),
        &mut rx_buffer,
        &mut tx_buffer,
    );

    info!("uarte initialized!");

    let (mut rx, mut tx) = u.split();

    let mut buf = [0; 1024];
    for (j, b) in buf.iter_mut().enumerate() {
        *b = j as u8;
    }

    // Write 1024b. This causes the rx buffer to get exactly full.
    unwrap!(tx.write_all(&buf).await);
    unwrap!(tx.flush().await);

    // Read those 1024b.
    unwrap!(rx.read_exact(&mut buf).await);
    for (j, b) in buf.iter().enumerate() {
        assert_eq!(*b, j as u8);
    }

    // The buffer should now be unclogged. Write 1024b again.
    unwrap!(tx.write_all(&buf).await);
    unwrap!(tx.flush().await);

    // Read should work again.
    unwrap!(rx.read_exact(&mut buf).await);
    for (j, b) in buf.iter().enumerate() {
        assert_eq!(*b, j as u8);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
