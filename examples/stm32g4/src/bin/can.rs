#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::*;
use embassy_stm32::{bind_interrupts, can, Config};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<FDCAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();

    let peripherals = embassy_stm32::init(config);

    let mut can = can::Fdcan::new(peripherals.FDCAN1, peripherals.PA11, peripherals.PA12, Irqs);

    // 250k bps
    can.set_bitrate(250_000);

    info!("Configured");

    //let mut can = can.into_external_loopback_mode();
    let mut can = can.into_normal_mode();

    let mut i = 0;
    loop {
        let frame = can::TxFrame::new(
            can::TxFrameHeader {
                len: 1,
                frame_format: can::FrameFormat::Standard,
                id: can::StandardId::new(0x123).unwrap().into(),
                bit_rate_switching: false,
                marker: None,
            },
            &[i],
        )
        .unwrap();
        info!("Writing frame");
        _ = can.write(&frame).await;

        match can.read().await {
            Ok(rx_frame) => info!("Rx: {}", rx_frame.data()[0]),
            Err(_err) => error!("Error in frame"),
        }

        Timer::after_millis(250).await;

        i += 1;
    }
}
