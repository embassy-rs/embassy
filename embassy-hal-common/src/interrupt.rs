//! Interrupt handling for cortex-m devices.
use core::mem;
use core::sync::atomic::{compiler_fence, Ordering};

use cortex_m::interrupt::InterruptNumber;
use cortex_m::peripheral::NVIC;

/// Generate a standard `mod interrupt` for a HAL.
#[macro_export]
macro_rules! interrupt_mod {
    ($($irqs:ident),* $(,)?) => {
        #[cfg(feature = "rt")]
        pub use cortex_m_rt::interrupt;

        /// Interrupt definitions.
        pub mod interrupt {
            pub use $crate::interrupt::{InterruptExt, Priority};
            pub use crate::pac::Interrupt::*;
            pub use crate::pac::Interrupt;

            /// Type-level interrupt infrastructure.
            ///
            /// This module contains one *type* per interrupt. This is used for checking at compile time that
            /// the interrupts are correctly bound to HAL drivers.
            ///
            /// As an end user, you shouldn't need to use this module directly. Use the [`crate::bind_interrupts!`] macro
            /// to bind interrupts, and the [`crate::interrupt`] module to manually register interrupt handlers and manipulate
            /// interrupts directly (pending/unpending, enabling/disabling, setting the priority, etc...)
            pub mod typelevel {
                use super::InterruptExt;

                mod sealed {
                    pub trait Interrupt {}
                }

                /// Type-level interrupt.
                ///
                /// This trait is implemented for all typelevel interrupt types in this module.
                pub trait Interrupt: sealed::Interrupt {

                    /// Interrupt enum variant.
                    ///
                    /// This allows going from typelevel interrupts (one type per interrupt) to
                    /// non-typelevel interrupts (a single `Interrupt` enum type, with one variant per interrupt).
                    const IRQ: super::Interrupt;

                    /// Enable the interrupt.
                    #[inline]
                    unsafe fn enable() {
                        Self::IRQ.enable()
                    }

                    /// Disable the interrupt.
                    #[inline]
                    fn disable() {
                        Self::IRQ.disable()
                    }

                    /// Check if interrupt is enabled.
                    #[inline]
                    fn is_enabled() -> bool {
                        Self::IRQ.is_enabled()
                    }

                    /// Check if interrupt is pending.
                    #[inline]
                    fn is_pending() -> bool {
                        Self::IRQ.is_pending()
                    }

                    /// Set interrupt pending.
                    #[inline]
                    fn pend() {
                        Self::IRQ.pend()
                    }

                    /// Unset interrupt pending.
                    #[inline]
                    fn unpend() {
                        Self::IRQ.unpend()
                    }

                    /// Get the priority of the interrupt.
                    #[inline]
                    fn get_priority() -> crate::interrupt::Priority {
                        Self::IRQ.get_priority()
                    }

                    /// Set the interrupt priority.
                    #[inline]
                    fn set_priority(prio: crate::interrupt::Priority) {
                        Self::IRQ.set_priority(prio)
                    }
                }

                $(
                    #[allow(non_camel_case_types)]
                    #[doc=stringify!($irqs)]
                    #[doc=" typelevel interrupt."]
                    pub enum $irqs {}
                    impl sealed::Interrupt for $irqs{}
                    impl Interrupt for $irqs {
                        const IRQ: super::Interrupt = super::Interrupt::$irqs;
                    }
                )*

                /// Interrupt handler trait.
                ///
                /// Drivers that need to handle interrupts implement this trait.
                /// The user must ensure `on_interrupt()` is called every time the interrupt fires.
                /// Drivers must use use [`Binding`] to assert at compile time that the user has done so.
                pub trait Handler<I: Interrupt> {
                    /// Interrupt handler function.
                    ///
                    /// Must be called every time the `I` interrupt fires, synchronously from
                    /// the interrupt handler context.
                    ///
                    /// # Safety
                    ///
                    /// This function must ONLY be called from the interrupt handler for `I`.
                    unsafe fn on_interrupt();
                }

                /// Compile-time assertion that an interrupt has been bound to a handler.
                ///
                /// For the vast majority of cases, you should use the `bind_interrupts!`
                /// macro instead of writing `unsafe impl`s of this trait.
                ///
                /// # Safety
                ///
                /// By implementing this trait, you are asserting that you have arranged for `H::on_interrupt()`
                /// to be called every time the `I` interrupt fires.
                ///
                /// This allows drivers to check bindings at compile-time.
                pub unsafe trait Binding<I: Interrupt, H: Handler<I>> {}
            }
        }
    };
}

