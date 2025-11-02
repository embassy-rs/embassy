use embassy_time::Timer;
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

/// Physical interface trait for communicating with the ESP chip.
pub trait Interface {
    /// Wait for the HANDSHAKE signal indicating the ESP is ready for a new transaction.
    async fn wait_for_handshake(&mut self);

    /// Wait for the READY signal indicating the ESP has data to send.
    async fn wait_for_ready(&mut self);

    /// Perform a SPI transfer, exchanging data with the ESP chip.
    async fn transfer(&mut self, rx: &mut [u8], tx: &[u8]);
}

/// Standard SPI interface.
///
/// This interface is what's implemented in the upstream `esp-hosted-fg` firmware. It uses:
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
    async fn wait_for_handshake(&mut self) {
        self.handshake.wait_for_high().await.unwrap();
    }

    async fn wait_for_ready(&mut self) {
        self.ready.wait_for_high().await.unwrap();
    }

    async fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) {
        self.spi.transfer(rx, tx).await.unwrap();

        // The esp-hosted firmware deasserts the HANDSHAKE pin a few us AFTER ending the SPI transfer
        // If we check it again too fast, we'll see it's high from the previous transfer, and if we send it
        // data it will get lost.
        Timer::after_micros(100).await;
    }
}
