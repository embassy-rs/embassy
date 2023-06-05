use core::marker::PhantomData;

use embassy_hal_common::Peripheral;
use embedded_storage::nor_flash::{
    check_erase, check_read, check_write, ErrorType, MultiwriteNorFlash, NorFlash, NorFlashError, NorFlashErrorKind,
    ReadNorFlash,
};

use crate::pac;
use crate::peripherals::FLASH;

pub const FLASH_BASE: *const u32 = 0x10000000 as _;

// **NOTE**:
//
// These limitations are currently enforced because of using the
// RP2040 boot-rom flash functions, that are optimized for flash compatibility
// rather than performance.
pub const PAGE_SIZE: usize = 256;
pub const WRITE_SIZE: usize = 1;
pub const READ_SIZE: usize = 1;
pub const ERASE_SIZE: usize = 4096;

/// Error type for NVMC operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Operation using a location not in flash.
    OutOfBounds,
    /// Unaligned operation or using unaligned buffers.
    Unaligned,
    InvalidCore,
    Other,
}

impl From<NorFlashErrorKind> for Error {
    fn from(e: NorFlashErrorKind) -> Self {
        match e {
            NorFlashErrorKind::NotAligned => Self::Unaligned,
            NorFlashErrorKind::OutOfBounds => Self::OutOfBounds,
            _ => Self::Other,
        }
    }
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Self::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            Self::Unaligned => NorFlashErrorKind::NotAligned,
            _ => NorFlashErrorKind::Other,
        }
    }
}

pub struct Flash<'d, T: Instance, const FLASH_SIZE: usize>(PhantomData<&'d mut T>);

impl<'d, T: Instance, const FLASH_SIZE: usize> Flash<'d, T, FLASH_SIZE> {
    pub fn new(_flash: impl Peripheral<P = T> + 'd) -> Self {
        Self(PhantomData)
    }

    pub fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        trace!(
            "Reading from 0x{:x} to 0x{:x}",
            FLASH_BASE as u32 + offset,
            FLASH_BASE as u32 + offset + bytes.len() as u32
        );
        check_read(self, offset, bytes.len())?;

        let flash_data = unsafe { core::slice::from_raw_parts((FLASH_BASE as u32 + offset) as *const u8, bytes.len()) };

