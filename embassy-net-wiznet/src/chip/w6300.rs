use crate::wiznet_spi_interface::{SpiType, WiznetSpiBus, WiznetSpiOperation};

#[repr(u8)]
pub enum RegisterBlock {
    Common = 0x00,
    Socket0 = 0x01,
    TxBuf = 0x02,
    RxBuf = 0x03,
}

/// Return bits to be ORed with instruction byte to set SPI mode corresponding to given SPI type
const fn spi_mode_instruction_bits(spi_type: &SpiType) -> u8 {
    // page 75 of datasheet
    // https://docs.wiznet.io/pdf-viewer?file=%2Fassets%2Ffiles%2F20251204_W6300_DS_V101E-4f4cd2e75de8d76f51a741f6a492ea01.pdf
    match spi_type {
        SpiType::Single => 0b0000_0000,
        SpiType::Dual => 0b0100_0000,
        SpiType::Quad => 0b1000_0000,
    }
}

/// Wiznet W6300 chip.
pub enum W6300 {}

impl super::Chip for W6300 {}
impl super::SealedChip for W6300 {
    type Address = (RegisterBlock, u16);

    // CIDR2 Minor Chip ID
    const CHIP_VERSION: u8 = 0x11;

    const COMMON_MODE: Self::Address = (RegisterBlock::Common, 0x2004);
    // SHAR0 (Source Hardware Address Register)
    const COMMON_MAC: Self::Address = (RegisterBlock::Common, 0x4120);
    // SIMR (SOCKET Interrupt Mask Register)
    const COMMON_SOCKET_INTR: Self::Address = (RegisterBlock::Common, 0x2114);
    // PHYSR (PHY Status Register)
    const COMMON_PHY_CFG: Self::Address = (RegisterBlock::Common, 0x3000);
    // CIDR2 (Minor Chip IP Register)
    const COMMON_VERSION: Self::Address = (RegisterBlock::Common, 0x0004);

    // Sn_MR (SOCKET n Mode Register)
    const SOCKET_MODE: Self::Address = (RegisterBlock::Socket0, 0x0000);
    // Sn_CR (SOCKET n Command Register)
    const SOCKET_COMMAND: Self::Address = (RegisterBlock::Socket0, 0x0010);
    // Sn_RX_BSR (SOCKET n RX Buffer Size Register)
    const SOCKET_RXBUF_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0220);
    // Sn_TX_BSR (SOCKET n TX Buffer Size Register)
    const SOCKET_TXBUF_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0200);
    // Sn_TX_FSR0 (SOCKET n TX Free Size Register)
    const SOCKET_TX_FREE_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0204);
    // Sn_TX_WR0 (SOCKET n TX Write Pointer Register)
    const SOCKET_TX_DATA_WRITE_PTR: Self::Address = (RegisterBlock::Socket0, 0x020C);
    // Sn_RX_RSR0 (SOCKET n RX Received Size Register)
    const SOCKET_RECVD_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0224);
    // Sn_RX_RD0 (SOCKET n RX Read Pointer Register)
    const SOCKET_RX_DATA_READ_PTR: Self::Address = (RegisterBlock::Socket0, 0x0228);
    // Sn_IMR (SOCKET n Interrupt Mask Register)
    const SOCKET_INTR_MASK: Self::Address = (RegisterBlock::Socket0, 0x0024);
    // Sn_IR (SOCKET n Interrupt Register)
    const SOCKET_INTR: Self::Address = (RegisterBlock::Socket0, 0x0020);
    // Sn_IRCLR (Sn_IR Clear Register)
    const SOCKET_INTR_CLR: Self::Address = (RegisterBlock::Socket0, 0x0028);

    // MACRAW mode. See Page 57 of https://docs.wiznet.io/pdf-viewer?file=%2Fassets%2Ffiles%2F20251204_W6300_DS_V101E-4f4cd2e75de8d76f51a741f6a492ea01.pdf
    // Note: Bit 7 is MAC filter. On the W5500 this is normally turned ON however the W6300 will not successfully retrieve an IP address with this enabled. Disabling for now and will have live with the extra noise.
    const SOCKET_MODE_VALUE: u8 = 0b0000_0111;

    const BUF_SIZE: u16 = 0x8000;
    const AUTO_WRAP: bool = true;

    fn rx_addr(addr: u16) -> Self::Address {
        (RegisterBlock::RxBuf, addr)
    }

    fn tx_addr(addr: u16) -> Self::Address {
        (RegisterBlock::TxBuf, addr)
    }

    async fn bus_read<SPI: WiznetSpiBus>(
        spi: &mut SPI,
        address: Self::Address,
        data: &mut [u8],
    ) -> Result<(), SPI::Error> {
        let spi_mode_bits: u8 = spi_mode_instruction_bits(&SPI::SPI_TYPE);
        let instruction_phase = [(address.0 as u8) | spi_mode_bits];
        let address_phase = address.1.to_be_bytes();
        let dummy_phase = [0u8];

        let operations = [
            WiznetSpiOperation::WriteSingleLine(&instruction_phase),
            WiznetSpiOperation::Write(&address_phase),
            WiznetSpiOperation::Write(&dummy_phase),
            WiznetSpiOperation::Read(data),
        ];
        spi.transaction(operations).await
    }

    async fn bus_write<SPI: WiznetSpiBus>(
        spi: &mut SPI,
        address: Self::Address,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        const WRITE_ACCESS_BIT: u8 = 0b0010_0000;
        let spi_mode_bits: u8 = spi_mode_instruction_bits(&SPI::SPI_TYPE);
        // Set the SPI Mode and Write Access bits
        let instruction_phase = [(address.0 as u8) | spi_mode_bits | WRITE_ACCESS_BIT];
        let address_phase = address.1.to_be_bytes();
        let dummy_phase = [0u8];

        let operations = [
            WiznetSpiOperation::WriteSingleLine(&instruction_phase),
            WiznetSpiOperation::Write(&address_phase),
            WiznetSpiOperation::Write(&dummy_phase),
            WiznetSpiOperation::Write(data),
        ];
        spi.transaction(operations).await
    }
}
