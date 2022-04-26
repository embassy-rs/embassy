use crate::pac;
use crate::peripherals::FLASH;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::ptr::write_volatile;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

pub use crate::pac::ERASE_SIZE;
pub use crate::pac::ERASE_VALUE;
pub use crate::pac::FLASH_BASE;
pub use crate::pac::FLASH_SIZE;
pub use crate::pac::WRITE_SIZE;
const FLASH_END: usize = FLASH_BASE + FLASH_SIZE;

pub struct Flash<'d> {
    _inner: FLASH,
    _phantom: PhantomData<&'d mut FLASH>,
}

impl<'d> Flash<'d> {
    pub fn new(p: impl Unborrow<Target = FLASH>) -> Self {
        unborrow!(p);
        Self {
            _inner: p,
            _phantom: PhantomData,
        }
    }

    pub fn unlock(p: impl Unborrow<Target = FLASH>) -> Self {
        let flash = Self::new(p);
        #[cfg(any(flash_wl, flash_wb, flash_l4))]
        unsafe {
            pac::FLASH.keyr().write(|w| w.set_keyr(0x4567_0123));
            pac::FLASH.keyr().write(|w| w.set_keyr(0xCDEF_89AB));
        }

        #[cfg(any(flash_l0))]
        unsafe {
            pac::FLASH.pekeyr().write(|w| w.set_pekeyr(0x89ABCDEF));
            pac::FLASH.pekeyr().write(|w| w.set_pekeyr(0x02030405));

            pac::FLASH.prgkeyr().write(|w| w.set_prgkeyr(0x8C9DAEBF));
            pac::FLASH.prgkeyr().write(|w| w.set_prgkeyr(0x13141516));
        }
        flash
    }

    pub fn lock(&mut self) {
        #[cfg(any(flash_wl, flash_wb, flash_l4))]
        unsafe {
            pac::FLASH.cr().modify(|w| w.set_lock(true));
        }

        #[cfg(any(flash_l0))]
        unsafe {
            pac::FLASH.pecr().modify(|w| {
                w.set_optlock(true);
                w.set_prglock(true);
                w.set_pelock(true);
            });
        }
    }

    pub fn blocking_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Error> {
        let offset = FLASH_BASE as u32 + offset;
        if offset as usize >= FLASH_END || offset as usize + bytes.len() > FLASH_END {
            return Err(Error::Size);
        }

        let flash_data = unsafe { core::slice::from_raw_parts(offset as *const u8, bytes.len()) };
        bytes.copy_from_slice(flash_data);
        Ok(())
    }

    pub fn blocking_write(&mut self, offset: u32, buf: &[u8]) -> Result<(), Error> {
        let offset = FLASH_BASE as u32 + offset;
        if offset as usize + buf.len() > FLASH_END {
            return Err(Error::Size);
        }
        if offset as usize % WRITE_SIZE != 0 || buf.len() as usize % WRITE_SIZE != 0 {
            return Err(Error::Unaligned);
        }
        trace!("Writing {} bytes at 0x{:x}", buf.len(), offset);

        self.clear_all_err();

        #[cfg(any(flash_wl, flash_wb, flash_l4))]
        unsafe {
            pac::FLASH.cr().write(|w| w.set_pg(true))
        }

        let mut ret: Result<(), Error> = Ok(());
        let mut offset = offset;
        for chunk in buf.chunks(WRITE_SIZE) {
            for val in chunk.chunks(4) {
                unsafe {
                    write_volatile(
                        offset as *mut u32,
                        u32::from_le_bytes(val[0..4].try_into().unwrap()),
                    );
                }
                offset += val.len() as u32;
            }

            ret = self.blocking_wait_ready();
            if ret.is_err() {
                break;
            }
        }

        #[cfg(any(flash_wl, flash_wb, flash_l4))]
        unsafe {
            pac::FLASH.cr().write(|w| w.set_pg(false))
        }

        ret
    }

    pub fn blocking_erase(&mut self, from: u32, to: u32) -> Result<(), Error> {
        let from = FLASH_BASE as u32 + from;
        let to = FLASH_BASE as u32 + to;
        if to < from || to as usize > FLASH_END {
            return Err(Error::Size);
        }
        if from as usize % ERASE_SIZE != 0 || to as usize % ERASE_SIZE != 0 {
            return Err(Error::Unaligned);
        }

        self.clear_all_err();

        for page in (from..to).step_by(ERASE_SIZE) {
            #[cfg(any(flash_l0, flash_l1))]
            unsafe {
                pac::FLASH.pecr().modify(|w| {
                    w.set_erase(true);
                    w.set_prog(true);
                });

                write_volatile(page as *mut u32, 0xFFFFFFFF);
            }

            #[cfg(any(flash_wl, flash_wb, flash_l4))]
            unsafe {
                let idx = page / ERASE_SIZE as u32;

                pac::FLASH.cr().modify(|w| {
                    w.set_per(true);
                    w.set_pnb(idx as u8);
                    #[cfg(any(flash_wl, flash_wb))]
                    w.set_strt(true);
                    #[cfg(any(flash_l4))]
                    w.set_start(true);
                });
            }

            let ret: Result<(), Error> = self.blocking_wait_ready();

            #[cfg(any(flash_wl, flash_wb, flash_l4))]
            unsafe {
                pac::FLASH.cr().modify(|w| w.set_per(false));
            }

            #[cfg(any(flash_l0, flash_l1))]
            unsafe {
                pac::FLASH.pecr().modify(|w| {
                    w.set_erase(false);
                    w.set_prog(false);
                });
            }

            self.clear_all_err();
            if ret.is_err() {
                return ret;
            }
        }

        Ok(())
    }

