#![macro_use]
//! Nonvolatile memory/Flash controller

use crate::{Peri, pac};
use core::future::{Future, poll_fn};
use core::marker::PhantomData;
use core::task::Poll;
use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;
use mspm0_metapac::flashctl::{Flashctl, regs, vals};

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

pub enum FlashError {
    WriteProtected,
    VerifyFailed,
    IllegalAddr,
    BankModeMismatch,
    AddrNotErased,
    MiscError,
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
        // TODO: not sure this is correct ordering
        self.regs.cmddata(0).write(|w| *w = (value & 0xffff) as u32);
        self.regs.cmddata(1).write(|w| *w = (value >> 32) as u32);
        self.regs.cmdexec().write(|w| w.set_val(true));
        self.status_future()
    }

    /// NOTE: this will enable all dynamic write-protects
    pub fn erase_page(&mut self, addr: &*mut u8) -> impl Future<Output = Result<(), FlashError>> {
        self.regs.cmdtype().write(|w| {
            w.set_command(vals::Command::ERASE);
            w.set_size(vals::Size::SECTOR);
        });
        self.regs.cmdaddr().write(|w| *w = addr as *const _ as u32);
        self.regs.cmdexec().write(|w| w.set_val(true));

        self.status_future()
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
