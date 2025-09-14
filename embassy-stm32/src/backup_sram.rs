//! Battary backed SRAM

use core::slice;
use core::{
    arch::asm,
    mem::{self, MaybeUninit},
};

use embassy_hal_internal::Peri;

use crate::peripherals::BKPSRAM;

pub unsafe trait Storable {
    fn layout_checksum() -> u32;
}

pub enum ReasonForDataReset<'a> {
    /// This is the first time we run or the first time after battery power was lost
    BackupRamDisabled,

    /// The struct layout does not seem to match
    ///
    /// `all_bytes` may be used to try to recover if the firmware was updated on purpose
    StructureMissmatch { old_bytes: &'a [u8] },
}

//const ADDRESS: *mut u32 = 0x4003_6400 as *mut u32;

//#[cfg(any(feature = "stm32h523", feature = "stm32h533"))]
const SIZE: usize = 2048;

// TODO: Setup these symbols in the linker script. BKPSRAM needs to be at 0x4003_6400 and BKPSRAM_LAYOUT_CHECKSUM should probably be at 0x4003_6400 + SIZE - 4
unsafe extern "C" {
    static mut BKPSRAM: [MaybeUninit<u8>; SIZE - mem::size_of::<u32>()];
    static mut BKPSRAM_LAYOUT_CHECKSUM: u32;
}

fn is_bkpsram_powered_by_battery() -> bool {
    todo!()
}

/// Setup data to be stored in battery backed SRAM
///
///
pub fn init<T: Storable>(
    _backup_sram: Peri<'static, BKPSRAM>,
    f: impl FnOnce(ReasonForDataReset) -> T,
) -> &'static mut T {
    let size = mem::size_of::<T>();
    let size_of_checksum = 4;
    assert!(size < 2048 - size_of_checksum);

    // TODO: Enable RCC bits(or is this done by the RTC? If so require a reference to an RTC instance to ensure that it has been setup)

    let ptr = unsafe { BKPSRAM.as_mut_ptr() };

    // Trick the compiler to think the memory is initialized
    // TODO: Is this safe? Is there a better way?
    // https://github.com/rust-lang/unsafe-code-guidelines/issues/397 seems relevant
    unsafe {
        asm!("/*{0}*/", in(reg) ptr);
    }

    let ptr = ptr as *const u8;

    // This bit will only be 0 the first time or when the battery power has been lost
    if !is_bkpsram_powered_by_battery() {
        // TODO: Set RCC bits to enable battery power to BKPSRAM

        let new_val = f(ReasonForDataReset::BackupRamDisabled);
        unsafe {
            let ptr = ptr as *mut T;
            *ptr = new_val;
            BKPSRAM_LAYOUT_CHECKSUM = T::layout_checksum();
        }
    } else if unsafe { BKPSRAM_LAYOUT_CHECKSUM } != T::layout_checksum() {
        let old_bytes = unsafe { slice::from_raw_parts(ptr, SIZE / mem::size_of::<u32>()) };

        let new_val = f(ReasonForDataReset::StructureMissmatch { old_bytes });
        unsafe {
            let ptr = ptr as *mut T;
            *ptr = new_val;
            BKPSRAM_LAYOUT_CHECKSUM = T::layout_checksum();
        }
    }

    unsafe { &mut *(ptr as *mut T) }
}
