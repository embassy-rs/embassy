#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::time::khz;
use embassy_stm32::timer::CaptureCompareInterruptHandler;
use embassy_stm32::timer::input_capture::{CaptureInput, InputCapture};
use embassy_stm32::triggers::COMP1_OUT;
use embassy_stm32::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TIM1_CC => CaptureCompareInterruptHandler<peripherals::TIM1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let trigger = CaptureInput::from(COMP1_OUT);

    let _pwm = InputCapture::new(
        p.TIM1,
        Some(trigger),
        None,
        None,
        None,
        Irqs,
        khz(10),
        Default::default(),
    );
}
