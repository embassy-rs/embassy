use embedded_hal_async::spi::{Operation, SpiDevice};

#[repr(u8)]
pub enum RegisterBlock {
    Common = 0x00,
    Socket0 = 0x01,
    TxBuf = 0x02,
    RxBuf = 0x03,
}

/// Wiznet W5500 chip.
pub enum W5500 {}

impl super::Chip for W5500 {}
impl super::SealedChip for W5500 {
    type Address = (RegisterBlock, u16);

    const CHIP_VERSION: u8 = 0x04;

    const COMMON_MODE: Self::Address = (RegisterBlock::Common, 0x00);
    const COMMON_MAC: Self::Address = (RegisterBlock::Common, 0x09);
    const COMMON_SOCKET_INTR: Self::Address = (RegisterBlock::Common, 0x18);
    const COMMON_PHY_CFG: Self::Address = (RegisterBlock::Common, 0x2E);
    const COMMON_VERSION: Self::Address = (RegisterBlock::Common, 0x39);

    const SOCKET_MODE: Self::Address = (RegisterBlock::Socket0, 0x00);
    const SOCKET_COMMAND: Self::Address = (RegisterBlock::Socket0, 0x01);
    const SOCKET_RXBUF_SIZE: Self::Address = (RegisterBlock::Socket0, 0x1E);
    const SOCKET_TXBUF_SIZE: Self::Address = (RegisterBlock::Socket0, 0x1F);
    const SOCKET_TX_FREE_SIZE: Self::Address = (RegisterBlock::Socket0, 0x20);
    const SOCKET_TX_DATA_WRITE_PTR: Self::Address = (RegisterBlock::Socket0, 0x24);
    const SOCKET_RECVD_SIZE: Self::Address = (RegisterBlock::Socket0, 0x26);
    const SOCKET_RX_DATA_READ_PTR: Self::Address = (RegisterBlock::Socket0, 0x28);
    const SOCKET_INTR_MASK: Self::Address = (RegisterBlock::Socket0, 0x2C);
    const SOCKET_INTR: Self::Address = (RegisterBlock::Socket0, 0x02);

    const SOCKET_MODE_VALUE: u8 = (1 << 2) | (1 << 7);

    const BUF_SIZE: u16 = 0x4000;
    const AUTO_WRAP: bool = true;

    fn rx_addr(addr: u16) -> Self::Address {
        (RegisterBlock::RxBuf, addr)
    }

    fn tx_addr(addr: u16) -> Self::Address {
        (RegisterBlock::TxBuf, addr)
    }

    async fn bus_read<SPI: SpiDevice>(
        spi: &mut SPI,
        address: Self::Address,
        data: &mut [u8],
    ) -> Result<(), SPI::Error> {
        let address_phase = address.1.to_be_bytes();
        let control_phase = [(address.0 as u8) << 3];
        let operations = &mut [
            Operation::Write(&address_phase),
            Operation::Write(&control_phase),
            Operation::TransferInPlace(data),
        ];
        spi.transaction(operations).await
    }

    async fn bus_write<SPI: SpiDevice>(spi: &mut SPI, address: Self::Address, data: &[u8]) -> Result<(), SPI::Error> {
        let address_phase = address.1.to_be_bytes();
        let control_phase = [(address.0 as u8) << 3 | 0b0000_0100];
        let data_phase = data;
        let operations = &mut [
            Operation::Write(&address_phase[..]),
            Operation::Write(&control_phase),
            Operation::Write(&data_phase),
        ];
        spi.transaction(operations).await
    }
}