/// Represents an interrupt type that can be configured by embassy to handle
/// interrupts.
pub unsafe trait InterruptExt: InterruptNumber + Copy {
    /// Enable the interrupt.
    #[inline]
    unsafe fn enable(self) {
        compiler_fence(Ordering::SeqCst);
        NVIC::unmask(self)
    }

    /// Disable the interrupt.
    #[inline]
    fn disable(self) {
        NVIC::mask(self);
        compiler_fence(Ordering::SeqCst);
    }

    /// Check if interrupt is being handled.
    #[inline]
    #[cfg(not(armv6m))]
    fn is_active(self) -> bool {
        NVIC::is_active(self)
    }

    /// Check if interrupt is enabled.
    #[inline]
    fn is_enabled(self) -> bool {
        NVIC::is_enabled(self)
    }

    /// Check if interrupt is pending.
    #[inline]
    fn is_pending(self) -> bool {
        NVIC::is_pending(self)
    }

    /// Set interrupt pending.
    #[inline]
    fn pend(self) {
        NVIC::pend(self)
    }

    /// Unset interrupt pending.
    #[inline]
    fn unpend(self) {
        NVIC::unpend(self)
    }

    /// Get the priority of the interrupt.
    #[inline]
    fn get_priority(self) -> Priority {
        Priority::from(NVIC::get_priority(self))
    }

    /// Set the interrupt priority.
    #[inline]
    fn set_priority(self, prio: Priority) {
        critical_section::with(|_| unsafe {
            let mut nvic: cortex_m::peripheral::NVIC = mem::transmute(());
            nvic.set_priority(self, prio.into())
        })
    }
}

unsafe impl<T: InterruptNumber + Copy> InterruptExt for T {}

impl From<u8> for Priority {
    fn from(priority: u8) -> Self {
        unsafe { mem::transmute(priority & PRIO_MASK) }
    }
}

impl From<Priority> for u8 {
    fn from(p: Priority) -> Self {
        p as u8
    }
}

#[cfg(feature = "prio-bits-0")]
const PRIO_MASK: u8 = 0x00;
#[cfg(feature = "prio-bits-1")]
const PRIO_MASK: u8 = 0x80;
#[cfg(feature = "prio-bits-2")]
const PRIO_MASK: u8 = 0xc0;
#[cfg(feature = "prio-bits-3")]
const PRIO_MASK: u8 = 0xe0;
#[cfg(feature = "prio-bits-4")]
const PRIO_MASK: u8 = 0xf0;
#[cfg(feature = "prio-bits-5")]
const PRIO_MASK: u8 = 0xf8;
#[cfg(feature = "prio-bits-6")]
const PRIO_MASK: u8 = 0xfc;
#[cfg(feature = "prio-bits-7")]
const PRIO_MASK: u8 = 0xfe;
#[cfg(feature = "prio-bits-8")]
const PRIO_MASK: u8 = 0xff;

/// The interrupt priority level.
///
/// NOTE: The contents of this enum differ according to the set `prio-bits-*` Cargo feature.
#[cfg(feature = "prio-bits-0")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Priority {
    P0 = 0x0,
}

/// The interrupt priority level.
///
/// NOTE: The contents of this enum differ according to the set `prio-bits-*` Cargo feature.
#[cfg(feature = "prio-bits-1")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Priority {
    P0 = 0x0,
    P1 = 0x80,
}

