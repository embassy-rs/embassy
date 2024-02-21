//! ### Frame types

use crc::CRC_8_ROHC;
use embedded_io_async::Error as _;

const FLAG: u8 = 0xF9;
const EA: u8 = 0x01;
const CR_MASK: u8 = 0x02;

const FCS: crc::Crc<u8> = crc::Crc::<u8>::new(&CRC_8_ROHC);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CR {
    Command = 0x00,
    Response = 0x02,
}

impl From<u8> for CR {
    fn from(value: u8) -> Self {
        match (value >> CR_MASK) & 0x01 {
            0 => Self::Command,
            _ => Self::Response,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InformationType {
    /// DLC parameter negotiation (PN)
    ParameterNegotiation = 0x83,
    /// Power Saving Control (PSC)
    PowerSavingControl = 0x43,
    /// Multiplexer close down (CLD)
    MultiplexerCloseDown = 0xC3,
    /// Test Command (Test)
    TestCommand = 0x23,
    /// Flow Control On Command (FCon)
    FlowControlOnCommand = 0xA3,
    /// Modem Status Command (MSC)
    ModemStatusCommand = 0xE3,
    /// Non Supported Command Response (NSC)
    NonSupportedCommandResponse = 0x13,
    /// Remote Port Negotiation Command (RPN)
    RemotePortNegotiationCommand = 0x93,
    /// Remote Line Status Command(RLS)
    RemoteLineStatusCommand = 0x53,
    /// Service Negotiation Command (SNC)
    ServiceNegotiationCommand = 0xD3,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Read(embedded_io_async::ErrorKind),
    Write(embedded_io_async::ErrorKind),
    Crc,
    BufferSize,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Information<'a> {
    typ_byte: u8,
    pub len: usize,
    pub data: &'a [u8],
}

impl<'a> Information<'a> {
    pub fn typ(&self) -> InformationType {
        match self.typ_byte >> 2 {
            0x20 => InformationType::ParameterNegotiation,
            0x10 => InformationType::PowerSavingControl,
            0x30 => InformationType::MultiplexerCloseDown,
            0x08 => InformationType::TestCommand,
            0x28 => InformationType::FlowControlOnCommand,
            0x38 => InformationType::ModemStatusCommand,
            0x04 => InformationType::NonSupportedCommandResponse,
            0x24 => InformationType::RemotePortNegotiationCommand,
            0x14 => InformationType::RemoteLineStatusCommand,
            0x34 => InformationType::ServiceNegotiationCommand,
            n => panic!("Unknown information type {:08b}", n),
        }
    }

    pub fn is_command(&self) -> bool {
        (self.typ_byte & 0x02) == 0x02
    }
}

pub(crate) struct RxHeader<'a, R: embedded_io_async::BufRead> {
    id: u8,
    pub control: u8,
    pub len: usize,
    expected_fcs: u8,
    reader: &'a mut R,
}

#[cfg(feature = "defmt")]
impl<'a, R: embedded_io_async::BufRead> defmt::Format for RxHeader<'a, R> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "RxHeader {{ id: {}, control: 0x{:02x}, len: {}, fcs: 0x{:02x}}}",
            self.id,
            self.control,
            self.len,
            self.expected_fcs
        )
    }
}

impl<'a, R: embedded_io_async::BufRead> RxHeader<'a, R> {
    pub(crate) async fn read(reader: &'a mut R) -> Result<Self, Error> {
        let mut header = [0xf9; 4];
        while header[0] == 0xf9 {
            Self::read_exact(reader, &mut header[..1]).await?;
        }
        Self::read_exact(reader, &mut header[1..3]).await?;

        let id = (header[0] & 0xFC) >> 2;

        let control = header[1];

        let len = if (header[2] & EA) != EA {
            Self::read_exact(reader, &mut header[3..4]).await?;
            (header[3] as usize) << 7 | ((header[2] & 0xFE) >> 1) as usize
        } else {
            ((header[2] & 0xFE) >> 1) as usize
        };

        Ok(Self {
            id,
            control,
            len,
            reader,
            expected_fcs: 0xFF - FCS.checksum(&header[0..3]),
        })
    }

    pub(crate) fn is_control(&self) -> bool {
        self.id == 0
    }

    pub(crate) fn id(&self) -> u8 {
        self.id
    }

