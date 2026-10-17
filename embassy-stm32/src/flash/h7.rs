use core::ptr::write_volatile;
use core::sync::atomic::{Ordering, fence};

use embassy_sync::waitqueue::AtomicWaker;
use pac::flash::regs::Sr;

use super::{BANK1_REGION, FLASH_REGIONS, Flash, FlashBank, FlashSector, WRITE_SIZE};
use crate::dma::{Channel, TransferOptions};
use crate::flash::Error;
use crate::pac;

static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) unsafe fn on_interrupt() {
    // Clear IRQ flags
    pac::FLASH.bank(0).ccr().write(|w| {
        w.set_clr_eop(true);
        w.set_clr_operr(true);
    });
    if is_dual_bank() {
        pac::FLASH.bank(1).ccr().write(|w| {
            w.set_clr_eop(true);
            w.set_clr_operr(true);
        });
    }

    WAKER.wake();
}

const fn is_dual_bank() -> bool {
    FLASH_REGIONS.len() >= 2
}

pub(crate) unsafe fn lock() {
    pac::FLASH.bank(0).cr().modify(|w| w.set_lock(true));
    if is_dual_bank() {
        pac::FLASH.bank(1).cr().modify(|w| w.set_lock(true));
    }
}

pub(crate) unsafe fn unlock() {
    if pac::FLASH.bank(0).cr().read().lock() {
        pac::FLASH.bank(0).keyr().write_value(0x4567_0123);
        pac::FLASH.bank(0).keyr().write_value(0xCDEF_89AB);
    }
    if is_dual_bank() {
        if pac::FLASH.bank(1).cr().read().lock() {
            pac::FLASH.bank(1).keyr().write_value(0x4567_0123);
            pac::FLASH.bank(1).keyr().write_value(0xCDEF_89AB);
        }
    }
}

pub(crate) unsafe fn enable_write() {
    enable_blocking_write();
}

pub(crate) unsafe fn disable_write() {
    disable_blocking_write();
}

pub(crate) unsafe fn enable_blocking_write() {
    assert_eq!(0, WRITE_SIZE % 4);
}

pub(crate) unsafe fn disable_blocking_write() {}

pub(crate) async unsafe fn write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    // We cannot have the write setup sequence in begin_write as it depends on the address
    let bank = if start_address >= BANK1_REGION.base() && start_address < BANK1_REGION.end() {
        pac::FLASH.bank(0)
    } else {
        pac::FLASH.bank(1)
    };
    bank.cr().write(|w| {
        w.set_pg(true);
        #[cfg(flash_h7)]
        w.set_psize(2); // 32 bits at once
        w.set_eopie(true);
        w.set_operrie(true);
    });
    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let mut res = None;
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;

        res = Some(wait_ready(bank).await);
        bank.sr().modify(|w| {
            if w.eop() {
                w.set_eop(true);
            }
        });
        if unwrap!(res).is_err() {
            break;
        }
    }

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    bank.cr().modify(|w| {
        w.set_pg(false);
        w.set_eopie(false);
        w.set_operrie(false);
    });

    unwrap!(res)
}

pub(crate) unsafe fn blocking_write(start_address: u32, buf: &[u8; WRITE_SIZE]) -> Result<(), Error> {
    // We cannot have the write setup sequence in begin_write as it depends on the address
    let bank = if start_address >= BANK1_REGION.base() && start_address < BANK1_REGION.end() {
        pac::FLASH.bank(0)
    } else {
        pac::FLASH.bank(1)
    };
    bank.cr().write(|w| {
        w.set_pg(true);
        #[cfg(flash_h7)]
        w.set_psize(2); // 32 bits at once
    });
    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let mut res = None;
    let mut address = start_address;
    for val in buf.chunks(4) {
        write_volatile(address as *mut u32, u32::from_le_bytes(unwrap!(val.try_into())));
        address += val.len() as u32;

        res = Some(blocking_wait_ready(bank));
        bank.sr().modify(|w| {
            if w.eop() {
                w.set_eop(true);
            }
        });
        if unwrap!(res).is_err() {
            break;
        }
    }

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    bank.cr().modify(|w| w.set_pg(false));

    unwrap!(res)
}

