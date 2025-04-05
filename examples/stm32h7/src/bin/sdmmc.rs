#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::sdmmc::Sdmmc;
use embassy_stm32::time::mhz;
use embassy_stm32::{bind_interrupts, peripherals, sdmmc, Config};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SDMMC1 => sdmmc::InterruptHandler<peripherals::SDMMC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV4), // default clock chosen by SDMMCSEL. 200 Mhz
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut sdmmc = Sdmmc::new_4bit(
        p.SDMMC1,
        Irqs,
        p.PC12,
        p.PD2,
        p.PC8,
        p.PC9,
        p.PC10,
        p.PC11,
        Default::default(),
    );

    // Should print 400kHz for initialization
    info!("Configured clock: {}", sdmmc.clock().0);

    unwrap!(sdmmc.init_sd_card(mhz(25)).await);

    let card = unwrap!(sdmmc.card());

    info!("Card: {:#?}", Debug2Format(card));

    loop {}
}
