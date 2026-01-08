//! LPSPI driver for MCXA276.
//!
//! This module provides SPI master and slave drivers with both blocking and async
//! (interrupt-driven) modes. The async APIs are interrupt-driven: the ISR services
//! the FIFOs and wakes the awaiting task via a `WaitCell`.
//!
//! # DMA Support
//!
//! - [`SpiDma`] provides DMA-based SPI **master** transfers.
//! - [`SpiSlaveDma`] provides DMA-based SPI **slave** transfers.
//!
//! The master DMA implementation uses scatter/gather DMA to handle PCS (chip select)
//! de-assertion automatically at the end of a burst.
//!
//! ## Transfer Modes
//!
//! LPSPI is electrically full-duplex (every transmitted frame clocks in a received
//! frame), but many "half-duplex" *protocols* are implemented by sequencing phases.
//!
//! - **TX-only**: transmit bytes while discarding the concurrently received bytes
//!   (e.g. [`Spi::write`], [`SpiDma::write_dma`]).
//! - **RX-only**: receive bytes by transmitting dummy bytes (0x00) to generate clocks
//!   (e.g. [`Spi::read`], [`SpiDma::read_dma`]).
//! - **Full-duplex**: transmit and receive at the same time
//!   (e.g. [`Spi::transfer`], [`SpiDma::transfer_dma`]).
//!
//! For "write-then-read" protocols that require chip-select held across both phases,
//! prefer a single full-duplex burst (send the command bytes followed by dummy bytes,
//! then ignore the initial received bytes).
//!
//! # Examples
//!
//! See the MCXA examples:
//! - `examples/mcxa/src/bin/spi_master_blocking.rs`
//! - `examples/mcxa/src/bin/spi_interrupt_master.rs`
//! - `examples/mcxa/src/bin/spi_master_dma.rs`
//! - `examples/mcxa/src/bin/spi_slave_blocking.rs`
//! - `examples/mcxa/src/bin/spi_interrupt_slave.rs`
//! - `examples/mcxa/src/bin/spi_slave_dma.rs`
//! - `examples/mcxa/src/bin/spi_b2b_master.rs` / `spi_b2b_slave.rs`

// Sub-modules
mod common;
mod dma_master;
mod dma_slave;
mod master;
mod pins;
mod slave;

// Re-export all public items
pub use common::*;
pub use dma_master::*;
pub use dma_slave::*;
pub use master::*;
pub use pins::*;
pub use slave::*;

// Re-export SPI mode constants from embedded-hal
pub use embedded_hal_02::spi::{MODE_0, MODE_1, MODE_2, MODE_3, Mode as SpiMode, Phase, Polarity};