pub(crate) async unsafe fn erase_sector(
    sector: &FlashSector,
    #[cfg(flash_h7)] parallelism: Option<super::EraseParallelism>,
) -> Result<(), Error> {
    let bank = pac::FLASH.bank(sector.bank as usize);
    bank.cr().modify(|w| {
        #[cfg(flash_h7)]
        if let Some(parallelism) = parallelism {
            w.set_psize(parallelism.psize());
        }
        w.set_ser(true);
        #[cfg(flash_h7)]
        w.set_snb(sector.index_in_bank);
        #[cfg(flash_h7ab)]
        w.set_ssn(sector.index_in_bank);
        w.set_eopie(true);
        w.set_operrie(true);
    });

    bank.cr().modify(|w| {
        w.set_start(true);
    });

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let ret: Result<(), Error> = wait_ready(bank).await;
    bank.cr().modify(|w| {
        w.set_ser(false);
        w.set_eopie(false);
        w.set_operrie(false);
    });
    bank_clear_all_err(bank);
    ret
}

pub(crate) unsafe fn blocking_erase_sector(
    sector: &FlashSector,
    #[cfg(flash_h7)] parallelism: Option<super::EraseParallelism>,
) -> Result<(), Error> {
    let bank = pac::FLASH.bank(sector.bank as usize);
    bank.cr().modify(|w| {
        #[cfg(flash_h7)]
        if let Some(parallelism) = parallelism {
            w.set_psize(parallelism.psize());
        }
        w.set_ser(true);
        #[cfg(flash_h7)]
        w.set_snb(sector.index_in_bank);
        #[cfg(flash_h7ab)]
        w.set_ssn(sector.index_in_bank);
    });

    bank.cr().modify(|w| {
        w.set_start(true);
    });

    cortex_m::asm::isb();
    cortex_m::asm::dsb();
    fence(Ordering::SeqCst);

    let ret: Result<(), Error> = blocking_wait_ready(bank);
    bank.cr().modify(|w| w.set_ser(false));
    bank_clear_all_err(bank);
    ret
}

pub(crate) unsafe fn clear_all_err() {
    bank_clear_all_err(pac::FLASH.bank(0));
    bank_clear_all_err(pac::FLASH.bank(1));
}

unsafe fn bank_clear_all_err(bank: pac::flash::Bank) {
    // The status flags are cleared through the clear control register, which has the
    // clear bits at the same positions as the flags in the status register.
    // Mask out the busy bits (15:0), which are reserved in CCR.
    const FLAGS_MASK: u32 = 0x1FFF_0000;
    let sr = bank.sr().read();
    bank.ccr().write_value(pac::flash::regs::Ccr(sr.0 & FLAGS_MASK));
}

/// An ECC error reported by the flash memory.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EccError {
    /// The bank in which the error occurred.
    pub bank: FlashBank,
    /// At least one single bit error was detected and corrected.
    pub single: bool,
    /// At least one double bit error was detected. The data read from this location is invalid.
    pub double: bool,
    /// Absolute address of the flash word that caused the first ECC error.
    pub address: u32,
}

/// Errors returned by [`Flash::ecc_check`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EccCheckError {
    /// The requested range is not valid.
    Flash(Error),
    /// An ECC error was detected while reading the requested range.
    Ecc(EccError),
    /// The DMA transfer failed, but the flash did not report an ECC error.
    Dma,
}

fn bank_regs(bank: FlashBank) -> pac::flash::Bank {
    match bank {
        FlashBank::Bank2 => pac::FLASH.bank(1),
        _ => pac::FLASH.bank(0),
    }
}

fn bank_base(bank: FlashBank) -> u32 {
    unwrap!(FLASH_REGIONS.iter().find(|r| r.bank == bank)).base()
}

fn banks() -> &'static [FlashBank] {
    if is_dual_bank() {
        &[FlashBank::Bank1, FlashBank::Bank2]
    } else {
        &[FlashBank::Bank1]
    }
}

fn bank_ecc_error(bank: FlashBank) -> Option<EccError> {
    let regs = bank_regs(bank);
    let sr = regs.sr().read();
    if !sr.sneccerr1() && !sr.dbeccerr() {
        return None;
    }

    // FAR contains the index of the failing flash word, relative to the start of the bank.
    let far = regs.far().read().fail_ecc_addr() as u32;
    Some(EccError {
        bank,
        single: sr.sneccerr1(),
        double: sr.dbeccerr(),
        address: bank_base(bank) + far * WRITE_SIZE as u32,
    })
}

impl<'d, MODE> Flash<'d, MODE> {
    /// Get the ECC error that is currently latched by the flash, if any.
    ///
    /// If both banks have an ECC error latched, the error of bank 1 is returned.
    ///
    /// Use [`Flash::clear_ecc_error`] to clear it.
    pub fn ecc_error(&self) -> Option<EccError> {
        banks().iter().find_map(|&bank| bank_ecc_error(bank))
    }

