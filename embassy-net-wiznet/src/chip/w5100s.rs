use embedded_hal_async::spi::{Operation, SpiDevice};

const SOCKET_BASE: u16 = 0x400;
const TX_BASE: u16 = 0x4000;
const RX_BASE: u16 = 0x6000;

/// Wizard W5100S chip.
pub enum W5100S {}

impl super::Chip for W5100S {}
impl super::SealedChip for W5100S {
    type Address = u16;

    const CHIP_VERSION: u8 = 0x51;

    const COMMON_MODE: Self::Address = 0x00;
    const COMMON_MAC: Self::Address = 0x09;
    const COMMON_SOCKET_INTR: Self::Address = 0x16;
    const COMMON_PHY_CFG: Self::Address = 0x3c;
    const COMMON_VERSION: Self::Address = 0x80;

    const SOCKET_MODE: Self::Address = SOCKET_BASE + 0x00;
    const SOCKET_COMMAND: Self::Address = SOCKET_BASE + 0x01;
    const SOCKET_RXBUF_SIZE: Self::Address = SOCKET_BASE + 0x1E;
    const SOCKET_TXBUF_SIZE: Self::Address = SOCKET_BASE + 0x1F;
    const SOCKET_TX_FREE_SIZE: Self::Address = SOCKET_BASE + 0x20;
    const SOCKET_TX_DATA_WRITE_PTR: Self::Address = SOCKET_BASE + 0x24;
    const SOCKET_RECVD_SIZE: Self::Address = SOCKET_BASE + 0x26;
    const SOCKET_RX_DATA_READ_PTR: Self::Address = SOCKET_BASE + 0x28;
    const SOCKET_INTR_MASK: Self::Address = SOCKET_BASE + 0x2C;
    const SOCKET_INTR: Self::Address = SOCKET_BASE + 0x02;

    const SOCKET_MODE_VALUE: u8 = (1 << 2) | (1 << 6);

    const BUF_SIZE: u16 = 0x2000;
    const AUTO_WRAP: bool = false;

    fn rx_addr(addr: u16) -> Self::Address {
        RX_BASE + addr
    }

    fn tx_addr(addr: u16) -> Self::Address {
        TX_BASE + addr
    }

    async fn bus_read<SPI: SpiDevice>(
        spi: &mut SPI,
        address: Self::Address,
        data: &mut [u8],
    ) -> Result<(), SPI::Error> {
        spi.transaction(&mut [
            Operation::Write(&[0x0F, (address >> 8) as u8, address as u8]),
            Operation::Read(data),
        ])
        .await
    }

    async fn bus_write<SPI: SpiDevice>(spi: &mut SPI, address: Self::Address, data: &[u8]) -> Result<(), SPI::Error> {
        spi.transaction(&mut [
            Operation::Write(&[0xF0, (address >> 8) as u8, address as u8]),
            Operation::Write(data),
        ])
        .await
    }
}
