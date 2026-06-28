//! Half-duplex SPI interface.
//!
//! Only supported by `esp-hosted-mcu`.

use aligned::{A4, Aligned};
use embassy_time::Timer;
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

use crate::Interface;

/// Represents a register on the half-duplex SPI interface.
#[derive(Clone, Copy)]
pub enum HdRegister {
    /// Indicates if co-processor is ready
    CoprocessorReady = 0x00,

    /// The length of data the co-processor can transmit.
    ///
    /// Updated whenever co-processor wants to transmit data.
    TxBufLen = 0x0C,

    /// The length of data co-processor can receive.
    ///
    /// Updated whenever co-processor can receive data.
    RxBufLen = 0x10,

    /// Controls co-processor operation.
    CoprocessorControl = 0x14,
}

/// Represents a command on the half-duplex SPI interface.
#[derive(Clone, Copy)]
pub enum HdCommand {
    /// Write to a 32-bit buffer register on the co-processor.
    WriteReg = 0x01,

    /// Read from a 32-bit buffer register on the co-processor
    ReadReg = 0x02,

    /// Write data to the co-processor.
    WriteDma = 0x03,

    /// Read data from the co-processor.
    ReadDma = 0x04,

    /// End of write.
    WriteDone = 0x07,

    /// CMD8 - End of read.
    ReadDone = 0x08,

    /// CMD9 - The host is done with the register read. The co-processor can de-assert `Data_Ready`.
    Int1 = 0x09,
}

impl HdCommand {
    /// Returns the command opcode for Dual SPI mode.
    pub fn dual_spi(self) -> u8 {
        (self as u8) | 0x50
    }

    /// Returns the command opcode for Quad SPI mode.
    pub fn quad_spi(self) -> u8 {
        (self as u8) | 0xA0
    }
}

/// Trait for a half-duplex SPI interface.
///
/// The device must be configured to use the same number of data lines as the hosted firmware.
pub trait HdSpi {
    /// Read data from the SPI device using the given command and address.
    ///
    /// esp-hosted requires 8 dummy bits to be inserted between the address and the data.
    async fn read(&mut self, command: HdCommand, addr: u32, buf: &mut [u8]);

    /// Write data to the SPI device using the given command and address.
    ///
    /// esp-hosted requires 8 dummy bits to be inserted between the address and the data.
    async fn write(&mut self, command: HdCommand, addr: u32, buf: &[u8]);
}

/// Half-duplex SPI interface.
///
/// This interface is what's implemented in the upstream `esp-hosted` firmware. It uses:
/// - An `HdSpi` implementation for SPI communication (CS is handled by the device)
/// - A `Data_Ready` pin that indicates when the ESP has data to send
pub struct HdSpiInterface<D, DR> {
    spi: D,
    data_ready: DR,
    tx_buf_count: u32,
    rx_byte_count: u32,
    /// Last buffer-grant count read from `RxBufLen`. Cached so TX doesn't read
    /// the register on every packet; refreshed only when credits run out.
    cached_granted: u32,
}

