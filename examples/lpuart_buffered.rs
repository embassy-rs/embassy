#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa276 as hal;
use embassy_mcxa276::interrupt::typelevel::Handler;
use embassy_mcxa276::lpuart;
use embassy_mcxa276::lpuart::buffered::BufferedLpuart;

use embedded_io_async::{Read, Write};

use embassy_mcxa276::bind_interrupts;

mod common;

// Bind OS_EVENT for timers plus LPUART2 IRQ for the buffered driver
bind_interrupts!(struct Irqs {
    LPUART2 => lpuart::buffered::BufferedInterruptHandler::<lpuart::lib::peripherals::LPUART2>;
});

// Wrapper function for the interrupt handler
unsafe extern "C" fn lpuart2_handler() {
    lpuart::buffered::BufferedInterruptHandler::<lpuart::lib::peripherals::LPUART2>::on_interrupt();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = hal::init(hal::config::Config::default());
    let p2 = lpuart::lib::init();

    unsafe {
        hal::interrupt::install_irq_handler(mcxa276_pac::Interrupt::LPUART2, lpuart2_handler);
    }

    // Configure NVIC for LPUART2
    hal::interrupt::LPUART2.configure_for_uart(hal::interrupt::Priority::P3);

    unsafe {
        common::init_uart2(hal::pac());
        common::init_ostimer0(hal::pac());
    }

    // UART configuration (enable both TX and RX)
    let config = lpuart::Config {
        baudrate_bps: 115_200,
        enable_tx: true,
        enable_rx: true,
        rx_fifo_watermark: 0,
        tx_fifo_watermark: 0,
        ..Default::default()
    };

    let mut tx_buf = [0u8; 256];
    let mut rx_buf = [0u8; 256];

    // Create a buffered LPUART2 instance with both TX and RX
    let mut uart = BufferedLpuart::new(
        p2.LPUART2,
        p2.PIO2_2, // TX pin
        p2.PIO2_3, // RX pin
        Irqs,
        &mut tx_buf,
        &mut rx_buf,
        config,
    )
    .unwrap();

    // Split into TX and RX parts
    let (tx, rx) = uart.split_ref();

    tx.write(b"Hello buffered LPUART.\r\n").await.unwrap();
    tx.write(b"Type characters to echo them back.\r\n")
        .await
        .unwrap();

    // Echo loop
    let mut buf = [0u8; 4];
    loop {
        rx.read_exact(&mut buf[..]).await.unwrap();
        tx.write_all(&buf[..]).await.unwrap();
    }
}
