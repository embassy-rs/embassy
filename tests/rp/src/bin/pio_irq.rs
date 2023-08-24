#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
teleprobe_meta::target!(b"rpi-pico");

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Config, InterruptHandler, Pio};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let pio = p.PIO0;
    let Pio {
        mut common,
        sm0: mut sm,
        irq_flags,
        ..
    } = Pio::new(pio, Irqs);

    let prg = pio_proc::pio_asm!(
        "irq set 0",
        "irq wait 0",
        "irq set 1",
        // pause execution here
        "irq wait 1",
    );

    let mut cfg = Config::default();
    cfg.use_program(&common.load_program(&prg.program), &[]);
    sm.set_config(&cfg);
    sm.set_enable(true);

    // not using the wait futures on purpose because they clear the irq bits,
    // and we want to see in which order they are set.
    while !irq_flags.check(0) {}
    cortex_m::asm::nop();
    assert!(!irq_flags.check(1));
    irq_flags.clear(0);
    cortex_m::asm::nop();
    assert!(irq_flags.check(1));

    info!("Test OK");
    cortex_m::asm::bkpt();
}
