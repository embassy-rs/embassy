use core::marker::PhantomData;

use embedded_hal_async::spi::SpiDevice;

use crate::chip::Chip;

#[repr(u8)]
enum Command {
    Open = 0x01,
    Send = 0x20,
    Receive = 0x40,
}

#[repr(u8)]
enum Interrupt {
    Receive = 0b00100_u8,
}

/// Wiznet chip in MACRAW mode
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct WiznetDevice<C, SPI> {
    spi: SPI,
    _phantom: PhantomData<C>,
}

impl<C: Chip, SPI: SpiDevice> WiznetDevice<C, SPI> {
    /// Create and initialize the driver
    pub async fn new(spi: SPI, mac_addr: [u8; 6]) -> Result<Self, SPI::Error> {
        let mut this = Self {
            spi,
            _phantom: PhantomData,
        };

        // Reset device
        this.bus_write(C::COMMON_MODE, &[0x80]).await?;

        // Enable interrupt pin
        this.bus_write(C::COMMON_SOCKET_INTR, &[0x01]).await?;
        // Enable receive interrupt
        this.bus_write(C::SOCKET_INTR_MASK, &[Interrupt::Receive as u8]).await?;

        // Set MAC address
        this.bus_write(C::COMMON_MAC, &mac_addr).await?;

        // Set the raw socket RX/TX buffer sizes.
        let buf_kbs = (C::BUF_SIZE / 1024) as u8;
        this.bus_write(C::SOCKET_TXBUF_SIZE, &[buf_kbs]).await?;
        this.bus_write(C::SOCKET_RXBUF_SIZE, &[buf_kbs]).await?;

        // MACRAW mode with MAC filtering.
        this.bus_write(C::SOCKET_MODE, &[C::SOCKET_MODE_VALUE]).await?;
        this.command(Command::Open).await?;

        Ok(this)
    }

    async fn bus_read(&mut self, address: C::Address, data: &mut [u8]) -> Result<(), SPI::Error> {
        C::bus_read(&mut self.spi, address, data).await
    }

    async fn bus_write(&mut self, address: C::Address, data: &[u8]) -> Result<(), SPI::Error> {
        C::bus_write(&mut self.spi, address, data).await
    }

    async fn reset_interrupt(&mut self, code: Interrupt) -> Result<(), SPI::Error> {
        let data = [code as u8];
        self.bus_write(C::SOCKET_INTR, &data).await
    }

    async fn get_tx_write_ptr(&mut self) -> Result<u16, SPI::Error> {
        let mut data = [0u8; 2];
        self.bus_read(C::SOCKET_TX_DATA_WRITE_PTR, &mut data).await?;
        Ok(u16::from_be_bytes(data))
    }

    async fn set_tx_write_ptr(&mut self, ptr: u16) -> Result<(), SPI::Error> {
        let data = ptr.to_be_bytes();
        self.bus_write(C::SOCKET_TX_DATA_WRITE_PTR, &data).await
    }

    async fn get_rx_read_ptr(&mut self) -> Result<u16, SPI::Error> {
        let mut data = [0u8; 2];
        self.bus_read(C::SOCKET_RX_DATA_READ_PTR, &mut data).await?;
        Ok(u16::from_be_bytes(data))
    }

    async fn set_rx_read_ptr(&mut self, ptr: u16) -> Result<(), SPI::Error> {
        let data = ptr.to_be_bytes();
        self.bus_write(C::SOCKET_RX_DATA_READ_PTR, &data).await
    }

    async fn command(&mut self, command: Command) -> Result<(), SPI::Error> {
        let data = [command as u8];
        self.bus_write(C::SOCKET_COMMAND, &data).await
    }

    async fn get_rx_size(&mut self) -> Result<u16, SPI::Error> {
        loop {
            // Wait until two sequential reads are equal
            let mut res0 = [0u8; 2];
            self.bus_read(C::SOCKET_RECVD_SIZE, &mut res0).await?;
            let mut res1 = [0u8; 2];
            self.bus_read(C::SOCKET_RECVD_SIZE, &mut res1).await?;
            if res0 == res1 {
                break Ok(u16::from_be_bytes(res0));
            }
        }
    }

    async fn get_tx_free_size(&mut self) -> Result<u16, SPI::Error> {
        let mut data = [0; 2];
        self.bus_read(C::SOCKET_TX_FREE_SIZE, &mut data).await?;
        Ok(u16::from_be_bytes(data))
    }

    /// Read bytes from the RX buffer.
    async fn read_bytes(&mut self, read_ptr: &mut u16, buffer: &mut [u8]) -> Result<(), SPI::Error> {
        if C::AUTO_WRAP {
            self.bus_read(C::rx_addr(*read_ptr), buffer).await?;
        } else {
            let addr = *read_ptr % C::BUF_SIZE;
            if addr as usize + buffer.len() <= C::BUF_SIZE as usize {
                self.bus_read(C::rx_addr(addr), buffer).await?;
            } else {
                let n = C::BUF_SIZE - addr;
                self.bus_read(C::rx_addr(addr), &mut buffer[..n as usize]).await?;
                self.bus_read(C::rx_addr(0), &mut buffer[n as usize..]).await?;
            }
        }

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

        if C::AUTO_WRAP {
            self.bus_write(C::tx_addr(write_ptr), frame).await?;
        } else {
            let addr = write_ptr % C::BUF_SIZE;
            if addr as usize + frame.len() <= C::BUF_SIZE as usize {
                self.bus_write(C::tx_addr(addr), frame).await?;
            } else {
                let n = C::BUF_SIZE - addr;
                self.bus_write(C::tx_addr(addr), &frame[..n as usize]).await?;
                self.bus_write(C::tx_addr(0), &frame[n as usize..]).await?;
            }
        }

        self.set_tx_write_ptr(write_ptr.wrapping_add(frame.len() as u16))
            .await?;
        self.command(Command::Send).await?;
        Ok(frame.len())
    }

    pub async fn is_link_up(&mut self) -> bool {
        let mut link = [0];
        self.bus_read(C::COMMON_PHY_CFG, &mut link).await.ok();
        link[0] & 1 == 1
    }
}
