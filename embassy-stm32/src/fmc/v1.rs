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
