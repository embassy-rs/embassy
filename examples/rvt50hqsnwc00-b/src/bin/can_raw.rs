#![no_std]
#![no_main]

//! Raw FDCAN demo for the Riverdi RVT50 CAN connector (P5).
//!
//! Periodically transmits an 8-byte test pattern on CAN ID `0x123`. Listens on
//! CAN ID `0x124` for LED state (byte 0 bit 0) and drives the user LED (`PE5`).

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{
    self, can_frame_standard_id, can_led_state_from_payload, can_pattern_payload, CAN_BITRATE,
    CAN_LED_STATE_RX_ID, CAN_PATTERN_INTERVAL_MS, CAN_PATTERN_TX_ID,
};
use embassy_stm32::can::{self, CanRx, CanTx};
use embassy_stm32::gpio::Output;
use embassy_stm32::Peripherals;
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static CAN_TX: StaticCell<CanTx<'static>> = StaticCell::new();
static CAN_RX: StaticCell<CanRx<'static>> = StaticCell::new();

#[embassy_executor::task]
async fn pattern_tx_task(tx: &'static mut CanTx<'static>) {
    let mut seq = 0u8;

    loop {
        let payload = can_pattern_payload(seq);
        let frame = can::frame::Frame::new_standard(CAN_PATTERN_TX_ID, &payload).unwrap();
        _ = tx.write(&frame).await;
        info!(
            "TX pattern id=0x{:x} seq={} walk=0x{:x}",
            CAN_PATTERN_TX_ID,
            seq,
            payload[7],
        );

        seq = seq.wrapping_add(1);
        Timer::after_millis(CAN_PATTERN_INTERVAL_MS).await;
    }
}

#[embassy_executor::task]
async fn led_rx_task(mut led: Output<'static>, rx: &'static mut CanRx<'static>) {
    loop {
        if let Ok(envelope) = rx.read().await {
            let (rx_frame, _) = envelope.parts();
            let data = rx_frame.data();

            match can_frame_standard_id(&rx_frame) {
                Some(CAN_LED_STATE_RX_ID) => {
                    if let Some(on) = can_led_state_from_payload(data) {
                        if on {
                            led.set_high();
                        } else {
                            led.set_low();
                        }
                        info!("RX LED state id=0x{:x} on={}", CAN_LED_STATE_RX_ID, on);
                    }
                }
                Some(id) => {
                    info!(
                        "RX id=0x{:x} len={} data={:x} {:x} {:x} {:x}",
                        id,
                        data.len(),
                        data.first().copied().unwrap_or(0),
                        data.get(1).copied().unwrap_or(0),
                        data.get(2).copied().unwrap_or(0),
                        data.get(3).copied().unwrap_or(0),
                    );
                }
                None => info!("RX extended/unknown id len={}", data.len()),
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = rvt50_board::init_clocks();

    let Peripherals {
        FDCAN1,
        PB8,
        PB9,
        PI6,
        PE5,
        ..
    } = p;

    let mut can = rvt50_board::init_can(FDCAN1, PB8, PB9, PI6);
    can.set_bitrate(CAN_BITRATE);
    let can = can.into_normal_mode();
    let (tx, rx, _) = can.split();

    let tx = CAN_TX.init(tx);
    let rx = CAN_RX.init(rx);
    let led = rvt50_board::init_user_led(PE5);

    spawner.spawn(unwrap!(pattern_tx_task(tx)));
    spawner.spawn(unwrap!(led_rx_task(led, rx)));

    info!(
        "RVT50 CAN demo: TX pattern 0x{:x} every {}ms, RX LED state 0x{:x} -> {}",
        CAN_PATTERN_TX_ID,
        CAN_PATTERN_INTERVAL_MS,
        CAN_LED_STATE_RX_ID,
        rvt50_board::pins::USER_LED,
    );

    loop {
        Timer::after_secs(60).await;
    }
}
