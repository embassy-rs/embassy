#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::can::filter::Mask32;
use embassy_stm32::can::{
    Can, Fifo, Frame, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, StandardId, TxInterruptHandler,
};
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::CAN1;
use embassy_time::Instant;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut p = embassy_stm32::init(Default::default());

    // The next two lines are a workaround for testing without transceiver.
    // To synchronise to the bus the RX input needs to see a high level.
    // Use `mem::forget()` to release the borrow on the pin but keep the
    // pull-up resistor enabled.
    let rx_pin = Input::new(p.PA11.reborrow(), Pull::Up);
    core::mem::forget(rx_pin);

    let mut can = Can::new(p.CAN1, p.PA11, p.PA12, Irqs);

    can.modify_filters().enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.modify_config()
        .set_loopback(true) // Receive own frames
        .set_silent(true)
        .set_bitrate(1_000_000);

    can.enable().await;

    let mut i: u8 = 0;
    loop {
        let tx_frame = Frame::new_data(unwrap!(StandardId::new(i as _)), &[i]).unwrap();
        let tx_ts = Instant::now();
        can.write(&tx_frame).await;

        let envelope = can.read().await.unwrap();

        // We can measure loopback latency by using receive timestamp in the `Envelope`.
        // Our frame is ~55 bits long (exlcuding bit stuffing), so at 1mbps loopback delay is at least 55 us.
        // When measured with `tick-hz-1_000_000` actual latency is 80~83 us, giving a combined hardware and software
        // overhead of ~25 us. Note that CPU frequency can greatly affect the result.
        let latency = envelope.ts.saturating_duration_since(tx_ts);

        info!(
            "loopback frame {=u8}, latency: {} us",
            envelope.frame.data()[0],
            latency.as_micros()
        );
        i = i.wrapping_add(1);
    }
}