        bytes.copy_from_slice(flash_data);
        Ok(())
    }

    pub fn capacity(&self) -> usize {
        FLASH_SIZE
    }

    pub fn erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        check_erase(self, from, to)?;

        trace!(
            "Erasing from 0x{:x} to 0x{:x}",
            FLASH_BASE as u32 + from,
            FLASH_BASE as u32 + to
        );

        let len = to - from;

        unsafe { self.in_ram(|| ram_helpers::flash_range_erase(from, len, true))? };

        Ok(())
    }

    pub fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
        check_write(self, offset, bytes.len())?;

        trace!("Writing {:?} bytes to 0x{:x}", bytes.len(), FLASH_BASE as u32 + offset);

        let end_offset = offset as usize + bytes.len();

        let padded_offset = (offset as *const u8).align_offset(PAGE_SIZE);
        let start_padding = core::cmp::min(padded_offset, bytes.len());

        // Pad in the beginning
        if start_padding > 0 {
            let start = PAGE_SIZE - padded_offset;
            let end = start + start_padding;

            let mut pad_buf = [0xFF_u8; PAGE_SIZE];
            pad_buf[start..end].copy_from_slice(&bytes[..start_padding]);

            let unaligned_offset = offset as usize - start;

            unsafe { self.in_ram(|| ram_helpers::flash_range_program(unaligned_offset as u32, &pad_buf, true))? }
        }

        let remaining_len = bytes.len() - start_padding;
        let end_padding = start_padding + PAGE_SIZE * (remaining_len / PAGE_SIZE);

        // Write aligned slice of length in multiples of 256 bytes
        // If the remaining bytes to be written is more than a full page.
        if remaining_len >= PAGE_SIZE {
            let mut aligned_offset = if start_padding > 0 {
                offset as usize + padded_offset
            } else {
                offset as usize
            };

            if bytes.as_ptr() as usize >= 0x2000_0000 {
                let aligned_data = &bytes[start_padding..end_padding];

                unsafe { self.in_ram(|| ram_helpers::flash_range_program(aligned_offset as u32, aligned_data, true))? }
            } else {
                for chunk in bytes[start_padding..end_padding].chunks_exact(PAGE_SIZE) {
                    let mut ram_buf = [0xFF_u8; PAGE_SIZE];
                    ram_buf.copy_from_slice(chunk);
                    unsafe { self.in_ram(|| ram_helpers::flash_range_program(aligned_offset as u32, &ram_buf, true))? }
                    aligned_offset += PAGE_SIZE;
                }
            }
        }

        // Pad in the end
        let rem_offset = (end_offset as *const u8).align_offset(PAGE_SIZE);
        let rem_padding = remaining_len % PAGE_SIZE;
        if rem_padding > 0 {
            let mut pad_buf = [0xFF_u8; PAGE_SIZE];
            pad_buf[..rem_padding].copy_from_slice(&bytes[end_padding..]);

            let unaligned_offset = end_offset - (PAGE_SIZE - rem_offset);

            unsafe { self.in_ram(|| ram_helpers::flash_range_program(unaligned_offset as u32, &pad_buf, true))? }
        }

        Ok(())
    }

    /// Make sure to uphold the contract points with rp2040-flash.
    /// - interrupts must be disabled
    /// - DMA must not access flash memory
    unsafe fn in_ram(&mut self, operation: impl FnOnce()) -> Result<(), Error> {
        // Make sure we're running on CORE0
        let core_id: u32 = unsafe { pac::SIO.cpuid().read() };
        if core_id != 0 {
            return Err(Error::InvalidCore);
        }

        // Make sure CORE1 is paused during the entire duration of the RAM function
        crate::multicore::pause_core1();

        critical_section::with(|_| {
            // Wait for all DMA channels in flash to finish before ram operation
            const SRAM_LOWER: u32 = 0x2000_0000;
            for n in 0..crate::dma::CHANNEL_COUNT {
                let ch = crate::pac::DMA.ch(n);
                while ch.read_addr().read() < SRAM_LOWER && ch.ctrl_trig().read().busy() {}
            }

            // Run our flash operation in RAM
            operation();
        });

        // Resume CORE1 execution
        crate::multicore::resume_core1();
        Ok(())
    }

    /// Read SPI flash unique ID
    pub fn unique_id(&mut self, uid: &mut [u8]) -> Result<(), Error> {
        unsafe { self.in_ram(|| ram_helpers::flash_unique_id(uid, true))? };
        Ok(())
    }

    /// Read SPI flash JEDEC ID
    pub fn jedec_id(&mut self) -> Result<u32, Error> {
        let mut jedec = None;
        unsafe {
            self.in_ram(|| {
                jedec.replace(ram_helpers::flash_jedec_id(true));
            })?;
        };
        Ok(jedec.unwrap())
    }
}

impl<'d, T: Instance, const FLASH_SIZE: usize> ErrorType for Flash<'d, T, FLASH_SIZE> {
    type Error = Error;
}

impl<'d, T: Instance, const FLASH_SIZE: usize> ReadNorFlash for Flash<'d, T, FLASH_SIZE> {
    const READ_SIZE: usize = READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }
}

impl<'d, T: Instance, const FLASH_SIZE: usize> MultiwriteNorFlash for Flash<'d, T, FLASH_SIZE> {}

impl<'d, T: Instance, const FLASH_SIZE: usize> NorFlash for Flash<'d, T, FLASH_SIZE> {
    const WRITE_SIZE: usize = WRITE_SIZE;

    const ERASE_SIZE: usize = ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.erase(from, to)
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write(offset, bytes)
    }
}

#[allow(dead_code)]
mod ram_helpers {
    use core::marker::PhantomData;

    use super::*;
    use crate::rom_data;

