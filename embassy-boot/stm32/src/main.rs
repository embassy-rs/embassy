#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception};

#[cfg(feature = "defmt")]
use defmt_rtt as _;

use embassy_boot_stm32::*;
use embassy_stm32::flash::Flash;

#[cfg(not(any(feature = "flash-2k", feature = "flash-256", feature = "flash-128")))]
compile_error!("No flash size specified. Must specify exactly one of the following features: flash-2k, flash-256, flash-128");

#[entry]
fn main() -> ! {
    let p = embassy_stm32::init(Default::default());

    // Uncomment this if you are debugging the bootloader with debugger/RTT attached,
    // as it prevents a hard fault when accessing flash 'too early' after boot.
    /*
        for i in 0..10000000 {
            cortex_m::asm::nop();
        }
    */

    #[cfg(feature = "flash-2k")]
    let mut bl: BootLoader<2048> = BootLoader::default();

    #[cfg(feature = "flash-256")]
    let mut bl: BootLoader<256> = BootLoader::default();

    #[cfg(feature = "flash-128")]
    let mut bl: BootLoader<128> = BootLoader::default();

    let mut flash = Flash::unlock(p.FLASH);
    let start = bl.prepare(&mut SingleFlashProvider::new(&mut flash));
    core::mem::drop(flash);
    unsafe { bl.load(start) }
}

#[no_mangle]
#[cfg_attr(target_os = "none", link_section = ".HardFault.user")]
unsafe extern "C" fn HardFault() {
    cortex_m::peripheral::SCB::sys_reset();
}

#[exception]
unsafe fn DefaultHandler(_: i16) -> ! {
    const SCB_ICSR: *const u32 = 0xE000_ED04 as *const u32;
    let irqn = core::ptr::read_volatile(SCB_ICSR) as u8 as i16 - 16;

    panic!("DefaultHandler #{:?}", irqn);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        cortex_m::asm::udf();
        core::hint::unreachable_unchecked();
    }
}
