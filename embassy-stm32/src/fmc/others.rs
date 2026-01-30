//! Flexible Memory Controller (FMC) types for FMC v2.1, v3.1, and v4
//!
//! These strongly differ from the types needed for v1.3, so they've
//! been seperated out here into their own module to reduce complexity.

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
    /// Bank5: SDRAM 1
    Bank5,

    /// Bank6: SDRAM 2
    Bank6,
}

impl FmcBank {
    /// Return a pointer to the base address of the FMC bank.
    pub fn ptr(self) -> *mut u32 {
        (match self {
            FmcBank::Bank1 => 0x6000_0000u32,
            FmcBank::Bank2 => 0x7000_0000u32,
            FmcBank::Bank3 => 0x8000_0000u32,
            // Bank 4 is not used.
            FmcBank::Bank5 => 0xC000_0000u32,
            FmcBank::Bank6 => 0xD000_0000u32,
        }) as *mut u32
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
    pub fn base_address(&self, mapping: FmcBankMapping) -> u32 {
        if mapping == FmcBankMapping::NorSdramSwapped {
            match self {
                FmcSramBank::Bank1 => 0xC0000000,
                FmcSramBank::Bank2 => 0xC4000000,
                FmcSramBank::Bank3 => 0xC8000000,
                FmcSramBank::Bank4 => 0xCC000000,
            }
        } else {
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
    }

    /// Returns the lower address bound of the NOR/PSRAM/SRAM bank.
    pub fn min_address(&self, mapping: FmcBankMapping) -> u32 {
        self.base_address(mapping)
    }

    /// Returns the size of the bank.
    pub const fn size(&self) -> u32 {
        0x3FFFFFF
    }

    /// Returns the upper address bound of the NOR/PSRAM/SRAM bank.
    pub fn max_address(&self, mapping: FmcBankMapping) -> u32 {
        self.min_address(mapping) + self.size()
    }

    /// Returns the address of the bank.
    pub fn addr(&self, mapping: FmcBankMapping) -> u32 {
        self.min_address(mapping)
    }
}