    #[repr(C)]
    struct FlashFunctionPointers<'a> {
        connect_internal_flash: unsafe extern "C" fn() -> (),
        flash_exit_xip: unsafe extern "C" fn() -> (),
        flash_range_erase: Option<unsafe extern "C" fn(addr: u32, count: usize, block_size: u32, block_cmd: u8) -> ()>,
        flash_range_program: Option<unsafe extern "C" fn(addr: u32, data: *const u8, count: usize) -> ()>,
        flash_flush_cache: unsafe extern "C" fn() -> (),
        flash_enter_cmd_xip: unsafe extern "C" fn() -> (),
        phantom: PhantomData<&'a ()>,
    }

    #[allow(unused)]
    fn flash_function_pointers(erase: bool, write: bool) -> FlashFunctionPointers<'static> {
        FlashFunctionPointers {
            connect_internal_flash: rom_data::connect_internal_flash::ptr(),
            flash_exit_xip: rom_data::flash_exit_xip::ptr(),
            flash_range_erase: if erase {
                Some(rom_data::flash_range_erase::ptr())
            } else {
                None
            },
            flash_range_program: if write {
                Some(rom_data::flash_range_program::ptr())
            } else {
                None
            },
            flash_flush_cache: rom_data::flash_flush_cache::ptr(),
            flash_enter_cmd_xip: rom_data::flash_enter_cmd_xip::ptr(),
            phantom: PhantomData,
        }
    }

    #[allow(unused)]
    /// # Safety
    ///
    /// `boot2` must contain a valid 2nd stage boot loader which can be called to re-initialize XIP mode
    unsafe fn flash_function_pointers_with_boot2(erase: bool, write: bool, boot2: &[u32; 64]) -> FlashFunctionPointers {
        let boot2_fn_ptr = (boot2 as *const u32 as *const u8).offset(1);
        let boot2_fn: unsafe extern "C" fn() -> () = core::mem::transmute(boot2_fn_ptr);
        FlashFunctionPointers {
            connect_internal_flash: rom_data::connect_internal_flash::ptr(),
            flash_exit_xip: rom_data::flash_exit_xip::ptr(),
            flash_range_erase: if erase {
                Some(rom_data::flash_range_erase::ptr())
            } else {
                None
            },
            flash_range_program: if write {
                Some(rom_data::flash_range_program::ptr())
            } else {
                None
            },
            flash_flush_cache: rom_data::flash_flush_cache::ptr(),
            flash_enter_cmd_xip: boot2_fn,
            phantom: PhantomData,
        }
    }

    /// Erase a flash range starting at `addr` with length `len`.
    ///
    /// `addr` and `len` must be multiples of 4096
    ///
    /// If `use_boot2` is `true`, a copy of the 2nd stage boot loader
    /// is used to re-initialize the XIP engine after flashing.
    ///
    /// # Safety
    ///
    /// Nothing must access flash while this is running.
    /// Usually this means:
    ///   - interrupts must be disabled
    ///   - 2nd core must be running code from RAM or ROM with interrupts disabled
    ///   - DMA must not access flash memory
    ///
    /// `addr` and `len` parameters must be valid and are not checked.
    pub unsafe fn flash_range_erase(addr: u32, len: u32, use_boot2: bool) {
        let mut boot2 = [0u32; 256 / 4];
        let ptrs = if use_boot2 {
            rom_data::memcpy44(&mut boot2 as *mut _, FLASH_BASE, 256);
            flash_function_pointers_with_boot2(true, false, &boot2)
        } else {
            flash_function_pointers(true, false)
        };

        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        write_flash_inner(addr, len, None, &ptrs as *const FlashFunctionPointers);
    }

    /// Erase and rewrite a flash range starting at `addr` with data `data`.
    ///
    /// `addr` and `data.len()` must be multiples of 4096
    ///
    /// If `use_boot2` is `true`, a copy of the 2nd stage boot loader
    /// is used to re-initialize the XIP engine after flashing.
    ///
    /// # Safety
    ///
    /// Nothing must access flash while this is running.
    /// Usually this means:
    ///   - interrupts must be disabled
    ///   - 2nd core must be running code from RAM or ROM with interrupts disabled
    ///   - DMA must not access flash memory
    ///
    /// `addr` and `len` parameters must be valid and are not checked.
    pub unsafe fn flash_range_erase_and_program(addr: u32, data: &[u8], use_boot2: bool) {
        let mut boot2 = [0u32; 256 / 4];
        let ptrs = if use_boot2 {
            rom_data::memcpy44(&mut boot2 as *mut _, FLASH_BASE, 256);
            flash_function_pointers_with_boot2(true, true, &boot2)
        } else {
            flash_function_pointers(true, true)
        };

        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        write_flash_inner(
            addr,
            data.len() as u32,
            Some(data),
            &ptrs as *const FlashFunctionPointers,
        );
    }

    /// Write a flash range starting at `addr` with data `data`.
    ///
    /// `addr` and `data.len()` must be multiples of 256
    ///
    /// If `use_boot2` is `true`, a copy of the 2nd stage boot loader
    /// is used to re-initialize the XIP engine after flashing.
    ///
    /// # Safety
    ///
    /// Nothing must access flash while this is running.
    /// Usually this means:
    ///   - interrupts must be disabled
    ///   - 2nd core must be running code from RAM or ROM with interrupts disabled
    ///   - DMA must not access flash memory
    ///
    /// `addr` and `len` parameters must be valid and are not checked.
    pub unsafe fn flash_range_program(addr: u32, data: &[u8], use_boot2: bool) {
        let mut boot2 = [0u32; 256 / 4];
        let ptrs = if use_boot2 {
            rom_data::memcpy44(&mut boot2 as *mut _, FLASH_BASE, 256);
            flash_function_pointers_with_boot2(false, true, &boot2)
        } else {
            flash_function_pointers(false, true)
        };

        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        write_flash_inner(
            addr,
            data.len() as u32,
            Some(data),
            &ptrs as *const FlashFunctionPointers,
        );
    }

    /// # Safety
    ///
    /// Nothing must access flash while this is running.
    /// Usually this means:
    ///   - interrupts must be disabled
    ///   - 2nd core must be running code from RAM or ROM with interrupts disabled
    ///   - DMA must not access flash memory
    /// Length of data must be a multiple of 4096
    /// addr must be aligned to 4096
    #[inline(never)]
    #[link_section = ".data.ram_func"]
    unsafe fn write_flash_inner(addr: u32, len: u32, data: Option<&[u8]>, ptrs: *const FlashFunctionPointers) {
        /*
         Should be equivalent to:
            rom_data::connect_internal_flash();
            rom_data::flash_exit_xip();
            rom_data::flash_range_erase(addr, len, 1 << 31, 0); // if selected
            rom_data::flash_range_program(addr, data as *const _, len); // if selected
            rom_data::flash_flush_cache();
            rom_data::flash_enter_cmd_xip();
        */
        #[cfg(target_arch = "arm")]
        core::arch::asm!(
            "mov r8, r0",
            "mov r9, r2",
            "mov r10, r1",
            "ldr r4, [{ptrs}, #0]",
            "blx r4", // connect_internal_flash()

            "ldr r4, [{ptrs}, #4]",
            "blx r4", // flash_exit_xip()

            "mov r0, r8", // r0 = addr
            "mov r1, r10", // r1 = len
            "movs r2, #1",
            "lsls r2, r2, #31", // r2 = 1 << 31
            "movs r3, #0", // r3 = 0
            "ldr r4, [{ptrs}, #8]",
            "cmp r4, #0",
            "beq 1f",
            "blx r4", // flash_range_erase(addr, len, 1 << 31, 0)
            "1:",

            "mov r0, r8", // r0 = addr
            "mov r1, r9", // r0 = data
            "mov r2, r10", // r2 = len
            "ldr r4, [{ptrs}, #12]",
            "cmp r4, #0",
            "beq 1f",
            "blx r4", // flash_range_program(addr, data, len);
            "1:",

            "ldr r4, [{ptrs}, #16]",
            "blx r4", // flash_flush_cache();

            "ldr r4, [{ptrs}, #20]",
            "blx r4", // flash_enter_cmd_xip();
            ptrs = in(reg) ptrs,
            // Registers r8-r15 are not allocated automatically,
            // so assign them manually. We need to use them as
            // otherwise there are not enough registers available.
            in("r0") addr,
            in("r2") data.map(|d| d.as_ptr()).unwrap_or(core::ptr::null()),
            in("r1") len,
            out("r3") _,
            out("r4") _,
            lateout("r8") _,
            lateout("r9") _,
            lateout("r10") _,
            clobber_abi("C"),
        );
    }

    #[repr(C)]
    struct FlashCommand {
        cmd_addr: *const u8,
        cmd_addr_len: u32,
        dummy_len: u32,
        data: *mut u8,
        data_len: u32,
    }

    /// Return SPI flash unique ID
    ///
    /// Not all SPI flashes implement this command, so check the JEDEC
    /// ID before relying on it. The Winbond parts commonly seen on
    /// RP2040 devboards (JEDEC=0xEF7015) support an 8-byte unique ID;
    /// https://forums.raspberrypi.com/viewtopic.php?t=331949 suggests
    /// that LCSC (Zetta) parts have a 16-byte unique ID (which is
    /// *not* unique in just its first 8 bytes),
    /// JEDEC=0xBA6015. Macronix and Spansion parts do not have a
    /// unique ID.
    ///
    /// The returned bytes are relatively predictable and should be
    /// salted and hashed before use if that is an issue (e.g. for MAC
    /// addresses).
    ///
    /// # Safety
    ///
    /// Nothing must access flash while this is running.
    /// Usually this means:
    ///   - interrupts must be disabled
    ///   - 2nd core must be running code from RAM or ROM with interrupts disabled
    ///   - DMA must not access flash memory
    ///
    /// Credit: taken from `rp2040-flash` (also licensed Apache+MIT)
    pub unsafe fn flash_unique_id(out: &mut [u8], use_boot2: bool) {
        let mut boot2 = [0u32; 256 / 4];
        let ptrs = if use_boot2 {
            rom_data::memcpy44(&mut boot2 as *mut _, FLASH_BASE, 256);
            flash_function_pointers_with_boot2(false, false, &boot2)
        } else {
            flash_function_pointers(false, false)
        };
        // 4B - read unique ID
        let cmd = [0x4B];
        read_flash(&cmd[..], 4, out, &ptrs as *const FlashFunctionPointers);
    }

    /// Return SPI flash JEDEC ID
    ///
    /// This is the three-byte manufacturer-and-model identifier
    /// commonly used to check before using manufacturer-specific SPI
    /// flash features, e.g. 0xEF7015 for Winbond W25Q16JV.
    ///
    /// # Safety
    ///
    /// Nothing must access flash while this is running.
    /// Usually this means:
    ///   - interrupts must be disabled
    ///   - 2nd core must be running code from RAM or ROM with interrupts disabled
    ///   - DMA must not access flash memory
    ///
    /// Credit: taken from `rp2040-flash` (also licensed Apache+MIT)
    pub unsafe fn flash_jedec_id(use_boot2: bool) -> u32 {
        let mut boot2 = [0u32; 256 / 4];
        let ptrs = if use_boot2 {
            rom_data::memcpy44(&mut boot2 as *mut _, FLASH_BASE, 256);
            flash_function_pointers_with_boot2(false, false, &boot2)
        } else {
            flash_function_pointers(false, false)
        };
        let mut id = [0u8; 4];
        // 9F - read JEDEC ID
        let cmd = [0x9F];
        read_flash(&cmd[..], 0, &mut id[1..4], &ptrs as *const FlashFunctionPointers);
        u32::from_be_bytes(id)
    }

    unsafe fn read_flash(cmd_addr: &[u8], dummy_len: u32, out: &mut [u8], ptrs: *const FlashFunctionPointers) {
        read_flash_inner(
            FlashCommand {
                cmd_addr: cmd_addr.as_ptr(),
                cmd_addr_len: cmd_addr.len() as u32,
                dummy_len,
                data: out.as_mut_ptr(),
                data_len: out.len() as u32,
            },
            ptrs,
        );
    }

    /// Issue a generic SPI flash read command
    ///
    /// # Arguments
    ///
    /// * `cmd` - `FlashCommand` structure
    /// * `ptrs` - Flash function pointers as per `write_flash_inner`
    ///
    /// Credit: taken from `rp2040-flash` (also licensed Apache+MIT)
    #[inline(never)]
    #[link_section = ".data.ram_func"]
    unsafe fn read_flash_inner(cmd: FlashCommand, ptrs: *const FlashFunctionPointers) {
        #[cfg(target_arch = "arm")]
        core::arch::asm!(
            "mov r10, r0", // cmd
            "mov r5, r1", // ptrs

            "ldr r4, [r5, #0]",
            "blx r4", // connect_internal_flash()

            "ldr r4, [r5, #4]",
            "blx r4", // flash_exit_xip()


            "movs r4, #0x18",
            "lsls r4, r4, #24", // 0x18000000, SSI, RP2040 datasheet 4.10.13

            // Disable, write 0 to SSIENR
            "movs r0, #0",
            "str r0, [r4, #8]", // SSIENR

            // Write ctrlr0
            "movs r0, #0x3",
            "lsls r0, r0, #8", // TMOD=0x300
            "ldr r1, [r4, #0]", // CTRLR0
            "orrs r1, r0",
            "str r1, [r4, #0]",

            // Write ctrlr1 with len-1
            "mov r3, r10", // cmd
            "ldr r0, [r3, #8]", // dummy_len
            "ldr r1, [r3, #16]", // data_len
            "add r0, r1",
            "subs r0, #1",
            "str r0, [r4, #0x04]", // CTRLR1

            // Enable, write 1 to ssienr
            "movs r0, #1",
            "str r0, [r4, #8]", // SSIENR

            // Write cmd/addr phase to DR
            "mov r2, r4",
            "adds r2, 0x60", // &DR
            "ldr r0, [r3, #0]", // cmd_addr
            "ldr r1, [r3, #4]", // cmd_addr_len
            "10:",
            "ldrb r3, [r0]",
            "strb r3, [r2]", // DR
            "adds r0, #1",
            "subs r1, #1",
            "bne 10b",

            // Skip any dummy cycles
            "mov r3, r10", // cmd
            "ldr r1, [r3, #8]", // dummy_len
            "cmp r1, #0",
            "beq 9f",
            "4:",
            "ldr r3, [r4, #0x28]", // SR
            "movs r2, #0x8",
            "tst r3, r2", // SR.RFNE
            "beq 4b",

            "mov r2, r4",
            "adds r2, 0x60", // &DR
            "ldrb r3, [r2]", // DR
            "subs r1, #1",
            "bne 4b",

            // Read RX fifo
            "9:",
            "mov r2, r10", // cmd
            "ldr r0, [r2, #12]", // data
            "ldr r1, [r2, #16]", // data_len

            "2:",
            "ldr r3, [r4, #0x28]", // SR
            "movs r2, #0x8",
            "tst r3, r2", // SR.RFNE
            "beq 2b",

            "mov r2, r4",
            "adds r2, 0x60", // &DR
            "ldrb r3, [r2]", // DR
            "strb r3, [r0]",
            "adds r0, #1",
            "subs r1, #1",
            "bne 2b",

            // Disable, write 0 to ssienr
            "movs r0, #0",
            "str r0, [r4, #8]", // SSIENR

            // Write 0 to CTRLR1 (returning to its default value)
            //
            // flash_enter_cmd_xip does NOT do this, and everything goes
            // wrong unless we do it here
            "str r0, [r4, #4]", // CTRLR1

            "ldr r4, [r5, #20]",
            "blx r4", // flash_enter_cmd_xip();

            in("r0") &cmd as *const FlashCommand,
            in("r1") ptrs,
            out("r2") _,
            out("r3") _,
            out("r4") _,
            out("r5") _,
            // Registers r8-r10 are used to store values
            // from r0-r2 in registers not clobbered by
            // function calls.
            // The values can't be passed in using r8-r10 directly
            // due to https://github.com/rust-lang/rust/issues/99071
            out("r10") _,
            clobber_abi("C"),
        );
    }
}

mod sealed {
    pub trait Instance {}
}

pub trait Instance: sealed::Instance {}

impl sealed::Instance for FLASH {}
impl Instance for FLASH {}
