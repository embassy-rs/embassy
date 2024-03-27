//! ### Frame types

use bitfield_struct::bitfield;
use crc::CRC_8_ROHC;
use embedded_io_async::Error as _;

const FLAG: u8 = 0xF9;
const EA: u8 = 0x01;
const CR: u8 = 0x02;
const PF: u8 = 0x10;

const FCS: crc::Crc<u8> = crc::Crc::<u8>::new(&CRC_8_ROHC);
const GOOD_FCS: u8 = 0xCF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CR {
    Command = 0x02,
    Response = 0x00,
}

impl From<u8> for CR {
    fn from(value: u8) -> Self {
        match (value & CR) == CR {
            false => Self::Command,
            _ => Self::Response,
        }
    }
}

fn read_ea(buf: &[u8]) -> &[u8] {
    let mut len = 0;
    let mut i = 0;
    for b in buf {
        len = (len << 7) | ((b & !EA) >> 1) as usize;
        if (b & EA) == EA {
            break;
        }
        i += 1;
    }
    i += 1;

    &buf[i..i + len]
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InformationType {
    /// DLC parameter negotiation (PN)
    ParameterNegotiation = 0x80,
    /// Power Saving Control (PSC)
    PowerSavingControl = 0x40,
    /// Multiplexer close down (CLD)
    MultiplexerCloseDown = 0xC0,
    /// Test Command (Test)
    TestCommand = 0x20,
    /// Flow Control On Command (FCon)
    FlowControlOnCommand = 0xA0,
    /// Flow Control Off Command (FCoff)
    FlowControlOffCommand = 0x60,
    /// Modem Status Command (MSC)
    ModemStatusCommand = 0xE0,
    /// Non Supported Command Response (NSC)
    NonSupportedCommandResponse = 0x10,
    /// Remote Port Negotiation Command (RPN)
    RemotePortNegotiationCommand = 0x90,
    /// Remote Line Status Command(RLS)
    RemoteLineStatusCommand = 0x50,
    /// Service Negotiation Command (SNC)
    ServiceNegotiationCommand = 0xD0,
}

impl InformationType {
    const fn max_data_len(&self) -> usize {
        match self {
            Self::ParameterNegotiation => 8,
            Self::ModemStatusCommand => 3,
            Self::NonSupportedCommandResponse => 1,
            Self::RemoteLineStatusCommand => 2,
            _ => 0,
        }
    }
}

impl From<u8> for InformationType {
    fn from(value: u8) -> Self {
        match value & !(CR | EA) {
            0x80 => Self::ParameterNegotiation,
            0x40 => Self::PowerSavingControl,
            0xC0 => Self::MultiplexerCloseDown,
            0x20 => Self::TestCommand,
            0xA0 => Self::FlowControlOnCommand,
            0x60 => Self::FlowControlOffCommand,
            0xE0 => Self::ModemStatusCommand,
            0x10 => Self::NonSupportedCommandResponse,
            0x90 => Self::RemotePortNegotiationCommand,
            0x50 => Self::RemoteLineStatusCommand,
            0xD0 => Self::ServiceNegotiationCommand,
            n => panic!("Unknown information type {:#02x}", n),
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Information<'a> {
    /// DLC parameter negotiation (PN)
    ParameterNegotiation(ParameterNegotiation),
    /// Power Saving Control (PSC)
    PowerSavingControl,
    /// Multiplexer close down (CLD)
    MultiplexerCloseDown,
    /// Test Command (Test)
    TestCommand,
    /// Flow Control On Command (FCon)
    FlowControlOnCommand(FlowControlOnCommand),
    /// Flow Control Off Command (FCoff)
    FlowControlOffCommand(FlowControlOffCommand),
    /// Modem Status Command (MSC)
    ModemStatusCommand(ModemStatusCommand),
    /// Non Supported Command Response (NSC)
    NonSupportedCommandResponse(NonSupportedCommandResponse),
    /// Remote Port Negotiation Command (RPN)
    RemotePortNegotiationCommand,
    /// Remote Line Status Command(RLS)
    RemoteLineStatusCommand(RemoteLineStatusCommand),
    /// Service Negotiation Command (SNC)
    ServiceNegotiationCommand,
    Data(&'a [u8]),
}

impl<'a> Information<'a> {
    pub fn is_command(&self) -> bool {
        match self {
            Information::ParameterNegotiation(i) => i.is_command(),
            Information::FlowControlOnCommand(i) => i.is_command(),
            Information::FlowControlOffCommand(i) => i.is_command(),
            Information::ModemStatusCommand(i) => i.is_command(),
            Information::NonSupportedCommandResponse(i) => i.is_command(),
            Information::RemoteLineStatusCommand(i) => i.is_command(),
            _ => true,
        }
    }

    const fn max_data_len(&self) -> usize {
        match self {
            Information::ParameterNegotiation(_) => InformationType::ParameterNegotiation.max_data_len(),
            Information::PowerSavingControl => InformationType::ParameterNegotiation.max_data_len(),
            Information::MultiplexerCloseDown => InformationType::ParameterNegotiation.max_data_len(),
            Information::TestCommand => InformationType::ParameterNegotiation.max_data_len(),
            Information::FlowControlOnCommand(_) => InformationType::ParameterNegotiation.max_data_len(),
            Information::FlowControlOffCommand(_) => InformationType::ParameterNegotiation.max_data_len(),
            Information::ModemStatusCommand(_) => InformationType::ParameterNegotiation.max_data_len(),
            Information::NonSupportedCommandResponse(_) => InformationType::ParameterNegotiation.max_data_len(),
            Information::RemotePortNegotiationCommand => InformationType::ParameterNegotiation.max_data_len(),
            Information::RemoteLineStatusCommand(_) => InformationType::ParameterNegotiation.max_data_len(),
            Information::ServiceNegotiationCommand => InformationType::ParameterNegotiation.max_data_len(),
            Information::Data(d) => d.len(),
        }
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        match self {
            Information::ParameterNegotiation(inner) => inner.write(writer).await,
            Information::FlowControlOnCommand(inner) => inner.write(writer).await,
            Information::FlowControlOffCommand(inner) => inner.write(writer).await,
            Information::ModemStatusCommand(inner) => inner.write(writer).await,
            Information::NonSupportedCommandResponse(inner) => inner.write(writer).await,
            Information::RemoteLineStatusCommand(inner) => inner.write(writer).await,
            Information::Data(d) => writer.write_all(d).await.map_err(|e| Error::Write(e.kind())),
            Information::RemotePortNegotiationCommand => todo!(),
            Information::PowerSavingControl => todo!(),
            Information::MultiplexerCloseDown => todo!(),
            Information::TestCommand => todo!(),
            Information::ServiceNegotiationCommand => todo!(),
        }
    }

    pub fn parse(buf: &[u8]) -> Result<Self, Error> {
        let info_type = InformationType::from(buf[0]);
        let cr = CR::from(buf[0]);

        // get length
        let inner_data = read_ea(&buf[1..]);
        if inner_data.len() > info_type.max_data_len() {
            return Err(Error::MalformedFrame);
        }

        Ok(match info_type {
            InformationType::ParameterNegotiation => Self::ParameterNegotiation(ParameterNegotiation { cr }),
            InformationType::PowerSavingControl => Self::PowerSavingControl,
            InformationType::MultiplexerCloseDown => Self::MultiplexerCloseDown,
            InformationType::TestCommand => Self::TestCommand,
            InformationType::FlowControlOnCommand => Self::FlowControlOnCommand(FlowControlOnCommand { cr }),
            InformationType::FlowControlOffCommand => Self::FlowControlOffCommand(FlowControlOffCommand { cr }),
            InformationType::ModemStatusCommand => {
                let brk = if inner_data.len() == 3 {
                    Some(Break::from_bits(inner_data[2]))
                } else {
                    None
                };
                Self::ModemStatusCommand(ModemStatusCommand {
                    cr,
                    dlci: (inner_data[0] & !(EA | CR)) >> 2,
                    control: Control::from_bits(inner_data[1]),
                    brk,
                })
            }
            InformationType::NonSupportedCommandResponse => {
                Self::NonSupportedCommandResponse(NonSupportedCommandResponse {
                    cr,
                    command_type: InformationType::from(inner_data[0]),
                })
            }
            InformationType::RemotePortNegotiationCommand => Self::RemotePortNegotiationCommand,
            InformationType::RemoteLineStatusCommand => Self::RemoteLineStatusCommand(RemoteLineStatusCommand {
                cr,
                dlci: (inner_data[0] & !(EA | CR)) >> 2,
                remote_line_status: RemoteLineStatus::from(inner_data[1]),
            }),
            InformationType::ServiceNegotiationCommand => Self::ServiceNegotiationCommand,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FrameType {
    /// Set Asynchronous Balanced Mode (SABM) command
    Sabm = 0x2F,
    /// Unnumbered Acknowledgement (UA) response
    Ua = 0x63,
    Disc = 0x43,
    Dm = 0x0F,
    /// Unnumbered information (UI) command and response
    Ui = 0x03,
    /// Unnumbered information with header check (UIH) command and response
    Uih = 0xEF,
}

impl From<u8> for FrameType {
    fn from(value: u8) -> Self {
        match value & !PF {
            0x2F => Self::Sabm,
            0x63 => Self::Ua,
            0x43 => Self::Disc,
            0x0F => Self::Dm,
            0x03 => Self::Ui,
            0xEF => Self::Uih,
            n => panic!("Unknown frame type {:#02x}", n),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Read(embedded_io_async::ErrorKind),
    Write(embedded_io_async::ErrorKind),
    Crc,
    UnexpectedFrameType,
    MalformedFrame,
}

pub trait Info {
    const INFORMATION_TYPE: InformationType;

    fn is_command(&self) -> bool;

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error>;
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ParameterNegotiation {
    cr: CR,
}

impl Info for ParameterNegotiation {
    const INFORMATION_TYPE: InformationType = InformationType::ParameterNegotiation;

    fn is_command(&self) -> bool {
        self.cr == CR::Command
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        let buf = [0u8; Self::INFORMATION_TYPE.max_data_len()];

        // TODO: Add Parameters!

        writer.write_all(&buf).await.map_err(|e| Error::Write(e.kind()))
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlowControlOffCommand {
    cr: CR,
}

impl Info for FlowControlOffCommand {
    const INFORMATION_TYPE: InformationType = InformationType::FlowControlOffCommand;

    fn is_command(&self) -> bool {
        self.cr == CR::Command
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer
            .write_all(&[Self::INFORMATION_TYPE as u8 | self.cr as u8 | EA])
            .await
            .map_err(|e| Error::Write(e.kind()))
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlowControlOnCommand {
    cr: CR,
}

impl Info for FlowControlOnCommand {
    const INFORMATION_TYPE: InformationType = InformationType::FlowControlOnCommand;

    fn is_command(&self) -> bool {
        self.cr == CR::Command
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer
            .write_all(&[Self::INFORMATION_TYPE as u8 | self.cr as u8 | EA])
            .await
            .map_err(|e| Error::Write(e.kind()))
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModemStatusCommand {
    pub cr: CR,
    pub dlci: u8,
    pub control: Control,
    pub brk: Option<Break>,
}

impl Info for ModemStatusCommand {
    const INFORMATION_TYPE: InformationType = InformationType::ModemStatusCommand;

    fn is_command(&self) -> bool {
        self.cr == CR::Command
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer
            .write_all(&[
                Self::INFORMATION_TYPE as u8 | self.cr as u8 | EA,
                self.dlci << 2 | CR | EA,
                self.control.into_bits(),
            ])
            .await
            .map_err(|e| Error::Write(e.kind()))?;

        if let Some(brk) = self.brk {
            writer
                .write_all(&[brk.into_bits()])
                .await
                .map_err(|e| Error::Write(e.kind()))?;
        }

        Ok(())
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NonSupportedCommandResponse {
    pub cr: CR,
    pub command_type: InformationType,
}

impl Info for NonSupportedCommandResponse {
    const INFORMATION_TYPE: InformationType = InformationType::NonSupportedCommandResponse;

    fn is_command(&self) -> bool {
        self.cr == CR::Command
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer
            .write_all(&[
                Self::INFORMATION_TYPE as u8 | self.cr as u8 | EA,
                self.command_type as u8 | self.cr as u8 | EA,
            ])
            .await
            .map_err(|e| Error::Write(e.kind()))
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RemoteLineStatusCommand {
    pub cr: CR,
    pub dlci: u8,
    pub remote_line_status: RemoteLineStatus,
}

impl Info for RemoteLineStatusCommand {
    const INFORMATION_TYPE: InformationType = InformationType::RemoteLineStatusCommand;

    fn is_command(&self) -> bool {
        self.cr == CR::Command
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer
            .write_all(&[
                Self::INFORMATION_TYPE as u8 | self.cr as u8 | EA,
                self.dlci << 2 | CR | EA,
                self.remote_line_status.into_bits(),
            ])
            .await
            .map_err(|e| Error::Write(e.kind()))
    }
}

//   val   bit NAME   RX         TX
// 0x0001   0  FC     -          -
// 0x0002   1  RTC    107.DSR    108/2.DTR
// 0x0004   2  RTR    106.CTS    133.RFR / 105.RTS
// 0x0008   3  RFU1   -          -
// 0x0010   4  RFU2   -          -
// 0x0020   5  IC     125.RI     always 0
// 0x0040   6  DV     109.DCD    always 1
// 0x0080   7  B1     1 = signal break
// 0x0100   8  B2     reserved, always 0
// 0x0200   9  B3     reserved, always 0
// 0x0400  10  L1     |
// 0x0800  11  L2     | break length
// 0x1000  12  L3     | units of 200ms
// 0x2000  13  L4     |

/// Control signal octet
#[bitfield(u8, order = Lsb)]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Control {
    /// The EA bit is set to 1 in the last octet of the sequence; in other
    /// octets EA is set to 0. If only one octet is transmitted EA is set to 1
    ea: bool,
    /// Flow Control (FC). The bit is set to 1(one) when the device is unable to
    /// accept frames
    pub fc: bool,
    /// Ready To Communicate (RTC). The bit is set to 1 when the device is ready
    /// to communicate
    pub rtc: bool,
    /// Ready To Receive (RTR). The bit is set to 1 when the device is ready to
    /// receive data
    pub rtr: bool,
    /// Reserved for future use. Set to zero by the sender, ignored by the
    /// receiver
    #[bits(2, access = None)]
    reserved: u8,
    /// Incoming call indicator (IC). The bit is set to 1 to indicate an
    /// incoming call.
    pub ic: bool,
    /// Data Valid (DV). The bit is set to 1 to indicate that valid data is
    /// being sent
    pub dv: bool,
}

/// Break signal octet
#[bitfield(u8, order = Lsb)]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Break {
    /// The EA bit is set to 1 in the last octet of the sequence; in other
    /// octets EA is set to 0. If only one octet is transmitted EA is set to 1
    ea: bool,
    pub brk: bool,
    #[bits(2, access = None)]
    b2: u8,
    /// Length of break in units of 200ms
    #[bits(4)]
    pub len: u8,
}

/// Remote Line Status Octets
#[bitfield(u8, order = Lsb)]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RemoteLineStatus {
    #[bits(4)]
    pub l: u8,
    /// The res bits are set to zero for the sender and ignored by the reciever.
    #[bits(4, access = None)]
    reserved: u8,
}

pub(crate) struct RxHeader<'a, R: embedded_io_async::BufRead> {
    id: u8,
    frame_type: FrameType,
    pub len: usize,
    fcs: crc::Digest<'a, u8>,
    reader: &'a mut R,
}

#[cfg(feature = "defmt")]
impl<'a, R: embedded_io_async::BufRead> defmt::Format for RxHeader<'a, R> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "RxHeader {{ id: {}, frame_type: {:?}, len: {}}}",
            self.id,
            self.frame_type,
            self.len,
        )
    }
}

impl<'a, R: embedded_io_async::BufRead> core::fmt::Debug for RxHeader<'a, R> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        write!(
            fmt,
            "RxHeader {{ id: {}, frame_type: {:?}, len: {}}}",
            self.id, self.frame_type, self.len
        )
    }
}

impl<'a, R: embedded_io_async::BufRead> RxHeader<'a, R> {
    pub(crate) async fn read(reader: &'a mut R) -> Result<Self, Error> {
        let mut fcs = FCS.digest();

        let mut header = [FLAG; 4];
        while header[0] == FLAG {
            Self::read_exact(reader, &mut header[..1]).await?;
        }
        Self::read_exact(reader, &mut header[1..3]).await?;

        let id = (header[0] & !(EA | CR)) >> 2;

        let frame_type = FrameType::from(header[1]);

        let len = if (header[2] & EA) == EA {
            fcs.update(&header[..3]);
            ((header[2] & !EA) >> 1) as usize
        } else {
            Self::read_exact(reader, &mut header[3..4]).await?;
            fcs.update(&header[..4]);
            (header[3] as usize) << 7 | ((header[2] & !EA) >> 1) as usize
        };

        Ok(Self {
            id,
            frame_type,
            len,
            reader,
            fcs,
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

    pub(crate) async fn read_information<'d>(mut self) -> Result<Information<'d>, Error> {
        let fcs_over_data = match self.frame_type {
            FrameType::Ui => true,
            FrameType::Uih => false,
            _ => return Err(Error::UnexpectedFrameType),
        };

        assert!(self.len <= 24);

        let mut buf = [0u8; 24];
        Self::read_exact(self.reader, &mut buf[..self.len]).await?;

        if fcs_over_data {
            self.fcs.update(&buf[..self.len]);
        }

        let info = Information::parse(&buf[..self.len]).unwrap();

        // Make sure we cannot call this twice, or call `copy`, to over-read data
        self.len = 0;

        self.finalize().await?;

        Ok(info)
    }

    pub(crate) async fn copy<W: embedded_io_async::Write>(mut self, w: &mut W) -> Result<(), Error> {
        while self.len != 0 {
            let buf = self.reader.fill_buf().await.map_err(|e| Error::Read(e.kind()))?;
            if buf.is_empty() {
                panic!("EOF");
            }
            let n = buf.len().min(self.len);
            w.write_all(&buf[..n]).await.map_err(|e| Error::Write(e.kind()))?;
            self.reader.consume(n);
            self.len -= n;
        }
        w.flush().await.map_err(|e| Error::Write(e.kind()))?;
        self.finalize().await?;

        Ok(())
    }

    pub async fn finalize(mut self) -> Result<(), Error> {
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

        self.fcs.update(&[trailer[0]]);
        let expected_fcs = self.fcs.finalize();

        if trailer[1] != FLAG {
            error!("Malformed packet! Expected {:#02x} but got {:#02x}", FLAG, trailer[1]);
            return Err(Error::MalformedFrame);
        }

        if expected_fcs != GOOD_FCS {
            error!("bad crc! {:#02x} != {:#02x}", expected_fcs, GOOD_FCS);
            return Err(Error::Crc);
        }

        Ok(())
    }
}

pub trait Frame {
    const FRAME_TYPE: FrameType;

    fn cr(&self) -> u8;
    fn id(&self) -> u8;

    fn information(&self) -> Option<&Information> {
        None
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        let information_len = if let Some(info) = self.information() {
            info.max_data_len()
        } else {
            0
        };

        let fcs = if information_len > 127 {
            let [b1, b2] = ((information_len as u16) << 1).to_le_bytes();

            let header = [
                FLAG,
                self.id() << 2 | EA | self.cr(),
                Self::FRAME_TYPE as u8 | PF,
                b1,
                b2,
            ];

            writer.write_all(&header).await.map_err(|e| Error::Write(e.kind()))?;

            0xFF - FCS.checksum(&header[1..])
        } else {
            let header = [
                FLAG,
                self.id() << 2 | EA | self.cr(),
                Self::FRAME_TYPE as u8 | PF,
                (information_len as u8) << 1 | EA,
            ];
            writer.write_all(&header).await.map_err(|e| Error::Write(e.kind()))?;

            0xFF - FCS.checksum(&header[1..])
        };

        if let Some(info) = self.information() {
            info.write(writer).await?;
        }

        writer
            .write_all(&[fcs, FLAG])
            .await
            .map_err(|e| Error::Write(e.kind()))?;

        writer.flush().await.map_err(|e| Error::Write(e.kind()))?;

        Ok(())
    }
}

/// Set Asynchronous Balanced Mode (SABM) command
pub struct Sabm {
    pub id: u8,
}

impl Frame for Sabm {
    const FRAME_TYPE: FrameType = FrameType::Sabm;

    fn cr(&self) -> u8 {
        CR::Command as u8
    }

    fn id(&self) -> u8 {
        self.id
    }
}

/// Unnumbered information with header check (UIH) command and response
pub struct Uih<'d> {
    pub cr: CR,
    pub id: u8,
    pub information: Information<'d>,
}

impl<'d> Frame for Uih<'d> {
    const FRAME_TYPE: FrameType = FrameType::Uih;

    fn cr(&self) -> u8 {
        self.cr as u8
    }

    fn id(&self) -> u8 {
        self.id
    }

    fn information(&self) -> Option<&Information> {
        Some(&self.information)
    }
}

#[test]
fn parse_out() {
    Information::parse(&[
        0x41, 0x54, 0x44, 0x2a, 0x39, 0x39, 0x2a, 0x2a, 0x2a, 0x31, 0x23, 0x0d, 0x0a,
    ])
    .unwrap();
}
