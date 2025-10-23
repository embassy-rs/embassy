use embedded_hal_async::spi::{Operation, SpiDevice};
use heapless::Vec;

use super::Adin1110Protocol;
use crate::regs::SpiRegisters as sr;
use crate::{AdinError, ETH_MIN_LEN, FCS_LEN, FRAME_HEADER_LEN, MAX_BUFF, PORT_ID_BYTE};

/// TC6 (OPEN Alliance 10BASE-T1x MAC-PHY Serial Interface) protocol implementation
///
/// TC6 uses a chunk-based protocol where each chunk consists of:
/// - 4 bytes of overhead (header for TX, footer for RX)
/// - 64 bytes of payload
///
/// For control transactions (register access): DNC bit = 0
/// For data transactions (Ethernet frames): DNC bit = 1
pub struct Tc6<SPI> {
    spi: SPI,
}

/// TC6 chunk payload size (64 bytes as per OPEN Alliance spec)
const TC6_CHUNK_PAYLOAD_SIZE: usize = 64;

/// TC6 header/footer size (4 bytes)
const TC6_HDR_SIZE: usize = 4;

/// Total chunk size including header and payload
const TC6_CHUNK_SIZE: usize = TC6_HDR_SIZE + TC6_CHUNK_PAYLOAD_SIZE;

/// TC6 Control Command Header
///
/// Bit 31: DNC (Data-Not-Control) = 0 for control commands
/// Bit 30: WNR (Write-Not-Read) = 1 for write, 0 for read
/// Bit 29: AID (Address Increment Disable)
/// Bit 28: MMS (Memory Map Selector)
/// Bits 27-16: ADDR (Register Address)
/// Bits 15-8: LEN (Length in DWORDs)
/// Bits 7-1: Reserved
/// Bit 0: P (Parity bit - even parity over bits 31:1)
#[derive(Debug, Clone, Copy)]
struct Tc6ControlHeader(u32);

impl Tc6ControlHeader {
    /// Create a new control header for register read
    fn new_read(addr: u16) -> Self {
        let mut val = 0u32;
        // DNC = 0 (control command)
        // WNR = 0 (read)
        // ADDR
        val |= u32::from(addr) << 16;
        // LEN = 1 (one DWORD)
        val |= 1u32 << 8;

        // Calculate even parity over bits 31:1
        let parity = (val >> 1).count_ones() & 1;
        val |= parity;

        Self(val)
    }

    /// Create a new control header for register write
    fn new_write(addr: u16) -> Self {
        let mut val = 0u32;
        // DNC = 0 (control command)
        // WNR = 1 (write)
        val |= 1u32 << 30;
        // ADDR
        val |= u32::from(addr) << 16;
        // LEN = 1 (one DWORD)
        val |= 1u32 << 8;

        // Calculate even parity over bits 31:1
        let parity = (val >> 1).count_ones() & 1;
        val |= parity;

        Self(val)
    }

    fn to_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
}

/// TC6 Data Chunk Header
///
/// Bit 31: DNC (Data-Not-Control) = 1 for data chunks
/// Bit 30: SEQ (Sequence bit for even/odd indication)
/// Bit 29: NORX (No Receive - prevents MAC-PHY from sending RX data)
/// Bits 28-24: Reserved
/// Bits 23-16: DV (Data Valid - number of valid bytes in payload, 0 = no data)
/// Bits 15-11: SV (Start Valid - byte offset to frame start)
/// Bits 10-6: SWO (Start Word Offset)
/// Bits 5-1: EV (End Valid - byte offset after frame end)
/// Bit 0: EBO (End Byte Offset)
/// Bit 0: P (Parity bit - even parity over bits 31:1)
#[derive(Debug, Clone, Copy)]
struct Tc6DataHeader(u32);

impl Tc6DataHeader {
    /// Create a new data chunk header
    fn new(dv: u8, sv: u8, ev: u8) -> Self {
        let mut val = 0u32;
        // DNC = 1 (data chunk)
        val |= 1u32 << 31;
        // SEQ = 0 (we'll toggle this as needed)
        // NORX = 0 (we want to receive data)
        // DV (Data Valid bytes)
        val |= u32::from(dv) << 16;
        // SV (Start Valid)
        val |= u32::from(sv) << 11;
        // EV (End Valid)
        val |= u32::from(ev) << 1;

        // Calculate even parity over bits 31:1
        let parity = (val >> 1).count_ones() & 1;
        val |= parity;

        Self(val)
    }

    fn to_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
}

