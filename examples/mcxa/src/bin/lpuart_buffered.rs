#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::lpuart::{Config, Lpuart};
use embassy_mcxa::{bind_interrupts, lpuart};
use embedded_io_async::Write;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// Bind OS_EVENT for timers plus LPUART2 IRQ for the buffered driver
bind_interrupts!(struct Irqs {
    LPUART2 => lpuart::BufferedInterruptHandler::<hal::peripherals::LPUART2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    // UART configuration (enable both TX and RX)
    let config = Config {
        baudrate_bps: 115_200,
        rx_fifo_watermark: 0,
        tx_fifo_watermark: 0,
        ..Default::default()
    };

    let mut tx_buf = [0u8; 256];
    let mut rx_buf = [0u8; 256];

    // Create a buffered LPUART2 instance with both TX and RX
    let mut uart = Lpuart::new_buffered(
        p.LPUART2,
        p.P2_2, // TX pin
        p.P2_3, // RX pin
        Irqs,
        &mut tx_buf,
        &mut rx_buf,
        config,
    )
    .unwrap();

    // Split into TX and RX parts
    let (tx, rx) = uart.split_ref();

    tx.write(b"Hello buffered LPUART.\r\n").await.unwrap();
    tx.write(b"Type characters to echo them back.\r\n").await.unwrap();

    // Echo loop
    let mut buf = [0u8; 4];
    loop {
        let used = rx.read(&mut buf).await.unwrap();
        tx.write_all(&buf[..used]).await.unwrap();
    }
}
