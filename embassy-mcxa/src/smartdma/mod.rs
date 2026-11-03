//! Compile-time SmartDMA (EZH Architecture B) programs.
//!
//! Use [`crate::smartdma_asm!`] or [`crate::smartdma_asm_file!`] to assemble
//! application-authored programs. The result is an immutable [`program::Program`]
//! description with exported entries and explicit relocation records.
//!
//! This module currently provides assembly and program metadata only. It does
//! **not** load programs, start the engine, configure pins, or establish safe
//! completion/cancellation. In particular, valid instruction encodings do not
//! sandbox SmartDMA: it is a separate bus master capable of accessing memory and
//! peripherals. No program in these examples has been executed on hardware.

pub mod program;
