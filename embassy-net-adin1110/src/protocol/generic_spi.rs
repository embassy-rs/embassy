use embedded_hal_async::spi::{Operation, SpiDevice};
use heapless::Vec;

use super::Adin1110Protocol;
use crate::crc8::crc8;
use crate::crc32::ETH_FCS;
use crate::fmt::Bytes;
use crate::regs::{SpiHeader, SpiRegisters as sr};
use crate::{
    AdinError, DONT_CARE_BYTE, ETH_MIN_LEN, FCS_LEN, FRAME_HEADER_LEN, MAX_BUFF, PORT_ID_BYTE, TURN_AROUND_BYTE,
};

/// Packet minimal frame/packet length without `Frame Check Sequence` length
const ETH_MIN_WITHOUT_FCS_LEN: usize = ETH_MIN_LEN - FCS_LEN;

/// SPI Header, contains SPI action and register id.
const SPI_HEADER_LEN: usize = 2;
/// SPI Header CRC length
const SPI_HEADER_CRC_LEN: usize = 1;
/// SPI Header Turn Around length
const SPI_HEADER_TA_LEN: usize = 1;

/// Space for last bytes to create multipule 4 bytes on the end of a FIFO read/write.
const SPI_SPACE_MULTIPULE: usize = 3;

/// Generic SPI protocol implementation for ADIN1110
pub struct GenericSpi<SPI> {
    spi: SPI,
    crc_enabled: bool,
    append_fcs_on_tx: bool,
}

impl<SPI> GenericSpi<SPI> {
    /// Create a new `GenericSpi` protocol handler
    pub fn new(spi: SPI, crc_enabled: bool, append_fcs_on_tx: bool) -> Self {
        Self {
            spi,
            crc_enabled,
            append_fcs_on_tx,
        }
    }
}

impl<SPI: SpiDevice> Adin1110Protocol for GenericSpi<SPI> {
    type SpiError = SPI::Error;

    async fn read_reg(&mut self, addr: u16) -> Result<u32, AdinError<Self::SpiError>> {
        let mut tx_buf = Vec::<u8, 16>::new();

        let mut spi_hdr = SpiHeader(0);
        spi_hdr.set_control(true);
        spi_hdr.set_addr(sr::from(addr));
        let _ = tx_buf.extend_from_slice(spi_hdr.0.to_be_bytes().as_slice());

        if self.crc_enabled {
            // Add CRC for header data
            let _ = tx_buf.push(crc8(&tx_buf));
        }

        // Turn around byte, give the chip the time to access/setup the answer data.
        let _ = tx_buf.push(TURN_AROUND_BYTE);

        let mut rx_buf = [0; 5];

        let spi_read_len = if self.crc_enabled {
            rx_buf.len()
        } else {
            rx_buf.len() - 1
        };

        let mut spi_op = [Operation::Write(&tx_buf), Operation::Read(&mut rx_buf[0..spi_read_len])];

        self.spi.transaction(&mut spi_op).await.map_err(AdinError::Spi)?;

        if self.crc_enabled {
            let crc = crc8(&rx_buf[0..4]);
            if crc != rx_buf[4] {
                return Err(AdinError::SPI_CRC);
            }
        }

        let value = u32::from_be_bytes(rx_buf[0..4].try_into().unwrap());

        trace!("REG Read {} = {:08x} SPI {}", addr, value, Bytes(&tx_buf));

        Ok(value)
    }

    async fn write_reg(&mut self, addr: u16, value: u32) -> Result<(), AdinError<Self::SpiError>> {
        let mut tx_buf = Vec::<u8, 16>::new();

        let mut spi_hdr = SpiHeader(0);
        spi_hdr.set_control(true);
        spi_hdr.set_write(true);
        spi_hdr.set_addr(sr::from(addr));
        let _ = tx_buf.extend_from_slice(spi_hdr.0.to_be_bytes().as_slice());

        if self.crc_enabled {
            // Add CRC for header data
            let _ = tx_buf.push(crc8(&tx_buf));
        }

        let val = value.to_be_bytes();
        let _ = tx_buf.extend_from_slice(val.as_slice());

        if self.crc_enabled {
            // Add CRC for header data
            let _ = tx_buf.push(crc8(val.as_slice()));
        }

        trace!("REG Write {} = {:08x} SPI {}", addr, value, Bytes(&tx_buf));

        self.spi.write(&tx_buf).await.map_err(AdinError::Spi)
    }

