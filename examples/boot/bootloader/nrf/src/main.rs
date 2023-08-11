#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m_rt::{entry, exception};
#[cfg(feature = "defmt")]
use defmt_rtt as _;
use embassy_boot_nrf::*;
use embassy_nrf::nvmc::Nvmc;
use embassy_nrf::wdt;
use embassy_sync::blocking_mutex::Mutex;

#[entry]
fn main() -> ! {
    let p = embassy_nrf::init(Default::default());

    // Uncomment this if you are debugging the bootloader with debugger/RTT attached,
    // as it prevents a hard fault when accessing flash 'too early' after boot.
    /*
        for i in 0..10000000 {
            cortex_m::asm::nop();
        }
    */

    let mut wdt_config = wdt::Config::default();
    wdt_config.timeout_ticks = 32768 * 5; // timeout seconds
    wdt_config.run_during_sleep = true;
    wdt_config.run_during_debug_halt = false;

    let flash = WatchdogFlash::start(Nvmc::new(p.NVMC), p.WDT, wdt_config);
    let flash = Mutex::new(RefCell::new(flash));

    let config = BootLoaderConfig::from_linkerfile_blocking(&flash);
    let active_offset = config.active.offset();
    let bl: BootLoader = BootLoader::prepare(config);

    unsafe { bl.load(active_offset) }
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
    cortex_m::asm::udf();
}
