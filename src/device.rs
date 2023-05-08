use crate::socket;
use crate::spi::SpiInterface;
use embedded_hal_async::spi::SpiDevice;

pub const MODE: u16 = 0x00;
pub const MAC: u16 = 0x09;
pub const SOCKET_INTR: u16 = 0x18;
pub const PHY_CFG: u16 = 0x2E;

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
        bus.write_frame(RegisterBlock::Common, MODE, &[0x80])
            .await?;

        // Enable interrupt pin
        bus.write_frame(RegisterBlock::Common, SOCKET_INTR, &[0x01])
            .await?;
        // Enable receive interrupt
        bus.write_frame(
            RegisterBlock::Socket0,
            socket::SOCKET_INTR_MASK,
            &[socket::Interrupt::Receive as u8],
        )
        .await?;

        // Set MAC address
        bus.write_frame(RegisterBlock::Common, MAC, &mac_addr)
            .await?;

        // Set the raw socket RX/TX buffer sizes to  16KB
        bus.write_frame(RegisterBlock::Socket0, socket::TXBUF_SIZE, &[16])
            .await?;
        bus.write_frame(RegisterBlock::Socket0, socket::RXBUF_SIZE, &[16])
            .await?;

        // MACRAW mode with MAC filtering.
        let mode: u8 = (1 << 2) | (1 << 7);
        bus.write_frame(RegisterBlock::Socket0, socket::MODE, &[mode])
            .await?;
        socket::command(&mut bus, socket::Command::Open).await?;

        Ok(Self { bus })
    }

    /// Read bytes from the RX buffer. Returns the number of bytes read.
    async fn read_bytes(&mut self, buffer: &mut [u8], offset: u16) -> Result<usize, SPI::Error> {
        let rx_size = socket::get_rx_size(&mut self.bus).await? as usize;

        let read_buffer = if rx_size > buffer.len() + offset as usize {
            buffer
        } else {
            &mut buffer[..rx_size - offset as usize]
        };

        let read_ptr = socket::get_rx_read_ptr(&mut self.bus)
            .await?
            .wrapping_add(offset);
        self.bus
            .read_frame(RegisterBlock::RxBuf, read_ptr, read_buffer)
            .await?;
        socket::set_rx_read_ptr(
            &mut self.bus,
            read_ptr.wrapping_add(read_buffer.len() as u16),
        )
        .await?;

        Ok(read_buffer.len())
    }

    /// Read an ethernet frame from the device. Returns the number of bytes read.
    pub async fn read_frame(&mut self, frame: &mut [u8]) -> Result<usize, SPI::Error> {
        let rx_size = socket::get_rx_size(&mut self.bus).await? as usize;
        if rx_size == 0 {
            return Ok(0);
        }

        socket::reset_interrupt(&mut self.bus, socket::Interrupt::Receive).await?;

        // First two bytes gives the size of the received ethernet frame
        let expected_frame_size: usize = {
            let mut frame_bytes = [0u8; 2];
            assert!(self.read_bytes(&mut frame_bytes[..], 0).await? == 2);
            u16::from_be_bytes(frame_bytes) as usize - 2
        };

        // Read the ethernet frame
        let read_buffer = if frame.len() > expected_frame_size {
            &mut frame[..expected_frame_size]
        } else {
            frame
        };

        let recvd_frame_size = self.read_bytes(read_buffer, 2).await?;

        // Register RX as completed
        socket::command(&mut self.bus, socket::Command::Receive).await?;

        // If the whole frame wasn't read, drop it
        if recvd_frame_size < expected_frame_size {
            Ok(0)
        } else {
            Ok(recvd_frame_size)
        }
    }

    /// Write an ethernet frame to the device. Returns number of bytes written
    pub async fn write_frame(&mut self, frame: &[u8]) -> Result<usize, SPI::Error> {
        let max_size = socket::get_tx_free_size(&mut self.bus).await? as usize;

        let write_data = if frame.len() < max_size {
            frame
        } else {
            &frame[..max_size]
        };

        let write_ptr = socket::get_tx_write_ptr(&mut self.bus).await?;
        self.bus
            .write_frame(RegisterBlock::TxBuf, write_ptr, write_data)
            .await?;
        socket::set_tx_write_ptr(
            &mut self.bus,
            write_ptr.wrapping_add(write_data.len() as u16),
        )
        .await?;

        socket::reset_interrupt(&mut self.bus, socket::Interrupt::SendOk).await?;
        socket::command(&mut self.bus, socket::Command::Send).await?;
        // Wait for TX to complete
        while !socket::is_interrupt(&mut self.bus, socket::Interrupt::SendOk).await? {}
        socket::reset_interrupt(&mut self.bus, socket::Interrupt::SendOk).await?;

        Ok(write_data.len())
    }

    pub async fn is_link_up(&mut self) -> bool {
        let mut link = [0];
        self.bus
            .read_frame(RegisterBlock::Common, PHY_CFG, &mut link)
            .await
            .ok();
        link[0] & 1 == 1
    }
}
