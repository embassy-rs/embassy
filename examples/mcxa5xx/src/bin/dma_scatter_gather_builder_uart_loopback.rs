//! DMA scatter-gather loopback: mem-to-peripheral (gather TX) and
//! peripheral-to-mem (scatter RX) running concurrently over LPUART2.
//!
//! # Features demonstrated:
//! - `PeripheralSegment::memory_to_peripheral()` gather TX segments
//! - `PeripheralSegment::peripheral_to_memory()` scatter RX segments
//! - `build_borrowed()` to start a chain without consuming the channel
//!
//! Connect LPUART2 TX and RX pins together to form a loopback.
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_mcxa::clocks::config::Div8;
use embassy_mcxa::dma::{DmaChannel, DmaRequest, PeripheralPaced, PeripheralSegment, ScatterGatherBuilder};
use embassy_mcxa::lpuart::{Config, Lpuart};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

const PAYLOAD1_SIZE: usize = 4;
const PAYLOAD2_SIZE: usize = 8;
const PAYLOAD3_SIZE: usize = 4;

const PAYLOAD1: [u8; PAYLOAD1_SIZE] = [0x11, 0x22, 0x33, 0x44];
const PAYLOAD2: [u8; PAYLOAD2_SIZE] = [0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC];
const PAYLOAD3: [u8; PAYLOAD3_SIZE] = [0xDD, 0xEE, 0xFF, 0x00];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(cfg);

    defmt::info!("DMA scatter-gather loopback (TX gather + RX scatter, concurrent)");

    let _lpuart = Lpuart::new_blocking(
        p.LPUART2,
        p.P2_10, // TX
        p.P2_11, // RX
        Config {
            baudrate_bps: 115_200,
            // RXWATER must be 0 for DMA-paced RX: RDRF (and therefore the RX DMA
            // request) only asserts while RXCOUNT > RXWATER. With the default
            // watermark of 1 the final byte of a transfer is stranded in the RX
            // FIFO, no DMA request is generated, and the scatter chain stalls
            // with CITER=1.
            rx_fifo_watermark: 0,
            ..Default::default()
        },
    )
    .unwrap();

    // LPUART2 DATA register address for DMA.
    //
    // We access the PAC directly here because the `Lpuart` HAL
    // does not  expose the data register address or DMA enable methods. This is only used for the purpose of this example.
    let data_tx = hal::pac::LPUART2.data().as_ptr() as *mut u8;
    let data_rx = hal::pac::LPUART2.data().as_ptr() as *const u8;

    let mut tx_ch = DmaChannel::new(p.DMA0_CH0);
    let mut rx_ch = DmaChannel::new(p.DMA0_CH1);

    // SAFETY: each channel is used exclusively for one direction of LPUART2.
    unsafe {
        tx_ch.set_request_source(DmaRequest::Lpuart2Tx);
        rx_ch.set_request_source(DmaRequest::Lpuart2Rx);
    }

    // Destination buffers for RX scatter segments
    let mut rx1 = [0u8; PAYLOAD1_SIZE];
    let mut rx2 = [0u8; PAYLOAD2_SIZE];
    let mut rx3 = [0u8; PAYLOAD3_SIZE];

    // Build both scatter-gather chains.
    let mut rx_sg = ScatterGatherBuilder::<u8, PeripheralPaced>::new();
    rx_sg
        .add_transfer_segment(PeripheralSegment::peripheral_to_memory(data_rx, &mut rx1))
        .unwrap()
        .add_transfer_segment(PeripheralSegment::peripheral_to_memory(data_rx, &mut rx2))
        .unwrap()
        .add_transfer_segment(PeripheralSegment::peripheral_to_memory(data_rx, &mut rx3))
        .unwrap();

    let mut tx_sg = ScatterGatherBuilder::<u8, PeripheralPaced>::new();
    tx_sg
        .add_transfer_segment(PeripheralSegment::memory_to_peripheral(&PAYLOAD1, data_tx))
        .unwrap()
        .add_transfer_segment(PeripheralSegment::memory_to_peripheral(&PAYLOAD2, data_tx))
        .unwrap()
        .add_transfer_segment(PeripheralSegment::memory_to_peripheral(&PAYLOAD3, data_tx))
        .unwrap();

    defmt::info!(
        "TX chain: {} segments, RX chain: {} segments",
        tx_sg.segment_count(),
        rx_sg.segment_count()
    );

    // Enable LPUART2 DMA request lines directly via PAC — the HAL does not
    // yet provide a method for this on `Lpuart`. This is only used for the purpose of this example.
    hal::pac::LPUART2.baud().modify(|w| {
        w.set_tdmae(true);
        w.set_rdmae(true);
    });

    // Run TX gather and RX scatter concurrently.
    let rx_fut = async {
        defmt::info!("RX: scatter chain armed");
        let t = rx_sg.build_borrowed(&mut rx_ch).unwrap();
        let res = t.await;
        defmt::info!("RX: transfer done, result ok={}", res.is_ok());
        res
    };

    let tx_fut = async {
        defmt::info!("TX: gather chain starting");
        let t = tx_sg.build_borrowed(&mut tx_ch).unwrap();
        let res = t.await;
        defmt::info!("TX: transfer done, result ok={}", res.is_ok());
        res
    };

    let (rx_res, tx_res) = join(rx_fut, tx_fut).await;

    // Disable LPUART2 DMA request lines.
    hal::pac::LPUART2.baud().modify(|w| {
        w.set_tdmae(false);
        w.set_rdmae(false);
    });

    if let Err(e) = tx_res {
        defmt::error!("TX error: {:?}", e);
    }

    if let Err(e) = rx_res {
        defmt::error!("RX error: {:?}", e);
    }

    // Verify that the received data matches the transmitted data.
    if rx1 != PAYLOAD1 {
        defmt::error!(
            "payload1 mismatch: {=[u8]:x} != {=[u8]:x}",
            rx1.as_slice(),
            PAYLOAD1.as_slice()
        );
    }

    if rx2 != PAYLOAD2 {
        defmt::error!(
            "payload2 mismatch: {=[u8]:x} != {=[u8]:x}",
            rx2.as_slice(),
            PAYLOAD2.as_slice()
        );
    }

    if rx3 != PAYLOAD3 {
        defmt::error!(
            "payload3 mismatch: {=[u8]:x} != {=[u8]:x}",
            rx3.as_slice(),
            PAYLOAD3.as_slice()
        );
    }
}
