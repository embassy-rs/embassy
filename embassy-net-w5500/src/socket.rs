use embedded_hal_async::spi::SpiDevice;

use crate::device::RegisterBlock;
use crate::spi::SpiInterface;

pub const MODE: u16 = 0x00;
pub const COMMAND: u16 = 0x01;
pub const RXBUF_SIZE: u16 = 0x1E;
pub const TXBUF_SIZE: u16 = 0x1F;
pub const TX_FREE_SIZE: u16 = 0x20;
pub const TX_DATA_WRITE_PTR: u16 = 0x24;
pub const RECVD_SIZE: u16 = 0x26;
pub const RX_DATA_READ_PTR: u16 = 0x28;
pub const SOCKET_INTR_MASK: u16 = 0x2C;

#[repr(u8)]
pub enum Command {
    Open = 0x01,
    Send = 0x20,
    Receive = 0x40,
}

pub const INTR: u16 = 0x02;
#[repr(u8)]
pub enum Interrupt {
    Receive = 0b00100_u8,
}

pub async fn reset_interrupt<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>, code: Interrupt) -> Result<(), SPI::Error> {
    let data = [code as u8];
    bus.write_frame(RegisterBlock::Socket0, INTR, &data).await
}

pub async fn get_tx_write_ptr<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>) -> Result<u16, SPI::Error> {
    let mut data = [0u8; 2];
    bus.read_frame(RegisterBlock::Socket0, TX_DATA_WRITE_PTR, &mut data)
        .await?;
    Ok(u16::from_be_bytes(data))
}

pub async fn set_tx_write_ptr<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>, ptr: u16) -> Result<(), SPI::Error> {
    let data = ptr.to_be_bytes();
    bus.write_frame(RegisterBlock::Socket0, TX_DATA_WRITE_PTR, &data).await
}

pub async fn get_rx_read_ptr<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>) -> Result<u16, SPI::Error> {
    let mut data = [0u8; 2];
    bus.read_frame(RegisterBlock::Socket0, RX_DATA_READ_PTR, &mut data)
        .await?;
    Ok(u16::from_be_bytes(data))
}

pub async fn set_rx_read_ptr<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>, ptr: u16) -> Result<(), SPI::Error> {
    let data = ptr.to_be_bytes();
    bus.write_frame(RegisterBlock::Socket0, RX_DATA_READ_PTR, &data).await
}

pub async fn command<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>, command: Command) -> Result<(), SPI::Error> {
    let data = [command as u8];
    bus.write_frame(RegisterBlock::Socket0, COMMAND, &data).await
}

pub async fn get_rx_size<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>) -> Result<u16, SPI::Error> {
    loop {
        // Wait until two sequential reads are equal
        let mut res0 = [0u8; 2];
        bus.read_frame(RegisterBlock::Socket0, RECVD_SIZE, &mut res0).await?;
        let mut res1 = [0u8; 2];
        bus.read_frame(RegisterBlock::Socket0, RECVD_SIZE, &mut res1).await?;
        if res0 == res1 {
            break Ok(u16::from_be_bytes(res0));
        }
    }
}

pub async fn get_tx_free_size<SPI: SpiDevice>(bus: &mut SpiInterface<SPI>) -> Result<u16, SPI::Error> {
    let mut data = [0; 2];
    bus.read_frame(RegisterBlock::Socket0, TX_FREE_SIZE, &mut data).await?;
    Ok(u16::from_be_bytes(data))
}
