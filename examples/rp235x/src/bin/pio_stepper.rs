//! This example shows how to use the PIO module in the RP235x to implement a stepper motor driver
//! for a 5-wire stepper such as the 28BYJ-48. You can halt an ongoing rotation by dropping the future.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::stepper::{PioStepper, PioStepperProgram};
use embassy_time::{with_timeout, Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio {
        mut common, irq0, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    let prg = PioStepperProgram::new(&mut common);
    let mut stepper = PioStepper::new(&mut common, sm0, irq0, p.PIN_4, p.PIN_5, p.PIN_6, p.PIN_7, &prg);
    stepper.set_frequency(120);
    loop {
        info!("CW full steps");
        stepper.step(1000).await;

        info!("CCW full steps, drop after 1 sec");
        if with_timeout(Duration::from_secs(1), stepper.step(-i32::MAX))
            .await
            .is_err()
        {
            info!("Time's up!");
            Timer::after(Duration::from_secs(1)).await;
        }

        info!("CW half steps");
        stepper.step_half(1000).await;

        info!("CCW half steps");
        stepper.step_half(-1000).await;
    }
}
