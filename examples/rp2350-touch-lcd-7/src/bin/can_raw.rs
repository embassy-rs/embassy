#![no_std]
#![no_main]

use defmt::info;
use embassy_rp2350_touch_lcd_7_examples::board::{self, CAN_BITRATE};
use embassy_rp2350_touch_lcd_7_examples::can_driver;
use embassy_rp2350_touch_lcd_7_examples::xl2515::CanSpi;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) -> ! {
    let p = board::init();
    board::log_board_info();

    let can = CanSpi::new(p.SPI0, p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5);
    can_driver::install(can, CAN_BITRATE).await;

    let mut seq = 0u8;
    info!("CAN raw demo — TX id 0x123 every 500 ms @ {} bit/s", CAN_BITRATE);

    loop {
        let payload = [seq, 0xDE, 0xAD, 0xBE, 0xEF, 0x55, 0xAA, 1 << (seq % 8)];
        let _ = can_driver::with_can(|c| c.send_standard(0x123, &payload));

        if seq % 10 == 0 {
            info!(
                "CAN TX id=0x123 seq={} data[0]={:02x} @ {} bit/s",
                seq,
                payload[0],
                CAN_BITRATE
            );
        }

        if let Some((id, len, data)) = can_driver::with_can(|c| c.try_receive()).flatten() {
            info!("RX id=0x{:03x} len={} data[0]={:02x}", id, len, data[0]);
        }

        seq = seq.wrapping_add(1);
        Timer::after(Duration::from_millis(500)).await;
    }
}
