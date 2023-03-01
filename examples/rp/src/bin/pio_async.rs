#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::gpio::{AnyPin, Pin};
use embassy_rp::pio::{Pio0, PioPeripheral, PioStateMachine, PioStateMachineInstance, ShiftDirection, Sm0, Sm1, Sm2};
use embassy_rp::pio_instr_util;
use embassy_rp::relocate::RelocatedProgram;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn pio_task_sm0(mut sm: PioStateMachineInstance<Pio0, Sm0>, pin: AnyPin) {
    // Setup sm0

    // Send data serially to pin
    let prg = pio_proc::pio_asm!(
        ".origin 16",
        "set pindirs, 1",
        ".wrap_target",
        "out pins,1 [19]",
        ".wrap",
    );

    let relocated = RelocatedProgram::new(&prg.program);
    let out_pin = sm.make_pio_pin(pin);
    let pio_pins = [&out_pin];
    sm.set_out_pins(&pio_pins);
    sm.write_instr(relocated.origin() as usize, relocated.code());
    pio_instr_util::exec_jmp(&mut sm, relocated.origin());
    sm.set_clkdiv((125e6 / 20.0 / 2e2 * 256.0) as u32);
    sm.set_set_range(0, 1);
    let pio::Wrap { source, target } = relocated.wrap();
    sm.set_wrap(source, target);

    sm.set_autopull(true);
    sm.set_out_shift_dir(ShiftDirection::Left);

    sm.set_enable(true);

    let mut v = 0x0f0caffa;
    loop {
        sm.wait_push(v).await;
        v ^= 0xffff;
        info!("Pushed {:032b} to FIFO", v);
    }
}

#[embassy_executor::task]
async fn pio_task_sm1(mut sm: PioStateMachineInstance<Pio0, Sm1>) {
    // Setupm sm1

    // Read 0b10101 repeatedly until ISR is full
    let prg = pio_proc::pio_asm!(".origin 8", "set x, 0x15", ".wrap_target", "in x, 5 [31]", ".wrap",);

    let relocated = RelocatedProgram::new(&prg.program);
    sm.write_instr(relocated.origin() as usize, relocated.code());
    pio_instr_util::exec_jmp(&mut sm, relocated.origin());
    sm.set_clkdiv((125e6 / 2e3 * 256.0) as u32);
    sm.set_set_range(0, 0);
    let pio::Wrap { source, target } = relocated.wrap();
    sm.set_wrap(source, target);

    sm.set_autopush(true);
    sm.set_in_shift_dir(ShiftDirection::Right);
    sm.set_enable(true);
    loop {
        let rx = sm.wait_pull().await;
        info!("Pulled {:032b} from FIFO", rx);
    }
}

#[embassy_executor::task]
async fn pio_task_sm2(mut sm: PioStateMachineInstance<Pio0, Sm2>) {
    // Setup sm2

    // Repeatedly trigger IRQ 3
    let prg = pio_proc::pio_asm!(
        ".origin 0",
        ".wrap_target",
        "set x,10",
        "delay:",
        "jmp x-- delay [15]",
        "irq 3 [15]",
        ".wrap",
    );
    let relocated = RelocatedProgram::new(&prg.program);
    sm.write_instr(relocated.origin() as usize, relocated.code());

    let pio::Wrap { source, target } = relocated.wrap();
    sm.set_wrap(source, target);

    pio_instr_util::exec_jmp(&mut sm, relocated.origin());
    sm.set_clkdiv((125e6 / 2e3 * 256.0) as u32);
    sm.set_enable(true);
    loop {
        sm.wait_irq(3).await;
        info!("IRQ trigged");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let pio = p.PIO0;

    let (_, sm0, sm1, sm2, ..) = pio.split();

    spawner.spawn(pio_task_sm0(sm0, p.PIN_0.degrade())).unwrap();
    spawner.spawn(pio_task_sm1(sm1)).unwrap();
    spawner.spawn(pio_task_sm2(sm2)).unwrap();
}
