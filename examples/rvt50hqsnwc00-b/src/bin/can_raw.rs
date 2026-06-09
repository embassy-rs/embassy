#![no_std]
#![no_main]

//! Raw FDCAN demo for the Riverdi RVT50 CAN connector (P5).
//!
//! Press the user button (`PH3`) to transmit the press count on CAN ID `0x123`.
//! The user LED (`PE5`) flashes on each press. Received frames are logged on RTT.

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rvt50hqsnwc00_b_examples::rvt50_board::{self, CAN_BITRATE, CAN_BUTTON_FRAME_ID};
use embassy_stm32::can::{self, CanRx, CanTx};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::Peripherals;
use embassy_time::Timer;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static CAN_TX: StaticCell<CanTx<'static>> = StaticCell::new();
static CAN_RX: StaticCell<CanRx<'static>> = StaticCell::new();

#[embassy_executor::task]
async fn button_task(mut button: ExtiInput<'static, Async>, mut led: Output<'static>, tx: &'static mut CanTx<'static>) {
    let mut count = 0u32;

    loop {
        button.wait_for_rising_edge().await;
        button.wait_for_falling_edge().await;

        count = count.wrapping_add(1);

        let payload = count.to_le_bytes();
        let frame = can::frame::Frame::new_standard(CAN_BUTTON_FRAME_ID, &payload).unwrap();
        _ = tx.write(&frame).await;
        info!("Button pressed, TX count={}", count);

        led.set_high();
        Timer::after_millis(100).await;
        led.set_low();
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
        PH3,
        EXTI3,
        PE5,
        ..
    } = p;

    let mut can = rvt50_board::init_can(FDCAN1, PB8, PB9, PI6);
    can.set_bitrate(CAN_BITRATE);
    let can = can.into_normal_mode();
    let (tx, rx, _) = can.split();

    let tx = CAN_TX.init(tx);
    let rx = CAN_RX.init(rx);

    let button = rvt50_board::init_user_button(PH3, EXTI3);
    let led = rvt50_board::init_user_led(PE5);

    spawner.spawn(unwrap!(button_task(button, led, tx)));

    info!(
        "RVT50 CAN demo: {} sends count on 0x{:x}, {} blinks on press",
        rvt50_board::pins::USER_BUTTON,
        CAN_BUTTON_FRAME_ID,
        rvt50_board::pins::USER_LED,
    );

    loop {
        if let Ok(envelope) = rx.read().await {
            let (rx_frame, _) = envelope.parts();
            let data = rx_frame.data();
            info!(
                "RX len={} data={:x} {:x} {:x} {:x}",
                data.len(),
                data.first().copied().unwrap_or(0),
                data.get(1).copied().unwrap_or(0),
                data.get(2).copied().unwrap_or(0),
                data.get(3).copied().unwrap_or(0),
            );
        }
    }
}
