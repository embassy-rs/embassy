use embedded_storage::nor_flash::{
    check_erase, check_read, check_write, ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

pub const FLASH_BASE: usize = 0x10000000;

// **NOTE**:
//
// These limitations are currently enforced because of using the
// RP2040 boot-rom flash functions, that are optimized for flash compatibility
// rather than performance.
pub const WRITE_SIZE: usize = 256;
pub const READ_SIZE: usize = 1;
pub const ERASE_SIZE: usize = 4096;

/// Error type for NVMC operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Opration using a location not in flash.
    OutOfBounds,
    /// Unaligned operation or using unaligned buffers.
    Unaligned,
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
            Self::Other => NorFlashErrorKind::Other,
        }
    }
}

pub struct Flash<const FLASH_SIZE: usize>;

impl<const FLASH_SIZE: usize> Flash<FLASH_SIZE> {
    /// Make sure to uphold the contract points with rp2040-flash.
    /// - interrupts must be disabled
    /// - DMA must not access flash memory
    unsafe fn in_ram(&mut self, operation: impl FnOnce()) {
        let dma_status = &mut [false; crate::dma::CHANNEL_COUNT];

        // TODO: Make sure CORE1 is paused during the entire duration of the RAM function

        critical_section::with(|_| {
            // Pause all DMA channels for the duration of the ram operation
            for (number, status) in dma_status.iter_mut().enumerate() {
                let ch = crate::pac::DMA.ch(number as _);
                *status = ch.ctrl_trig().read().en();
                if *status {
                    ch.ctrl_trig().modify(|w| w.set_en(false));
                }
            }

            // Run our flash operation in RAM
            operation();

            // Re-enable previously enabled DMA channels
            for (number, status) in dma_status.iter().enumerate() {
                let ch = crate::pac::DMA.ch(number as _);
                if *status {
                    ch.ctrl_trig().modify(|w| w.set_en(true));
                }
            }
        });
    }
}

impl<const FLASH_SIZE: usize> ErrorType for Flash<FLASH_SIZE> {
    type Error = Error;
}

impl<const FLASH_SIZE: usize> ReadNorFlash for Flash<FLASH_SIZE> {
    const READ_SIZE: usize = READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        check_read(self, offset, bytes.len())?;

        let flash_data = unsafe { core::slice::from_raw_parts((FLASH_BASE as u32 + offset) as *const u8, bytes.len()) };

        bytes.copy_from_slice(flash_data);
        Ok(())
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl<const FLASH_SIZE: usize> NorFlash for Flash<FLASH_SIZE> {
    const WRITE_SIZE: usize = WRITE_SIZE;

    const ERASE_SIZE: usize = ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        check_erase(self, from, to)?;

        let len = to - from;

        unsafe { self.in_ram(|| ram_helpers::flash_range_erase(from, len, true)) };

        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        check_write(self, offset, bytes.len())?;

        trace!("Writing {:?} bytes to 0x{:x}", bytes.len(), offset);

        unsafe { self.in_ram(|| ram_helpers::flash_range_program(offset, bytes, true)) };

        Ok(())
    }
}

mod ram_helpers {
    use core::marker::PhantomData;

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
            rom_data::memcpy44(&mut boot2 as *mut _, 0x10000000 as *const _, 256);
            flash_function_pointers_with_boot2(true, false, &boot2)
        } else {
            flash_function_pointers(true, false)
        };
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
            rom_data::memcpy44(&mut boot2 as *mut _, 0x10000000 as *const _, 256);
            flash_function_pointers_with_boot2(true, true, &boot2)
        } else {
            flash_function_pointers(true, true)
        };
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
            rom_data::memcpy44(&mut boot2 as *mut _, 0x10000000 as *const _, 256);
            flash_function_pointers_with_boot2(false, true, &boot2)
        } else {
            flash_function_pointers(false, true)
        };
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
}