    async fn read_exact(r: &mut R, mut data: &mut [u8]) -> Result<(), Error> {
        while !data.is_empty() {
            let buf = r.fill_buf().await.map_err(|e| Error::Read(e.kind()))?;
            if buf.is_empty() {
                panic!("EOF");
            }
            let n = buf.len().min(data.len());
            data[..n].copy_from_slice(&buf[..n]);
            data = &mut data[n..];
            r.consume(n);
        }
        Ok(())
    }

    pub(crate) async fn read_information<'d>(&mut self, buf: &'d mut [u8]) -> Result<Information<'d>, Error> {
        if self.len > buf.len() {
            return Err(Error::BufferSize);
        }

        Self::read_exact(self.reader, &mut buf[..self.len]).await?;
        self.len = 0;

        let typ_byte = buf[0];

        // check length
        let mut i = 1;
        let mut inner_len = 0;
        for b in &buf[i..] {
            inner_len = (inner_len << 7) + ((b & 0x7F) >> 1) as usize;
            if b & EA == EA {
                break;
            }
            i += 1;
        }
        i += 1;

        self.finalize().await?;

        Ok(Information {
            typ_byte,
            len: inner_len,
            data: &buf[i..],
        })
    }

    pub(crate) async fn copy<W: embedded_io_async::Write>(&mut self, w: &mut W) -> Result<(), Error> {
        while self.len != 0 {
            let buf = self.reader.fill_buf().await.map_err(|e| Error::Read(e.kind()))?;
            if buf.is_empty() {
                panic!("EOF");
            }
            let n = buf.len().min(self.len);
            let n = w.write(&buf[..n]).await.map_err(|e| Error::Write(e.kind()))?;
            if n == 0 {
                panic!("Write zero!");
            }
            self.reader.consume(n);
            self.len -= n;
        }
        self.finalize().await?;

        Ok(())
    }

    pub async fn finalize(&mut self) -> Result<(), Error> {
        while self.len > 0 {
            // Discard any information here
            let buf = self.reader.fill_buf().await.map_err(|e| Error::Read(e.kind()))?;
            if buf.is_empty() {
                panic!("EOF");
            }
            let n = buf.len().min(self.len);
            self.reader.consume(n);
            self.len -= n;
        }

        let mut trailer = [0; 2];
        Self::read_exact(&mut self.reader, &mut trailer).await?;

        if trailer != [self.expected_fcs, FLAG] {
            warn!("bad crc");
            return Err(Error::Crc);
        }

        Ok(())
    }
}

pub trait Frame {
    const CONTROL: u8;

    fn id(&self) -> u8;
    fn information(&self) -> &[u8] {
        &[]
    }

    fn cr(&self) -> CR {
        CR::Command
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let len = self.information().len().min(32767);

        let fcs = if len > 127 {
            let [b1, b2] = ((len as u16) << 1).to_be_bytes();

            let header = [FLAG, self.id() << 2 | EA | self.cr() as u8, Self::CONTROL, b1, b2];
            writer.write_all(&header).await.map_err(|e| Error::Write(e.kind()))?;
            0xFF - FCS.checksum(&header[1..])
        } else {
            let header = [
                FLAG,
                self.id() << 2 | EA | self.cr() as u8,
                Self::CONTROL,
                (len as u8) << 1 | EA,
            ];
            writer.write_all(&header).await.map_err(|e| Error::Write(e.kind()))?;
            0xFF - FCS.checksum(&header[1..])
        };

        writer
            .write_all(&self.information()[..len])
            .await
            .map_err(|e| Error::Write(e.kind()))?;
        writer
            .write_all(&[fcs, FLAG])
            .await
            .map_err(|e| Error::Write(e.kind()))?;
        Ok(len)
    }
}

/// Set Asynchronous Balanced Mode (SABM) command
pub struct Sabm {
    pub id: u8,
}

impl Frame for Sabm {
    const CONTROL: u8 = 0x3F;

    fn id(&self) -> u8 {
        self.id
    }
}

/// Unnumbered information with header check (UIH) command and response
pub struct Uih<'a> {
    pub id: u8,
    pub information: &'a [u8],
    pub cr: CR,
}

impl<'a> Frame for Uih<'a> {
    const CONTROL: u8 = 0xEF;

    fn cr(&self) -> CR {
        self.cr
    }

    fn id(&self) -> u8 {
        self.id
    }

    fn information(&self) -> &[u8] {
        self.information
    }
}