/// The interrupt priority level.
///
/// NOTE: The contents of this enum differ according to the set `prio-bits-*` Cargo feature.
#[cfg(feature = "prio-bits-2")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Priority {
    P0 = 0x0,
    P1 = 0x40,
    P2 = 0x80,
    P3 = 0xc0,
}

/// The interrupt priority level.
///
/// NOTE: The contents of this enum differ according to the set `prio-bits-*` Cargo feature.
#[cfg(feature = "prio-bits-3")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Priority {
    P0 = 0x0,
    P1 = 0x20,
    P2 = 0x40,
    P3 = 0x60,
    P4 = 0x80,
    P5 = 0xa0,
    P6 = 0xc0,
    P7 = 0xe0,
}

/// The interrupt priority level.
///
/// NOTE: The contents of this enum differ according to the set `prio-bits-*` Cargo feature.
#[cfg(feature = "prio-bits-4")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Priority {
    P0 = 0x0,
    P1 = 0x10,
    P2 = 0x20,
    P3 = 0x30,
    P4 = 0x40,
    P5 = 0x50,
    P6 = 0x60,
    P7 = 0x70,
    P8 = 0x80,
    P9 = 0x90,
    P10 = 0xa0,
    P11 = 0xb0,
    P12 = 0xc0,
    P13 = 0xd0,
    P14 = 0xe0,
    P15 = 0xf0,
}

/// The interrupt priority level.
///
/// NOTE: The contents of this enum differ according to the set `prio-bits-*` Cargo feature.
#[cfg(feature = "prio-bits-5")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Priority {
    P0 = 0x0,
    P1 = 0x8,
    P2 = 0x10,
    P3 = 0x18,
    P4 = 0x20,
    P5 = 0x28,
    P6 = 0x30,
    P7 = 0x38,
    P8 = 0x40,
    P9 = 0x48,
    P10 = 0x50,
    P11 = 0x58,
    P12 = 0x60,
    P13 = 0x68,
    P14 = 0x70,
    P15 = 0x78,
    P16 = 0x80,
    P17 = 0x88,
    P18 = 0x90,
    P19 = 0x98,
    P20 = 0xa0,
    P21 = 0xa8,
    P22 = 0xb0,
    P23 = 0xb8,
    P24 = 0xc0,
    P25 = 0xc8,
    P26 = 0xd0,
    P27 = 0xd8,
    P28 = 0xe0,
    P29 = 0xe8,
    P30 = 0xf0,
    P31 = 0xf8,
}

/// The interrupt priority level.
///
/// NOTE: The contents of this enum differ according to the set `prio-bits-*` Cargo feature.
#[cfg(feature = "prio-bits-6")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Priority {
    P0 = 0x0,
    P1 = 0x4,
    P2 = 0x8,
    P3 = 0xc,
    P4 = 0x10,
    P5 = 0x14,
    P6 = 0x18,
    P7 = 0x1c,
    P8 = 0x20,
    P9 = 0x24,
    P10 = 0x28,
    P11 = 0x2c,
    P12 = 0x30,
    P13 = 0x34,
    P14 = 0x38,
    P15 = 0x3c,
    P16 = 0x40,
    P17 = 0x44,
    P18 = 0x48,
    P19 = 0x4c,
    P20 = 0x50,
    P21 = 0x54,
    P22 = 0x58,
    P23 = 0x5c,
    P24 = 0x60,
    P25 = 0x64,
    P26 = 0x68,
    P27 = 0x6c,
    P28 = 0x70,
    P29 = 0x74,
    P30 = 0x78,
    P31 = 0x7c,
    P32 = 0x80,
    P33 = 0x84,
    P34 = 0x88,
    P35 = 0x8c,
    P36 = 0x90,
    P37 = 0x94,
    P38 = 0x98,
    P39 = 0x9c,
    P40 = 0xa0,
    P41 = 0xa4,
    P42 = 0xa8,
    P43 = 0xac,
    P44 = 0xb0,
    P45 = 0xb4,
    P46 = 0xb8,
    P47 = 0xbc,
    P48 = 0xc0,
    P49 = 0xc4,
    P50 = 0xc8,
    P51 = 0xcc,
    P52 = 0xd0,
    P53 = 0xd4,
    P54 = 0xd8,
    P55 = 0xdc,
    P56 = 0xe0,
    P57 = 0xe4,
    P58 = 0xe8,
    P59 = 0xec,
    P60 = 0xf0,
    P61 = 0xf4,
    P62 = 0xf8,
    P63 = 0xfc,
}

