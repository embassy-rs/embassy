use core::ptr::write_volatile;
use core::sync::atomic::{fence, Ordering};

use super::{FlashSector, WRITE_SIZE};
use crate::flash::Error;
use crate::pac;

// pub(crate) const fn is_default_layout() -> bool {
//     true
// }

// pub(crate) fn get_flash_regions() -> &'static [&'static FlashRegion] {
//     &FLASH_REGIONS
// }

pub(crate) unsafe fn lock() {
    pac::FLASH.cr().modify(|w| w.set_lock(true));
}

pub(crate) unsafe fn unlock() {
    if pac::FLASH.cr().read().lock() {
        pac::FLASH.keyr().write_value(stm32_metapac::flash::regs::Keyr(0x4567_0123));
        pac::FLASH.keyr().write_value(stm32_metapac::flash::regs::Keyr(0xCDEF_89AB));
    }
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);
}

pub(crate) unsafe fn disable_blocking_write() {}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    // We cannot have the write setup sequence in begin_write as it depends on the address
    let bank = pac::FLASH;

    bank.cr().write(|w| {
        w.set_pg(true);
        #[cfg(flash_h7)]
        w.set_psize(2); // 32 bits at once
    });
    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let mut res = None;
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;

        res = Some(blocking_wait_ready());
        // bank.sr().modify(|w| {
        //     if w.eop() {
        //         w.set_eop(true);
        //     }
        // });
        if unwrap!(res).is_err() {
            break;
        }
    }

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    bank.cr().write(|w| w.set_pg(false));

    unwrap!(res)
}

pub(crate) unsafe fn blocking_erase_sector(sector: &FlashSector) -> Result<(), Error> {
    let bank = pac::FLASH;
    bank.cr().modify(|w| {
        w.set_ser(true);
        w.set_ssn(sector.index_in_bank);
        // #[cfg(flash_h7)]
        // w.set_snb(sector.index_in_bank);
        // #[cfg(flash_h7ab)]
        // w.set_ssn(sector.index_in_bank);
    });

    bank.cr().modify(|w| {
        w.set_start(true);
    });

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let ret: Result<(), Error> = blocking_wait_ready();
    bank.cr().modify(|w| w.set_ser(false));
    bank_clear_all_err();
    ret
}

pub(crate) unsafe fn clear_all_err() {
    bank_clear_all_err();
}

unsafe fn bank_clear_all_err() {
    // read and write back the same value.
    // This clears all "write 1 to clear" bits.
    pac::FLASH.sr().modify(|_| {});
}

unsafe fn blocking_wait_ready() -> Result<(), Error> {
    let bank = pac::FLASH;
    loop {
        let sr = bank.sr().read();

        if !sr.busy() && !sr.qw() {
            return Ok(());
        }
    }
}
