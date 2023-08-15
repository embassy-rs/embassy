use embedded_hal_async::spi::SpiDevice;

use crate::device::RegisterBlock;
use crate::spi::{Address, SpiInterface};

pub const MODE: Address = (RegisterBlock::Socket0, 0x00);
pub const COMMAND: Address = (RegisterBlock::Socket0, 0x01);
pub const RXBUF_SIZE: Address = (RegisterBlock::Socket0, 0x1E);
pub const TXBUF_SIZE: Address = (RegisterBlock::Socket0, 0x1F);
pub const TX_FREE_SIZE: Address = (RegisterBlock::Socket0, 0x20);
pub const TX_DATA_WRITE_PTR: Address = (RegisterBlock::Socket0, 0x24);
pub const RECVD_SIZE: Address = (RegisterBlock::Socket0, 0x26);
pub const RX_DATA_READ_PTR: Address = (RegisterBlock::Socket0, 0x28);
pub const SOCKET_INTR_MASK: Address = (RegisterBlock::Socket0, 0x2C);
pub const INTR: Address = (RegisterBlock::Socket0, 0x02);

#[repr(u8)]
pub enum Command {
    Open = 0x01,
    Send = 0x20,
    Receive = 0x40,
}

#[repr(u8)]
pub enum Interrupt {
    Receive = 0b00100_u8,
}

pub async fn reset_interrupt<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>, code: Interrupt) -> Result<(), SPI::Error> {
    let data = [code as u8];
    bus.write_frame(INTR, &data).await
}

pub async fn get_tx_write_ptr<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>) -> Result<u16, SPI::Error> {
    let mut data = [0u8; 2];
    bus.read_frame(TX_DATA_WRITE_PTR, &mut data).await?;
    Ok(u16::from_be_bytes(data))
}

pub async fn set_tx_write_ptr<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>, ptr: u16) -> Result<(), SPI::Error> {
    let data = ptr.to_be_bytes();
    bus.write_frame(TX_DATA_WRITE_PTR, &data).await
}

pub async fn get_rx_read_ptr<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>) -> Result<u16, SPI::Error> {
    let mut data = [0u8; 2];
    bus.read_frame(RX_DATA_READ_PTR, &mut data).await?;
    Ok(u16::from_be_bytes(data))
}

pub async fn set_rx_read_ptr<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>, ptr: u16) -> Result<(), SPI::Error> {
    let data = ptr.to_be_bytes();
    bus.write_frame(RX_DATA_READ_PTR, &data).await
}

pub async fn command<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>, command: Command) -> Result<(), SPI::Error> {
    let data = [command as u8];
    bus.write_frame(COMMAND, &data).await
}

pub async fn get_rx_size<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>) -> Result<u16, SPI::Error> {
    loop {
        // Wait until two sequential reads are equal
        let mut res0 = [0u8; 2];
        bus.read_frame(RECVD_SIZE, &mut res0).await?;
        let mut res1 = [0u8; 2];
        bus.read_frame(RECVD_SIZE, &mut res1).await?;
        if res0 == res1 {
            break Ok(u16::from_be_bytes(res0));
        }
    }
}

pub async fn get_tx_free_size<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>) -> Result<u16, SPI::Error> {
    let mut data = [0; 2];
    bus.read_frame(TX_FREE_SIZE, &mut data).await?;
    Ok(u16::from_be_bytes(data))
}
