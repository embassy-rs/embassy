//! UART DMA peripheral stress test for MCXA5xx.
//!
//! Connect LPUART2 TX and RX pins together to form a loopback.
//!
//! What the test exercises:
//! - Regular DMA loopback transfers.
//! - Scatter-gather DMA loopback transfers.
//! - Dropping in-flight transfers after randomized delays.
//! - Reusing the UART and DMA channels after cancellation.

#![no_std]
#![no_main]

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_futures::select::select;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::clocks::periph_helpers::LpuartClockSel;
use embassy_mcxa::lpuart::{Config, Dma, Error, Lpuart, LpuartRx, LpuartTx};
use embassy_mcxa::trng::{InterruptHandler, Trng};
use embassy_mcxa5xx_examples::util::verify_equal;
use embassy_time::Timer;
use panic_probe as _;

hal::bind_interrupts!(
    struct Irqs {
        TRNG0 => InterruptHandler<hal::peripherals::TRNG0>;
    }
);

const REPORT_EVERY: usize = 1;
const STRESS_PHASES: usize = 100_000;
const REGULAR_PAYLOAD_SIZE: usize = 256;
const SG_PAYLOAD1_SIZE: usize = 37;
const SG_PAYLOAD2_SIZE: usize = 173;
const SG_PAYLOAD3_SIZE: usize = 509;
const SG_PAYLOAD4_SIZE: usize = 83;
const UART_BAUD_BPS: u64 = 3_000_000;
const UART_BITS_PER_BYTE: u64 = 10;
const UART_BYTE_TIME_US: u64 = (UART_BITS_PER_BYTE * 1_000_000 + UART_BAUD_BPS - 1) / UART_BAUD_BPS;
const CANCEL_WINDOW_LOWER_FRACTION: u64 = 10;
const CANCEL_WINDOW_UPPER_FRACTION: u64 = 90;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    cfg.clock_cfg.firc.as_mut().unwrap().fro_hf_div = Some(Div8::no_div());
    let mut p = hal::init(cfg);
    let mut trng = Trng::new_async(p.TRNG0.reborrow(), Irqs, Default::default());

    let lpuart = Lpuart::new_async_with_dma(
        p.LPUART2,
        p.P2_10,
        p.P2_11,
        p.DMA0_CH0,
        p.DMA0_CH1,
        Config {
            source: LpuartClockSel::FroHfDiv,
            baudrate_bps: UART_BAUD_BPS as u32,
            rx_fifo_watermark: 0,
            ..Default::default()
        },
    )
    .unwrap();
    let (mut tx, mut rx) = lpuart.split();

    info!(
        "UART DMA peripheral stress test starting ({=usize} phases)",
        STRESS_PHASES
    );

    for phase in 1..=STRESS_PHASES {
        if phase % REPORT_EVERY == 0 || phase == 1 {
            info!("phase {=usize}: starting", phase);
        }

        if let Err(err) = run_round(&mut tx, &mut rx, &mut trng).await {
            error!("phase {=usize} failed: {:?}", phase, err);
            panic!();
        }
    }

    info!(
        "UART DMA peripheral stress test complete: {=usize} phases passed",
        STRESS_PHASES
    );

    core::future::pending::<()>().await;
}

async fn run_round<'a>(
    tx: &mut LpuartTx<'a, Dma<'a>>,
    rx: &mut LpuartRx<'a, Dma<'a>>,
    trng: &mut Trng<'_, hal::trng::Async>,
) -> Result<(), Error> {
    regular_dma_loopback(tx, rx).await?;
    scatter_gather_dma_loopback(tx, rx).await?;
    cancel_regular_dma_then_recover(tx, rx, trng).await?;
    cancel_scatter_gather_then_recover(tx, rx, trng).await?;

    Ok(())
}

async fn regular_dma_loopback<'a>(tx: &mut LpuartTx<'a, Dma<'a>>, rx: &mut LpuartRx<'a, Dma<'a>>) -> Result<(), Error> {
    let mut tx_buf = [0u8; REGULAR_PAYLOAD_SIZE];
    let mut rx_buf = [0u8; REGULAR_PAYLOAD_SIZE];
    fill_pattern(&mut tx_buf, 0x11);

    let receive = rx.read(&mut rx_buf);
    let transmit = tx.write(&tx_buf);
    let (read_result, write_result) = join(receive, transmit).await;

    read_result?;
    write_result?;
    verify_equal(&rx_buf, &tx_buf, "regular_dma");
    Ok(())
}

