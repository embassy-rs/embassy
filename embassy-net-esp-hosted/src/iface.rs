//! Transport interface.
//!
//! The esp-hosted firmware supports multiple interfaces, selected at compile time.
//! This module provides the [`Interface`] trait and implementations for SPI and HD-SPI.

pub mod hd_spi;
pub mod spi;

/// Physical interface trait for communicating with the ESP chip.
pub trait Interface {
    /// Initialize or re-initialize the transport.
    async fn init(&mut self, cold_boot: bool);

    /// Wait for the ESP to indicate readiness for a new transaction.
    async fn wait_for_handshake(&mut self);

    /// Wait for the ESP to indicate that it has data to send.
    async fn wait_for_ready(&mut self);

    /// Perform a transfer, exchanging data with the ESP chip.
    ///
    /// `tx_len` is the number of bytes in the payload.
    ///
    /// The payload bytes contain the valid length of the received buffer.
    async fn transfer(&mut self, buffer: &mut [u8], tx_len: usize);
}
