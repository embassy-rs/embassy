use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use embedded_storage::nor_flash::{
    check_erase, check_read, check_write, ErrorType, MultiwriteNorFlash, NorFlash, NorFlashError, NorFlashErrorKind,
    ReadNorFlash,
};

use crate::dma::{AnyChannel, Channel, Transfer};
use crate::pac;
use crate::peripherals::FLASH;

pub const FLASH_BASE: *const u32 = 0x10000000 as _;

// If running from RAM, we might have no boot2. Use bootrom `flash_enter_cmd_xip` instead.
// TODO: when run-from-ram is set, completely skip the "pause cores and jumpp to RAM" dance.
pub const USE_BOOT2: bool = !cfg!(feature = "run-from-ram");

// **NOTE**:
//
// These limitations are currently enforced because of using the
// RP2040 boot-rom flash functions, that are optimized for flash compatibility
// rather than performance.
pub const PAGE_SIZE: usize = 256;
pub const WRITE_SIZE: usize = 1;
pub const READ_SIZE: usize = 1;
pub const ERASE_SIZE: usize = 4096;
pub const ASYNC_READ_SIZE: usize = 4;

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

/// Future that waits for completion of a background read
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct BackgroundRead<'a, 'd, T: Instance, const FLASH_SIZE: usize> {
    flash: PhantomData<&'a mut Flash<'d, T, Async, FLASH_SIZE>>,
    transfer: Transfer<'a, AnyChannel>,
}

impl<'a, 'd, T: Instance, const FLASH_SIZE: usize> Future for BackgroundRead<'a, 'd, T, FLASH_SIZE> {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.transfer).poll(cx)
    }
}

impl<'a, 'd, T: Instance, const FLASH_SIZE: usize> Drop for BackgroundRead<'a, 'd, T, FLASH_SIZE> {
    fn drop(&mut self) {
        if pac::XIP_CTRL.stream_ctr().read().0 == 0 {
            return;
        }
        pac::XIP_CTRL
            .stream_ctr()
            .write_value(pac::xip_ctrl::regs::StreamCtr(0));
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        // Errata RP2040-E8: Perform an uncached read to make sure there's not a transfer in
        // flight that might effect an address written to start a new transfer.  This stalls
        // until after any transfer is complete, so the address will not change anymore.
        const XIP_NOCACHE_NOALLOC_BASE: *const u32 = 0x13000000 as *const _;
        unsafe {
            core::ptr::read_volatile(XIP_NOCACHE_NOALLOC_BASE);
        }
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}

pub struct Flash<'d, T: Instance, M: Mode, const FLASH_SIZE: usize> {
    dma: Option<PeripheralRef<'d, AnyChannel>>,
    phantom: PhantomData<(&'d mut T, M)>,
}

impl<'d, T: Instance, M: Mode, const FLASH_SIZE: usize> Flash<'d, T, M, FLASH_SIZE> {
    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
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

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        check_erase(self, from, to)?;

        trace!(
            "Erasing from 0x{:x} to 0x{:x}",
            FLASH_BASE as u32 + from,
            FLASH_BASE as u32 + to
        );

        let len = to - from;

        unsafe { self.in_ram(|| ram_helpers::flash_range_erase(from, len))? };

        Ok(())
    }

    pub fn blocking_write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Error> {
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

            unsafe { self.in_ram(|| ram_helpers::flash_range_program(unaligned_offset as u32, &pad_buf))? }
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

                unsafe { self.in_ram(|| ram_helpers::flash_range_program(aligned_offset as u32, aligned_data))? }
            } else {
                for chunk in bytes[start_padding..end_padding].chunks_exact(PAGE_SIZE) {
                    let mut ram_buf = [0xFF_u8; PAGE_SIZE];
                    ram_buf.copy_from_slice(chunk);
                    unsafe { self.in_ram(|| ram_helpers::flash_range_program(aligned_offset as u32, &ram_buf))? }
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

            unsafe { self.in_ram(|| ram_helpers::flash_range_program(unaligned_offset as u32, &pad_buf))? }
        }

        Ok(())
    }

