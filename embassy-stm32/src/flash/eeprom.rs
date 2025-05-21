use super::{family, Blocking, Error, Flash, EEPROM_BASE, EEPROM_SIZE};

#[cfg(eeprom)]
impl<'d> Flash<'d, Blocking> {
    // --- Internal helpers ---

    fn check_eeprom_offset(&self, offset: u32, size: u32) -> Result<(), Error> {
        if offset
            .checked_add(size)
            .filter(|&end| end <= EEPROM_SIZE as u32)
            .is_some()
        {
            Ok(())
        } else {
            Err(Error::Size)
        }
    }

    // --- Unlocked (unsafe, internal) functions ---

    unsafe fn eeprom_write_u8_slice_unlocked(&self, offset: u32, data: &[u8]) -> Result<(), Error> {
        for (i, &byte) in data.iter().enumerate() {
            let addr = EEPROM_BASE as u32 + offset + i as u32;
            core::ptr::write_volatile(addr as *mut u8, byte);
            family::wait_ready_blocking()?;
            family::clear_all_err();
        }
        Ok(())
    }

    unsafe fn eeprom_write_u16_slice_unlocked(&self, offset: u32, data: &[u16]) -> Result<(), Error> {
        for (i, &value) in data.iter().enumerate() {
            let addr = EEPROM_BASE as u32 + offset + i as u32 * 2;
            core::ptr::write_volatile(addr as *mut u16, value);
            family::wait_ready_blocking()?;
            family::clear_all_err();
        }
        Ok(())
    }

    unsafe fn eeprom_write_u32_slice_unlocked(&self, offset: u32, data: &[u32]) -> Result<(), Error> {
        for (i, &value) in data.iter().enumerate() {
            let addr = EEPROM_BASE as u32 + offset + i as u32 * 4;
            core::ptr::write_volatile(addr as *mut u32, value);
            family::wait_ready_blocking()?;
            family::clear_all_err();
        }
        Ok(())
    }

    // --- Public, safe API ---

    pub fn eeprom_write_u8(&mut self, offset: u32, value: u8) -> Result<(), Error> {
        self.check_eeprom_offset(offset, 1)?;
        unsafe {
            family::unlock();
        }
        unsafe {
            self.eeprom_write_u8_slice_unlocked(offset, core::slice::from_ref(&value))?;
        }
        unsafe {
            family::lock();
        }
        Ok(())
    }

    pub fn eeprom_write_u16(&mut self, offset: u32, value: u16) -> Result<(), Error> {
        if offset % 2 != 0 {
            return Err(Error::Unaligned);
        }
        self.check_eeprom_offset(offset, 2)?;
        unsafe {
            family::unlock();
        }
        unsafe {
            self.eeprom_write_u16_slice_unlocked(offset, core::slice::from_ref(&value))?;
        }
        unsafe {
            family::lock();
        }
        Ok(())
    }

    pub fn eeprom_write_u32(&mut self, offset: u32, value: u32) -> Result<(), Error> {
        if offset % 4 != 0 {
            return Err(Error::Unaligned);
        }
        self.check_eeprom_offset(offset, 4)?;
        unsafe {
            family::unlock();
        }
        unsafe {
            self.eeprom_write_u32_slice_unlocked(offset, core::slice::from_ref(&value))?;
        }
        unsafe {
            family::lock();
        }
        Ok(())
    }

    pub fn eeprom_write_u8_slice(&mut self, offset: u32, data: &[u8]) -> Result<(), Error> {
        self.check_eeprom_offset(offset, data.len() as u32)?;
        unsafe {
            family::unlock();
        }
        unsafe {
            self.eeprom_write_u8_slice_unlocked(offset, data)?;
        }
        unsafe {
            family::lock();
        }
        Ok(())
    }

    pub fn eeprom_write_u16_slice(&mut self, offset: u32, data: &[u16]) -> Result<(), Error> {
        if offset % 2 != 0 {
            return Err(Error::Unaligned);
        }
        self.check_eeprom_offset(offset, data.len() as u32 * 2)?;
        unsafe {
            family::unlock();
        }
        unsafe {
            self.eeprom_write_u16_slice_unlocked(offset, data)?;
        }
        unsafe {
            family::lock();
        }
        Ok(())
    }

    pub fn eeprom_write_u32_slice(&mut self, offset: u32, data: &[u32]) -> Result<(), Error> {
        if offset % 4 != 0 {
            return Err(Error::Unaligned);
        }
        self.check_eeprom_offset(offset, data.len() as u32 * 4)?;
        unsafe {
            family::unlock();
        }
        unsafe {
            self.eeprom_write_u32_slice_unlocked(offset, data)?;
        }
        unsafe {
            family::lock();
        }
        Ok(())
    }

    pub fn eeprom_write(&mut self, offset: u32, data: &[u8]) -> Result<(), Error> {
        let start = offset;
        let end = offset.checked_add(data.len() as u32).ok_or(Error::Size)?;
        if end > EEPROM_SIZE as u32 {
            return Err(Error::Size);
        }

        let misalign = (start % 4) as usize;
        let prefix_len = if misalign == 0 {
            0
        } else {
            (4 - misalign).min(data.len())
        };
        let (prefix, rest) = data.split_at(prefix_len);
        let aligned_len = (rest.len() / 4) * 4;
        let (aligned, suffix) = rest.split_at(aligned_len);

        unsafe {
            family::unlock();
        }
        if !prefix.is_empty() {
            unsafe {
                self.eeprom_write_u8_slice_unlocked(start, prefix)?;
            }
        }
        if !aligned.is_empty() {
            let aligned_offset = start + prefix_len as u32;
            let u32_data = unsafe { core::slice::from_raw_parts(aligned.as_ptr() as *const u32, aligned.len() / 4) };
            unsafe {
                self.eeprom_write_u32_slice_unlocked(aligned_offset, u32_data)?;
            }
        }
        if !suffix.is_empty() {
            let suffix_offset = start + (prefix_len + aligned_len) as u32;
            unsafe {
                self.eeprom_write_u8_slice_unlocked(suffix_offset, suffix)?;
            }
        }
        unsafe {
            family::lock();
        }
        Ok(())
    }

    pub fn eeprom_read_u8(&self, offset: u32) -> Result<u8, Error> {
        self.check_eeprom_offset(offset, 1)?;
        let addr = EEPROM_BASE as u32 + offset;
        Ok(unsafe { core::ptr::read_volatile(addr as *const u8) })
    }

    pub fn eeprom_read_u16(&self, offset: u32) -> Result<u16, Error> {
        if offset % 2 != 0 {
            return Err(Error::Unaligned);
        }
        self.check_eeprom_offset(offset, 2)?;
        let addr = EEPROM_BASE as u32 + offset;
        Ok(unsafe { core::ptr::read_volatile(addr as *const u16) })
    }

    pub fn eeprom_read_u32(&self, offset: u32) -> Result<u32, Error> {
        if offset % 4 != 0 {
            return Err(Error::Unaligned);
        }
        self.check_eeprom_offset(offset, 4)?;
        let addr = EEPROM_BASE as u32 + offset;
        Ok(unsafe { core::ptr::read_volatile(addr as *const u32) })
    }

    pub fn eeprom_read_slice(&self, offset: u32, buf: &mut [u8]) -> Result<(), Error> {
        self.check_eeprom_offset(offset, buf.len() as u32)?;
        let addr = EEPROM_BASE as u32 + offset;
        let src = unsafe { core::slice::from_raw_parts(addr as *const u8, buf.len()) };
        buf.copy_from_slice(src);
        Ok(())
    }
}
