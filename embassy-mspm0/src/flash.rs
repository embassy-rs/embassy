#![macro_use]
//! Nonvolatile memory/Flash controller

use core::future::{Future, poll_fn};
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use embedded_storage_async::nor_flash::{ErrorType, MultiwriteNorFlash, NorFlash, NorFlashError, ReadNorFlash};
use mspm0_metapac::flashctl::{Flashctl, regs, vals};

use crate::{Peri, pac};

pub struct FlashController<'d> {
    regs: Flashctl,
    _phantom: PhantomData<&'d mut ()>,
}

static FLASH_WAKER: AtomicWaker = AtomicWaker::new();

#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType {}
pub(crate) trait SealedInstance {
    fn regs() -> Flashctl;
}

macro_rules! impl_flash_instance {
    ($instance: ident) => {
        impl crate::flash::SealedInstance for crate::peripherals::$instance {
            fn regs() -> crate::pac::flashctl::Flashctl {
                crate::pac::$instance
            }
        }

        impl crate::flash::Instance for crate::peripherals::$instance {}
    };
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub enum FlashError {
    WriteProtected,
    VerifyFailed,
    IllegalAddr,
    BankModeMismatch,
    AddrNotErased,
    AddrOutOfRange,
    BufferInvalidSize,
    MiscError,
}

impl NorFlashError for FlashError {
    fn kind(&self) -> embedded_storage_async::nor_flash::NorFlashErrorKind {
        match self {
            FlashError::WriteProtected => embedded_storage_async::nor_flash::NorFlashErrorKind::Other,
            FlashError::VerifyFailed => embedded_storage_async::nor_flash::NorFlashErrorKind::Other,
            FlashError::IllegalAddr => embedded_storage_async::nor_flash::NorFlashErrorKind::NotAligned,
            FlashError::BankModeMismatch => embedded_storage_async::nor_flash::NorFlashErrorKind::Other,
            FlashError::AddrNotErased => embedded_storage_async::nor_flash::NorFlashErrorKind::Other,
            FlashError::MiscError => embedded_storage_async::nor_flash::NorFlashErrorKind::Other,
            FlashError::AddrOutOfRange => embedded_storage_async::nor_flash::NorFlashErrorKind::OutOfBounds,
            FlashError::BufferInvalidSize => embedded_storage_async::nor_flash::NorFlashErrorKind::NotAligned,
        }
    }
}

impl<'d> FlashController<'d> {
    pub fn new<T: Instance>(_instance: Peri<'d, T>) -> Self {
        let regs = T::regs();

        #[cfg(feature = "rt")]
        regs.imask().write(|w| w.set_done(true));

        Self {
            regs,
            _phantom: PhantomData,
        }

    }

    pub fn disable_dyn_writeprotect(&mut self) {
        self.regs.cmdweprota().write(|w| w.0 = 0);
        self.regs.cmdweprotb().write(|w| w.0 = 0);
        self.regs.cmdweprotc().write(|w| w.0 = 0);
        // Non-main is config data, this is usually not intended to overwrite
        // self.regs.cmdweprotnm().write(|w| w.0 = 0);
    }

    fn status_to_error(cmd: regs::Statcmd) -> Option<FlashError> {
        Some(if cmd.failweprot() {
            FlashError::WriteProtected
        } else if cmd.failmode() {
            FlashError::BankModeMismatch
        } else if cmd.faililladdr() {
            FlashError::IllegalAddr
        } else if cmd.failverify() {
            FlashError::VerifyFailed
        } else if cmd.failinvdata() {
            FlashError::AddrNotErased
        } else if cmd.failmisc() {
            FlashError::MiscError
        } else {
            None?
        })
    }

    fn status_future(&self) -> impl Future<Output = Result<(), FlashError>> {
        poll_fn(move |cx| {
            let status = self.regs.statcmd().read();
            if !status.inprogress() {
                Poll::Ready(Ok(()))
            } else if let Some(err) = Self::status_to_error(status) {
                Poll::Ready(Err(err))
            } else {
                FLASH_WAKER.register(cx.waker());
                Poll::Pending
            }
        })
    }

    /// NOTE: this will enable all dynamic write-protects
    pub fn blank_verify_word(&mut self, addr: *mut u8) -> impl Future<Output = Result<(), FlashError>> {
        debug_assert!(
            (addr as *mut u64).is_aligned(),
            "Address should be flash word (8 byte) aligned"
        );

        self.regs.cmdtype().write(|w| {
            w.set_command(vals::Command::BLANKVERIFY);
            w.set_size(vals::Size::ONEWORD);
        });
        self.regs.cmdaddr().write(|w| *w = addr as *const _ as u32);
        self.regs.cmdexec().write(|w| w.set_val(true));

        self.status_future()
    }

    /// Address should be flash word aligned (8 byte)
    ///
    /// NOTE: this will enable all dynamic write-protects
    pub fn program_word(&mut self, addr: *mut u8, value: u64) -> impl Future<Output = Result<(), FlashError>> {
        debug_assert!(
            (addr as *mut u64).is_aligned(),
            "Address should be flash word (8 byte) aligned"
        );

        self.regs.cmdtype().write(|w| {
            w.set_command(vals::Command::PROGRAM);
            w.set_size(vals::Size::ONEWORD);
        });

        self.regs.cmddataindex().write(|w| w.set_val(0));
        self.regs.cmdaddr().write(|w| *w = addr as *const _ as u32);
        self.regs.cmdbyten().write(|w| w.0 |= 0xff); // Write 8 bytes
        // This ends up in cmddata(0) ends up in ptr+0x0000
        self.regs.cmddata(0).write(|w| *w = (value >> 32) as u32);
        // This ends up in cmddata(1) ends up in ptr+0x0004
        self.regs.cmddata(1).write(|w| *w = (value & 0xffffffff) as u32);
        self.regs.cmdexec().write(|w| w.set_val(true));
        self.status_future()
    }

    /// NOTE: this will enable all dynamic write-protects
    pub fn erase_page(&mut self, addr: *mut u8) -> impl Future<Output = Result<(), FlashError>> {
        self.regs.cmdtype().write(|w| {
            w.set_command(vals::Command::ERASE);
            w.set_size(vals::Size::SECTOR);
        });
        self.regs.cmdaddr().write(|w| *w = addr as *const _ as u32);
        self.regs.cmdexec().write(|w| w.set_val(true));

        self.status_future()
    }

    pub fn as_range(&mut self, start: *mut u8, capacity: usize) -> FlashRange<'_, 'd> {
        FlashRange {
            flashctl: self,
            start,
            capacity,
        }
    }
}

pub struct FlashRange<'a, 'b> {
    flashctl: &'a mut FlashController<'b>,
    start: *mut u8,
    capacity: usize,
}

impl ErrorType for FlashRange<'_, '_> {
    type Error = FlashError;
}

impl ReadNorFlash for FlashRange<'_, '_> {
    const READ_SIZE: usize = 1;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if !(0..self.capacity).contains(&(offset as usize)) {
            return Err(FlashError::AddrOutOfRange);
        }

        // TODO: convert start into mut slice
        unsafe {
            core::ptr::copy(
                self.start.wrapping_add(offset as usize) as *const u8,
                bytes.as_mut_ptr(),
                bytes.len(),
            );
        }
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.capacity
    }
}

