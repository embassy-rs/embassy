//! Flexible Memory Controller (FMC)

use embassy_hal_internal::PeripheralType;

use crate::{Peri, rcc};

// Shadow the metapac values to make them more convenient to access.
pub use crate::pac::fmc::vals;

// Implements the SDRAM functionality.
//
// SDRAM registers are not supported by FSMC peripherals, only FMC.
#[cfg(any(fmc_v1x3, fmc_v2x1, fmc_v3x1, fmc_v4))]
pub mod sdram;

#[cfg(any(fmc_v1x3, fmc_v2x1, fmc_v3x1, fmc_v4))]
pub mod nand;

#[cfg(any(fmc_v1x3, fmc_v2x1, fmc_v3x1, fmc_v4))]
pub mod nor_sram;

// fmc_v1x3 has some very different structures to the later FMCs version,
// notably how mapping is handled, and the presence of a PC/CompactFlash
// card controller in the NAND configuration registers.
//
// So the types for fmc_v1x3 have been seperated out into their own module
// to avoid having a huge amount of predicates making the code complex.

#[cfg(fmc_v1x3)]
pub mod v1;
#[cfg(fmc_v1x3)]
pub use v1::*;

#[cfg(not(fmc_v1x3))]
pub mod others;
#[cfg(not(fmc_v1x3))]
pub use others::*;

/// FMC driver
pub struct Fmc<'d, T: Instance> {
    #[allow(unused)]
    peri: Peri<'d, T>,

    // Specifies the bank mapping in used by the FMC.
    #[cfg(not(fmc_v1x3))]
    mapping: FmcBankMapping,
}

unsafe impl<'d, T> Send for Fmc<'d, T> where T: Instance {}

