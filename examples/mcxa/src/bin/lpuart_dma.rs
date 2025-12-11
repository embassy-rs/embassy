//! LPUART DMA example for MCXA276.
//!
//! This example demonstrates using DMA for UART TX and RX operations.
//! It sends a message using DMA, then waits for 16 characters to be received
//! via DMA and echoes them back.
//!
//! The DMA request sources are automatically derived from the LPUART instance type.
//! DMA clock/reset/init is handled automatically by the HAL.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::lpuart::{Config, LpuartDma};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("LPUART DMA example starting...");

    // Create UART configuration
    let config = Config {
        baudrate_bps: 115_200,
        ..Default::default()
    };

    // Create UART instance with DMA channels
    let mut lpuart = LpuartDma::new(
        p.LPUART2, // Instance
        p.P2_2,    // TX pin
        p.P2_3,    // RX pin
        p.DMA_CH0, // TX DMA channel
        p.DMA_CH1, // RX DMA channel
        config,
    )
    .unwrap();

    // Send a message using DMA (DMA request source is automatically derived from LPUART2)
    let tx_msg = b"Hello from LPUART2 DMA TX!\r\n";
    lpuart.write_dma(tx_msg).await.unwrap();

    defmt::info!("TX DMA complete");

    // Send prompt
    let prompt = b"Type 16 characters to echo via DMA:\r\n";
    lpuart.write_dma(prompt).await.unwrap();

    // Receive 16 characters using DMA
    let mut rx_buf = [0u8; 16];
    lpuart.read_dma(&mut rx_buf).await.unwrap();

    defmt::info!("RX DMA complete");

    // Echo back the received data
    let echo_prefix = b"\r\nReceived: ";
    lpuart.write_dma(echo_prefix).await.unwrap();
    lpuart.write_dma(&rx_buf).await.unwrap();
    let done_msg = b"\r\nDone!\r\n";
    lpuart.write_dma(done_msg).await.unwrap();

    defmt::info!("Example complete");
}
