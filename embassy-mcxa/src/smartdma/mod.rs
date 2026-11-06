//! Compile-time SmartDMA (EZH Architecture B) programs.
//!
//! Use [`crate::smartdma_asm!`] or [`crate::smartdma_asm_file!`] to assemble
//! application-authored programs. The result is an immutable [`program::Program`]
//! description with exported entries and explicit relocation records.
//!
//! [`LoadedProgram`] installs an image into caller-provided, permanently borrowed
//! ordinary SRAM. [`SmartDma`] reserves the single engine and compatible pins;
//! its unsafe one-shot launch retains all resources until a full device reset or
//! power cycle. No stop, restart, unload, completion/drain or async cancellation
//! guarantee is provided yet. Dropping a running handle does not stop execution.
//!
//! Valid instruction encodings do not sandbox SmartDMA: it is a separate bus
//! master capable of accessing memory and peripherals. Generic execution remains
//! unsafe. Per-chip instruction-fetch accessibility and external waveform timing
//! require validation under the application's existing memory configuration.
//!
//! # Current restrictions
//!
//! Only the ordinary nonsecure SRAM aperture is accepted; no SRAMX backdoor,
//! cache/execute-permission configuration, DMA scheduling or interrupt handler
//! is supplied. The entry must begin with an explicit NOP and initialize SP
//! through the documented parameter ABI. Ambiguous PAC pin/mux mappings are
//! omitted rather than guessed. The waveform examples use P0_4/PIO0 and retain
//! resources permanently; this is not yet a safe protocol wrapper or WS2812
//! timing-conformance implementation.

mod loader;
pub mod program;
pub use loader::{LoadError, LoadedProgram};

#[cfg(any(mcxa_smartdma, mcxa_smartdma0))]
pub(crate) mod engine;
#[cfg(any(mcxa_smartdma, mcxa_smartdma0))]
pub use engine::{Instance, Pin, Running, SetupError, SmartDma, SmartDmaPin, StartError, Status};