impl<'d, T> Fmc<'d, T>
where
    T: Instance,
{
    /// Create an FMC instance.
    pub fn new(peri: Peri<'d, T>) -> Self {
        Self {
            peri,
            #[cfg(not(fmc_v1x3))]
            mapping: FmcBankMapping::Default,
        }
    }

    /// Returns the bank mapping in use by the FMC.
    ///
    /// This can be used to derrive the correct mapped
    /// memory addresses of the various banks.
    #[cfg(not(fmc_v1x3))]
    pub fn mapping(&self) -> FmcBankMapping {
        self.mapping
    }

    /// Returns the pointer to the specified SDRAM bank.
    #[cfg(not(fmc_v1x3))]
    pub fn sdram_ptr(&self, bank: FmcSdramBank) -> *mut u32 {
        (match self.mapping {
            FmcBankMapping::Default => match bank {
                // Note Bank 1 is mapped to 0x7000_0000 and 0xC000_0000.
                FmcSdramBank::Bank1 => FmcBank::Bank5.ptr(), // 0xC000_0000
                FmcSdramBank::Bank2 => FmcBank::Bank6.ptr(), // 0xD000_0000
            },
            FmcBankMapping::NorSdramSwapped => match bank {
                FmcSdramBank::Bank1 => FmcBank::Bank1.ptr(), // 0x6000_0000
                FmcSdramBank::Bank2 => FmcBank::Bank6.ptr(), // 0xD000_0000
            },
            FmcBankMapping::Sdram2Swapped => match bank {
                FmcSdramBank::Bank1 => FmcBank::Bank5.ptr(), // 0xC000_0000
                // Note Bank 1 is mapped to 0x7000_0000 and 0xD000_0000.
                FmcSdramBank::Bank2 => FmcBank::Bank6.ptr(), // 0xD000_0000
            },
        } as *mut u32)
    }

    /// Returns the pointer to the specified SDRAM bank.
    #[cfg(fmc_v1x3)]
    pub fn sdram_ptr(&self, bank: FmcSdramBank) -> *mut u32 {
        match bank {
            // Note Bank 1 is mapped to 0x7000_0000 and 0xC000_0000.
            FmcSdramBank::Bank1 => FmcBank::Bank5.ptr(), // 0xC000_0000
            FmcSdramBank::Bank2 => FmcBank::Bank6.ptr(), // 0xD000_0000
        }
    }

    /// Returns a pointer to the address for the specified NAND bank.
    #[cfg(fmc_v1x3)]
    pub fn nand_ptr(&self, bank: FmcNandBank) -> *mut u32 {
        match bank {
            FmcNandBank::Bank1 => FmcBank::Bank2.ptr(),
            FmcNandBank::Bank2 => FmcBank::Bank3.ptr(),
        }
    }

    /// Returns the address to the NOR/PSRAM/SRAM bank.
    #[cfg(not(fmc_v1x3))]
    pub fn nor_sram_addr(&self, bank: FmcSramBank) -> u32 {
        bank.addr(self.mapping)
    }

    /// Returns the base pointer to the NOR/PSRAM/SRAM bank.
    #[cfg(not(fmc_v1x3))]
    pub fn nor_sram_ptr(&self, bank: FmcSramBank) -> *mut u32 {
        self.nor_sram_addr(bank) as *mut u32
    }

    /// Returns the address to the NOR/PSRAM/SRAM bank.
    #[cfg(fmc_v1x3)]
    pub fn nor_sram_addr(&self, bank: FmcSramBank) -> u32 {
        bank.addr()
    }

    /// Returns the base pointer to the NOR/PSRAM/SRAM bank.
    #[cfg(fmc_v1x3)]
    pub fn nor_sram_ptr(&self, bank: FmcSramBank) -> *mut u32 {
        self.nor_sram_addr(bank) as *mut u32
    }

    /// Enable the FMC peripheral and reset it.
    ///
    /// This should be called before configuring any of the FMC memory device controllers.
    pub fn enable(&mut self) {
        rcc::enable_and_reset::<T>();
    }

    /// Enable the memory controller on applicable chips.
    pub fn memory_controller_enable(&mut self) {
        // fmc v1 and v2 does not have the fmcen bit
        // fsmc v1, v2 and v3 does not have the fmcen bit
        // This is a "not" because it is expected that all future versions have this bit
        #[cfg(not(any(fmc_v1x3, fmc_v2x1, fsmc_v1x0, fsmc_v1x3, fmc_v4, fmc_n6)))]
        T::regs().bcr1().modify(|r| r.set_fmcen(true));
        #[cfg(any(fmc_v4, fmc_n6))]
        T::regs().nor_psram().bcr1().modify(|r| r.set_fmcen(true));
    }

    /// Disable the memory controller on applicable chips.
    ///
    /// This is typically called when changes need to be made
    /// to the FMC clock registers, which requires the FMC
    /// to be disabled.
    pub fn memory_controller_disable(&mut self) {
        // fmc v1 and v2 does not have the fmcen bit
        // fsmc v1, v2 and v3 does not have the fmcen bit
        // This is a "not" because it is expected that all future versions have this bit
        #[cfg(not(any(fmc_v1x3, fmc_v2x1, fsmc_v1x0, fsmc_v1x3, fmc_v4, fmc_n6)))]
        T::regs().bcr1().modify(|r| r.set_fmcen(false));
        #[cfg(any(fmc_v4, fmc_n6))]
        T::regs().nor_psram().bcr1().modify(|r| r.set_fmcen(false));
    }

    /// Get the kernel clock currently in use for this FMC instance.
    pub fn source_clock_hz(&self) -> u32 {
        <T as crate::rcc::SealedRccPeripheral>::frequency().0
    }
}

pub(crate) trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> crate::pac::fmc::Fmc;
}

/// FMC instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {}

foreach_peripheral!(
    (fmc, $inst:ident) => {
        impl crate::fmc::SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::fmc::Fmc {
                crate::pac::$inst
            }
        }
        impl crate::fmc::Instance for crate::peripherals::$inst {}
    };
);

pin_trait!(SDNWEPin, Instance);
pin_trait!(SDNCASPin, Instance);
pin_trait!(SDNRASPin, Instance);

pin_trait!(SDNE0Pin, Instance);
pin_trait!(SDNE1Pin, Instance);

pin_trait!(SDCKE0Pin, Instance);
pin_trait!(SDCKE1Pin, Instance);

pin_trait!(SDCLKPin, Instance);