/// The interrupt priority level.
///
/// NOTE: The contents of this enum differ according to the set `prio-bits-*` Cargo feature.
#[cfg(feature = "prio-bits-7")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Priority {
    P0 = 0x0,
    P1 = 0x2,
    P2 = 0x4,
    P3 = 0x6,
    P4 = 0x8,
    P5 = 0xa,
    P6 = 0xc,
    P7 = 0xe,
    P8 = 0x10,
    P9 = 0x12,
    P10 = 0x14,
    P11 = 0x16,
    P12 = 0x18,
    P13 = 0x1a,
    P14 = 0x1c,
    P15 = 0x1e,
    P16 = 0x20,
    P17 = 0x22,
    P18 = 0x24,
    P19 = 0x26,
    P20 = 0x28,
    P21 = 0x2a,
    P22 = 0x2c,
    P23 = 0x2e,
    P24 = 0x30,
    P25 = 0x32,
    P26 = 0x34,
    P27 = 0x36,
    P28 = 0x38,
    P29 = 0x3a,
    P30 = 0x3c,
    P31 = 0x3e,
    P32 = 0x40,
    P33 = 0x42,
    P34 = 0x44,
    P35 = 0x46,
    P36 = 0x48,
    P37 = 0x4a,
    P38 = 0x4c,
    P39 = 0x4e,
    P40 = 0x50,
    P41 = 0x52,
    P42 = 0x54,
    P43 = 0x56,
    P44 = 0x58,
    P45 = 0x5a,
    P46 = 0x5c,
    P47 = 0x5e,
    P48 = 0x60,
    P49 = 0x62,
    P50 = 0x64,
    P51 = 0x66,
    P52 = 0x68,
    P53 = 0x6a,
    P54 = 0x6c,
    P55 = 0x6e,
    P56 = 0x70,
    P57 = 0x72,
    P58 = 0x74,
    P59 = 0x76,
    P60 = 0x78,
    P61 = 0x7a,
    P62 = 0x7c,
    P63 = 0x7e,
    P64 = 0x80,
    P65 = 0x82,
    P66 = 0x84,
    P67 = 0x86,
    P68 = 0x88,
    P69 = 0x8a,
    P70 = 0x8c,
    P71 = 0x8e,
    P72 = 0x90,
    P73 = 0x92,
    P74 = 0x94,
    P75 = 0x96,
    P76 = 0x98,
    P77 = 0x9a,
    P78 = 0x9c,
    P79 = 0x9e,
    P80 = 0xa0,
    P81 = 0xa2,
    P82 = 0xa4,
    P83 = 0xa6,
    P84 = 0xa8,
    P85 = 0xaa,
    P86 = 0xac,
    P87 = 0xae,
    P88 = 0xb0,
    P89 = 0xb2,
    P90 = 0xb4,
    P91 = 0xb6,
    P92 = 0xb8,
    P93 = 0xba,
    P94 = 0xbc,
    P95 = 0xbe,
    P96 = 0xc0,
    P97 = 0xc2,
    P98 = 0xc4,
    P99 = 0xc6,
    P100 = 0xc8,
    P101 = 0xca,
    P102 = 0xcc,
    P103 = 0xce,
    P104 = 0xd0,
    P105 = 0xd2,
    P106 = 0xd4,
    P107 = 0xd6,
    P108 = 0xd8,
    P109 = 0xda,
    P110 = 0xdc,
    P111 = 0xde,
    P112 = 0xe0,
    P113 = 0xe2,
    P114 = 0xe4,
    P115 = 0xe6,
    P116 = 0xe8,
    P117 = 0xea,
    P118 = 0xec,
    P119 = 0xee,
    P120 = 0xf0,
    P121 = 0xf2,
    P122 = 0xf4,
    P123 = 0xf6,
    P124 = 0xf8,
    P125 = 0xfa,
    P126 = 0xfc,
    P127 = 0xfe,
}

