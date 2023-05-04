#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::pio::{Pio, PioStateMachine, ShiftDirection};
use embassy_rp::relocate::RelocatedProgram;
use embassy_rp::{pio_instr_util, Peripheral};
use {defmt_rtt as _, panic_probe as _};

fn swap_nibbles(v: u32) -> u32 {
    let v = (v & 0x0f0f_0f0f) << 4 | (v & 0xf0f0_f0f0) >> 4;
    let v = (v & 0x00ff_00ff) << 8 | (v & 0xff00_ff00) >> 8;
    (v & 0x0000_ffff) << 16 | (v & 0xffff_0000) >> 16
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let pio = p.PIO0;
    let Pio {
        mut common,
        sm0: mut sm,
        ..
    } = Pio::new(pio);

    let prg = pio_proc::pio_asm!(
        ".origin 0",
        "set pindirs,1",
        ".wrap_target",
        "set y,7",
        "loop:",
        "out x,4",
        "in x,4",
        "jmp y--, loop",
        ".wrap",
    );

    let relocated = RelocatedProgram::new(&prg.program);
    common.write_instr(relocated.origin() as usize, relocated.code());
    pio_instr_util::exec_jmp(&mut sm, relocated.origin());
    sm.set_clkdiv((125e6 / 10e3 * 256.0) as u32);
    let pio::Wrap { source, target } = relocated.wrap();
    sm.set_wrap(source, target);
    sm.set_autopull(true);
    sm.set_autopush(true);
    sm.set_pull_threshold(32);
    sm.set_push_threshold(32);
    sm.set_out_shift_dir(ShiftDirection::Right);
    sm.set_in_shift_dir(ShiftDirection::Left);

    sm.set_enable(true);

    let mut dma_out_ref = p.DMA_CH0.into_ref();
    let mut dma_in_ref = p.DMA_CH1.into_ref();
    let mut dout = [0x12345678u32; 29];
    for i in 1..dout.len() {
        dout[i] = (dout[i - 1] & 0x0fff_ffff) * 13 + 7;
    }
    let mut din = [0u32; 29];
    loop {
        join(
            sm.dma_push(dma_out_ref.reborrow(), &dout),
            sm.dma_pull(dma_in_ref.reborrow(), &mut din),
        )
        .await;
        for i in 0..din.len() {
            assert_eq!(din[i], swap_nibbles(dout[i]));
        }
        info!("Swapped {} words", dout.len());
    }
}
