#![no_std]
#![no_main]

use cortex_m_rt::entry;
#[cfg(feature = "defmt")]
use defmt_rtt as _;
use embassy_boot_rp::*;
use embassy_rp::flash::{Flash, ERASE_SIZE};

const ERASE_VALUE: u8 = 0xFF;

#[entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());

    // Uncomment this if you are debugging the bootloader with debugger/RTT attached,
    // as it prevents a hard fault when accessing flash 'too early' after boot.
    /*
        for i in 0..10000000 {
            cortex_m::asm::nop();
        }
    */

    let mut bl: BootLoader = BootLoader::default();
    let flash = Flash::<_, { 2 * 1024 * 1024 }>::new(p.FLASH);
    let mut flash = BootFlash::<_, ERASE_SIZE, ERASE_VALUE>::new(flash);
    let start = bl.prepare(&mut SingleFlashConfig::new(&mut flash));
    core::mem::drop(flash);
    unsafe { bl.load(start) }
}

#[no_mangle]
#[cfg_attr(target_os = "none", link_section = ".HardFault.user")]
unsafe extern "C" fn HardFault() {
    cortex_m::peripheral::SCB::sys_reset();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::asm::udf();
}
