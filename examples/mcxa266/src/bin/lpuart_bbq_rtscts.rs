//! LPUART BBQueue example with RTS/CTS hardware flow control for MCXA2.
//!
//! Demonstrates hardware flow control by flooding the UART with more data than
//! the receiver drains. The `LpuartBbq` is `split()` into a TX and RX half, and
//! two concurrent futures are `join`ed:
//!
//! * the TX future floods a large payload as fast as it can,
//! * the RX future drains slowly (small reads with delays between them).
//!
//! Because the RX side falls behind, its ring buffer fills, the peripheral
//! deasserts RTS, and the connected CTS pauses the TX flood. The transfer only
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
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::clocks::periph_helpers::LpuartClockSel;
use embassy_mcxa::dma::DmaChannel;
use embassy_mcxa::lpuart::{BbqConfig, BbqParts, BbqRxMode, LpuartBbq};
use embassy_mcxa::{bind_interrupts, lpuart};
use embassy_time::Timer;
use panic_probe as _;
use static_cell::ConstStaticCell;

bind_interrupts!(struct Irqs {
    LPUART2 => lpuart::BbqInterruptHandler::<hal::peripherals::LPUART2>;
});

const SIZE: usize = 1024;
static TX_BUF: ConstStaticCell<[u8; SIZE]> = ConstStaticCell::new([0u8; SIZE]);
static RX_BUF: ConstStaticCell<[u8; SIZE]> = ConstStaticCell::new([0u8; SIZE]);

// Total number of bytes to flood through the link.
const TOTAL: usize = 4096;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("LPUART BBQ RTS/CTS flood example starting...");

    let mut config = BbqConfig::default();
    config.baudrate_bps = 115_200;
    config.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    config.source = LpuartClockSel::FroLfDiv;

    let tx_buf = TX_BUF.take();
    let rx_buf = RX_BUF.take();

    let parts = BbqParts::new_with_rtscts(
        p.LPUART2, // Instance
        Irqs,
        p.P2_2, // TX pin
        tx_buf,
        DmaChannel::new(p.DMA0_CH0), // TX DMA channel
        p.P2_3,                      // RX pin
        rx_buf,
        DmaChannel::new(p.DMA0_CH1), // RX DMA channel
        p.P2_4,                      // CTS pin
        p.P2_5,                      // RTS pin
    )
    .unwrap();

    let uart = LpuartBbq::new(parts, config, BbqRxMode::Efficiency).unwrap();

    // Split into independent TX and RX halves so they can run concurrently.
    let (mut tx, mut rx) = uart.split();

    // TX future: flood a large payload as fast as the link allows. When the
    // receiver falls behind, CTS (driven by the loopbacked RTS) will pause us.
    let flood = async {
        let payload = [0xA5u8; 256];
        let mut sent = 0usize;
        while sent < TOTAL {
            // `write` may accept only part of the slice, so advance a window.
            let mut window = payload.as_slice();
            while !window.is_empty() && sent < TOTAL {
                let n = tx.write(window).await.unwrap();
                window = &window[n..];
                sent += n;
            }
        }
        tx.flush().await;
        defmt::info!("TX flood complete: {} bytes sent", sent);
    };

    // RX future: drain slowly. The small buffer plus the delay between reads
    // means the RX side cannot keep up, forcing hardware flow control to engage.
    let drain = async {
        let mut buf = [0u8; 16];
        let mut received = 0usize;
        while received < TOTAL {
            let n = rx.read(&mut buf).await.unwrap();
            received += n;
            defmt::info!("RX drained {} bytes (total {})", n, received);
            // Deliberately slow: let the RX ring fill and RTS deassert.
            Timer::after_millis(10).await;
        }
        defmt::info!("RX drain complete: {} bytes received", received);
    };

    join(flood, drain).await;

    defmt::info!("Example complete: flow control exercised over {} bytes", TOTAL);
}
