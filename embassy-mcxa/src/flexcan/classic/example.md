<details>

<summary><h4>Example Setup</h4></summary>

Here's a short example program that demonstrates how to set up a FlexCAN peripheral for Classic CAN using this HAL:

```rust,no_run
#![no_std]
#![no_main]

use panic_probe as _;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use embassy_mcxa::{
    bind_interrupts, config::Config, peripherals::CAN0,
    flexcan::filter::{filters, Filter,},
    flexcan::classic::{
        FlexCan, FlexCanConfig, FlexCanRx, FlexCanTx, InterruptHandler,
        frame::{Frame, StandardId, ExtendedId},
    },
};

bind_interrupts!(struct Irqs {
    CAN0 => InterruptHandler<CAN0>;
});

// Outgoing messages
const EXAMPLE_MESSAGE_ONE: StandardId = StandardId::new(0x01).unwrap();
const EXAMPLE_MESSAGE_TWO: ExtendedId = ExtendedId::new(0xFAF).unwrap();

// Incoming messages
const EXAMPLE_MESSAGE_THREE: StandardId = StandardId::new(0x02).unwrap();
const EXAMPLE_MESSAGE_FOUR: ExtendedId = ExtendedId::new(0x1232).unwrap();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_mcxa::init(Config::default());

    // Create and configure a `FlexCan` instance for CAN0.
    let can0 = FlexCan::new(p.CAN0, p.P1_11, p.P1_2, FlexCanConfig {
        filters: filters!(
            Filter::Standard(EXAMPLE_MESSAGE_THREE), Filter::Extended(EXAMPLE_MESSAGE_FOUR),
        ),
        bitrate: 1_000_000,
        ..FlexCanConfig::default()
    }).expect("Failed to init FlexCan!!");

    // Split your `FlexCan` into separate `FlexCanTx` and `FlexCanRx` halves, and pass them to their respective tasks.
    let (tx0, rx0) = can0.split();
    spawner.spawn(can0_tx(tx0).expect("Failed to spawn `can0_tx()`."));
    spawner.spawn(can0_rx(rx0).expect("Failed to spawn `can0_rx()`."));
}

#[embassy_executor::task]
async fn can0_tx(mut tx0: FlexCanTx<'static>) {
    // Task for sending outgoing messages
    loop {
        let frame1 = Frame::new(EXAMPLE_MESSAGE_ONE, &[0, 1, 2]).expect("Message payload too long!");
        let frame2 = Frame::new(EXAMPLE_MESSAGE_TWO, &[3, 4, 5, 6]).expect("Message payload too long!");
        tx0.send(&frame1).await;
        tx0.send(&frame2).await;

        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn can0_rx(rx0: FlexCanRx<'static>) {
    // Task for receiving incoming messages
    loop {
        let frame = rx0.receive().await;
        defmt::info!("CAN0 RX id={:?} len={}", frame.id(), frame.dlc());
    }
}
```
</details>