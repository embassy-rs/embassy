//! Detect flash ECC errors using DMA.
//!
//! Reading a flash word with a double ECC error using the CPU causes a bus fault, and the
//! flash CRC unit does not raise ECC errors at all. Reading the flash with (M)DMA instead
//! aborts the transfer and latches the error in the flash status registers, which allows
//! scrubbing the flash for errors without faulting (see AN5342).
//!
//! This example corrupts the ECC of two flash words in the last sector by programming them
//! twice (see AN5342, appendix A), and then detects the errors using MDMA.
//!
//! WARNING: this erases the last flash sector.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dma::{self, Channel};
use embassy_stm32::flash::{Blocking, EccCheckError, FLASH_BASE, Flash, WRITE_SIZE};
use embassy_stm32::{bind_interrupts, peripherals};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    MDMA => dma::InterruptHandler<peripherals::MDMA_CH0>;
});

/// Offset of the last 128K sector of the STM32H723ZG.
const SECTOR_START: u32 = 0xE_0000;
const SECTOR_END: u32 = SECTOR_START + 128 * 1024;

const SINGLE_ERROR_OFFSET: u32 = SECTOR_START + 0xC0;
const DOUBLE_ERROR_OFFSET: u32 = SECTOR_START + 0x2000;

/// Check `start..end` for ECC errors, and return the number of errors found.
///
/// The region is read in chunks of the size of `buf`. A check stops at the first error,
/// so continue after the failing flash word until the whole region has been checked.
async fn scrub(flash: &mut Flash<'_, Blocking>, dma: &mut Channel<'_>, start: u32, end: u32, buf: &mut [u8]) -> usize {
    let mut errors = 0;
    let mut offset = start;
    while offset < end {
        let len = buf.len().min((end - offset) as usize);
        match flash.read_with_ecc_check(dma, offset, &mut buf[..len]).await {
            Ok(()) => offset += len as u32,
            Err(EccCheckError::Ecc(e)) => {
                errors += 1;
                let error_offset = e.address - FLASH_BASE as u32;
                if e.double {
                    warn!("Double ECC error at {:#x}", e.address);
                } else {
                    // Single errors are corrected, so it is safe to read the data with the CPU.
                    let mut data = [0; WRITE_SIZE];
                    unwrap!(flash.blocking_read(error_offset, &mut data));
                    warn!("Single ECC error at {:#x}, corrected data: {:x}", e.address, data);
                }
                offset = error_offset + WRITE_SIZE as u32;
            }
            Err(e) => defmt::panic!("ECC check failed: {}", e),
        }
    }
    errors
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut flash = Flash::new_blocking(p.FLASH);
    let mut dma = Channel::new(p.MDMA_CH0, Irqs);
    let mut buf = [0u8; 1024];

    info!("Erasing sector at offset {:#x}", SECTOR_START);
    unwrap!(flash.blocking_erase(SECTOR_START, SECTOR_END));

    info!("Checking erased sector");
    let errors = scrub(&mut flash, &mut dma, SECTOR_START, SECTOR_END, &mut buf).await;
    defmt::assert_eq!(errors, 0);
    info!("No ECC errors");

    // Writing a flash word twice leads to an inconsistent ECC code.
    //
    // For this data pattern and address, clearing a single data bit in the second write results in a
    // single (correctable) error, so reads return the data of the first write. This pattern was found
    // experimentally on an STM32H723ZG; other patterns or addresses may result in a double error.
    let first = [0xCC; WRITE_SIZE];
    let mut second = first;
    second[1] &= !(1 << 6);
    info!("Creating single ECC error at offset {:#x}", SINGLE_ERROR_OFFSET);
    unwrap!(flash.blocking_write(SINGLE_ERROR_OFFSET, &first));
    unwrap!(flash.blocking_write(SINGLE_ERROR_OFFSET, &second));

    // Most rewrites result in a double (uncorrectable) error.
    let mut first = [0xCC; WRITE_SIZE];
    first[0] = 0xCB;
    let second = [0xCC; WRITE_SIZE];
    info!("Creating double ECC error at offset {:#x}", DOUBLE_ERROR_OFFSET);
    unwrap!(flash.blocking_write(DOUBLE_ERROR_OFFSET, &first));
    unwrap!(flash.blocking_write(DOUBLE_ERROR_OFFSET, &second));

    info!("Checking sector");
    let errors = scrub(&mut flash, &mut dma, SECTOR_START, SECTOR_END, &mut buf).await;
    info!("Sector checked, found {} ECC errors", errors);

    info!("Erasing sector to clean up");
    unwrap!(flash.blocking_erase(SECTOR_START, SECTOR_END));
    let errors = scrub(&mut flash, &mut dma, SECTOR_START, SECTOR_END, &mut buf).await;
    defmt::assert_eq!(errors, 0);
    info!("Done, no ECC errors left");

    // Call semihosting SYS_EXIT with ADP_Stopped_ApplicationExit.
    unsafe { cortex_m::asm::semihosting_syscall(0x18, 0x20026) };
}
