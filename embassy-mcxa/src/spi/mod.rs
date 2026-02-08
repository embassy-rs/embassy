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
//! # Chip Select Modes
//!
//! The SPI master driver uses type-level distinction to track chip select management:
//!
//! - [`HardwareCs`]: LPSPI hardware controls the PCS signal automatically
//! - [`NoCs`]: User manages chip select externally via GPIO
//!
//! Only `Spi<..., NoCs>` implements [`embedded_hal_1::spi::SpiBus`], following the
//! embedded-hal semantics where `SpiBus` represents exclusive bus access without CS management.
//!
//! ## Hardware CS (`new_blocking` / `new_async`)
//!
//! Use the standard constructors when the LPSPI hardware should control the PCS signal.
//! The [`ChipSelect`] enum in the configuration selects which hardware PCS line (PCS0-PCS3) to use.
//!
//! **Use hardware CS when:**
//! - You have a single SPI device on the bus
//! - You want simpler code with automatic CS timing
//! - You don't need to share the SPI bus between multiple devices
//!
//! ```ignore
//! // Hardware CS - peripheral controls chip select automatically
//! let spi = Spi::new_blocking(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, p.P3_11, config)?;
//! ```
//!
//! ## External CS (`new_blocking_no_cs` / `new_async_no_cs`)
//!
//! Use the `_no_cs` constructors when you manage chip select externally via GPIO.
//! This is required for sharing the SPI bus between multiple devices using
//! `embassy-embedded-hal::shared_bus::SpiDevice`.
//!
//! **Use external CS when:**
//! - You have multiple devices on the same SPI bus
//! - You need to use [`embassy_embedded_hal::shared_bus::SpiDevice`]
//! - You need custom CS timing or behavior
//!
//! ```ignore
//! use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
//! use embassy_sync::blocking_mutex::{NoopMutex, raw::NoopRawMutex};
//!
//! // Create SPI without hardware CS (implements SpiBus)
//! let spi = Spi::new_blocking_no_cs(p.LPSPI1, p.P3_10, p.P3_8, p.P3_9, config)?;
//! let spi_bus = NoopMutex::new(RefCell::new(spi));
//!
//! // Create GPIO CS pins for each device
//! let cs_a = Output::new(p.P1_4, Level::High);
//! let cs_b = Output::new(p.P1_5, Level::High);
//!
//! // Wrap with SpiDevice for each peripheral
//! let spi_dev_a = SpiDevice::new(&spi_bus, cs_a);
//! let spi_dev_b = SpiDevice::new(&spi_bus, cs_b);
//! ```
//!
//! **Note:** For SPI slave devices, a CS signal is typically required for the slave to
//! know when it is being addressed.
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
// Re-export SPI mode constants from embedded-hal
pub use embedded_hal_02::spi::{MODE_0, MODE_1, MODE_2, MODE_3, Mode as SpiMode, Phase, Polarity};
pub use master::*;
pub use pins::*;
pub use slave::*;