pin_trait!(NBL0Pin, Instance);
pin_trait!(NBL1Pin, Instance);
pin_trait!(NBL2Pin, Instance);
pin_trait!(NBL3Pin, Instance);

pin_trait!(INTPin, Instance);
pin_trait!(NLPin, Instance);
pin_trait!(NWaitPin, Instance);

pin_trait!(NE1Pin, Instance);
pin_trait!(NE2Pin, Instance);
pin_trait!(NE3Pin, Instance);
pin_trait!(NE4Pin, Instance);

pin_trait!(NCEPin, Instance);
pin_trait!(NOEPin, Instance);
pin_trait!(NWEPin, Instance);
pin_trait!(ClkPin, Instance);

pin_trait!(BA0Pin, Instance);
pin_trait!(BA1Pin, Instance);

pin_trait!(D0Pin, Instance);
pin_trait!(D1Pin, Instance);
pin_trait!(D2Pin, Instance);
pin_trait!(D3Pin, Instance);
pin_trait!(D4Pin, Instance);
pin_trait!(D5Pin, Instance);
pin_trait!(D6Pin, Instance);
pin_trait!(D7Pin, Instance);
pin_trait!(D8Pin, Instance);
pin_trait!(D9Pin, Instance);
pin_trait!(D10Pin, Instance);
pin_trait!(D11Pin, Instance);
pin_trait!(D12Pin, Instance);
pin_trait!(D13Pin, Instance);
pin_trait!(D14Pin, Instance);
pin_trait!(D15Pin, Instance);
pin_trait!(D16Pin, Instance);
pin_trait!(D17Pin, Instance);
pin_trait!(D18Pin, Instance);
pin_trait!(D19Pin, Instance);
pin_trait!(D20Pin, Instance);
pin_trait!(D21Pin, Instance);
pin_trait!(D22Pin, Instance);
pin_trait!(D23Pin, Instance);
pin_trait!(D24Pin, Instance);
pin_trait!(D25Pin, Instance);
pin_trait!(D26Pin, Instance);
pin_trait!(D27Pin, Instance);
pin_trait!(D28Pin, Instance);
pin_trait!(D29Pin, Instance);
pin_trait!(D30Pin, Instance);
pin_trait!(D31Pin, Instance);

pin_trait!(DA0Pin, Instance);
pin_trait!(DA1Pin, Instance);
pin_trait!(DA2Pin, Instance);
pin_trait!(DA3Pin, Instance);
pin_trait!(DA4Pin, Instance);
pin_trait!(DA5Pin, Instance);
pin_trait!(DA6Pin, Instance);
pin_trait!(DA7Pin, Instance);
pin_trait!(DA8Pin, Instance);
pin_trait!(DA9Pin, Instance);
pin_trait!(DA10Pin, Instance);
pin_trait!(DA11Pin, Instance);
pin_trait!(DA12Pin, Instance);
pin_trait!(DA13Pin, Instance);
pin_trait!(DA14Pin, Instance);
pin_trait!(DA15Pin, Instance);

pin_trait!(A0Pin, Instance);
pin_trait!(A1Pin, Instance);
pin_trait!(A2Pin, Instance);
pin_trait!(A3Pin, Instance);
pin_trait!(A4Pin, Instance);
pin_trait!(A5Pin, Instance);
pin_trait!(A6Pin, Instance);
pin_trait!(A7Pin, Instance);
pin_trait!(A8Pin, Instance);
pin_trait!(A9Pin, Instance);
pin_trait!(A10Pin, Instance);
pin_trait!(A11Pin, Instance);
pin_trait!(A12Pin, Instance);
pin_trait!(A13Pin, Instance);
pin_trait!(A14Pin, Instance);
pin_trait!(A15Pin, Instance);
pin_trait!(A16Pin, Instance);
pin_trait!(A17Pin, Instance);
pin_trait!(A18Pin, Instance);
pin_trait!(A19Pin, Instance);
pin_trait!(A20Pin, Instance);
pin_trait!(A21Pin, Instance);
pin_trait!(A22Pin, Instance);
pin_trait!(A23Pin, Instance);
pin_trait!(A24Pin, Instance);
pin_trait!(A25Pin, Instance);