async fn scatter_gather_dma_loopback<'a>(
    tx: &mut LpuartTx<'a, Dma<'a>>,
    rx: &mut LpuartRx<'a, Dma<'a>>,
) -> Result<(), Error> {
    let mut rx_buf_1 = [0u8; SG_PAYLOAD1_SIZE];
    let mut rx_buf_2 = [0u8; SG_PAYLOAD2_SIZE];
    let mut rx_buf_3 = [0u8; SG_PAYLOAD3_SIZE];
    let mut rx_buf_4 = [0u8; SG_PAYLOAD4_SIZE];
    let mut tx_buf_1 = [0u8; SG_PAYLOAD1_SIZE];
    let mut tx_buf_2 = [0u8; SG_PAYLOAD2_SIZE];
    let mut tx_buf_3 = [0u8; SG_PAYLOAD3_SIZE];
    let mut tx_buf_4 = [0u8; SG_PAYLOAD4_SIZE];

    fill_pattern(&mut tx_buf_1, 0x21);
    fill_pattern(&mut tx_buf_2, 0x21 + SG_PAYLOAD1_SIZE as u32);
    fill_pattern(&mut tx_buf_3, 0x21 + (SG_PAYLOAD1_SIZE + SG_PAYLOAD2_SIZE) as u32);
    fill_pattern(
        &mut tx_buf_4,
        0x21 + (SG_PAYLOAD1_SIZE + SG_PAYLOAD2_SIZE + SG_PAYLOAD3_SIZE) as u32,
    );

    let receive = rx.read_scatter_gather([
        rx_buf_1.as_mut_slice(),
        rx_buf_2.as_mut_slice(),
        rx_buf_3.as_mut_slice(),
        rx_buf_4.as_mut_slice(),
    ]);
    let transmit = tx.write_scatter_gather([
        tx_buf_1.as_slice(),
        tx_buf_2.as_slice(),
        tx_buf_3.as_slice(),
        tx_buf_4.as_slice(),
    ]);

    let (read_result, write_result) = join(receive, transmit).await;
    read_result?;
    write_result?;

    verify_equal(&rx_buf_1, &tx_buf_1, "scatter_dma[0]");
    verify_equal(&rx_buf_2, &tx_buf_2, "scatter_dma[1]");
    verify_equal(&rx_buf_3, &tx_buf_3, "scatter_dma[2]");
    verify_equal(&rx_buf_4, &tx_buf_4, "scatter_dma[3]");
    Ok(())
}

async fn cancel_regular_dma_then_recover<'a>(
    tx: &mut LpuartTx<'a, Dma<'a>>,
    rx: &mut LpuartRx<'a, Dma<'a>>,
    trng: &mut Trng<'_, hal::trng::Async>,
) -> Result<(), Error> {
    let mut cancel_tx = [0u8; REGULAR_PAYLOAD_SIZE];
    let mut cancel_rx = [0u8; REGULAR_PAYLOAD_SIZE];
    fill_pattern(&mut cancel_tx, 0x31);

    let cancel_delay = random_cancel_delay_us(trng, REGULAR_PAYLOAD_SIZE).await;
    let cancelled_transfer = join(rx.read(&mut cancel_rx), tx.write(&cancel_tx));
    let cancel_timeout = Timer::after_micros(cancel_delay);
    let _ = select(cancelled_transfer, cancel_timeout).await;

    // Wait for the finite loopback tail, then discard it before recovery.
    tx.blocking_flush()?;
    rx.blocking_flush()?;

    let receive = rx.read(&mut cancel_rx);
    let transmit = tx.write(&cancel_tx);
    let (read_result, write_result) = join(receive, transmit).await;

    read_result?;
    write_result?;
    verify_equal(&cancel_rx, &cancel_tx, "cancel_regular_dma");
    Ok(())
}

