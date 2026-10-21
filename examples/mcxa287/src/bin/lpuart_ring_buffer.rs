//! Circular DMA UART RX with overrun recovery.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::lpuart::{Config, Dma, Lpuart, LpuartTx};
use panic_probe as _;
use static_cell::ConstStaticCell;

static RX_RING_BUFFER: ConstStaticCell<[u8; 64]> = ConstStaticCell::new([0; 64]);

async fn write_hex<'a>(tx: &mut LpuartTx<'a, Dma<'a>>, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let buf = [HEX[(byte >> 4) as usize], HEX[(byte & 0x0F) as usize]];
    tx.write(&buf).await.unwrap();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Allow probe-rs to attach.
    for _ in 0..100_000 {
        cortex_m::asm::nop();
    }

    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("LPUART Ring Buffer DMA example starting...");

    let config = Config {
        baudrate_bps: 115_200,
        ..Default::default()
    };

    let lpuart = Lpuart::new_async_with_dma(p.LPUART1, p.P1_9, p.P1_8, p.DMA0_CH1, p.DMA0_CH0, config).unwrap();
    let (mut tx, mut rx) = lpuart.split();

    tx.write(b"LPUART Ring Buffer DMA Example\r\n").await.unwrap();
    tx.write(b"==============================\r\n\r\n").await.unwrap();

    tx.write(b"Setting up circular DMA for UART RX...\r\n").await.unwrap();

    let buf = RX_RING_BUFFER.take();
    let mut ring_buf = rx.into_ring_dma_rx(buf).unwrap();

    tx.write(b"Ring buffer ready! Type characters to see them echoed.\r\n")
        .await
        .unwrap();
    tx.write(b"The DMA continuously receives in the background.\r\n\r\n")
        .await
        .unwrap();

    let mut read_buf = [0u8; 16];
    let mut total_received: usize = 0;

    loop {
        match ring_buf.read(&mut read_buf).await {
            Ok(n) if n > 0 => {
                total_received += n;

                tx.write(b"RX[").await.unwrap();
                for (i, &byte) in read_buf.iter().enumerate().take(n) {
                    write_hex(&mut tx, byte).await;
                    if i < n - 1 {
                        tx.write(b" ").await.unwrap();
                    }
                }
                tx.write(b"]: ").await.unwrap();
                tx.write(&read_buf[..n]).await.unwrap();
                tx.write(b"\r\n").await.unwrap();

                defmt::info!("Received {} bytes, total: {}", n, total_received);
            }
            Ok(_) => {}
            Err(_) => {
                tx.write(b"ERROR: Ring buffer overrun!\r\n").await.unwrap();
                defmt::error!("Ring buffer overrun!");
                ring_buf.clear();
            }
        }
    }
}
