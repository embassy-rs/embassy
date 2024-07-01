#![no_std]
#![no_main]
teleprobe_meta::target!(b"rpi-pico");

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Config, InterruptHandler, LoadError, Pio};
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
        mut sm0,
        mut sm1,
        mut sm2,
        irq_flags,
        ..
    } = Pio::new(pio, Irqs);

    // load with explicit origin works
    let prg1 = pio_proc::pio_asm!(
        ".origin 4"
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "irq 0",
        "nop",
        "nop",
    );
    let loaded1 = common.load_program(&prg1.program);
    assert_eq!(loaded1.origin, 4);
    assert_eq!(loaded1.wrap.source, 13);
    assert_eq!(loaded1.wrap.target, 4);

    // load without origin chooses a free space
    let prg2 = pio_proc::pio_asm!("nop", "nop", "nop", "nop", "nop", "nop", "nop", "irq 1", "nop", "nop",);
    let loaded2 = common.load_program(&prg2.program);
    assert_eq!(loaded2.origin, 14);
    assert_eq!(loaded2.wrap.source, 23);
    assert_eq!(loaded2.wrap.target, 14);

    // wrapping around the end of program space automatically works
    let prg3 =
        pio_proc::pio_asm!("nop", "nop", "nop", "nop", "nop", "nop", "nop", "nop", "nop", "nop", "nop", "irq 2",);
    let loaded3 = common.load_program(&prg3.program);
    assert_eq!(loaded3.origin, 24);
    assert_eq!(loaded3.wrap.source, 3);
    assert_eq!(loaded3.wrap.target, 24);

    // check that the programs actually work
    {
        let mut cfg = Config::default();
        cfg.use_program(&loaded1, &[]);
        sm0.set_config(&cfg);
        sm0.set_enable(true);
        while !irq_flags.check(0) {}
        sm0.set_enable(false);
    }
    {
        let mut cfg = Config::default();
        cfg.use_program(&loaded2, &[]);
        sm1.set_config(&cfg);
        sm1.set_enable(true);
        while !irq_flags.check(1) {}
        sm1.set_enable(false);
    }
    {
        let mut cfg = Config::default();
        cfg.use_program(&loaded3, &[]);
        sm2.set_config(&cfg);
        sm2.set_enable(true);
        while !irq_flags.check(2) {}
        sm2.set_enable(false);
    }

    // instruction memory is full now. all loads should fail.
    {
        let prg = pio_proc::pio_asm!(".origin 0", "nop");
        match common.try_load_program(&prg.program) {
            Err(LoadError::AddressInUse(0)) => (),
            _ => panic!("program loaded when it shouldn't"),
        };

        let prg = pio_proc::pio_asm!("nop");
        match common.try_load_program(&prg.program) {
            Err(LoadError::InsufficientSpace) => (),
            _ => panic!("program loaded when it shouldn't"),
        };
    }

    // freeing some memory should allow further loads though.
    unsafe {
        common.free_instr(loaded3.used_memory);
    }
    {
        let prg = pio_proc::pio_asm!(".origin 0", "nop");
        match common.try_load_program(&prg.program) {
            Ok(_) => (),
            _ => panic!("program didn't loaded when it shouldn"),
        };

        let prg = pio_proc::pio_asm!("nop");
        match common.try_load_program(&prg.program) {
            Ok(_) => (),
            _ => panic!("program didn't loaded when it shouldn"),
        };
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}
