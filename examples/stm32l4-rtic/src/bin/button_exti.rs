// An example of using RTIC and Embassy together with HW EXTI interrupts
#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

#[rtic::app(device = embassy_stm32, peripherals = true)]
mod app {
    use defmt::info;
    use embassy_stm32::exti::blocking::ExtiGroupMask;
    use embassy_stm32::exti::{ExtiInput, TriggerEdge};
    use embassy_stm32::gpio::{Level, Output, Pull, Speed};
    use embassy_stm32::mode::Blocking;

    #[shared]
    struct Shared {
        led1: Output<'static>,
        led2: Output<'static>,
    }

    #[local]
    struct Local {
        button1: ExtiInput<'static, Blocking>,
        button2: ExtiInput<'static, Blocking>,
        button3: ExtiInput<'static, Blocking>,
        exti_pending_mask_15_10: ExtiGroupMask, // Pre-computed mask for bulk clearing
    }

    #[init]
    fn init(_ctx: init::Context) -> (Shared, Local) {
        let device_config = embassy_stm32::Config::default();
        let stm32_peripherals = embassy_stm32::init(device_config);

        // setting up the user-leds on the nucleo board
        let led1 = Output::new(stm32_peripherals.PC7, Level::Low, Speed::Low);
        let led2 = Output::new(stm32_peripherals.PB7, Level::Low, Speed::Low);

        // setting up the user-button on the nucleo board (shared exti irq line 10-15)
        let button1 = ExtiInput::new_blocking(
            stm32_peripherals.PC13,
            stm32_peripherals.EXTI13,
            Pull::Down,
            TriggerEdge::Rising,
        );

        // setting up an external button connected to the nucleo board (shared exti irq line 10-15)
        let button2 = ExtiInput::new_blocking(
            stm32_peripherals.PB10,
            stm32_peripherals.EXTI10,
            Pull::Up,
            TriggerEdge::Falling,
        );

        // Computing the mask for clearing pending interrupts on exti 15_10
        let exti_pending_mask_15_10 = ExtiGroupMask::new(&[&button1, &button2]);

        // setting up an external button connected to the nucleo board (shared exti irq line 5-9)
        let button3 = ExtiInput::new_blocking(
            stm32_peripherals.PC8,
            stm32_peripherals.EXTI8,
            Pull::Up,
            TriggerEdge::Any,
        );

        (
            Shared { led1, led2 },
            Local {
                button1,
                button2,
                button3,
                exti_pending_mask_15_10,
            },
        )
    }

    // Setting up `hardware task` to handle interrupts in the exti group 15-10
    #[task(binds = EXTI15_10, local = [button1, button2, exti_pending_mask_15_10], shared = [led1])]
    fn button1_exti_handler(mut ctx: button1_exti_handler::Context) {
        if ctx.local.button1.is_pending() {
            info!("button1 triggered");
        }
        if ctx.local.button2.is_pending() {
            info!("button2 triggered");
        }

        ctx.local.exti_pending_mask_15_10.clear();

        ctx.shared.led1.lock(|led| led.toggle());
    }

    // Setting up `hardware task` to handle interrupts in the exti group 9-5
    #[task(binds = EXTI9_5, local = [button3], shared = [led2])]
    fn button3_exti_handler(mut ctx: button3_exti_handler::Context) {
        let button = ctx.local.button3;
        // clear the interrupt flag
        button.clear_pending();

        let l = button.get_level();
        info!("button3 triggered, {}", l);

        ctx.shared.led2.lock(|led| led.toggle());
    }
}