    fn blocking_wait_ready(&self) -> Result<(), Error> {
        loop {
            let sr = unsafe { pac::FLASH.sr().read() };

            if !sr.bsy() {
                #[cfg(any(flash_wl, flash_wb, flash_l4))]
                if sr.progerr() {
                    return Err(Error::Prog);
                }

                if sr.wrperr() {
                    return Err(Error::Protected);
                }

                if sr.pgaerr() {
                    return Err(Error::Unaligned);
                }

                if sr.sizerr() {
                    return Err(Error::Size);
                }

                #[cfg(any(flash_wl, flash_wb, flash_l4))]
                if sr.miserr() {
                    return Err(Error::Miss);
                }

                #[cfg(any(flash_wl, flash_wb, flash_l4))]
                if sr.pgserr() {
                    return Err(Error::Seq);
                }
                return Ok(());
            }
        }
    }

    fn clear_all_err(&mut self) {
        unsafe {
            pac::FLASH.sr().modify(|w| {
                #[cfg(any(flash_wl, flash_wb, flash_l4, flash_l0))]
                if w.rderr() {
                    w.set_rderr(false);
                }
                #[cfg(any(flash_wl, flash_wb, flash_l4))]
                if w.fasterr() {
                    w.set_fasterr(false);
                }
                #[cfg(any(flash_wl, flash_wb, flash_l4))]
                if w.miserr() {
                    w.set_miserr(false);
                }
                #[cfg(any(flash_wl, flash_wb, flash_l4))]
                if w.pgserr() {
                    w.set_pgserr(false);
                }
                if w.sizerr() {
                    w.set_sizerr(false);
                }
                if w.pgaerr() {
                    w.set_pgaerr(false);
                }
                if w.wrperr() {
                    w.set_wrperr(false);
                }
                #[cfg(any(flash_wl, flash_wb, flash_l4))]
                if w.progerr() {
                    w.set_progerr(false);
                }
                #[cfg(any(flash_wl, flash_wb, flash_l4))]
                if w.operr() {
                    w.set_operr(false);
                }
            });
        }
    }
}

impl Drop for Flash<'_> {
    fn drop(&mut self) {
        self.lock();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Prog,
    Size,
    Miss,
    Seq,
    Protected,
    Unaligned,
}

impl<'d> ErrorType for Flash<'d> {
    type Error = Error;
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Self::Size => NorFlashErrorKind::OutOfBounds,
            Self::Unaligned => NorFlashErrorKind::NotAligned,
            _ => NorFlashErrorKind::Other,
        }
    }
}

impl<'d> ReadNorFlash for Flash<'d> {
    const READ_SIZE: usize = WRITE_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl<'d> NorFlash for Flash<'d> {
    const WRITE_SIZE: usize = WRITE_SIZE;
    const ERASE_SIZE: usize = ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        self.blocking_erase(from, to)
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(offset, bytes)
    }
}

/*
cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")]
    {
        use embedded_storage_async::nor_flash::{AsyncNorFlash, AsyncReadNorFlash};
        use core::future::Future;

        impl<'d> AsyncNorFlash for Flash<'d> {
            const WRITE_SIZE: usize = <Self as NorFlash>::WRITE_SIZE;
            const ERASE_SIZE: usize = <Self as NorFlash>::ERASE_SIZE;

            type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
            fn write<'a>(&'a mut self, offset: u32, data: &'a [u8]) -> Self::WriteFuture<'a> {
                async move {
                    todo!()
                }
            }

            type EraseFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
            fn erase<'a>(&'a mut self, from: u32, to: u32) -> Self::EraseFuture<'a> {
                async move {
                    todo!()
                }
            }
        }

        impl<'d> AsyncReadNorFlash for Flash<'d> {
            const READ_SIZE: usize = <Self as ReadNorFlash>::READ_SIZE;
            type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a where Self: 'a;
            fn read<'a>(&'a mut self, address: u32, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
                async move {
                    todo!()
                }
            }

            fn capacity(&self) -> usize {
                FLASH_SIZE
            }
        }
    }
}
*/