impl<D, DR> HdSpiInterface<D, DR>
where
    D: HdSpi,
    DR: InputPin + Wait,
{
    const HEADER_LEN: usize = 12;
    const MAX_SPI_HD_BUFFER_SIZE: usize = 1600;
    const TX_BUF_LEN_MASK: u32 = 0x00FF_FFFF;
    const MAX_WRITE_BUF_RETRIES: u8 = 25;
    const POLLING_READ: u8 = 3;
    const COPROC_READY_FLAG: u32 = 0xEE;
    const CTRL_DATAPATH_ON: u32 = 1 << 0;

    /// Creates a new half-duplex SPI interface with the given SPI DMA driver and `Data_Ready` pin.
    pub fn new(spi: D, data_ready: DR) -> Self {
        Self {
            spi,
            data_ready,
            tx_buf_count: 0,
            rx_byte_count: 0,
            cached_granted: 0,
        }
    }

    async fn read_reg_once(&mut self, reg: HdRegister) -> u32 {
        let mut buf = [0u8; 4];
        self.spi.read(HdCommand::ReadReg, reg as u32, &mut buf).await;
        u32::from_le_bytes(buf)
    }

    async fn write_reg(&mut self, reg: HdRegister, value: u32) {
        self.spi
            .write(HdCommand::WriteReg, reg as u32, &value.to_le_bytes())
            .await;
    }

    // WR_DONE / CMD8 / CMD9: command + address + dummy, no data.
    fn command(&mut self, cmd: HdCommand) -> impl Future<Output = ()> {
        self.spi.write(cmd, 0, &[])
    }

    // Re-read until two reads agree; the co-processor may update mid-read.
    async fn read_reg(&mut self, reg: HdRegister) -> u32 {
        let mut value = self.read_reg_once(reg).await;
        for _ in 0..Self::POLLING_READ {
            let next = self.read_reg_once(reg).await;
            if next == value {
                break;
            }
            value = next;
        }
        value
    }

    fn has_credit(&self, buf_needed: u32) -> bool {
        self.cached_granted.wrapping_sub(self.tx_buf_count) >= buf_needed
    }

    async fn write_frame(&mut self, frame: &[u8]) {
        let buf_needed = frame.len().div_ceil(Self::MAX_SPI_HD_BUFFER_SIZE) as u32;

        let mut retries = Self::MAX_WRITE_BUF_RETRIES;
        loop {
            // Common path: spend a cached credit without touching the bus.
            if self.has_credit(buf_needed) {
                break;
            }
            // Out of cached credits: refresh from the co-processor.
            self.cached_granted = self.read_reg(HdRegister::RxBufLen).await;
            if self.has_credit(buf_needed) {
                break;
            }
            retries -= 1;
            if retries == 0 {
                warn!("spi-hd: no write buffers on co-processor, dropping packet");
                return;
            }
            Timer::after_millis(1).await;
        }

        self.spi.write(HdCommand::WriteDma, 0, frame).await;

        self.command(HdCommand::WriteDone).await;
        self.tx_buf_count = self.tx_buf_count.wrapping_add(buf_needed);
    }

    async fn read_frame(&mut self, buffer: &mut [u8]) -> usize {
        let tx_buf_len = self.read_reg(HdRegister::TxBufLen).await;
        self.command(HdCommand::Int1).await;

        let total = tx_buf_len & Self::TX_BUF_LEN_MASK;
        let size = (total.wrapping_sub(self.rx_byte_count) & Self::TX_BUF_LEN_MASK) as usize;
        if size == 0 {
            return 0;
        }
        if size > buffer.len() {
            warn!("spi-hd: rx size {} exceeds buffer, dropping", size);
            return 0;
        }

        self.spi.read(HdCommand::ReadDma, 0, &mut buffer[..size]).await;

        self.command(HdCommand::ReadDone).await;
        self.rx_byte_count = self.rx_byte_count.wrapping_add(size as u32) & Self::TX_BUF_LEN_MASK;
        size
    }
}

impl<D, DR> Interface for HdSpiInterface<D, DR>
where
    D: HdSpi,
    DR: InputPin + Wait,
{
    async fn init(&mut self, cold_boot: bool) {
        if !cold_boot {
            // Give the device time to reboot.
            Timer::after_secs(2).await;
        }

        self.tx_buf_count = 0;
        self.rx_byte_count = 0;
        self.cached_granted = 0;

        while self.read_reg_once(HdRegister::CoprocessorReady).await != Self::COPROC_READY_FLAG {
            Timer::after_millis(100).await;
        }
        self.write_reg(HdRegister::CoprocessorControl, Self::CTRL_DATAPATH_ON)
            .await;
    }

    async fn wait_for_handshake(&mut self) {
        // Half-duplex has no per-transaction handshake line.
    }

    async fn wait_for_ready(&mut self) {
        self.data_ready.wait_for_high().await.unwrap();
    }

    async fn transfer(&mut self, buffer: &mut Aligned<A4, [u8]>, tx_len: usize) {
        if tx_len > 0 {
            self.write_frame(&buffer[..tx_len]).await;
        }

        let received = if self.data_ready.is_high().unwrap_or(false) {
            self.read_frame(buffer).await
        } else {
            0
        };

        if received == 0 {
            buffer[..Self::HEADER_LEN].fill(0);
        }
    }
}
