//! This example shows how to use the PIO step/dir program to drive a stepper driver.
//! STEP pulses are output on PIN_4, and DIR is driven on PIN_5.

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::step_dir::{PioStepDir, PioStepDirProgram, StepDir, StepPulseTiming};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

static STEPS_FWD: u32 = 10;
static STEPS_REV: u32 = 20;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio {
        mut common, irq0, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    // the PioStepDirProgram has a default timing of 66 cycles, which is good for many applications, but you can also specify a different timing if needed
    // let prg = PioStepDirProgram::new(&mut common); -> default timing of 66 cycles
    // the timing determines how long the STEP pulse will be, and also affects the minimum frequency that can be achieved. The longer the pulse, the lower the minimum frequency.
    // here we specify the timing explicitly, other options are available in the StepPulseTiming enum
    let prg = PioStepDirProgram::new_with_timing(&mut common, StepPulseTiming::Cycles66);

    // create the stepper driver, specifying the pins for STEP and DIR, and the program to use
    let mut stepper = PioStepDir::new(&mut common, sm0, irq0, p.PIN_4, p.PIN_5, &prg);

    // you must set the frequency at which the stepper will run
    // zero, too high and too low a frequency will return an error
    if stepper.set_frequency(2000).is_err() {
        let min_freq = stepper.min_frequency();
        info!("Failed to set frequency, using {} instead.", min_freq);
    }

    loop {
        info!("Forward: {} steps", STEPS_FWD);
        // one option is to use a convenience method that sets the direction and moves a number of steps in one call
        stepper.move_with_dir(STEPS_FWD, StepDir::Forward).await;

        Timer::after(Duration::from_millis(500)).await;

        info!("Reverse: {} steps", STEPS_REV);
        // or you can set the direction and then move steps in separate calls
        stepper.set_direction(StepDir::Reverse);
        stepper.move_steps(STEPS_REV).await;

        Timer::after(Duration::from_millis(500)).await;

        info!("Forward: continuous, then stop");
        // one option is to use a convenience method that sets the direction and moves a max number of steps
        stepper.start_continuous_with_dir(StepDir::Forward);
        Timer::after(Duration::from_secs(2)).await;
        stepper.stop_continuous();

        Timer::after(Duration::from_secs(1)).await;

        info!("Reverse: continuous, then stop");
        // or you can set the direction and then move steps in separate calls
        stepper.set_direction(StepDir::Reverse);
        stepper.start_continuous();
        Timer::after(Duration::from_secs(1)).await;
        stepper.stop_continuous();

        Timer::after(Duration::from_secs(1)).await;
    }
}
