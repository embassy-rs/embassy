use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, Ordering};

use super::Status;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SpiFlashConfig {
    /// - `tag [31:28]`: Must be 0x0C (kSpiMem_ConfigOption_Tag)
    /// - `option_size [27:24]`: Option size in terms of uint32_t, actual size = (option_size + 1) × 4 bytes
    /// - `spi_index [23:20]`: Index of SPI module instance
    /// - `pcs_index [19:16]`: PCS (chip select) instance of SPI module
    /// - `memory_type [15:12]`: Determines the connected memory type (NOR/EEPROM)
    /// - `memory_size [11:8]`: NOR/EEPROM memory size
    /// - `sector_size [7:4]`: NOR/EEPROM sector size
    /// - `page_size [3:0]`: NOR/EEPROM page size
    pub option0: u32,
    /// - `spi_speed [3:0]`: SPI transfer speed to connected NOR/EEPROM
    ///   - 0 = 22.5 MHz
    ///   - 1 = 11.25 MHz
    ///   - 2 = 5.625 MHz
    ///   - 3 = 2.8125 MHz
    /// - reserved [31:4]: Reserved for future use (set to 0)
    pub option1: u32,
}

#[repr(C)]
pub struct SpiFlashVtable {
    init: unsafe extern "C" fn() -> Status,
    read: unsafe extern "C" fn(address: u32, no_of_bytes: u32, buffer: *mut u8) -> Status,
    write: unsafe extern "C" fn(address: u32, no_of_bytes: u32, buffer: *const u8) -> Status,
    erase: unsafe extern "C" fn(address: u32, length: u32) -> Status,
    config: unsafe extern "C" fn(config: *mut SpiFlashConfig) -> Status,
    flush: unsafe extern "C" fn() -> Status,
    reserved0: *mut c_void,
    erase_all: unsafe extern "C" fn() -> Status,
}

pub struct SpiFlash {
    vtable: &'static SpiFlashVtable,
}

static TAKEN: AtomicBool = AtomicBool::new(false);

impl SpiFlash {
    pub(super) fn new(vtable: &'static SpiFlashVtable, mut config: SpiFlashConfig) -> Result<Self, SpiFlashError> {
        if TAKEN
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(SpiFlashError::Unavailable);
        }

        Result::from(unsafe { (vtable.init)() })?;
        Result::from(unsafe { (vtable.config)(&raw mut config) })?;
        Ok(Self { vtable })
    }

    pub fn read(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), SpiFlashError> {
        unsafe { (self.vtable.read)(address, buffer.len() as _, buffer.as_mut_ptr()) }.into()
    }

    pub fn write(&mut self, address: u32, buffer: &[u8]) -> Result<(), SpiFlashError> {
        unsafe { (self.vtable.write)(address, buffer.len() as _, buffer.as_ptr()) }.into()
    }

    pub fn erase(&mut self, address: u32, length: u32) -> Result<(), SpiFlashError> {
        unsafe { (self.vtable.erase)(address, length) }.into()
    }

    pub fn config(&mut self, mut config: SpiFlashConfig) -> Result<(), SpiFlashError> {
        unsafe { (self.vtable.config)(&raw mut config) }.into()
    }

    pub fn flush(&mut self) -> Result<(), SpiFlashError> {
        unsafe { (self.vtable.flush)() }.into()
    }

    pub fn erase_all(&mut self) -> Result<(), SpiFlashError> {
        unsafe { (self.vtable.erase_all)() }.into()
    }
}

impl Drop for SpiFlash {
    fn drop(&mut self) {
        TAKEN.store(false, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpiFlashError {
    Fail,
    Unknown(u32),
    Unavailable,
}

impl From<Status> for Result<(), SpiFlashError> {
    fn from(raw: Status) -> Self {
        match raw.0 {
            super::KSTATUS_SPIFLASH_SUCCESS => Ok(()),
            super::KSTATUS_SPIFLASH_FAIL => Err(SpiFlashError::Fail),
            other => Err(SpiFlashError::Unknown(other)),
        }
    }
}