async fn cancel_scatter_gather_then_recover<'a>(
    tx: &mut LpuartTx<'a, Dma<'a>>,
    rx: &mut LpuartRx<'a, Dma<'a>>,
    trng: &mut Trng<'_, hal::trng::Async>,
) -> Result<(), Error> {
    let mut rx_buf_1 = [0u8; SG_PAYLOAD1_SIZE];
    let mut rx_buf_2 = [0u8; SG_PAYLOAD2_SIZE];
    let mut rx_buf_3 = [0u8; SG_PAYLOAD3_SIZE];
    let mut rx_buf_4 = [0u8; SG_PAYLOAD4_SIZE];
    let mut tx_buf_1 = [0u8; SG_PAYLOAD1_SIZE];
    let mut tx_buf_2 = [0u8; SG_PAYLOAD2_SIZE];
    let mut tx_buf_3 = [0u8; SG_PAYLOAD3_SIZE];
    let mut tx_buf_4 = [0u8; SG_PAYLOAD4_SIZE];

    fill_pattern(&mut tx_buf_1, 0x41);
    fill_pattern(&mut tx_buf_2, 0x41 + SG_PAYLOAD1_SIZE as u32);
    fill_pattern(&mut tx_buf_3, 0x41 + (SG_PAYLOAD1_SIZE + SG_PAYLOAD2_SIZE) as u32);
    fill_pattern(
        &mut tx_buf_4,
        0x41 + (SG_PAYLOAD1_SIZE + SG_PAYLOAD2_SIZE + SG_PAYLOAD3_SIZE) as u32,
    );

    let cancel_delay = random_cancel_delay_us(
        trng,
        SG_PAYLOAD1_SIZE + SG_PAYLOAD2_SIZE + SG_PAYLOAD3_SIZE + SG_PAYLOAD4_SIZE,
    )
    .await;
    let cancelled_transfer = join(
        rx.read_scatter_gather([
            rx_buf_1.as_mut_slice(),
            rx_buf_2.as_mut_slice(),
            rx_buf_3.as_mut_slice(),
            rx_buf_4.as_mut_slice(),
        ]),
        tx.write_scatter_gather([
            tx_buf_1.as_slice(),
            tx_buf_2.as_slice(),
            tx_buf_3.as_slice(),
            tx_buf_4.as_slice(),
        ]),
    );
    let cancel_timeout = Timer::after_micros(cancel_delay);
    let _ = select(cancelled_transfer, cancel_timeout).await;

    // Wait for the finite loopback tail, then discard it before recovery.
    tx.blocking_flush()?;
    rx.blocking_flush()?;

    // info!("recovering from cancelled scatter-gather transfer");

    let receive = rx.read_scatter_gather([
        rx_buf_1.as_mut_slice(),
        rx_buf_2.as_mut_slice(),
        rx_buf_3.as_mut_slice(),
        rx_buf_4.as_mut_slice(),
    ]);
    let transmit = tx.write_scatter_gather([
        tx_buf_1.as_slice(),
        tx_buf_2.as_slice(),
        tx_buf_3.as_slice(),
        tx_buf_4.as_slice(),
    ]);

    let (read_result, write_result) = join(receive, transmit).await;
    read_result?;
    write_result?;

    verify_equal(&rx_buf_1, &tx_buf_1, "cancel_scatter_dma[0]");
    verify_equal(&rx_buf_2, &tx_buf_2, "cancel_scatter_dma[1]");
    verify_equal(&rx_buf_3, &tx_buf_3, "cancel_scatter_dma[2]");
    verify_equal(&rx_buf_4, &tx_buf_4, "cancel_scatter_dma[3]");
    Ok(())
}

async fn random_cancel_delay_us(trng: &mut Trng<'_, hal::trng::Async>, payload_len: usize) -> u64 {
    let total_time_us = (payload_len as u64).saturating_mul(UART_BYTE_TIME_US);
    let lower_bound_us = (total_time_us * CANCEL_WINDOW_LOWER_FRACTION / 100).max(1);
    let upper_bound_us = (total_time_us * CANCEL_WINDOW_UPPER_FRACTION / 100).max(1);
    let span = upper_bound_us.saturating_sub(lower_bound_us);
    let random_word = trng.async_next_u32().await.unwrap_or(0) as u64;
    lower_bound_us + (random_word % span.max(1))
}

fn fill_pattern(buffer: &mut [u8], seed: u32) {
    for (idx, byte) in buffer.iter_mut().enumerate() {
        *byte = seed.wrapping_add(idx as u32) as u8;
    }
}
