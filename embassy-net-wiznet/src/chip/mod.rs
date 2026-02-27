//! Wiznet W5100s, W5500, W6100 and W6300 family driver.
mod w5500;
pub use w5500::W5500;

mod w5100s;
pub use w5100s::W5100S;

mod w6100;
pub use w6100::W6100;

mod w6300;
use embedded_hal_async::spi::SpiDevice;
pub use w6300::W6300;

pub(crate) trait SealedChip {
    type Address;

    /// The version of the chip as reported by the VERSIONR register.
    /// This is used to verify that the chip is supported by the driver,
    /// and that SPI communication is working.
    const CHIP_VERSION: u8;

    const COMMON_MODE: Self::Address;
    const COMMON_MAC: Self::Address;
    const COMMON_SOCKET_INTR: Self::Address;
    const COMMON_PHY_CFG: Self::Address;
    const COMMON_VERSION: Self::Address;

    const SOCKET_MODE: Self::Address;
    const SOCKET_COMMAND: Self::Address;
    const SOCKET_RXBUF_SIZE: Self::Address;
    const SOCKET_TXBUF_SIZE: Self::Address;
    const SOCKET_TX_FREE_SIZE: Self::Address;
    const SOCKET_TX_DATA_WRITE_PTR: Self::Address;
    const SOCKET_RECVD_SIZE: Self::Address;
    const SOCKET_RX_DATA_READ_PTR: Self::Address;
    const SOCKET_INTR_MASK: Self::Address;
    #[allow(dead_code)]
    const SOCKET_INTR: Self::Address;
    const SOCKET_INTR_CLR: Self::Address;

    const SOCKET_MODE_VALUE: u8;

    const BUF_SIZE: u16;
    const AUTO_WRAP: bool;

    fn rx_addr(addr: u16) -> Self::Address;
    fn tx_addr(addr: u16) -> Self::Address;

    async fn bus_read<SPI: SpiDevice>(spi: &mut SPI, address: Self::Address, data: &mut [u8])
    -> Result<(), SPI::Error>;
    async fn bus_write<SPI: SpiDevice>(spi: &mut SPI, address: Self::Address, data: &[u8]) -> Result<(), SPI::Error>;
}

/// Trait for Wiznet chips.
#[allow(private_bounds)]
pub trait Chip: SealedChip {}
