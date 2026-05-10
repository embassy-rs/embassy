#![no_std]
#![no_main]

#[embassy_executor::main(entry = "aarch32_rt::entry")]
async fn main(_spawner: embassy_executor::Spawner) -> ! {
    let _p = embassy_mps3_an536_examples::Board::new().unwrap();
    loop {
        defmt::info!("Hello World!");
        embassy_time::Timer::after_secs(1).await;
    }
}

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