// TODO: wait for ptr.is_aligned_to to stablize
fn is_aligned_to(ptr: *const u8, align: usize) -> bool {
    (ptr as usize).is_multiple_of(align)
}

impl NorFlash for FlashRange<'_, '_> {
    const WRITE_SIZE: usize = 8;

    const ERASE_SIZE: usize = 1024;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if !is_aligned_to(self.start.wrapping_add(from as usize), Self::ERASE_SIZE) {
            return Err(FlashError::IllegalAddr);
        }

        if !is_aligned_to(self.start.wrapping_add(to as usize), Self::ERASE_SIZE) {
            return Err(FlashError::IllegalAddr);
        }

        for page in (from..to).step_by(Self::ERASE_SIZE) {
            self.flashctl.erase_page(self.start.wrapping_add(page as usize)).await?;
        }

        Ok(())
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if !is_aligned_to(self.start.wrapping_add(offset as usize), Self::WRITE_SIZE) {
            return Err(FlashError::IllegalAddr);
        }

        if !bytes.len().is_multiple_of(Self::WRITE_SIZE) {
            return Err(FlashError::BufferInvalidSize);
        }

        for (i, chunk) in bytes.chunks_exact(8).enumerate() {
            let val = u64::from_be_bytes(chunk.try_into().unwrap());
            // TODO: zip iter addr
            self.flashctl
                .program_word(self.start.wrapping_add(offset as usize + i * 8), val)
                .await?;
        }
        Ok(())
    }
}

// TODO: test; not sure if this works
#[cfg(feature = "rt")]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
fn FLASHCTL() {
    let events = pac::FLASHCTL.mis().read();

    if events.done() {
        // TODO: wake flash controller future
        FLASH_WAKER.wake();
    }

    // Clear done event for next operation
    pac::FLASHCTL.iclr().write(|w| {
        w.set_done(true);
    });
}
