#![no_std]
#![no_main]

//! Raw FDCAN demo for the Riverdi RVT50 CAN connector (P5).
//!
//! Transmits classic CAN frames on `PB9` and prints any received frames on `PB8`.
//! Connect a second CAN node or USB-CAN adapter to the P5 header for loopback testing.

use defmt::info;
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{self, CAN_BITRATE};
use embassy_stm32::can;
use embassy_stm32::Peripherals;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = rvt50_board::init_clocks();

    let Peripherals {
        FDCAN1, PB8, PB9, PI6, ..
    } = p;

    let mut can = rvt50_board::init_can(FDCAN1, PB8, PB9, PI6);
    can.set_bitrate(CAN_BITRATE);
    let mut can = can.into_normal_mode();

    info!("RVT50 raw CAN demo at {} bps", CAN_BITRATE);

    let mut seq = 0u8;
    loop {
        let frame = can::frame::Frame::new_standard(0x123, &[seq, seq.wrapping_add(1), 0xAA, 0x55]).unwrap();
        info!("TX id=0x123 data={:x}", seq);
        _ = can.write(&frame).await;

        if let Ok(envelope) = can.read().await {
            let (rx_frame, _) = envelope.parts();
            let data = rx_frame.data();
            info!(
                "RX len={} data0={:x} data1={:x}",
                data.len(),
                data.first().copied().unwrap_or(0),
                data.get(1).copied().unwrap_or(0),
            );
        }

        seq = seq.wrapping_add(1);
        Timer::after_millis(500).await;
    }
}
