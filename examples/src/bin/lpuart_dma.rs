//! LPUART DMA example for MCXA276.
//!
//! This example demonstrates using DMA for UART TX and RX operations.
//! It sends a message using DMA, then waits for 16 characters to be received
//! via DMA and echoes them back.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::clocks::Gate;
use embassy_mcxa::dma::{self, DMA_REQ_LPUART2_RX, DMA_REQ_LPUART2_TX};
use embassy_mcxa::lpuart::{Config, LpuartDma};
use embassy_mcxa::pac;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

// DMA interrupt handlers
#[no_mangle]
pub extern "C" fn DMA_CH0() {
    unsafe { dma::on_interrupt(0) };
}

#[no_mangle]
pub extern "C" fn DMA_CH1() {
    unsafe { dma::on_interrupt(1) };
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("LPUART DMA example starting...");

    // Enable DMA0 clock and release reset
    unsafe {
        hal::peripherals::DMA0::enable_clock();
        hal::peripherals::DMA0::release_reset();
    }

    // Get PAC peripherals for DMA init
    let pac_periphs = unsafe { pac::Peripherals::steal() };

    // Initialize DMA
    unsafe {
        dma::init(&pac_periphs);
    }

    // Get EDMA TCD register block for transfers
    let edma = &pac_periphs.edma_0_tcd0;

    // Enable DMA interrupts
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH0);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA_CH1);
    }

    // Create UART configuration
    let config = Config {
        baudrate_bps: 115_200,
        enable_tx: true,
        enable_rx: true,
        ..Default::default()
    };

    // Create UART instance with DMA channels
    let mut lpuart = LpuartDma::new(
        p.LPUART2,
        p.P2_2,    // TX pin
        p.P2_3,    // RX pin
        p.DMA_CH0, // TX DMA channel
        p.DMA_CH1, // RX DMA channel
        config,
    )
    .unwrap();

    // Send a message using DMA
    let tx_msg = b"Hello from LPUART2 DMA TX!\r\n";
    lpuart
        .write_dma(edma, DMA_REQ_LPUART2_TX, tx_msg)
        .await
        .unwrap();

    defmt::info!("TX DMA complete");

    // Send prompt
    let prompt = b"Type 16 characters to echo via DMA:\r\n";
    lpuart
        .write_dma(edma, DMA_REQ_LPUART2_TX, prompt)
        .await
        .unwrap();

    // Receive 16 characters using DMA
    let mut rx_buf = [0u8; 16];
    lpuart
        .read_dma(edma, DMA_REQ_LPUART2_RX, &mut rx_buf)
        .await
        .unwrap();

    defmt::info!("RX DMA complete");

    // Echo back the received data
    let echo_prefix = b"\r\nReceived: ";
    lpuart
        .write_dma(edma, DMA_REQ_LPUART2_TX, echo_prefix)
        .await
        .unwrap();
    lpuart
        .write_dma(edma, DMA_REQ_LPUART2_TX, &rx_buf)
        .await
        .unwrap();
    let done_msg = b"\r\nDone!\r\n";
    lpuart
        .write_dma(edma, DMA_REQ_LPUART2_TX, done_msg)
        .await
        .unwrap();

    defmt::info!("Example complete");

    loop {
        cortex_m::asm::wfe();
    }
}

