//! Flexible Memory Controller (FMC) / Flexible Static Memory Controller (FSMC)
use embassy_hal_internal::PeripheralType;

use crate::gpio::{AfType, OutputType, Pull, Speed};
use crate::{Peri, rcc};

// Shadow the metapac values to make them more convenient to access.
pub use crate::pac::fmc::vals;

/// Defines how the FMC banks are mapped using the BMAP register.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
pub enum FmcBankMapping {
    /// Default FMC bank mapping.
    Default = 0b00,
    /// Swaps SDRAM 1 into `0x6000_0000``, SDRAM 2 into `0x7000_0000`,
    /// and NOR/PSRAM into `0xC000_0000`.
    NorSdramSwapped = 0b01,
    /// Swaps SDRAM bank 2 into 0x7000 0000 instead of SDRAM bank 1.
    Sdram2Swapped = 0b10,
}

/// The possible FMC banks for memory mapping with FMC controllers.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
pub enum FmcBank {
    /// Bank1: NOR/PSRAM/SRAM
    Bank1,

    /// Bank2: Remapped SDRAM 1 or 2 depending on BMAP register configuration.
    Bank2,

    /// Bank3: NAND Flash
    ///
    /// Bank3 is always used as NAND flash.
    Bank3,

    // NOTE: Bank 4 is not normally used by the FMC outside
    // of fmc_v1x3 for the PC Card/CompactFlash interface.
    #[cfg(fmc_v1x3)]
    Bank4,

    /// Bank5: SDRAM 1
    #[cfg(not(fmc_v1x3))]
    Bank5,
    /// Bank6: SDRAM 2
    #[cfg(not(fmc_v1x3))]
    Bank6,
}

impl FmcBank {
    /// Return a pointer to the base address of the FMC bank.
    pub fn ptr(self) -> *mut u32 {
        // fmc_v1x3 supports 2 banks of NAND memory and a compact flash card
        #[cfg(fmc_v1x3)]
        return (match self {
            FmcBank::Bank1 => 0x6000_0000u32,
            FmcBank::Bank2 => 0x7000_0000u32,
            FmcBank::Bank3 => 0x8000_0000u32,
            FmcBank::Bank4 => 0x9000_0000u32,
        }) as *mut u32;

        #[cfg(not(fmc_v1x3))]
        return (match self {
            FmcBank::Bank1 => 0x6000_0000u32,
            FmcBank::Bank2 => 0x7000_0000u32,
            FmcBank::Bank3 => 0x7000_0000u32,
            #[cfg(not(fmc_v1x3))]
            FmcBank::Bank3 => 0x8000_0000u32,
            // Bank 4 is not used.
            FmcBank::Bank5 => 0xC000_0000u32,
            FmcBank::Bank6 => 0xD000_0000u32,
        }) as *mut u32;
    }
}

/// Target bank for SDRAM commands.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
pub enum FmcSdramBank {
    /// Targets the 1st SDRAM bank.
    Bank1,
    /// Targets the 2nd SDRAM bank
    Bank2,
}

impl FmcSdramBank {
    /// Return a pointer to the base address of the SDRAM bank.
    ///
    /// This takes into account if the memory banks have been re-mapped.
    pub fn ptr(self, mapping: FmcBankMapping) -> *mut u32 {
        (match mapping {
            FmcBankMapping::Default => match self {
                // Note Bank 1 is mapped to 0x7000_0000 and 0xC000_0000.
                FmcSdramBank::Bank1 => FmcBank::Bank5.ptr(), // 0xC000_0000
                FmcSdramBank::Bank2 => FmcBank::Bank6.ptr(), // 0xD000_0000
            },
            FmcBankMapping::NorSdramSwapped => match self {
                FmcSdramBank::Bank1 => FmcBank::Bank1.ptr(), // 0x6000_0000
                FmcSdramBank::Bank2 => FmcBank::Bank6.ptr(), // 0xD000_0000
            },
            FmcBankMapping::Sdram2Swapped => match self {
                FmcSdramBank::Bank1 => FmcBank::Bank5.ptr(), // 0xC000_0000
                // Note Bank 1 is mapped to 0x7000_0000 and 0xD000_0000.
                FmcSdramBank::Bank2 => FmcBank::Bank6.ptr(), // 0xD000_0000
            },
        } as *mut u32)
    }
}
