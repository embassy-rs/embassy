#![no_std]
#![no_main]

use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{self, Input, Pull};
use embassy_nrf::mode::Async;
use embassy_nrf::{bind_interrupts, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    GPIOTE20_0 => gpio::InterruptHandler<peripherals::P1_13>,
        gpio::InterruptHandler<peripherals::P1_09>,
        gpio::InterruptHandler<peripherals::P1_08>;
    GPIOTE30_0 => gpio::InterruptHandler<peripherals::P0_04>;
});

#[embassy_executor::task(pool_size = 4)]
async fn button_task(n: usize, mut pin: Input<'static, Async>) {
    loop {
        pin.wait_for_low().await;
        info!("Button {:?} pressed!", n);
        pin.wait_for_high().await;
        info!("Button {:?} released!", n);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Starting!");

    let btn1 = Input::new_async(p.P1_13, Irqs, Pull::Up);
    let btn2 = Input::new_async(p.P1_09, Irqs, Pull::Up);
    let btn3 = Input::new_async(p.P1_08, Irqs, Pull::Up);
    let btn4 = Input::new_async(p.P0_04, Irqs, Pull::Up);

    spawner.spawn(unwrap!(button_task(1, btn1)));
    spawner.spawn(unwrap!(button_task(2, btn2)));
    spawner.spawn(unwrap!(button_task(3, btn3)));
    spawner.spawn(unwrap!(button_task(4, btn4)));
}
