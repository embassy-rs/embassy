#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception};

#[cfg(feature = "defmt")]
use defmt_rtt as _;

use embassy_boot_nrf::*;
use embassy_nrf::gpio::*;
use embassy_nrf::nvmc::Nvmc;

#[used]
#[no_mangle]
#[link_section = ".uicr_bootloader_start_address"]
pub static UICR_BOOTLOADER_START_ADDRESS: usize = BOOTLOADER.from;

#[used]
#[no_mangle]
#[link_section = ".uicr_mbr_params_page"]
#[cfg(feature = "softdevice")]
pub static UICR_MBR_PARAMS_PAGE: usize = MBR_PARAMS_PAGE.from;

#[entry]
fn main() -> ! {
    cortex_m::interrupt::disable();
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_15.degrade(), Level::High, OutputDrive::Standard);
    led.set_low();
    for i in 0..10000000 {
        cortex_m::asm::nop();
    }
    led.set_high();

    let ledbefore = Output::new(p.P0_14.degrade(), Level::High, OutputDrive::Standard);
    let mut bl = BootLoader::new(ledbefore, led);
    bl.boot(Nvmc::new(p.NVMC));
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
        core::arch::asm!("udf #0");
        core::hint::unreachable_unchecked();
    }
}
