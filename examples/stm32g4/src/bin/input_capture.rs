#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::comp::{Comp, Config, Hysteresis, InvertingInput, OutputPolarity, PowerMode};
use embassy_stm32::dac::{Ch1, DacChannel, Value};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::khz;
use embassy_stm32::timer::input_capture::{CaptureInput, InputCapture};
use embassy_stm32::timer::{CaptureCompareInterruptHandler, Channel};
use embassy_stm32::triggers::COMP1_OUT;
use embassy_stm32::{bind_interrupts, comp, peripherals};
use embassy_time::{Duration, Timer, with_timeout};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TIM1_CC => CaptureCompareInterruptHandler<peripherals::TIM1>;
    COMP1_2_3 => comp::InterruptHandler<embassy_stm32::peripherals::COMP1>;
});

#[embassy_executor::task]
async fn toggle_pin(mut pin: Output<'static>) {
    loop {
        Timer::after_millis(500).await;

        pin.set_low();

        Timer::after_millis(500).await;

        pin.set_high();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // 1. setup dac as internal output and set dac output to constant value 3.3V/4
    let mut dac: DacChannel<'_, embassy_stm32::mode::Blocking> = DacChannel::new_internal_blocking::<_, Ch1>(p.DAC1);
    let constant_voltage: u16 = 1024;
    dac.set(Value::Bit12Right(constant_voltage));

    // 2. configure comparator and set its inverting input as DAC1 output
    let comp1_config = Config {
        power_mode: PowerMode::UltraLowPower,
        hysteresis: Hysteresis::None,
        output_polarity: OutputPolarity::NotInverted,
        inverting_input: InvertingInput::Dac1,
        ..Default::default()
    };

    let mut comp1 = Comp::new(p.COMP1, unsafe { p.PA1.clone_unchecked() }, Irqs, comp1_config);
    comp1.enable();

    spawner.spawn(toggle_pin(Output::new(p.PA1, Level::High, Speed::Low)).unwrap());

    //    loop {
    //        comp1.wait_for_low().await;
    //
    //        info!("comp1 low");
    //
    //        comp1.wait_for_high().await;
    //
    //        info!("comp1 high");
    //    }

    let mut ic = InputCapture::new(
        p.TIM1,
        CaptureInput::from_trigger(COMP1_OUT),
        None,
        None,
        None,
        Irqs,
        khz(10),
        Default::default(),
    );

    loop {
        // 5. capture falling edge and print out the return value from get_capture_value(Channel::Ch1) function
        match with_timeout(Duration::from_secs(3), ic.wait_for_falling_edge(Channel::Ch1)).await {
            Ok(val) => info!("Falling edge detected: {}", val), // Handle success }
            Err(_) => warn!("Timeout waiting for falling edge!"), // Handle timeout
        }
    }
}
