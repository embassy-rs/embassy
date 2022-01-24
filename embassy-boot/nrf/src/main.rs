#![no_std]
#![no_main]

use cortex_m_rt::entry;

#[cfg(feature = "defmt")]
use defmt_rtt as _;

use embassy_boot_nrf::*;
use embassy_nrf::{nvmc::Nvmc, pac};
use embedded_storage::nor_flash::{MultiwriteNorFlash, NorFlash, ReadNorFlash};

#[entry]
fn main() -> ! {
    /*
    for i in 0..10000000 {
        cortex_m::asm::nop();
    }
    */

    let p = embassy_nrf::init(Default::default());
    let mut bl = BootLoader::new();
    bl.boot(Nvmc::new(p.NVMC));
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("udf #0");
        core::hint::unreachable_unchecked();
    }
}
