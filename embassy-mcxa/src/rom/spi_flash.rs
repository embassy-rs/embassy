use core::ffi::c_void;

use super::Status;

// LPSPI external flash (SPI NOR/EEPROM) ROM API

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SpiMemConfigOption {
    option0: u32,
    option1: u32,
}

#[repr(C)]
pub struct SpiFlashDriver {
    spi_eeprom_init: unsafe extern "C" fn() -> Status,
    spi_eeprom_read: unsafe extern "C" fn(address: u32, NoOfBytes: u32, buffer: *mut u8) -> Status,
    spi_eeprom_write: unsafe extern "C" fn(address: u32, NoOfBytes: u32, buffer: *const u8) -> Status,
    spi_eeprom_erase: unsafe extern "C" fn(address: u32, length: u32) -> Status,
    spi_eeprom_config: unsafe extern "C" fn(config: *mut u32) -> Status,
    spi_eeprom_flush: unsafe extern "C" fn() -> Status,
    reserved0: *mut c_void,
    spi_eeprom_erase_all: unsafe extern "C" fn() -> Status,
}

impl SpiFlashDriver {
    fn spi_eeprom_init(&self) -> SpiFlashStatus {
        unsafe { SpiFlashStatus::from_raw((self.spi_eeprom_init)()) }
    }

    fn spi_eeprom_read(&self, address: u32, no_of_bytes: u32, buffer: *mut u8) -> SpiFlashStatus {
        unsafe { SpiFlashStatus::from_raw((self.spi_eeprom_read)(address, no_of_bytes, buffer)) }
    }

    fn spi_eeprom_write(&self, address: u32, no_of_bytes: u32, buffer: *const u8) -> SpiFlashStatus {
        unsafe { SpiFlashStatus::from_raw((self.spi_eeprom_write)(address, no_of_bytes, buffer)) }
    }

    fn spi_eeprom_erase(&self, address: u32, length: u32) -> SpiFlashStatus {
        unsafe { SpiFlashStatus::from_raw((self.spi_eeprom_erase)(address, length)) }
    }

    fn spi_eeprom_config(&self, config: *mut u32) -> SpiFlashStatus {
        unsafe { SpiFlashStatus::from_raw((self.spi_eeprom_config)(config)) }
    }

    fn spi_eeprom_flush(&self) -> SpiFlashStatus {
        unsafe { SpiFlashStatus::from_raw((self.spi_eeprom_flush)()) }
    }

    fn spi_eeprom_erase_all(&self) -> SpiFlashStatus {
        unsafe { SpiFlashStatus::from_raw((self.spi_eeprom_erase_all)()) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpiFlashStatus {
    Success,
    Fail,
    Unknown(u32),
}

impl SpiFlashStatus {
    const fn from_raw(raw: u32) -> Self {
        match raw {
            super::KSTATUS_SPIFLASH_SUCCESS => Self::Success,
            super::KSTATUS_SPIFLASH_FAIL => Self::Fail,
            other => Self::Unknown(other),
        }
    }
}
