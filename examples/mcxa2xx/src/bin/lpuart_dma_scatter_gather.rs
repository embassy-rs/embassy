//! DMA scatter-gather LPUART2 loopback test.
//!
//! Connect LPUART2 TX and RX pins together to form a loopback.
#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_mcxa as hal;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::lpuart::{Config, Lpuart};
use panic_probe as _;

const PAYLOAD1_SIZE: usize = 37;
const PAYLOAD2_SIZE: usize = 173;
const PAYLOAD3_SIZE: usize = 509;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("DMA scatter-gather loopback test");

    let lpuart = Lpuart::new_async_with_dma(
        p.LPUART2,
        p.P2_10,
        p.P2_11,
        p.DMA0_CH0,
        p.DMA0_CH1,
        Config {
            baudrate_bps: 115_200,
            rx_fifo_watermark: 0,
            ..Default::default()
        },
    )
    .unwrap();

    let (mut tx, mut rx) = lpuart.split();

    let mut round = 0u32;
    loop {
        round = round.wrapping_add(1);
        defmt::info!("round {}: starting scatter-gather loopback", round);

        let seed = round;
        let mut rx_buffer_1 = [0u8; PAYLOAD1_SIZE];
        let mut rx_buffer_2 = [0u8; PAYLOAD2_SIZE];
        let mut rx_buffer_3 = [0u8; PAYLOAD3_SIZE];
        let mut tx_buffer_1 = [0u8; PAYLOAD1_SIZE];
        let mut tx_buffer_2 = [0u8; PAYLOAD2_SIZE];
        let mut tx_buffer_3 = [0u8; PAYLOAD3_SIZE];

        fill_pattern(&mut tx_buffer_1, seed);
        fill_pattern(&mut tx_buffer_2, seed.wrapping_add(PAYLOAD1_SIZE as u32));
        fill_pattern(
            &mut tx_buffer_3,
            seed.wrapping_add((PAYLOAD1_SIZE + PAYLOAD2_SIZE) as u32),
        );

        let receive = rx.read_scatter_gather([
            rx_buffer_1.as_mut_slice(),
            rx_buffer_2.as_mut_slice(),
            rx_buffer_3.as_mut_slice(),
        ]);
        let transmit =
            tx.write_scatter_gather([tx_buffer_1.as_slice(), tx_buffer_2.as_slice(), tx_buffer_3.as_slice()]);
        let (received, sent) = join(receive, transmit).await;
        received.unwrap();
        sent.unwrap();

        verify(
            &rx_buffer_1,
            &rx_buffer_2,
            &rx_buffer_3,
            &tx_buffer_1,
            &tx_buffer_2,
            &tx_buffer_3,
        )
        .unwrap();

        defmt::info!(
            "loopback verified: {} bytes ({})",
            PAYLOAD1_SIZE + PAYLOAD2_SIZE + PAYLOAD3_SIZE,
            round
        );
    }
}

fn fill_pattern(buffer: &mut [u8], seed: u32) {
    for (offset, byte) in buffer.iter_mut().enumerate() {
        *byte = seed.wrapping_add(offset as u32) as u8;
    }
}

#[derive(defmt::Format, Debug)]
enum VerifyError {
    Payload1Mismatch,
    Payload2Mismatch,
    Payload3Mismatch,
}

/// Check the received buffers against the transmitted payloads.
fn verify(
    rx1: &[u8; PAYLOAD1_SIZE],
    rx2: &[u8; PAYLOAD2_SIZE],
    rx3: &[u8; PAYLOAD3_SIZE],
    payload1: &[u8; PAYLOAD1_SIZE],
    payload2: &[u8; PAYLOAD2_SIZE],
    payload3: &[u8; PAYLOAD3_SIZE],
) -> Result<(), VerifyError> {
    if rx1 != payload1 {
        defmt::error!(
            "payload1 mismatch: {=[u8]:x} != {=[u8]:x}",
            rx1.as_slice(),
            payload1.as_slice()
        );
        return Err(VerifyError::Payload1Mismatch);
    }

    if rx2 != payload2 {
        defmt::error!(
            "payload2 mismatch: {=[u8]:x} != {=[u8]:x}",
            rx2.as_slice(),
            payload2.as_slice()
        );
        return Err(VerifyError::Payload2Mismatch);
    }

    if rx3 != payload3 {
        defmt::error!(
            "payload3 mismatch: {=[u8]:x} != {=[u8]:x}",
            rx3.as_slice(),
            payload3.as_slice()
        );
        return Err(VerifyError::Payload3Mismatch);
    }

    defmt::info!("All payloads verified successfully");

    Ok(())
}
