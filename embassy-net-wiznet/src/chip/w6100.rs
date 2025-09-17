use embedded_hal_async::spi::{Operation, SpiDevice};

#[repr(u8)]
pub enum RegisterBlock {
    Common = 0x00,
    Socket0 = 0x01,
    TxBuf = 0x02,
    RxBuf = 0x03,
}

/// Wiznet W6100 chip.
pub enum W6100 {}

impl super::Chip for W6100 {}
impl super::SealedChip for W6100 {
    type Address = (RegisterBlock, u16);

    const CHIP_VERSION: u8 = 0x46;

    const COMMON_MODE: Self::Address = (RegisterBlock::Common, 0x2004);
    const COMMON_MAC: Self::Address = (RegisterBlock::Common, 0x4120);
    // SIMR (SOCKET Interrupt Mask Register)
    const COMMON_SOCKET_INTR: Self::Address = (RegisterBlock::Common, 0x2114);
    const COMMON_PHY_CFG: Self::Address = (RegisterBlock::Common, 0x3000);
    const COMMON_VERSION: Self::Address = (RegisterBlock::Common, 0x0002);

    const SOCKET_MODE: Self::Address = (RegisterBlock::Socket0, 0x0000);
    const SOCKET_COMMAND: Self::Address = (RegisterBlock::Socket0, 0x0010);
    const SOCKET_RXBUF_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0220);
    const SOCKET_TXBUF_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0200);
    const SOCKET_TX_FREE_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0204);
    const SOCKET_TX_DATA_WRITE_PTR: Self::Address = (RegisterBlock::Socket0, 0x020C);
    const SOCKET_RECVD_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0224);
    const SOCKET_RX_DATA_READ_PTR: Self::Address = (RegisterBlock::Socket0, 0x0228);
    // Sn_IMR (SOCKET n Interrupt Mask Register)
    const SOCKET_INTR_MASK: Self::Address = (RegisterBlock::Socket0, 0x0024);
    // Sn_IR (SOCKET n Interrupt Register)
    const SOCKET_INTR: Self::Address = (RegisterBlock::Socket0, 0x0020);
    // Sn_IRCLR (Sn_IR Clear Register)
    const SOCKET_INTR_CLR: Self::Address = (RegisterBlock::Socket0, 0x0028);

    // MACRAW mode. See Page 57 of https://docs.wiznet.io/img/products/w6100/w6100_ds_v105e.pdf
    // Note: Bit 7 is MAC filter. On the W5500 this is normally turned ON however the W6100 will not successfully retrieve an IP address with this enabled. Disabling for now and will have live with the extra noise.
    const SOCKET_MODE_VALUE: u8 = 0b0000_0111;

    const BUF_SIZE: u16 = 0x1000;
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
