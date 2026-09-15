//! LPUART BBQueue continuous DMA test between two UART instances on MCXA577.
//!
//! Wire the UARTs together on a single board:
//! * LPUART3 TX (P3_1) -> LPUART1 RX (P1_8)
//! * LPUART1 TX (P1_9) -> LPUART3 RX (P3_0)
//!
//! LPUART1 repeatedly sends 0x55. LPUART3 verifies it, then sends 0xaa
//! back for LPUART1 to verify. Both receive sides use continuous DMA.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::clocks::periph_helpers::LpuartClockSel;
use embassy_mcxa::dma::DmaChannel;
use embassy_mcxa::lpuart::{BbqConfig, BbqParts, BbqRxMode, LpuartBbq};
use embassy_mcxa::{bind_interrupts, lpuart};
use embassy_time::Timer;
use embedded_io_async::{Read, Write};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

const TX_BUFLEN: usize = 4096;
const RX_BUFLEN: usize = 16384;
const TRANSFER_LEN: usize = 1024;
const DMA_HALF_SIZE: usize = 1024;

bind_interrupts!(struct Irqs {
    LPUART3 => lpuart::BbqInterruptHandler::<hal::peripherals::LPUART3>;
    LPUART1 => lpuart::BbqInterruptHandler::<hal::peripherals::LPUART1>;
});

static LPUART3_TX_BUF: ConstStaticCell<[u8; TX_BUFLEN]> = ConstStaticCell::new([0; TX_BUFLEN]);
static LPUART3_RX_BUF: ConstStaticCell<[u8; RX_BUFLEN]> = ConstStaticCell::new([0; RX_BUFLEN]);
static LPUART1_TX_BUF: ConstStaticCell<[u8; TX_BUFLEN]> = ConstStaticCell::new([0; TX_BUFLEN]);
static LPUART1_RX_BUF: ConstStaticCell<[u8; RX_BUFLEN]> = ConstStaticCell::new([0; RX_BUFLEN]);

#[embassy_executor::task]
async fn lpuart3_task(mut uart: LpuartBbq) {
    loop {
        let mut rx_buf = [0; TRANSFER_LEN];
        uart.read_exact(&mut rx_buf).await.unwrap();
        assert!(rx_buf.iter().all(|b| *b == 0x55));
        info!("LPUART3 read {} bytes of 0x55", rx_buf.len());

        Timer::after_millis(10).await;

        let tx_buf = [0xaa; TRANSFER_LEN];
        uart.write_all(&tx_buf).await.unwrap();
        uart.flush().await;
        info!("LPUART3 wrote {} bytes of 0xaa", tx_buf.len());
    }
}

#[embassy_executor::task]
async fn lpuart1_task(mut uart: LpuartBbq) {
    loop {
        let tx_buf = [0x55; TRANSFER_LEN];
        uart.write_all(&tx_buf).await.unwrap();
        uart.flush().await;
        info!("LPUART1 wrote {} bytes of 0x55", tx_buf.len());

        Timer::after_millis(10).await;

        let mut rx_buf = [0; TRANSFER_LEN];
        uart.read_exact(&mut rx_buf).await.unwrap();
        assert!(rx_buf.iter().all(|b| *b == 0xaa));
        info!("LPUART1 read {} bytes of 0xaa", rx_buf.len());
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    info!("Dual LPUART BBQueue continuous DMA test start");

    let mut config = BbqConfig::default();
    config.baudrate_bps = 115_200;
    config.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    config.source = LpuartClockSel::FroLfDiv;

    let lpuart3_parts = BbqParts::new(
        p.LPUART3,
        Irqs,
        p.P3_1,
        LPUART3_TX_BUF.take(),
        DmaChannel::new(p.DMA0_CH4),
        p.P3_0,
        LPUART3_RX_BUF.take(),
        DmaChannel::new(p.DMA0_CH5),
    )
    .unwrap();
    let lpuart3 = LpuartBbq::new(
        lpuart3_parts,
        config,
        BbqRxMode::Continuous {
            half_size: DMA_HALF_SIZE,
        },
    )
    .unwrap();

    let lpuart1_parts = BbqParts::new(
        p.LPUART1,
        Irqs,
        p.P1_9,
        LPUART1_TX_BUF.take(),
        DmaChannel::new(p.DMA0_CH0),
        p.P1_8,
        LPUART1_RX_BUF.take(),
        DmaChannel::new(p.DMA0_CH1),
    )
    .unwrap();
    let lpuart1 = LpuartBbq::new(
        lpuart1_parts,
        config,
        BbqRxMode::Continuous {
            half_size: DMA_HALF_SIZE,
        },
    )
    .unwrap();

    spawner.spawn(lpuart3_task(lpuart3).unwrap());
    spawner.spawn(lpuart1_task(lpuart1).unwrap());
}
