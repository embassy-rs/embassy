//! Multicore utilities.
/// The enum values are identical to the bus master IDs / core Ids defined for each
/// chip family (i.e. stm32h747 see rm0399 table 95)
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CoreId {
    #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757))]
    /// Cortex-M7, core 1.
    Core0 = 0x3,

    #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757))]
    /// Cortex-M4, core 2.
    Core1 = 0x1,

    #[cfg(not(any(stm32h745, stm32h747, stm32h755, stm32h757)))]
    /// Cortex-M4, core 1
    Core0 = 0x4,

    #[cfg(any(stm32wb, stm32wl))]
    /// Cortex-M0+, core 2.
    Core1 = 0x8,
}

impl CoreId {
    /// Get the current core id
    /// This code assume that it is only executed on a Cortex-M M0+, M4 or M7 core.
    pub fn current() -> Self {
        let cpuid = unsafe { cortex_m::peripheral::CPUID::PTR.read_volatile().base.read() };
        match (cpuid & 0x000000F0) >> 4 {
            #[cfg(any(stm32wb, stm32wl))]
            0x0 => CoreId::Core1,

            #[cfg(not(any(stm32h745, stm32h747, stm32h755, stm32h757)))]
            0x4 => CoreId::Core0,

            #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757))]
            0x4 => CoreId::Core1,

            #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757))]
            0x7 => CoreId::Core0,
            _ => panic!("Unknown Cortex-M core"),
        }
    }

    #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757, stm32wb, stm32wl))]
    /// Get the other core id
    pub const fn other(&self) -> Self {
        match &self {
            Self::Core0 => Self::Core1,
            Self::Core1 => Self::Core0,
        }
    }

    /// Translates the core ID to an index into the interrupt registers.
    pub const fn to_index(&self) -> usize {
        match &self {
            CoreId::Core0 => 0,
            #[cfg(any(stm32h745, stm32h747, stm32h755, stm32h757, stm32wb, stm32wl))]
            CoreId::Core1 => 1,
        }
    }
}
