#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::{Config, uid};
use stm32_metapac::DESIG;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL30,
        divr: Some(PllDiv::DIV5),
        divq: None,
        divp: Some(PllDiv::DIV30),
        frac: Some(0),
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    config.rcc.sys = Sysclk::PLL1_R;

    let _p = embassy_stm32::init(config);
    info!("Device info example");

    // 96-bit unique ID
    let uid_bytes = uid::uid();
    let uid_hex = uid::uid_hex();
    info!("UID (bytes): {:02x}", uid_bytes);
    info!("UID (hex):   {}", uid_hex);

    // Flash and RAM size
    let flashsizer = DESIG.flashsizer().read();
    info!("Flash size:  {} KB", flashsizer.flash_size());
    info!("RAM size:    {} KB", flashsizer.ram_size());

    // Package type
    let pkgr = DESIG.pkgr().read();
    info!("Package:     0x{:02x}", pkgr.pkg().to_bits());

    // IEEE 64-bit unique ID
    let uid64r1 = DESIG.uid64r1().read();
    let uid64r2 = DESIG.uid64r2().read();
    info!("UID64 devnum: 0x{:08x}", uid64r1.devnum());
    info!("UID64 devid:  0x{:02x}", uid64r2.devid().to_bits());
    info!("UID64 stid:   0x{:06x}", uid64r2.stid().to_bits());
}
