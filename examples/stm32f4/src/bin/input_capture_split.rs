#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::Pull;
use embassy_stm32::time::khz;
use embassy_stm32::timer::input_capture::{CapturePin, InputCapture, InputCaptureChannel};
use embassy_stm32::{bind_interrupts, peripherals, timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TIM2 => timer::CaptureCompareInterruptHandler<peripherals::TIM2>;
});

#[embassy_executor::task]
async fn capture_task_ch1(mut ch: InputCaptureChannel<'static, peripherals::TIM2>) {
    loop {
        let val = ch.wait_for_rising_edge().await;
        info!("ch1: capture {}", val);
    }
}

#[embassy_executor::task]
async fn capture_task_ch2(mut ch: InputCaptureChannel<'static, peripherals::TIM2>) {
    loop {
        let val = ch.wait_for_rising_edge().await;
        info!("ch2: capture {}", val);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Input capture split example");

    let ch1 = CapturePin::new(p.PA0, Pull::None);
    let ch2 = CapturePin::new(p.PA1, Pull::None);
    let ic = InputCapture::new(
        p.TIM2,
        Some(ch1),
        Some(ch2),
        None,
        None,
        Irqs,
        khz(1000),
        Default::default(),
    );

    let chs = ic.split();
    spawner.spawn(unwrap!(capture_task_ch1(chs.ch1)));
    spawner.spawn(unwrap!(capture_task_ch2(chs.ch2)));
}
