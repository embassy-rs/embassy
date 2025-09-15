//! Battary backed SRAM

use core::slice;
use core::{arch::asm, mem::MaybeUninit};

use embassy_hal_internal::Peri;

use crate::peripherals::BKPSRAM;

use crate::_generated::BKPSRAM_SIZE;

/// Status of battery backed memory
pub enum Status {
    /// This is the first time we run or the first time after battery power was lost
    ///
    /// The backup RAM was just enabled. The memory will not contain any valid data from the last run
    BackupRamDisabled,

    /// The battery backed SRAM was already active
    ///
    /// You can expect the memory to contain data from the last run
    AlreadyActive,
}

// TODO: Setup these symbols in the linker script. BKPSRAM needs to be at 0x4003_6400
unsafe extern "C" {
    static mut BKPSRAM: [MaybeUninit<u8>; BKPSRAM_SIZE];
}

fn is_bkpsram_powered_by_battery() -> bool {
    todo!()
}

/// Setup battery backed sram
pub fn init(_backup_sram: Peri<'static, BKPSRAM>) -> (&'static mut [u8], Status) {
    // TODO: Enable RCC bits(or is this done by the RTC? If so require a reference to an RTC instance to ensure that it has been setup)

    #[allow(static_mut_refs)]
    let ptr = unsafe { BKPSRAM.as_mut_ptr() };

    // Trick the compiler to think the memory is initialized
    // TODO: Is this safe? Is there a better way?
    // https://github.com/rust-lang/unsafe-code-guidelines/issues/397 seems relevant
    unsafe {
        asm!("/*{0}*/", in(reg) ptr);
    }

    let ptr = ptr as *mut u8;

    // This bit will only be 0 the first time or when the battery power has been lost
    let status = if !is_bkpsram_powered_by_battery() {
        // TODO: Set RCC bits to enable battery power to BKPSRAM

        Status::BackupRamDisabled
    } else {
        Status::AlreadyActive
    };

    let bytes = unsafe { slice::from_raw_parts_mut(ptr, BKPSRAM_SIZE) };

    (bytes, status)
}
