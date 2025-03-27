#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

#[rtic::app(device = embassy_stm32, peripherals = true)]
mod app {
    use defmt::info;
    use embassy_stm32::exti::blocking::ExtiInput;
    use embassy_stm32::exti::TriggerEdge;
    use embassy_stm32::gpio::{Level, Output, Pull, Speed};

    #[shared]
    struct Shared {
        led: Output<'static>,
    }

    #[local]
    struct Local {
        button1: ExtiInput<'static>,
        button2: ExtiInput<'static>,
    }

    #[init]
    fn init(_ctx: init::Context) -> (Shared, Local) {
        let device_config = embassy_stm32::Config::default();
        let stm32_peripherals = embassy_stm32::init(device_config);

        // setting up the user led on the nucleo board
        let led = Output::new(stm32_peripherals.PB14, Level::High, Speed::Low);

        // setting up the user button on the nucleo board
        let button1 = ExtiInput::new(
            stm32_peripherals.PC13,
            stm32_peripherals.EXTI13,
            Pull::Down,
            TriggerEdge::Rising,
        );

        // setting up an external button connected to the nucleo board
        let button2 = ExtiInput::new(
            stm32_peripherals.PC8,
            stm32_peripherals.EXTI8,
            Pull::Down,
            TriggerEdge::Rising,
        );

        (Shared { led }, Local { button1, button2 })
    }

    // Setting up `hardware task` to handle interrupts in the exti group 15-10
    #[task(binds = EXTI15_10, local = [button1], shared = [led])]
    fn button1_exti_handler(mut ctx: button1_exti_handler::Context) {
        info!("button1 triggered");
        // clear the interrupt flag
        ctx.local.button1.clear_pending();

        ctx.shared.led.lock(|led| led.toggle());
    }

    // Setting up `hardware task` to handle interrupts in the exti group 9-5
    #[task(binds = EXTI9_5, local = [button2], shared = [led])]
    fn button2_exti_handler(mut ctx: button2_exti_handler::Context) {
        info!("button2 triggered");
        // clear the interrupt flag
        ctx.local.button2.clear_pending();

        ctx.shared.led.lock(|led| led.toggle());
    }
}
