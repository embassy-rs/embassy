//! Wiznet W5100s and W5500 family driver.
mod w5500;
pub use w5500::W5500;
mod w5100s;
pub use w5100s::W5100S;

pub(crate) mod sealed {
    use embedded_hal_async::spi::SpiDevice;

    pub trait Chip {
        type Address;

        const COMMON_MODE: Self::Address;
        const COMMON_MAC: Self::Address;
        const COMMON_SOCKET_INTR: Self::Address;
        const COMMON_PHY_CFG: Self::Address;
        const SOCKET_MODE: Self::Address;
        const SOCKET_COMMAND: Self::Address;
        const SOCKET_RXBUF_SIZE: Self::Address;
        const SOCKET_TXBUF_SIZE: Self::Address;
        const SOCKET_TX_FREE_SIZE: Self::Address;
        const SOCKET_TX_DATA_WRITE_PTR: Self::Address;
        const SOCKET_RECVD_SIZE: Self::Address;
        const SOCKET_RX_DATA_READ_PTR: Self::Address;
        const SOCKET_INTR_MASK: Self::Address;
        const SOCKET_INTR: Self::Address;

        const SOCKET_MODE_VALUE: u8;

        const BUF_SIZE: u16;
        const AUTO_WRAP: bool;

        fn rx_addr(addr: u16) -> Self::Address;
        fn tx_addr(addr: u16) -> Self::Address;

        async fn bus_read<SPI: SpiDevice>(
            spi: &mut SPI,
            address: Self::Address,
            data: &mut [u8],
        ) -> Result<(), SPI::Error>;
        async fn bus_write<SPI: SpiDevice>(
            spi: &mut SPI,
            address: Self::Address,
            data: &[u8],
        ) -> Result<(), SPI::Error>;
    }
}

/// Trait for Wiznet chips.
pub trait Chip: sealed::Chip {}
