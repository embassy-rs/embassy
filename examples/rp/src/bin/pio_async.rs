#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Common, Irq, Pio, PioPin, ShiftDirection, StateMachine};
use embassy_rp::pio_instr_util;
use embassy_rp::relocate::RelocatedProgram;
use {defmt_rtt as _, panic_probe as _};

fn setup_pio_task_sm0(pio: &mut Common<PIO0>, sm: &mut StateMachine<PIO0, 0>, pin: impl PioPin) {
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
    let out_pin = pio.make_pio_pin(pin);
    let pio_pins = [&out_pin];
    sm.set_out_pins(&pio_pins);
    pio.write_instr(relocated.origin() as usize, relocated.code());
    pio_instr_util::exec_jmp(sm, relocated.origin());
    sm.set_clkdiv((125e6 / 20.0 / 2e2 * 256.0) as u32);
    sm.set_set_range(0, 1);
    let pio::Wrap { source, target } = relocated.wrap();
    sm.set_wrap(source, target);

    sm.set_autopull(true);
    sm.set_out_shift_dir(ShiftDirection::Left);
}

#[embassy_executor::task]
async fn pio_task_sm0(mut sm: StateMachine<'static, PIO0, 0>) {
    sm.set_enable(true);

    let mut v = 0x0f0caffa;
    loop {
        sm.tx().wait_push(v).await;
        v ^= 0xffff;
        info!("Pushed {:032b} to FIFO", v);
    }
}

fn setup_pio_task_sm1(pio: &mut Common<PIO0>, sm: &mut StateMachine<PIO0, 1>) {
    // Setupm sm1

    // Read 0b10101 repeatedly until ISR is full
    let prg = pio_proc::pio_asm!(".origin 8", "set x, 0x15", ".wrap_target", "in x, 5 [31]", ".wrap",);

    let relocated = RelocatedProgram::new(&prg.program);
    pio.write_instr(relocated.origin() as usize, relocated.code());
    pio_instr_util::exec_jmp(sm, relocated.origin());
    sm.set_clkdiv((125e6 / 2e3 * 256.0) as u32);
    sm.set_set_range(0, 0);
    let pio::Wrap { source, target } = relocated.wrap();
    sm.set_wrap(source, target);

    sm.set_autopush(true);
    sm.set_in_shift_dir(ShiftDirection::Right);
}

#[embassy_executor::task]
async fn pio_task_sm1(mut sm: StateMachine<'static, PIO0, 1>) {
    sm.set_enable(true);
    loop {
        let rx = sm.rx().wait_pull().await;
        info!("Pulled {:032b} from FIFO", rx);
    }
}

fn setup_pio_task_sm2(pio: &mut Common<PIO0>, sm: &mut StateMachine<PIO0, 2>) {
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
    pio.write_instr(relocated.origin() as usize, relocated.code());

    let pio::Wrap { source, target } = relocated.wrap();
    sm.set_wrap(source, target);

    pio_instr_util::exec_jmp(sm, relocated.origin());
    sm.set_clkdiv((125e6 / 2e3 * 256.0) as u32);
}

#[embassy_executor::task]
async fn pio_task_sm2(mut irq: Irq<'static, PIO0, 3>, mut sm: StateMachine<'static, PIO0, 2>) {
    sm.set_enable(true);
    loop {
        irq.wait().await;
        info!("IRQ trigged");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let pio = p.PIO0;

    let Pio {
        mut common,
        irq3,
        mut sm0,
        mut sm1,
        mut sm2,
        ..
    } = Pio::new(pio);

    setup_pio_task_sm0(&mut common, &mut sm0, p.PIN_0);
    setup_pio_task_sm1(&mut common, &mut sm1);
    setup_pio_task_sm2(&mut common, &mut sm2);
    spawner.spawn(pio_task_sm0(sm0)).unwrap();
    spawner.spawn(pio_task_sm1(sm1)).unwrap();
    spawner.spawn(pio_task_sm2(irq3, sm2)).unwrap();
}
