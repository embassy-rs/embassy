//! DMA UART TX and fixed-length RX.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::lpuart::{Config, Lpuart};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("LPUART DMA example starting...");

    let config = Config {
        baudrate_bps: 115_200,
        rx_fifo_watermark: 0, // Drain every received byte.
        ..Default::default()
    };

    let mut lpuart = Lpuart::new_async_with_dma(p.LPUART1, p.P1_9, p.P1_8, p.DMA0_CH0, p.DMA0_CH1, config).unwrap();

    let tx_msg = b"Hello from LPUART1 DMA TX!\r\n";
    lpuart.write(tx_msg).await.unwrap();

    defmt::info!("TX DMA complete");

    let prompt = b"Type 16 characters to echo via DMA:\r\n";
    lpuart.write(prompt).await.unwrap();

    let mut rx_buf = [0u8; 16];
    lpuart.read(&mut rx_buf).await.unwrap();

    defmt::info!("RX DMA complete");

    let echo_prefix = b"\r\nReceived: ";
    lpuart.write(echo_prefix).await.unwrap();
    lpuart.write(&rx_buf).await.unwrap();
    let done_msg = b"\r\nDone!\r\n";
    lpuart.write(done_msg).await.unwrap();

    defmt::info!("Example complete");
}