    /// Make sure to uphold the contract points with rp2040-flash.
    /// - interrupts must be disabled
    /// - DMA must not access flash memory
    unsafe fn in_ram(&mut self, operation: impl FnOnce()) -> Result<(), Error> {
        // Make sure we're running on CORE0
        let core_id: u32 = pac::SIO.cpuid().read();
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
            // Wait for completion of any background reads
            while pac::XIP_CTRL.stream_ctr().read().0 > 0 {}

            // Run our flash operation in RAM
            operation();
        });

        // Resume CORE1 execution
        crate::multicore::resume_core1();
        Ok(())
    }

    /// Read SPI flash unique ID
    pub fn blocking_unique_id(&mut self, uid: &mut [u8]) -> Result<(), Error> {
        unsafe { self.in_ram(|| ram_helpers::flash_unique_id(uid))? };
        Ok(())
    }

    /// Read SPI flash JEDEC ID
    pub fn blocking_jedec_id(&mut self) -> Result<u32, Error> {
        let mut jedec = None;
        unsafe {
            self.in_ram(|| {
                jedec.replace(ram_helpers::flash_jedec_id());
            })?;
        };
        Ok(jedec.unwrap())
    }
}

impl<'d, T: Instance, const FLASH_SIZE: usize> Flash<'d, T, Blocking, FLASH_SIZE> {
    pub fn new_blocking(_flash: impl Peripheral<P = T> + 'd) -> Self {
        Self {
            dma: None,
            phantom: PhantomData,
        }
    }
}

impl<'d, T: Instance, const FLASH_SIZE: usize> Flash<'d, T, Async, FLASH_SIZE> {
    pub fn new(_flash: impl Peripheral<P = T> + 'd, dma: impl Peripheral<P = impl Channel> + 'd) -> Self {
        into_ref!(dma);
        Self {
            dma: Some(dma.map_into()),
            phantom: PhantomData,
        }
    }

    pub fn background_read<'a>(
        &'a mut self,
        offset: u32,
        data: &'a mut [u32],
    ) -> Result<BackgroundRead<'a, 'd, T, FLASH_SIZE>, Error> {
        trace!(
            "Reading in background from 0x{:x} to 0x{:x}",
            FLASH_BASE as u32 + offset,
            FLASH_BASE as u32 + offset + (data.len() * 4) as u32
        );
        // Can't use check_read because we need to enforce 4-byte alignment
        let offset = offset as usize;
        let length = data.len() * 4;
        if length > self.capacity() || offset > self.capacity() - length {
            return Err(Error::OutOfBounds);
        }
        if offset % 4 != 0 {
            return Err(Error::Unaligned);
        }

        while !pac::XIP_CTRL.stat().read().fifo_empty() {
            pac::XIP_CTRL.stream_fifo().read();
        }

        pac::XIP_CTRL
            .stream_addr()
            .write_value(pac::xip_ctrl::regs::StreamAddr(FLASH_BASE as u32 + offset as u32));
        pac::XIP_CTRL
            .stream_ctr()
            .write_value(pac::xip_ctrl::regs::StreamCtr(data.len() as u32));

        // Use the XIP AUX bus port, rather than the FIFO register access (e.x.
        // pac::XIP_CTRL.stream_fifo().as_ptr()) to avoid DMA stalling on
        // general XIP access.
        const XIP_AUX_BASE: *const u32 = 0x50400000 as *const _;
        let transfer = unsafe { crate::dma::read(self.dma.as_mut().unwrap(), XIP_AUX_BASE, data, 37) };

        Ok(BackgroundRead {
            flash: PhantomData,
            transfer,
        })
    }

    pub async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        use core::mem::MaybeUninit;

        // Checked early to simplify address validity checks
        if bytes.len() % 4 != 0 {
            return Err(Error::Unaligned);
        }

        // If the destination address is already aligned, then we can just DMA directly
        if (bytes.as_ptr() as u32) % 4 == 0 {
            // Safety: alignment and size have been checked for compatibility
            let mut buf: &mut [u32] =
                unsafe { core::slice::from_raw_parts_mut(bytes.as_mut_ptr() as *mut u32, bytes.len() / 4) };
            self.background_read(offset, &mut buf)?.await;
            return Ok(());
        }

        // Destination address is unaligned, so use an intermediate buffer
        const REALIGN_CHUNK: usize = PAGE_SIZE;
        // Safety: MaybeUninit requires no initialization
        let mut buf: [MaybeUninit<u32>; REALIGN_CHUNK / 4] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut chunk_offset: usize = 0;
        while chunk_offset < bytes.len() {
            let chunk_size = (bytes.len() - chunk_offset).min(REALIGN_CHUNK);
            let buf = &mut buf[..(chunk_size / 4)];

            // Safety: this is written to completely by DMA before any reads
            let buf = unsafe { &mut *(buf as *mut [MaybeUninit<u32>] as *mut [u32]) };
            self.background_read(offset + chunk_offset as u32, buf)?.await;

            // Safety: [u8] has more relaxed alignment and size requirements than [u32], so this is just aliasing
            let buf = unsafe { core::slice::from_raw_parts(buf.as_ptr() as *const _, buf.len() * 4) };
            bytes[chunk_offset..(chunk_offset + chunk_size)].copy_from_slice(&buf[..chunk_size]);

            chunk_offset += chunk_size;
        }

        Ok(())
    }
}