/// TC6 Data Footer (received from device)
///
/// Similar structure to header but received at end of RX data chunk
#[derive(Debug, Clone, Copy)]
struct Tc6DataFooter(u32);

impl Tc6DataFooter {
    fn from_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_be_bytes(bytes))
    }

    /// Extract number of valid data bytes in the received chunk
    fn data_valid(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    /// Extract start valid offset
    fn start_valid(&self) -> u8 {
        ((self.0 >> 11) & 0x1F) as u8
    }

    /// Extract end valid offset
    fn end_valid(&self) -> u8 {
        ((self.0 >> 1) & 0x1F) as u8
    }

    /// Check parity
    fn check_parity(&self) -> bool {
        let parity = (self.0 >> 1).count_ones() & 1;
        let expected_parity = self.0 & 1;
        parity == expected_parity
    }
}

impl<SPI> Tc6<SPI> {
    /// Create a new TC6 protocol handler
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }
}

impl<SPI: SpiDevice> Adin1110Protocol for Tc6<SPI> {
    type SpiError = SPI::Error;

    async fn read_reg(&mut self, addr: u16) -> Result<u32, AdinError<Self::SpiError>> {
        // Create control command header for read
        let header = Tc6ControlHeader::new_read(addr);
        let header_bytes = header.to_bytes();

        // Prepare buffers
        let mut rx_buf = [0u8; 4];

        // Perform SPI transaction: write header, read response
        let mut ops = [Operation::Write(&header_bytes), Operation::Read(&mut rx_buf)];

        self.spi.transaction(&mut ops).await.map_err(AdinError::Spi)?;

        // Parse response
        let value = u32::from_be_bytes(rx_buf);

        trace!("TC6 REG Read {} = {:08x}", addr, value);

        Ok(value)
    }

    async fn write_reg(&mut self, addr: u16, value: u32) -> Result<(), AdinError<Self::SpiError>> {
        // Create control command header for write
        let header = Tc6ControlHeader::new_write(addr);
        let header_bytes = header.to_bytes();
        let value_bytes = value.to_be_bytes();

        // Prepare write buffer
        let mut write_buf = [0u8; 8];
        write_buf[0..4].copy_from_slice(&header_bytes);
        write_buf[4..8].copy_from_slice(&value_bytes);

        trace!("TC6 REG Write {} = {:08x}", addr, value);

        self.spi.write(&write_buf).await.map_err(AdinError::Spi)
    }

    async fn read_fifo(&mut self, frame: &mut [u8]) -> Result<usize, AdinError<Self::SpiError>> {
        // Read the frame size from register
        let fifo_frame_size = self.read_reg(sr::RX_FSIZE.into()).await? as usize;

        if fifo_frame_size < ETH_MIN_LEN + FRAME_HEADER_LEN {
            return Err(AdinError::PACKET_TOO_SMALL);
        }

        let packet_size = fifo_frame_size - FRAME_HEADER_LEN - FCS_LEN;

        if packet_size > frame.len() {
            trace!("MAX: {} WANT: {}", frame.len(), packet_size);
            return Err(AdinError::PACKET_TOO_BIG);
        }

        // TC6 uses chunks: we need to read data in 64-byte chunks
        // For now, implement a basic version that reads the frame header + data

        // Read frame header (2 bytes)
        let mut frame_header = [0u8; 2];
        let mut bytes_read = 0;

        // Create data chunk header for reading
        let data_header = Tc6DataHeader::new(0, 0, 0); // Request to read data
        let header_bytes = data_header.to_bytes();

        // Read frame header first
        let mut chunk_buf = [0u8; TC6_CHUNK_SIZE];
        let mut ops = [Operation::Write(&header_bytes), Operation::Read(&mut chunk_buf)];

        self.spi.transaction(&mut ops).await.map_err(AdinError::Spi)?;

        // Extract footer (last 4 bytes of chunk)
        let footer_bytes: [u8; 4] = chunk_buf[TC6_CHUNK_PAYLOAD_SIZE..TC6_CHUNK_SIZE].try_into().unwrap();
        let footer = Tc6DataFooter::from_bytes(footer_bytes);

        if !footer.check_parity() {
            return Err(AdinError::SPI_CRC);
        }

        // Extract frame header from payload
        frame_header.copy_from_slice(&chunk_buf[0..2]);

        // Copy data to frame buffer
        let chunk_data_len = footer.data_valid() as usize;
        let to_copy = core::cmp::min(chunk_data_len.saturating_sub(2), packet_size);

        if to_copy > 0 {
            frame[0..to_copy].copy_from_slice(&chunk_buf[2..2 + to_copy]);
            bytes_read += to_copy;
        }

        // Read remaining chunks if needed
        while bytes_read < packet_size {
            let mut ops = [Operation::Write(&header_bytes), Operation::Read(&mut chunk_buf)];

            self.spi.transaction(&mut ops).await.map_err(AdinError::Spi)?;

            let footer_bytes: [u8; 4] = chunk_buf[TC6_CHUNK_PAYLOAD_SIZE..TC6_CHUNK_SIZE].try_into().unwrap();
            let footer = Tc6DataFooter::from_bytes(footer_bytes);

            if !footer.check_parity() {
                return Err(AdinError::SPI_CRC);
            }

            let chunk_data_len = footer.data_valid() as usize;
            let to_copy = core::cmp::min(chunk_data_len, packet_size - bytes_read);

            frame[bytes_read..bytes_read + to_copy].copy_from_slice(&chunk_buf[0..to_copy]);
            bytes_read += to_copy;

            if chunk_data_len < TC6_CHUNK_PAYLOAD_SIZE {
                break; // Last chunk
            }
        }

        // TODO: Verify FCS
        Ok(packet_size)
    }

