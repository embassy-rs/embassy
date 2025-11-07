//! Protocol abstraction for ADIN1110 SPI communication
//!
//! This module defines a trait-based abstraction for different SPI protocols
//! supported by the ADIN1110 chip:
//! - Generic SPI
//! - OPEN Alliance TC6

#[cfg(feature = "generic-spi")]
mod generic_spi;
#[cfg(feature = "generic-spi")]
pub use generic_spi::GenericSpi;

#[cfg(feature = "tc6")]
mod tc6;
#[cfg(feature = "tc6")]
pub use tc6::Tc6;

use crate::AdinError;

/// Protocol abstraction trait for ADIN1110 SPI communication
pub trait Adin1110Protocol {
    /// SPI error type
    type SpiError: core::fmt::Debug + embedded_hal_async::spi::Error;

    /// Read a register
    async fn read_reg(&mut self, addr: u16) -> Result<u32, AdinError<Self::SpiError>>;

    /// Write a register
    async fn write_reg(&mut self, addr: u16, val: u32) -> Result<(), AdinError<Self::SpiError>>;

    /// Read FIFO data into buffer, returns number of bytes read
    async fn read_fifo(&mut self, frame: &mut [u8]) -> Result<usize, AdinError<Self::SpiError>>;

    /// Write frame data to FIFO
    async fn write_fifo(&mut self, frame: &[u8]) -> Result<(), AdinError<Self::SpiError>>;
}
