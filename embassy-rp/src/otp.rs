//! Interface to the RP2350's One Time Programmable Memory

// Credit: taken from `rp-hal` (also licensed Apache+MIT)
// https://github.com/rp-rs/rp-hal/blob/main/rp235x-hal/src/rom_data.rs

/// The ways in which we can fail to read OTP
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The user passed an invalid index to a function.
    InvalidIndex,
    /// The hardware refused to let us read this word, probably due to
    /// read lock set earlier in the boot process.
    InvalidPermissions,
}

/// OTP read address, using automatic Error Correction.
///
/// A 32-bit read returns the ECC-corrected data for two neighbouring rows, or
/// all-ones on permission failure. Only the first 8 KiB is populated.
pub const OTP_DATA_BASE: *const u32 = 0x4013_0000 as *const u32;

/// OTP read address, without using any automatic Error Correction.
///
/// A 32-bit read returns 24-bits of raw data from the OTP word.
pub const OTP_DATA_RAW_BASE: *const u32 = 0x4013_4000 as *const u32;

/// How many pages in OTP (post error-correction)
pub const NUM_PAGES: usize = 64;

/// How many rows in one page in OTP (post error-correction)
pub const NUM_ROWS_PER_PAGE: usize = 64;

/// How many rows in OTP (post error-correction)
pub const NUM_ROWS: usize = NUM_PAGES * NUM_ROWS_PER_PAGE;

/// Read one ECC protected word from the OTP
pub fn read_ecc_word(row: usize) -> Result<u16, Error> {
    if row >= NUM_ROWS {
        return Err(Error::InvalidIndex);
    }
    // First do a raw read to check permissions
    let _ = read_raw_word(row)?;
    // One 32-bit read gets us two rows
    let offset = row >> 1;
    // # Safety
    //
    // We checked this offset was in range already.
    let value = unsafe { OTP_DATA_BASE.add(offset).read() };
    if (row & 1) == 0 {
        Ok(value as u16)
    } else {
        Ok((value >> 16) as u16)
    }
}

/// Read one raw word from the OTP
///
/// You get the 24-bit raw value in the lower part of the 32-bit result.
pub fn read_raw_word(row: usize) -> Result<u32, Error> {
    if row >= NUM_ROWS {
        return Err(Error::InvalidIndex);
    }
    // One 32-bit read gets us one row
    // # Safety
    //
    // We checked this offset was in range already.
    let value = unsafe { OTP_DATA_RAW_BASE.add(row).read() };
    if value == 0xFFFF_FFFF {
        Err(Error::InvalidPermissions)
    } else {
        Ok(value)
    }
}

/// Get the random 64bit chipid from rows 0x0-0x3.
pub fn get_chipid() -> Result<u64, Error> {
    let w0 = read_ecc_word(0x000)?.to_be_bytes();
    let w1 = read_ecc_word(0x001)?.to_be_bytes();
    let w2 = read_ecc_word(0x002)?.to_be_bytes();
    let w3 = read_ecc_word(0x003)?.to_be_bytes();

    Ok(u64::from_be_bytes([
        w3[0], w3[1], w2[0], w2[1], w1[0], w1[1], w0[0], w0[1],
    ]))
}

/// Get the 128bit private random number from rows 0x4-0xb.
///
/// This ID is not exposed through the USB PICOBOOT GET_INFO command
/// or the ROM get_sys_info() API. However note that the USB PICOBOOT OTP
/// access point can read the entirety of page 0, so this value is not
/// meaningfully private unless the USB PICOBOOT interface is disabled via the
//// DISABLE_BOOTSEL_USB_PICOBOOT_IFC flag in BOOT_FLAGS0
pub fn get_private_random_number() -> Result<u128, Error> {
    let w0 = read_ecc_word(0x004)?.to_be_bytes();
    let w1 = read_ecc_word(0x005)?.to_be_bytes();
    let w2 = read_ecc_word(0x006)?.to_be_bytes();
    let w3 = read_ecc_word(0x007)?.to_be_bytes();
    let w4 = read_ecc_word(0x008)?.to_be_bytes();
    let w5 = read_ecc_word(0x009)?.to_be_bytes();
    let w6 = read_ecc_word(0x00a)?.to_be_bytes();
    let w7 = read_ecc_word(0x00b)?.to_be_bytes();

    Ok(u128::from_be_bytes([
        w7[0], w7[1], w6[0], w6[1], w5[0], w5[1], w4[0], w4[1], w3[0], w3[1], w2[0], w2[1], w1[0], w1[1], w0[0], w0[1],
    ]))
}
