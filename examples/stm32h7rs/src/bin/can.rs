#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::*;
use embassy_stm32::{Config, bind_interrupts, can, rcc};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<FDCAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(25_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    config.rcc.mux.fdcansel = rcc::mux::Fdcansel::HSE;

    let peripherals = embassy_stm32::init(config);

    let mut can = can::CanConfigurator::new(peripherals.FDCAN1, peripherals.PA11, peripherals.PA12, Irqs);

    // 250k bps
    can.set_bitrate(250_000);

    //let mut can = can.into_internal_loopback_mode();
    let mut can = can.into_normal_mode();

    info!("CAN Configured");

    let mut i = 0;
    let mut last_read_ts = embassy_time::Instant::now();

    loop {
        let frame = can::frame::Frame::new_extended(0x123456F, &[i; 8]).unwrap();
        info!("Writing frame");
        _ = can.write(&frame).await;

        match can.read().await {
            Ok(envelope) => {
                let (rx_frame, ts) = envelope.parts();
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;
                info!(
                    "Rx: {:x} {:x} {:x} {:x} --- NEW {}",
                    rx_frame.data()[0],
                    rx_frame.data()[1],
                    rx_frame.data()[2],
                    rx_frame.data()[3],
                    delta,
                )
            }
            Err(_err) => error!("Error in frame"),
        }

        Timer::after_millis(250).await;

        i += 1;
        if i > 3 {
            break;
        }
    }

    let (mut tx, mut rx, _props) = can.split();
    // With split
    loop {
        let frame = can::frame::Frame::new_extended(0x123456F, &[i; 8]).unwrap();
        info!("Writing frame");
        _ = tx.write(&frame).await;

        match rx.read().await {
            Ok(envelope) => {
                let (rx_frame, ts) = envelope.parts();
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;
                info!(
                    "Rx: {:x} {:x} {:x} {:x} --- NEW {}",
                    rx_frame.data()[0],
                    rx_frame.data()[1],
                    rx_frame.data()[2],
                    rx_frame.data()[3],
                    delta,
                )
            }
            Err(_err) => error!("Error in frame"),
        }

        Timer::after_millis(250).await;

        i = i.wrapping_add(1);
    }
}
