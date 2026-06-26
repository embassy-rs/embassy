//! Full-duplex SPI interface.

use aligned::{A4, Aligned};
use embassy_futures::join::join;
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

use crate::Interface;

/// Standard SPI interface.
///
/// This interface is what's implemented in the upstream `esp-hosted` firmware. It uses:
/// - An `SpiDevice` for SPI communication (CS is handled by the device)
/// - A handshake pin that signals when the ESP is ready for a new transaction
/// - A ready pin that indicates when the ESP has data to send
pub struct SpiInterface<SPI, IN> {
    spi: SPI,
    handshake: IN,
    ready: IN,
}

impl<SPI, IN> SpiInterface<SPI, IN>
where
    SPI: SpiDevice,
    IN: InputPin + Wait,
{
    /// Create a new SpiInterface.
    pub fn new(spi: SPI, handshake: IN, ready: IN) -> Self {
        Self { spi, handshake, ready }
    }
}

impl<SPI, IN> Interface for SpiInterface<SPI, IN>
where
    SPI: SpiDevice,
    IN: InputPin + Wait,
{
    async fn init(&mut self, _cold_boot: bool) {
        // No-op
    }

    async fn wait_for_handshake(&mut self) {
        self.handshake.wait_for_high().await.unwrap();
    }

    async fn wait_for_ready(&mut self) {
        self.ready.wait_for_high().await.unwrap();
    }

    async fn transfer(&mut self, buffer: &mut Aligned<A4, [u8]>, _tx_len: usize) {
        let (a, b) = join(
            // The esp-hosted firmware deasserts the HANDSHAKE pin a few us AFTER ending the SPI transfer
            // If we check it again too fast, we'll see it's high from the previous transfer, and if we send it
            // data it will get lost.
            self.handshake.wait_for_low(),
            // Always transfer the full buffer.
            self.spi.transfer_in_place(buffer),
        )
        .await;

        a.unwrap();
        b.unwrap();
    }
}
