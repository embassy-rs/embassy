use embassy_hal_common::drop::OnDrop;
use embassy_hal_common::{into_ref, PeripheralRef};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};

use super::{family, Error, FlashLayout, FlashRegion, FLASH_BASE, FLASH_SIZE, WRITE_SIZE};
use crate::Peripheral;

pub struct Flash<'d> {
    inner: PeripheralRef<'d, crate::peripherals::FLASH>,
}

impl<'d> Flash<'d> {
    pub fn new(p: impl Peripheral<P = crate::peripherals::FLASH> + 'd) -> Self {
        into_ref!(p);
        Self { inner: p }
    }

    pub fn into_regions(self) -> FlashLayout<'d> {
        FlashLayout::new(self.release())
    }

    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        Self::blocking_read_inner(FLASH_BASE as u32 + offset, bytes)
    }

    fn blocking_read_inner(start_address: u32, bytes: &mut [u8]) -> Result<(), Error> {
        assert!(start_address >= FLASH_BASE as u32);
        if start_address as usize + bytes.len() > FLASH_BASE + FLASH_SIZE {
            return Err(Error::Size);
        }

        let flash_data = unsafe { core::slice::from_raw_parts(start_address as *const u8, bytes.len()) };
        bytes.copy_from_slice(flash_data);
        Ok(())
    }

    pub fn blocking_write(&mut self, offset: u32, buf: &[u8]) -> Result<(), Error> {
        let start_address = FLASH_BASE as u32 + offset;

        // No need to take lock here as we only have one mut flash reference.

        unsafe { Flash::blocking_write_inner(start_address, buf) }
    }

    unsafe fn blocking_write_inner(start_address: u32, buf: &[u8]) -> Result<(), Error> {
        assert!(start_address >= FLASH_BASE as u32);
        if start_address as usize + buf.len() > FLASH_BASE + FLASH_SIZE {
            return Err(Error::Size);
        }
        if (start_address as usize - FLASH_BASE) % WRITE_SIZE != 0 || buf.len() as usize % WRITE_SIZE != 0 {
            return Err(Error::Unaligned);
        }

        trace!("Writing {} bytes at 0x{:x}", buf.len(), start_address);

        family::clear_all_err();
        family::unlock();
        family::begin_write();

        let _ = OnDrop::new(|| {
            family::end_write();
            family::lock();
        });

        let mut address = start_address;
        for chunk in buf.chunks(WRITE_SIZE) {
            unsafe { family::blocking_write(address, chunk.try_into().unwrap())? };
            address += WRITE_SIZE as u32;
        }
        Ok(())
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        let start_address = FLASH_BASE as u32 + from;
        let end_address = FLASH_BASE as u32 + to;

        unsafe { Flash::blocking_erase_inner(start_address, end_address) }
    }

    unsafe fn blocking_erase_inner(start_address: u32, end_address: u32) -> Result<(), Error> {
        // Test if the address range is aligned at sector base addresses
        let mut address = start_address;
        while address < end_address {
            let sector = family::get_sector(address);
            if sector.start != address {
                return Err(Error::Unaligned);
            }
            address += sector.size;
        }
        if address != end_address {
            return Err(Error::Unaligned);
        }

        trace!("Erasing from 0x{:x} to 0x{:x}", start_address, end_address);

        family::clear_all_err();
        family::unlock();

        let _ = OnDrop::new(|| {
            family::lock();
        });

        let mut address = start_address;
        while address < end_address {
            let sector = family::get_sector(address);
            family::blocking_erase_sector(&sector)?;
            address += sector.size;
        }
        Ok(())
    }

    pub(crate) fn release(self) -> PeripheralRef<'d, crate::peripherals::FLASH> {
        let mut flash = self;
        unsafe { flash.inner.clone_unchecked() }
    }
}

impl Drop for Flash<'_> {
    fn drop(&mut self) {
        unsafe { family::lock() };
    }
}

impl Drop for FlashLayout<'_> {
    fn drop(&mut self) {
        unsafe { family::lock() };
    }
}

static REGION_LOCK: Mutex<CriticalSectionRawMutex, ()> = Mutex::new(());

fn take_lock_spin() -> MutexGuard<'static, CriticalSectionRawMutex, ()> {
    loop {
        if let Ok(guard) = REGION_LOCK.try_lock() {
            return guard;
        }
    }
}

impl FlashRegion {
    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        unsafe { self.blocking_read_inner(offset, bytes) }
    }

    pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        unsafe { self.blocking_write_inner(offset, bytes) }
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        unsafe { self.blocking_erase_inner(from, to) }
    }

    unsafe fn blocking_read_inner(&self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        Flash::blocking_read_inner(self.base + offset, bytes)
    }

    unsafe fn blocking_write_inner(&self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        let start_address = self.base + offset;

        // Protect agains simultaneous write/erase to multiple regions.
        let _guard = take_lock_spin();

        Flash::blocking_write_inner(start_address, bytes)
    }

    unsafe fn blocking_erase_inner(&self, from: u32, to: u32) -> Result<(), Error> {
        let start_address = self.base + from;
        let end_address = self.base + to;

        // Protect agains simultaneous write/erase to multiple regions.
        let _guard = take_lock_spin();

        Flash::blocking_erase_inner(start_address, end_address)
    }
}

foreach_flash_region! {
    ($type_name:ident, $write_size:ident, $erase_size:ident) => {
        impl crate::_generated::flash_regions::$type_name {
            pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
                unsafe { self.0.blocking_read_inner(offset, bytes) }
            }

            pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
                unsafe { self.0.blocking_write_inner(offset, bytes) }
            }

            pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
                unsafe { self.0.blocking_erase_inner(from, to) }
            }
        }

        impl ErrorType for crate::_generated::flash_regions::$type_name {
            type Error = Error;
        }

        impl ReadNorFlash for crate::_generated::flash_regions::$type_name {
            const READ_SIZE: usize = 1;

            fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
                unsafe { self.0.blocking_read_inner(offset, bytes) }
            }

            fn capacity(&self) -> usize {
                self.0.size as usize
            }
        }

        impl NorFlash for crate::_generated::flash_regions::$type_name {
            const WRITE_SIZE: usize = $write_size;
            const ERASE_SIZE: usize = $erase_size;

            fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
                unsafe { self.0.blocking_write_inner(offset, bytes) }
            }

            fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
                unsafe { self.0.blocking_erase_inner(from, to) }
            }
        }
    };
}
