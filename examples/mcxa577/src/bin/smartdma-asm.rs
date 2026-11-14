//! Assemble application-authored SmartDMA programs and inspect their metadata.
//! This does not load or execute SmartDMA, change pin muxing, or measure timing.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use hal::smartdma::program::Program;
use panic_probe as _;

const INLINE: Program = hal::smartdma_asm!(
    r#"
    .entry start
start:
    load_imm r0, 7
    load_simm r1, 1, 16
again:
    and_imms r2, r0, 255
    lsl r2, r2, 1
    sub_imms r0, r0, 1
    goto.nz again
    hold
"#
);

const FILE: Program = hal::smartdma_asm_file!("src/bin/smartdma-asm.ezh");

fn describe(name: &str, program: &Program) {
    defmt::info!(
        "{}: {} words, {} exported entries, {} relocations (not loaded)",
        name,
        program.words().len(),
        program.entries().len(),
        program.relocations().len()
    );
    for entry in program.entries() {
        defmt::info!("  entry {} at word {}", entry.name(), entry.word_offset());
    }
    for (offset, word) in program.words().iter().enumerate() {
        defmt::info!("  word {}: {=u32:08x}", offset, word);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = hal::init(Default::default());
    describe("inline", &INLINE);
    describe("file", &FILE);
    defmt::info!("Assembly metadata only; SmartDMA has not been started");
    core::future::pending::<()>().await;
}