    async fn write_fifo(&mut self, frame: &[u8]) -> Result<(), AdinError<Self::SpiError>> {
        if frame.len() < (6 + 6 + 2) {
            return Err(AdinError::PACKET_TOO_SMALL);
        }
        if frame.len() > (MAX_BUFF - FRAME_HEADER_LEN) {
            return Err(AdinError::PACKET_TOO_BIG);
        }

        // Calculate total size including frame header and FCS
        let total_size = frame.len() + FRAME_HEADER_LEN;
        let send_len = u32::try_from(total_size).map_err(|_| AdinError::PACKET_TOO_BIG)?;

        // Write TX frame size
        self.write_reg(sr::TX_FSIZE.into(), send_len).await?;

        // Prepare to send data in chunks
        let mut offset = 0;

        // First chunk includes frame header (2 bytes) + data
        let mut chunk_payload = [0u8; TC6_CHUNK_PAYLOAD_SIZE];

        // Add frame header (port ID)
        let port_header = u16::from(PORT_ID_BYTE).to_be_bytes();
        chunk_payload[0..2].copy_from_slice(&port_header);

        // Add frame data
        let first_chunk_data_len = core::cmp::min(frame.len(), TC6_CHUNK_PAYLOAD_SIZE - 2);
        chunk_payload[2..2 + first_chunk_data_len].copy_from_slice(&frame[0..first_chunk_data_len]);
        offset += first_chunk_data_len;

        // Calculate DV (data valid bytes in this chunk)
        let dv = (2 + first_chunk_data_len) as u8;
        let data_header = Tc6DataHeader::new(dv, 0, 0);
        let header_bytes = data_header.to_bytes();

        // Send first chunk
        let mut write_buf = [0u8; TC6_CHUNK_SIZE];
        write_buf[0..4].copy_from_slice(&header_bytes);
        write_buf[4..4 + TC6_CHUNK_PAYLOAD_SIZE].copy_from_slice(&chunk_payload);

        self.spi.write(&write_buf).await.map_err(AdinError::Spi)?;

        // Send remaining chunks
        while offset < frame.len() {
            let remaining = frame.len() - offset;
            let chunk_len = core::cmp::min(remaining, TC6_CHUNK_PAYLOAD_SIZE);

            let mut chunk_payload = [0u8; TC6_CHUNK_PAYLOAD_SIZE];
            chunk_payload[0..chunk_len].copy_from_slice(&frame[offset..offset + chunk_len]);

            let dv = chunk_len as u8;
            let data_header = Tc6DataHeader::new(dv, 0, 0);
            let header_bytes = data_header.to_bytes();

            let mut write_buf = [0u8; TC6_CHUNK_SIZE];
            write_buf[0..4].copy_from_slice(&header_bytes);
            write_buf[4..4 + TC6_CHUNK_PAYLOAD_SIZE].copy_from_slice(&chunk_payload);

            self.spi.write(&write_buf).await.map_err(AdinError::Spi)?;

            offset += chunk_len;
        }

        trace!(
            "TC6 TX: {} bytes in {} chunks",
            frame.len(),
            (frame.len() + TC6_CHUNK_PAYLOAD_SIZE - 1) / TC6_CHUNK_PAYLOAD_SIZE
        );

        Ok(())
    }
}
