#![no_std]
#![no_main]
use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::flash::Flash;
use panic_probe as _;

const TEST_OFFSET: u32 = 0;
const TEST_PAGE: u8 = 0;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut f = Flash::new_blocking(p.FLASH);

    info!("STM32C5 EDATA example");

    if !f.edata_is_enabled() {
        panic!("EDATA is disabled; enable EDATA_EN in the option bytes");
    }

    info!("Erasing EDATA bank 1, page 0...");
    unwrap!(f.edata_erase_page(embassy_stm32::flash::EDataBank::Bank1, TEST_PAGE));

    let expected: [u16; 4] = [0x1234, 0x5678, 0xabcd, 0xef01];

    info!("writing EDATA...");
    unwrap!(f.edata_write_u16_slice(embassy_stm32::flash::EDataBank::Bank1, TEST_OFFSET, &expected));

    info!("Reading EDATA...");
    let mut actual = [0u16; 4];
    unwrap!(f.edata_read_u16_slice(embassy_stm32::flash::EDataBank::Bank1, TEST_OFFSET, &mut actual));

    info!("Read values: {:?}", actual);
    assert_eq!(actual, expected);

    info!("EDATA test succeeded!");

    loop {
        cortex_m::asm::wfi();
    }
}