    async fn read_fifo(&mut self, frame: &mut [u8]) -> Result<usize, AdinError<Self::SpiError>> {
        const HEAD_LEN: usize = SPI_HEADER_LEN + SPI_HEADER_CRC_LEN + SPI_HEADER_TA_LEN;
        const TAIL_LEN: usize = FCS_LEN + SPI_SPACE_MULTIPULE;

        let mut tx_buf = Vec::<u8, HEAD_LEN>::new();

        // Size of the frame, also includes the `frame header` and `FCS`.
        let fifo_frame_size = self.read_reg(sr::RX_FSIZE.into()).await? as usize;

        if fifo_frame_size < ETH_MIN_LEN + FRAME_HEADER_LEN {
            return Err(AdinError::PACKET_TOO_SMALL);
        }

        let packet_size = fifo_frame_size - FRAME_HEADER_LEN - FCS_LEN;

        if packet_size > frame.len() {
            trace!("MAX: {} WANT: {}", frame.len(), packet_size);
            return Err(AdinError::PACKET_TOO_BIG);
        }

        let mut spi_hdr = SpiHeader(0);
        spi_hdr.set_control(true);
        spi_hdr.set_addr(sr::RX);
        let _ = tx_buf.extend_from_slice(spi_hdr.0.to_be_bytes().as_slice());

        if self.crc_enabled {
            // Add CRC for header data
            let _ = tx_buf.push(crc8(&tx_buf));
        }

        // Turn around byte, TODO: Unknown that this is.
        let _ = tx_buf.push(TURN_AROUND_BYTE);

        let mut frame_header = [0, 0];
        let mut fcs_and_extra = [0; TAIL_LEN];

        // Packet read of write to the MAC packet buffer must be a multipul of 4!
        let tail_size = (fifo_frame_size & 0x03) + FCS_LEN;

        let mut spi_op = [
            Operation::Write(&tx_buf),
            Operation::Read(&mut frame_header),
            Operation::Read(&mut frame[0..packet_size]),
            Operation::Read(&mut fcs_and_extra[0..tail_size]),
        ];

        self.spi.transaction(&mut spi_op).await.map_err(AdinError::Spi)?;

        // According to register `CONFIG2`, bit 5 `CRC_APPEND` discription:
        // "Similarly, on receive, the CRC32 is forwarded with the frame to the host where the host must verify it is correct."
        // The application must allways check the FCS. It seems that the MAC/PHY has no option to handle this.
        let fcs_calc = ETH_FCS::new(&frame[0..packet_size]);

        if fcs_calc.hton_bytes() == fcs_and_extra[0..4] {
            Ok(packet_size)
        } else {
            Err(AdinError::FCS)
        }
    }

    async fn write_fifo(&mut self, frame: &[u8]) -> Result<(), AdinError<Self::SpiError>> {
        const HEAD_LEN: usize = SPI_HEADER_LEN + SPI_HEADER_CRC_LEN + FRAME_HEADER_LEN;
        const TAIL_LEN: usize = ETH_MIN_LEN - FCS_LEN + FCS_LEN + SPI_SPACE_MULTIPULE;

        if frame.len() < (6 + 6 + 2) {
            return Err(AdinError::PACKET_TOO_SMALL);
        }
        if frame.len() > (MAX_BUFF - FRAME_HEADER_LEN) {
            return Err(AdinError::PACKET_TOO_BIG);
        }

        // SPI HEADER + [OPTIONAL SPI CRC] + FRAME HEADER
        let mut head_data = Vec::<u8, HEAD_LEN>::new();
        // [OPTIONAL PAD DATA] + FCS + [OPTINAL BYTES MAKE SPI FRAME EVEN]
        let mut tail_data = Vec::<u8, TAIL_LEN>::new();

        let mut spi_hdr = SpiHeader(0);
        spi_hdr.set_control(true);
        spi_hdr.set_write(true);
        spi_hdr.set_addr(sr::TX);

        head_data
            .extend_from_slice(spi_hdr.0.to_be_bytes().as_slice())
            .map_err(|_e| AdinError::PACKET_TOO_BIG)?;

        if self.crc_enabled {
            // Add CRC for header data
            head_data
                .push(crc8(&head_data[0..2]))
                .map_err(|_| AdinError::PACKET_TOO_BIG)?;
        }

        // Add port number, ADIN1110 its fixed to zero/P1, but for ADIN2111 has two ports.
        head_data
            .extend_from_slice(u16::from(PORT_ID_BYTE).to_be_bytes().as_slice())
            .map_err(|_e| AdinError::PACKET_TOO_BIG)?;

        // ADIN1110 MAC and PHY don´t accept ethernet packet smaller than 64 bytes.
        // So padded the data minus the FCS, FCS is automatilly added to by the MAC.
        if frame.len() < ETH_MIN_WITHOUT_FCS_LEN {
            let _ = tail_data.resize(ETH_MIN_WITHOUT_FCS_LEN - frame.len(), 0x00);
        }

        // Append FCS by the application
        if self.append_fcs_on_tx {
            let mut frame_fcs = ETH_FCS::new(frame);

            if !tail_data.is_empty() {
                frame_fcs = frame_fcs.update(&tail_data);
            }

            let _ = tail_data.extend_from_slice(frame_fcs.hton_bytes().as_slice());
        }

        // len = frame_size + optional padding + 2 bytes Frame header
        let send_len_orig = frame.len() + tail_data.len() + FRAME_HEADER_LEN;

        let send_len = u32::try_from(send_len_orig).map_err(|_| AdinError::PACKET_TOO_BIG)?;

        // Packet read of write to the MAC packet buffer must be a multipul of 4 bytes!
        let pad_len = send_len_orig & 0x03;
        if pad_len != 0 {
            let spi_pad_len = 4 - pad_len + tail_data.len();
            let _ = tail_data.resize(spi_pad_len, DONT_CARE_BYTE);
        }

        self.write_reg(sr::TX_FSIZE.into(), send_len).await?;

        trace!(
            "TX: hdr {} [{}] {}-{}-{} SIZE: {}",
            head_data.len(),
            frame.len(),
            Bytes(head_data.as_slice()),
            Bytes(frame),
            Bytes(tail_data.as_slice()),
            send_len,
        );

        let mut transaction = [
            Operation::Write(head_data.as_slice()),
            Operation::Write(frame),
            Operation::Write(tail_data.as_slice()),
        ];

        self.spi.transaction(&mut transaction).await.map_err(AdinError::Spi)
    }
}
