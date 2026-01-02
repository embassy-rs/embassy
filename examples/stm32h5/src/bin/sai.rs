#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{sai, Config};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello world.");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;

        config.rcc.pll2 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV16,
            mul: PllMul::MUL32,
            divp: Some(PllDiv::DIV16), // 8 MHz SAI clock
            divq: None,
            divr: None,
        });

        config.rcc.mux.sai1sel = mux::Saisel::PLL2_P;
    }
    let p = embassy_stm32::init(config);

    let mut write_buffer = [0u16; 1024];
    let (_, sai_b) = sai::split_subblocks(p.SAI1);

    let mut sai_b = sai::Sai::new_asynchronous(
        sai_b,
        p.PF8,
        p.PE3,
        p.PF9,
        p.GPDMA1_CH0,
        &mut write_buffer,
        Default::default(),
    );

    // Populate arbitrary data.
    let mut data = [0u16; 256];
    for (index, sample) in data.iter_mut().enumerate() {
        *sample = index as u16;
    }

    loop {
        sai_b.write(&data).await.unwrap();
    }
}