    /// Clear latched ECC errors of all banks.
    pub fn clear_ecc_error(&mut self) {
        for &bank in banks() {
            bank_regs(bank).ccr().write(|w| {
                w.set_clr_sneccerr(true);
                w.set_clr_dbeccerr(true);
            });
        }
    }

    /// Read a flash region using DMA, checking it for ECC errors.
    ///
    /// Reading a flash word that contains a double ECC error with the CPU causes a bus fault.
    /// Reading it with DMA instead only aborts the DMA transfer, which makes it possible to
    /// detect corruption without faulting. Note that the flash CRC unit can not be used for
    /// this, as CRC reads do not raise ECC errors.
    ///
    /// `offset` is an offset from the flash start, NOT an absolute address. The region that is
    /// checked is `offset..offset + buf.len()`. If this function returns `Ok`, `buf` contains the data
    /// of that region. `buf` must be accessible by the DMA channel.
    ///
    /// Any ECC errors that are latched before the check starts are cleared.
    ///
    /// The check stops at the first ECC error that is found. The flash only latches the address of
    /// the first error, so the returned [`EccError`] can also have its `double` flag set because of
    /// a later flash word in the same chunk. To continue checking after an error, call this
    /// function again starting from the flash word after [`EccError::address`].
    pub async fn read_with_ecc_check(
        &mut self,
        dma: &mut Channel<'_>,
        offset: u32,
        buf: &mut [u8],
    ) -> Result<(), EccCheckError> {
        let (base, size, offset) =
            super::check_flash_address(super::FLASH_BASE as u32, offset).map_err(EccCheckError::Flash)?;
        if offset + buf.len() as u32 > size {
            return Err(EccCheckError::Flash(Error::Size));
        }

        let mut options = TransferOptions::default();
        options.panic_on_error = false;

        #[cfg(dma)]
        {
            // Memory to memory transfers are not allowed in direct mode, which the DMA driver uses
            // if no FIFO threshold is set (and the source and destination word sizes are equal), so
            // we have to configure it.
            options.fifo_threshold = Some(crate::dma::FifoThreshold::Full);
        }

        self.clear_ecc_error();

        let start = base + offset;

        // The DMA can transfer at most 0xFFFF items at once.
        const CHUNK_SIZE: usize = 0xFFFF;
        for (i, chunk) in buf.chunks_mut(CHUNK_SIZE).enumerate() {
            let address = start + (i * CHUNK_SIZE) as u32;
            let src = core::ptr::slice_from_raw_parts(address as *const u8, chunk.len());

            let mut transfer = unsafe { dma.transfer(0, src, chunk.as_mut_ptr(), options) };
            (&mut transfer).await;
            let dma_error = transfer.is_error();
            drop(transfer);

            // Single errors don't abort the transfer, but stop here anyway: further errors
            // would be reported with the address of this one.
            if let Some(err) = self.ecc_error() {
                return Err(EccCheckError::Ecc(err));
            }
            if dma_error {
                return Err(EccCheckError::Dma);
            }
        }

        Ok(())
    }
}

async fn wait_ready(bank: pac::flash::Bank) -> Result<(), Error> {
    use core::future::poll_fn;
    use core::task::Poll;

    poll_fn(|cx| {
        WAKER.register(cx.waker());

        let sr = bank.sr().read();
        if !sr.bsy() && !sr.qw() {
            Poll::Ready(get_result(sr))
        } else {
            return Poll::Pending;
        }
    })
    .await
}

unsafe fn blocking_wait_ready(bank: pac::flash::Bank) -> Result<(), Error> {
    loop {
        let sr = bank.sr().read();

        if !sr.bsy() && !sr.qw() {
            return get_result(sr);
        }
    }
}

fn get_result(sr: Sr) -> Result<(), Error> {
    if sr.wrperr() {
        Err(Error::Protected)
    } else if sr.pgserr() {
        error!("pgserr");
        Err(Error::Seq)
    } else if sr.incerr() {
        // writing to a different address when programming 256 bit word was not finished
        error!("incerr");
        Err(Error::Seq)
    } else if sr.crcrderr() {
        error!("crcrderr");
        Err(Error::Seq)
    } else if sr.operr() {
        Err(Error::Prog)
    } else if sr.sneccerr1() {
        // single ECC error
        Err(Error::Prog)
    } else if sr.dbeccerr() {
        // double ECC error
        Err(Error::Prog)
    } else if sr.rdperr() {
        Err(Error::Protected)
    } else if sr.rdserr() {
        Err(Error::Protected)
    } else {
        Ok(())
    }
}
