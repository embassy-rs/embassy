use embedded_hal_async::spi::SpiDevice;

use crate::spi::{Address, SpiInterface};

pub const COMMON_MODE: Address = (RegisterBlock::Common, 0x00);
pub const COMMON_MAC: Address = (RegisterBlock::Common, 0x09);
pub const COMMON_SOCKET_INTR: Address = (RegisterBlock::Common, 0x18);
pub const COMMON_PHY_CFG: Address = (RegisterBlock::Common, 0x2E);

pub const SOCKET_MODE: Address = (RegisterBlock::Socket0, 0x00);
pub const SOCKET_COMMAND: Address = (RegisterBlock::Socket0, 0x01);
pub const SOCKET_RXBUF_SIZE: Address = (RegisterBlock::Socket0, 0x1E);
pub const SOCKET_TXBUF_SIZE: Address = (RegisterBlock::Socket0, 0x1F);
pub const SOCKET_TX_FREE_SIZE: Address = (RegisterBlock::Socket0, 0x20);
pub const SOCKET_TX_DATA_WRITE_PTR: Address = (RegisterBlock::Socket0, 0x24);
pub const SOCKET_RECVD_SIZE: Address = (RegisterBlock::Socket0, 0x26);
pub const SOCKET_RX_DATA_READ_PTR: Address = (RegisterBlock::Socket0, 0x28);
pub const SOCKET_INTR_MASK: Address = (RegisterBlock::Socket0, 0x2C);
pub const SOCKET_INTR: Address = (RegisterBlock::Socket0, 0x02);

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

#[repr(u8)]
pub enum RegisterBlock {
    Common = 0x00,
    Socket0 = 0x01,
    TxBuf = 0x02,
    RxBuf = 0x03,
}

/// W5500 in MACRAW mode
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct W5500<SPI> {
    bus: SpiInterface<SPI>,
}

impl<SPI: SpiDevice> W5500<SPI> {
    /// Create and initialize the W5500 driver
    pub async fn new(spi: SPI, mac_addr: [u8; 6]) -> Result<W5500<SPI>, SPI::Error> {
        let mut bus = SpiInterface(spi);
        // Reset device
        bus.write_frame(COMMON_MODE, &[0x80]).await?;

        // Enable interrupt pin
        bus.write_frame(COMMON_SOCKET_INTR, &[0x01]).await?;
        // Enable receive interrupt
        bus.write_frame(SOCKET_INTR_MASK, &[Interrupt::Receive as u8]).await?;

        // Set MAC address
        bus.write_frame(COMMON_MAC, &mac_addr).await?;

        // Set the raw socket RX/TX buffer sizes to  16KB
        bus.write_frame(SOCKET_TXBUF_SIZE, &[16]).await?;
        bus.write_frame(SOCKET_RXBUF_SIZE, &[16]).await?;

        // MACRAW mode with MAC filtering.
        let mode: u8 = (1 << 2) | (1 << 7);
        bus.write_frame(SOCKET_MODE, &[mode]).await?;
        let mut this = Self { bus };
        this.command(Command::Open).await?;

        Ok(this)
    }

    async fn reset_interrupt(&mut self, code: Interrupt) -> Result<(), SPI::Error> {
        let data = [code as u8];
        self.bus.write_frame(SOCKET_INTR, &data).await
    }

    async fn get_tx_write_ptr(&mut self) -> Result<u16, SPI::Error> {
        let mut data = [0u8; 2];
        self.bus.read_frame(SOCKET_TX_DATA_WRITE_PTR, &mut data).await?;
        Ok(u16::from_be_bytes(data))
    }

    async fn set_tx_write_ptr(&mut self, ptr: u16) -> Result<(), SPI::Error> {
        let data = ptr.to_be_bytes();
        self.bus.write_frame(SOCKET_TX_DATA_WRITE_PTR, &data).await
    }

    async fn get_rx_read_ptr(&mut self) -> Result<u16, SPI::Error> {
        let mut data = [0u8; 2];
        self.bus.read_frame(SOCKET_RX_DATA_READ_PTR, &mut data).await?;
        Ok(u16::from_be_bytes(data))
    }

    async fn set_rx_read_ptr(&mut self, ptr: u16) -> Result<(), SPI::Error> {
        let data = ptr.to_be_bytes();
        self.bus.write_frame(SOCKET_RX_DATA_READ_PTR, &data).await
    }

    async fn command(&mut self, command: Command) -> Result<(), SPI::Error> {
        let data = [command as u8];
        self.bus.write_frame(SOCKET_COMMAND, &data).await
    }

    async fn get_rx_size(&mut self) -> Result<u16, SPI::Error> {
        loop {
            // Wait until two sequential reads are equal
            let mut res0 = [0u8; 2];
            self.bus.read_frame(SOCKET_RECVD_SIZE, &mut res0).await?;
            let mut res1 = [0u8; 2];
            self.bus.read_frame(SOCKET_RECVD_SIZE, &mut res1).await?;
            if res0 == res1 {
                break Ok(u16::from_be_bytes(res0));
            }
        }
    }

    async fn get_tx_free_size(&mut self) -> Result<u16, SPI::Error> {
        let mut data = [0; 2];
        self.bus.read_frame(SOCKET_TX_FREE_SIZE, &mut data).await?;
        Ok(u16::from_be_bytes(data))
    }

    /// Read bytes from the RX buffer. Returns the number of bytes read.
    async fn read_bytes(&mut self, read_ptr: &mut u16, buffer: &mut [u8]) -> Result<(), SPI::Error> {
        self.bus.read_frame((RegisterBlock::RxBuf, *read_ptr), buffer).await?;
        *read_ptr = (*read_ptr).wrapping_add(buffer.len() as u16);

        Ok(())
    }

    /// Read an ethernet frame from the device. Returns the number of bytes read.
    pub async fn read_frame(&mut self, frame: &mut [u8]) -> Result<usize, SPI::Error> {
        let rx_size = self.get_rx_size().await? as usize;
        if rx_size == 0 {
            return Ok(0);
        }

        self.reset_interrupt(Interrupt::Receive).await?;

        let mut read_ptr = self.get_rx_read_ptr().await?;

        // First two bytes gives the size of the received ethernet frame
        let expected_frame_size: usize = {
            let mut frame_bytes = [0u8; 2];
            self.read_bytes(&mut read_ptr, &mut frame_bytes).await?;
            u16::from_be_bytes(frame_bytes) as usize - 2
        };

        // Read the ethernet frame
        self.read_bytes(&mut read_ptr, &mut frame[..expected_frame_size])
            .await?;

        // Register RX as completed
        self.set_rx_read_ptr(read_ptr).await?;
        self.command(Command::Receive).await?;

        Ok(expected_frame_size)
    }

    /// Write an ethernet frame to the device. Returns number of bytes written
    pub async fn write_frame(&mut self, frame: &[u8]) -> Result<usize, SPI::Error> {
        while self.get_tx_free_size().await? < frame.len() as u16 {}
        let write_ptr = self.get_tx_write_ptr().await?;
        self.bus.write_frame((RegisterBlock::TxBuf, write_ptr), frame).await?;
        self.set_tx_write_ptr(write_ptr.wrapping_add(frame.len() as u16))
            .await?;
        self.command(Command::Send).await?;
        Ok(frame.len())
    }

    pub async fn is_link_up(&mut self) -> bool {
        let mut link = [0];
        self.bus.read_frame(COMMON_PHY_CFG, &mut link).await.ok();
        link[0] & 1 == 1
    }
}
