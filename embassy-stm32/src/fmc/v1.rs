//! Flexible Memory Controller (FMC) types for FMC v1.3
//!
//! fmc_v1x3 has some very different structures to the later FMCs version,
//! notably how mapping is handled, and the presence of a PC/CompactFlash
//! card controller in the NAND configuration registers. Yhe types for
//! fmc_v1x3 have been seperated out into their own module here to reduce
//! the complexity of the root FMC module.

use embassy_hal_internal::PeripheralType;

use crate::gpio::{AfType, OutputType, Pull, Speed};
use crate::{Peri, rcc};

// Shadow the metapac values to make them more convenient to access.
pub use crate::pac::fmc::vals;

/// The possible FMC banks for memory mapping with FMC controllers.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
pub enum FmcBank {
    /// Bank1: NOR/PSRAM/SRAM
    Bank1,

    /// Bank2: NAND Flash.
    Bank2,

    /// Bank3: NAND Flash
    Bank3,

    /// Bank4: PC Card/CompactFlash interface.
    Bank4,

    /// Bank5: SDRAM 1
    Bank5,

    /// Bank6: SDRAM 2
    Bank6,
}

impl FmcBank {
    /// Return a pointer to the base address of the FMC bank.
    pub fn ptr(self) -> *mut u32 {
        return (match self {
            FmcBank::Bank1 => 0x6000_0000u32,
            FmcBank::Bank2 => 0x7000_0000u32,
            FmcBank::Bank3 => 0x8000_0000u32,
            FmcBank::Bank4 => 0x9000_0000u32,
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

/// Target bank for NAND commands.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
#[repr(u8)]
pub enum FmcNandBank {
    /// Targets the 1st NAND bank.
    Bank1 = 0,
    /// Targets the 2nd NAND bank
    Bank2 = 1,
}

impl Into<usize> for FmcNandBank {
    fn into(self) -> usize {
        match self {
            FmcNandBank::Bank1 => 0usize,
            FmcNandBank::Bank2 => 1usize,
        }
    }
}

impl FmcSdramBank {
    /// Return a pointer to the base address of the SDRAM bank.
    ///
    /// This takes into account if the memory banks have been re-mapped.
    pub fn ptr(self) -> *mut u32 {
        (match self {
            FmcSdramBank::Bank1 => FmcBank::Bank5.ptr(), // 0xC000_0000
            FmcSdramBank::Bank2 => FmcBank::Bank6.ptr(), // 0xD000_0000
        } as *mut u32)
    }
}

/// Target bank for NOR/PSRAM/SRAM commands.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(unused)]
pub enum FmcSramBank {
    /// Targets the 1st NOR/PSRAM/SRAM bank.
    Bank1,
    /// Targets the 2nd NOR/PSRAM/SRAM bank
    Bank2,
    /// Targets the 3nd NOR/PSRAM/SRAM bank
    Bank3,
    /// Targets the 4nd NOR/PSRAM/SRAM bank
    Bank4,
}

impl FmcSramBank {
    /// Returns the base address of the bank.
    pub const fn base_address(&self) -> u32 {
        // Banks start at 0x6000_0000 -> 01100000000000000000000000000000
        // Banks end at   0x6FFF_FFFF -> 01101111111111111111111111111111
        //
        // ADDR[27:26] Select bank:
        // 00 Bank 1 - NOR/PSRAM 1 0x60000000 -> 01100000000000000000000000000000
        // 01 Bank 1 - NOR/PSRAM 2 0x64000000 -> 01100100000000000000000000000000
        // 10 Bank 1 - NOR/PSRAM 3 0x68000000 -> 01101000000000000000000000000000
        // 11 Bank 1 - NOR/PSRAM 4 0x6C000000 -> 01101100000000000000000000000000
        match self {
            FmcSramBank::Bank1 => 0x60000000,
            FmcSramBank::Bank2 => 0x64000000,
            FmcSramBank::Bank3 => 0x68000000,
            FmcSramBank::Bank4 => 0x6C000000,
        }
    }

    /// Returns the lower address bound of the NOR/PSRAM/SRAM bank.
    pub const fn min_address(&self) -> u32 {
        self.base_address()
    }

    /// Returns the size of the bank.
    pub const fn size(&self) -> u32 {
        0x3FFFFFF
    }

    /// Returns the upper address bound of the NOR/PSRAM/SRAM bank.
    pub const fn max_address(&self) -> u32 {
        self.min_address() + self.size()
    }

    /// Returns the address of the bank.
    pub const fn addr(&self) -> u32 {
        self.min_address()
    }
}
