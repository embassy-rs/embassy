#![no_std]
#![no_main]

/// Core 0 entry point
mod core0 {
    use embassy_mps3_an536_examples::{Board, start_core1};

    #[embassy_executor::main()]
    async fn main(spawner: embassy_executor::Spawner) {
        defmt::info!("I am core0::main");

        let _p = Board::new().unwrap();

        defmt::info!("Setting prio mask");
        arm_gic::gicv3::GicCpuInterface::set_priority_mask(0xFF);

        defmt::info!("Enabling interrupts on Core 0...");
        unsafe {
            aarch32_cpu::interrupt::enable();
        }

        start_core1();

        spawner.spawn(fizz().unwrap());
        spawner.spawn(buzz().unwrap());
    }

    /// Core 0 entry point
    #[unsafe(no_mangle)]
    pub extern "C" fn kmain() -> ! {
        main();
    }

    #[embassy_executor::task]
    async fn fizz() {
        loop {
            defmt::info!("FIZZ");
            embassy_time::Timer::after_millis(300).await;
        }
    }

    #[embassy_executor::task]
    async fn buzz() {
        loop {
            defmt::info!("BUZZ");
            embassy_time::Timer::after_millis(500).await;
        }
    }
}

mod core1 {
    /// Core 1 entry point
    #[embassy_executor::main()]
    async fn main(spawner: embassy_executor::Spawner) {
        defmt::info!("I am core1::main");

        defmt::info!("Configuring this core's GIC...");
        arm_gic::gicv3::GicCpuInterface::set_priority_mask(0xFF);
        // the arm_gic::Gicv3::setup did this on the boot core, but not for our core
        arm_gic::gicv3::GicCpuInterface::enable_group1(true);

        defmt::info!("Enabling interrupts on core 1...");
        unsafe {
            aarch32_cpu::interrupt::enable();
        }

        spawner.spawn(fizz().unwrap());
        spawner.spawn(buzz().unwrap());
    }

    /// Core 1 entry point
    #[unsafe(no_mangle)]
    pub extern "C" fn kmain2() -> ! {
        main();
    }

    #[embassy_executor::task]
    async fn fizz() {
        loop {
            defmt::info!("fizz");
            embassy_time::Timer::after_millis(300).await;
        }
    }

    #[embassy_executor::task]
    async fn buzz() {
        loop {
            defmt::info!("buzz");
            embassy_time::Timer::after_millis(500).await;
        }
    }
}

/// Called from the assembly trampoline when either core gets an IRQ exception
#[aarch32_rt::irq]
fn irq_handler() {
    use arm_gic::gicv3::{GicCpuInterface, InterruptGroup};
    defmt::debug!("> IRQ");
    while let Some(int_id) = GicCpuInterface::get_and_acknowledge_interrupt(InterruptGroup::Group1) {
        match int_id {
            embassy_mps3_an536_examples::VIRTUAL_TIMER_PPI => {
                defmt::debug!("- Timer fired, resetting");
                embassy_mps3_an536_examples::timer_irq();
            }
            _ => unreachable!("We handle all enabled IRQs"),
        }
        GicCpuInterface::end_interrupt(int_id, InterruptGroup::Group1);
    }
    defmt::debug!("< IRQ");
}