/// The interrupt priority level.
///
/// NOTE: The contents of this enum differ according to the set `prio-bits-*` Cargo feature.
#[cfg(feature = "prio-bits-8")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Priority {
    P0 = 0x0,
    P1 = 0x1,
    P2 = 0x2,
    P3 = 0x3,
    P4 = 0x4,
    P5 = 0x5,
    P6 = 0x6,
    P7 = 0x7,
    P8 = 0x8,
    P9 = 0x9,
    P10 = 0xa,
    P11 = 0xb,
    P12 = 0xc,
    P13 = 0xd,
    P14 = 0xe,
    P15 = 0xf,
    P16 = 0x10,
    P17 = 0x11,
    P18 = 0x12,
    P19 = 0x13,
    P20 = 0x14,
    P21 = 0x15,
    P22 = 0x16,
    P23 = 0x17,
    P24 = 0x18,
    P25 = 0x19,
    P26 = 0x1a,
    P27 = 0x1b,
    P28 = 0x1c,
    P29 = 0x1d,
    P30 = 0x1e,
    P31 = 0x1f,
    P32 = 0x20,
    P33 = 0x21,
    P34 = 0x22,
    P35 = 0x23,
    P36 = 0x24,
    P37 = 0x25,
    P38 = 0x26,
    P39 = 0x27,
    P40 = 0x28,
    P41 = 0x29,
    P42 = 0x2a,
    P43 = 0x2b,
    P44 = 0x2c,
    P45 = 0x2d,
    P46 = 0x2e,
    P47 = 0x2f,
    P48 = 0x30,
    P49 = 0x31,
    P50 = 0x32,
    P51 = 0x33,
    P52 = 0x34,
    P53 = 0x35,
    P54 = 0x36,
    P55 = 0x37,
    P56 = 0x38,
    P57 = 0x39,
    P58 = 0x3a,
    P59 = 0x3b,
    P60 = 0x3c,
    P61 = 0x3d,
    P62 = 0x3e,
    P63 = 0x3f,
    P64 = 0x40,
    P65 = 0x41,
    P66 = 0x42,
    P67 = 0x43,
    P68 = 0x44,
    P69 = 0x45,
    P70 = 0x46,
    P71 = 0x47,
    P72 = 0x48,
    P73 = 0x49,
    P74 = 0x4a,
    P75 = 0x4b,
    P76 = 0x4c,
    P77 = 0x4d,
    P78 = 0x4e,
    P79 = 0x4f,
    P80 = 0x50,
    P81 = 0x51,
    P82 = 0x52,
    P83 = 0x53,
    P84 = 0x54,
    P85 = 0x55,
    P86 = 0x56,
    P87 = 0x57,
    P88 = 0x58,
    P89 = 0x59,
    P90 = 0x5a,
    P91 = 0x5b,
    P92 = 0x5c,
    P93 = 0x5d,
    P94 = 0x5e,
    P95 = 0x5f,
    P96 = 0x60,
    P97 = 0x61,
    P98 = 0x62,
    P99 = 0x63,
    P100 = 0x64,
    P101 = 0x65,
    P102 = 0x66,
    P103 = 0x67,
    P104 = 0x68,
    P105 = 0x69,
    P106 = 0x6a,
    P107 = 0x6b,
    P108 = 0x6c,
    P109 = 0x6d,
    P110 = 0x6e,
    P111 = 0x6f,
    P112 = 0x70,
    P113 = 0x71,
    P114 = 0x72,
    P115 = 0x73,
    P116 = 0x74,
    P117 = 0x75,
    P118 = 0x76,
    P119 = 0x77,
    P120 = 0x78,
    P121 = 0x79,
    P122 = 0x7a,
    P123 = 0x7b,
    P124 = 0x7c,
    P125 = 0x7d,
    P126 = 0x7e,
    P127 = 0x7f,
    P128 = 0x80,
    P129 = 0x81,
    P130 = 0x82,
    P131 = 0x83,
    P132 = 0x84,
    P133 = 0x85,
    P134 = 0x86,
    P135 = 0x87,
    P136 = 0x88,
    P137 = 0x89,
    P138 = 0x8a,
    P139 = 0x8b,
    P140 = 0x8c,
    P141 = 0x8d,
    P142 = 0x8e,
    P143 = 0x8f,
    P144 = 0x90,
    P145 = 0x91,
    P146 = 0x92,
    P147 = 0x93,
    P148 = 0x94,
    P149 = 0x95,
    P150 = 0x96,
    P151 = 0x97,
    P152 = 0x98,
    P153 = 0x99,
    P154 = 0x9a,
    P155 = 0x9b,
    P156 = 0x9c,
    P157 = 0x9d,
    P158 = 0x9e,
    P159 = 0x9f,
    P160 = 0xa0,
    P161 = 0xa1,
    P162 = 0xa2,
    P163 = 0xa3,
    P164 = 0xa4,
    P165 = 0xa5,
    P166 = 0xa6,
    P167 = 0xa7,
    P168 = 0xa8,
    P169 = 0xa9,
    P170 = 0xaa,
    P171 = 0xab,
    P172 = 0xac,
    P173 = 0xad,
    P174 = 0xae,
    P175 = 0xaf,
    P176 = 0xb0,
    P177 = 0xb1,
    P178 = 0xb2,
    P179 = 0xb3,
    P180 = 0xb4,
    P181 = 0xb5,
    P182 = 0xb6,
    P183 = 0xb7,
    P184 = 0xb8,
    P185 = 0xb9,
    P186 = 0xba,
    P187 = 0xbb,
    P188 = 0xbc,
    P189 = 0xbd,
    P190 = 0xbe,
    P191 = 0xbf,
    P192 = 0xc0,
    P193 = 0xc1,
    P194 = 0xc2,
    P195 = 0xc3,
    P196 = 0xc4,
    P197 = 0xc5,
    P198 = 0xc6,
    P199 = 0xc7,
    P200 = 0xc8,
    P201 = 0xc9,
    P202 = 0xca,
    P203 = 0xcb,
    P204 = 0xcc,
    P205 = 0xcd,
    P206 = 0xce,
    P207 = 0xcf,
    P208 = 0xd0,
    P209 = 0xd1,
    P210 = 0xd2,
    P211 = 0xd3,
    P212 = 0xd4,
    P213 = 0xd5,
    P214 = 0xd6,
    P215 = 0xd7,
    P216 = 0xd8,
    P217 = 0xd9,
    P218 = 0xda,
    P219 = 0xdb,
    P220 = 0xdc,
    P221 = 0xdd,
    P222 = 0xde,
    P223 = 0xdf,
    P224 = 0xe0,
    P225 = 0xe1,
    P226 = 0xe2,
    P227 = 0xe3,
    P228 = 0xe4,
    P229 = 0xe5,
    P230 = 0xe6,
    P231 = 0xe7,
    P232 = 0xe8,
    P233 = 0xe9,
    P234 = 0xea,
    P235 = 0xeb,
    P236 = 0xec,
    P237 = 0xed,
    P238 = 0xee,
    P239 = 0xef,
    P240 = 0xf0,
    P241 = 0xf1,
    P242 = 0xf2,
    P243 = 0xf3,
    P244 = 0xf4,
    P245 = 0xf5,
    P246 = 0xf6,
    P247 = 0xf7,
    P248 = 0xf8,
    P249 = 0xf9,
    P250 = 0xfa,
    P251 = 0xfb,
    P252 = 0xfc,
    P253 = 0xfd,
    P254 = 0xfe,
    P255 = 0xff,
}
