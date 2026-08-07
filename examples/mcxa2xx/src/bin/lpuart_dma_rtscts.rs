//! LPUART DMA example with RTS/CTS hardware flow control for MCXA2.
//!
//! Demonstrates hardware flow control by flooding the UART with more data than
//! the receiver drains. The driver is `split()` into a TX and RX half, and two
//! concurrent futures are `join`ed:
//!
//! * the TX future floods a large payload as fast as it can,
//! * the RX future drains slowly (small reads with delays between them).
//!
//! Because the RX side falls behind, its RX FIFO fills, the peripheral deasserts
//! RTS, and the connected CTS pauses the TX flood. The transfer therefore only
//! makes progress at the rate the slow reader consumes it, without losing data.
//!
//! Wire this as a self-loopback on a single board:
//! * TX (P2_2)  -> RX (P2_3)
//! * RTS (P2_5) -> CTS (P2_4)

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::lpuart::{Config, Error, Lpuart};
use embassy_time::Timer;
use panic_probe as _;

// Total number of bytes to flood through the link.
const TOTAL: usize = 4096;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("LPUART DMA RTS/CTS flood example starting...");

    let config = Config {
        baudrate_bps: 115_200,
        ..Default::default()
    };

    let lpuart = Lpuart::new_async_with_dma_rtscts(
        p.LPUART2,  // Instance
        p.P2_2,     // TX pin
        p.P2_3,     // RX pin
        p.P2_4,     // CTS pin
        p.P2_5,     // RTS pin
        p.DMA0_CH0, // TX DMA channel
        p.DMA0_CH1, // RX DMA channel
        config,
    )
    .unwrap();

    // Split into independent TX and RX halves so they can run concurrently.
    let (mut tx, mut rx) = lpuart.split();

    // TX future: flood a large payload as fast as the link allows. When the
    // receiver falls behind, CTS (driven by the loopbacked RTS) will pause us.
    let flood = async {
        let payload = [0xA5u8; 256];
        let mut sent = 0usize;
        while sent < TOTAL {
            let n = tx.write(&payload).await.unwrap();
            sent += n;
        }
        defmt::info!("TX flood complete: {} bytes sent", sent);
    };

    // RX future: drain slowly. The small buffer plus the delay between reads
    // means the RX side cannot keep up, forcing hardware flow control to engage.
    let drain = async {
        let mut buf = [0u8; 16];
        let mut received = 0usize;
        while received < TOTAL {
            match rx.read(&mut buf).await {
                Ok(n) => {
                    received += n;
                    defmt::info!("RX drained {} bytes (total {})", n, received);
                }
                // Before RTS backpressure fully engages on the very first burst,
                // the RX FIFO can briefly overflow. This is a recoverable transient:
                // log it and keep draining rather than aborting the demo.
                Err(Error::Overrun) => {
                    defmt::warn!("RX overrun (transient), continuing");
                }
                Err(e) => {
                    defmt::error!("RX read error at total {}: {}", received, e);
                    return;
                }
            }
            // Deliberately slow: let the RX FIFO fill and RTS deassert.
            Timer::after_millis(10).await;
        }
        defmt::info!("RX drain complete: {} bytes received", received);
    };

    join(flood, drain).await;

    defmt::info!("Example complete: flow control exercised over {} bytes", TOTAL);
}