impl<'d, T: Instance, M: Mode, const FLASH_SIZE: usize> ErrorType for Flash<'d, T, M, FLASH_SIZE> {
    type Error = Error;
}

impl<'d, T: Instance, M: Mode, const FLASH_SIZE: usize> ReadNorFlash for Flash<'d, T, M, FLASH_SIZE> {
    const READ_SIZE: usize = READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }
}

impl<'d, T: Instance, M: Mode, const FLASH_SIZE: usize> MultiwriteNorFlash for Flash<'d, T, M, FLASH_SIZE> {}

impl<'d, T: Instance, M: Mode, const FLASH_SIZE: usize> NorFlash for Flash<'d, T, M, FLASH_SIZE> {
    const WRITE_SIZE: usize = WRITE_SIZE;

    const ERASE_SIZE: usize = ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.blocking_erase(from, to)
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(offset, bytes)
    }
}

#[cfg(feature = "nightly")]
impl<'d, T: Instance, const FLASH_SIZE: usize> embedded_storage_async::nor_flash::ReadNorFlash
    for Flash<'d, T, Async, FLASH_SIZE>
{
    const READ_SIZE: usize = ASYNC_READ_SIZE;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read(offset, bytes).await
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }
}

#[cfg(feature = "nightly")]
impl<'d, T: Instance, const FLASH_SIZE: usize> embedded_storage_async::nor_flash::NorFlash
    for Flash<'d, T, Async, FLASH_SIZE>
{
    const WRITE_SIZE: usize = WRITE_SIZE;

    const ERASE_SIZE: usize = ERASE_SIZE;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.blocking_erase(from, to)
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(offset, bytes)
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
    /// If `USE_BOOT2` is `true`, a copy of the 2nd stage boot loader
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
    pub unsafe fn flash_range_erase(addr: u32, len: u32) {
        let mut boot2 = [0u32; 256 / 4];
        let ptrs = if USE_BOOT2 {
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
    /// If `USE_BOOT2` is `true`, a copy of the 2nd stage boot loader
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
    pub unsafe fn flash_range_erase_and_program(addr: u32, data: &[u8]) {
        let mut boot2 = [0u32; 256 / 4];
        let ptrs = if USE_BOOT2 {
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
    /// If `USE_BOOT2` is `true`, a copy of the 2nd stage boot loader
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
    pub unsafe fn flash_range_program(addr: u32, data: &[u8]) {
        let mut boot2 = [0u32; 256 / 4];
        let ptrs = if USE_BOOT2 {
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
    pub unsafe fn flash_unique_id(out: &mut [u8]) {
        let mut boot2 = [0u32; 256 / 4];
        let ptrs = if USE_BOOT2 {
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
    pub unsafe fn flash_jedec_id() -> u32 {
        let mut boot2 = [0u32; 256 / 4];
        let ptrs = if USE_BOOT2 {
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
    pub trait Mode {}
}

pub trait Instance: sealed::Instance {}
pub trait Mode: sealed::Mode {}

impl sealed::Instance for FLASH {}
impl Instance for FLASH {}

macro_rules! impl_mode {
    ($name:ident) => {
        impl sealed::Mode for $name {}
        impl Mode for $name {}
    };
}

pub struct Blocking;
pub struct Async;

impl_mode!(Blocking);
impl_mode!(Async);
