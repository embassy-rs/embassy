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
#[repr(u8)]
pub enum CR {
    Response = 0x00,
    Command = CR,
}

impl From<u8> for CR {
    fn from(value: u8) -> Self {
        if (value & CR) == CR {
            return Self::Command;
        }
        Self::Response
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PF {
    Final = 0x00,
    Poll = PF,
}

impl From<u8> for PF {
    fn from(value: u8) -> Self {
        if (value & PF) == PF {
            return Self::Poll;
        }
        Self::Final
    }
}

fn read_ea_len(buf: &[u8]) -> (usize, usize) {
    let mut len = 0;
    let mut i = 0;
    for b in buf {
        len <<= 7;
        len |= (b >> 1) as usize;
        if (b & EA) == EA {
            break;
        }
        i += 1;
    }
    i += 1;

    (i, len)
}

fn read_ea(buf: &[u8]) -> &[u8] {
    let (i, len) = read_ea_len(buf);
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

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Information<'a> {
    /// DLC parameter negotiation (PN)
    ParameterNegotiation(ParameterNegotiation),
    /// Power Saving Control (PSC)
    PowerSavingControl,
    /// Multiplexer close down (CLD)
    MultiplexerCloseDown(MultiplexerCloseDown),
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

    fn wire_len(&self) -> usize {
        match self {
            Information::ParameterNegotiation(inner) => inner.wire_len(),
            Information::PowerSavingControl => todo!(),
            Information::MultiplexerCloseDown(inner) => inner.wire_len(),
            Information::TestCommand => todo!(),
            Information::FlowControlOnCommand(inner) => inner.wire_len(),
            Information::FlowControlOffCommand(inner) => inner.wire_len(),
            Information::ModemStatusCommand(inner) => inner.wire_len(),
            Information::NonSupportedCommandResponse(inner) => inner.wire_len(),
            Information::RemotePortNegotiationCommand => todo!(),
            Information::RemoteLineStatusCommand(inner) => inner.wire_len(),
            Information::ServiceNegotiationCommand => todo!(),
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
            Information::MultiplexerCloseDown(inner) => inner.write(writer).await,
            Information::TestCommand => todo!(),
            Information::ServiceNegotiationCommand => todo!(),
        }
    }

    pub fn parse(buf: &[u8]) -> Result<Self, Error> {
        let info_type = InformationType::from(buf[0]);
        let cr = CR::from(buf[0]);

        // get length
        let inner_data = read_ea(&buf[1..]);

        Ok(match info_type {
            InformationType::ParameterNegotiation => Self::ParameterNegotiation(ParameterNegotiation { cr }),
            InformationType::PowerSavingControl => Self::PowerSavingControl,
            InformationType::MultiplexerCloseDown => Self::MultiplexerCloseDown(MultiplexerCloseDown { cr }),
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
                    dlci: inner_data[0] >> 2,
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
                dlci: inner_data[0] >> 2,
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
    /// Disconnected mode (DM)
    Dm = 0x0F,
    /// Disconnect (DISC)
    Disc = 0x43,
    /// Unnumbered information with header check (UIH) command and response
    Uih = 0xEF,
    /// Unnumbered information (UI) command and response
    Ui = 0x03,
}

impl From<u8> for FrameType {
    fn from(value: u8) -> Self {
        match value & !PF {
            0x2F => Self::Sabm,
            0x63 => Self::Ua,
            0x0F => Self::Dm,
            0x43 => Self::Disc,
            0xEF => Self::Uih,
            0x03 => Self::Ui,
            n => panic!("Unknown frame type {:#02x}", n),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Read(embedded_io_async::ErrorKind),
    Write(embedded_io_async::ErrorKind),
    Crc,
    MalformedFrame,
}

pub trait Info {
    const INFORMATION_TYPE: InformationType;

    fn is_command(&self) -> bool;

    fn wire_len(&self) -> usize;

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error>;
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ParameterNegotiation {
    cr: CR,
}

impl Info for ParameterNegotiation {
    const INFORMATION_TYPE: InformationType = InformationType::ParameterNegotiation;

    fn is_command(&self) -> bool {
        self.cr == CR::Command
    }

    fn wire_len(&self) -> usize {
        todo!()
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        let buf = [0u8; 8];

        // TODO: Add Parameters!

        writer.write_all(&buf).await.map_err(|e| Error::Write(e.kind()))
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MultiplexerCloseDown {
    pub cr: CR,
}

impl Info for MultiplexerCloseDown {
    const INFORMATION_TYPE: InformationType = InformationType::MultiplexerCloseDown;

    fn is_command(&self) -> bool {
        self.cr == CR::Command
    }

    fn wire_len(&self) -> usize {
        1
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer
            .write_all(&[Self::INFORMATION_TYPE as u8 | self.cr as u8 | EA])
            .await
            .map_err(|e| Error::Write(e.kind()))
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlowControlOffCommand {
    cr: CR,
}

impl Info for FlowControlOffCommand {
    const INFORMATION_TYPE: InformationType = InformationType::FlowControlOffCommand;

    fn is_command(&self) -> bool {
        self.cr == CR::Command
    }

    fn wire_len(&self) -> usize {
        1
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer
            .write_all(&[Self::INFORMATION_TYPE as u8 | self.cr as u8 | EA])
            .await
            .map_err(|e| Error::Write(e.kind()))
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlowControlOnCommand {
    cr: CR,
}

impl Info for FlowControlOnCommand {
    const INFORMATION_TYPE: InformationType = InformationType::FlowControlOnCommand;

    fn is_command(&self) -> bool {
        self.cr == CR::Command
    }

    fn wire_len(&self) -> usize {
        1
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer
            .write_all(&[Self::INFORMATION_TYPE as u8 | self.cr as u8 | EA])
            .await
            .map_err(|e| Error::Write(e.kind()))
    }
}

#[derive(Debug)]
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

    fn wire_len(&self) -> usize {
        self.brk.map_or(4, |_| 5)
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        let len = self.wire_len() as u8 - 2;

        writer
            .write_all(&[
                Self::INFORMATION_TYPE as u8 | self.cr as u8 | EA,
                len << 1 | EA,
                self.dlci << 2 | CR | EA,
                self.control.with_ea(true).into_bits(),
            ])
            .await
            .map_err(|e| Error::Write(e.kind()))?;

        if let Some(brk) = self.brk {
            writer
                .write_all(&[brk.with_ea(true).into_bits()])
                .await
                .map_err(|e| Error::Write(e.kind()))?;
        }

        Ok(())
    }
}

#[derive(Debug)]
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

    fn wire_len(&self) -> usize {
        2
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

#[derive(Debug)]
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

    fn wire_len(&self) -> usize {
        3
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

/// Control signal octet
#[bitfield(u8, order = Lsb)]
#[derive(PartialEq, Eq)]
pub struct Control {
    /// The EA bit is set to 1 in the last octet of the sequence; in other
    /// octets EA is set to 0. If only one octet is transmitted EA is set to 1
    pub ea: bool,
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

#[cfg(feature = "defmt")]
impl defmt::Format for Control {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Control {{ ea: {}, fc: {}, rtc: {}, rtr: {}, ic: {}, dv: {} }}",
            self.ea(),
            self.fc(),
            self.rtc(),
            self.rtr(),
            self.ic(),
            self.dv(),
        )
    }
}

/// Break signal octet
#[bitfield(u8, order = Lsb)]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Break {
    /// The EA bit is set to 1 in the last octet of the sequence; in other
    /// octets EA is set to 0. If only one octet is transmitted EA is set to 1
    pub ea: bool,
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
    pub frame_type: FrameType,
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

        let mut header = [FLAG; 3];
        while header[0] == FLAG {
            Self::read_exact(reader, &mut header[..1]).await?;
        }
        Self::read_exact(reader, &mut header[1..]).await?;

        let id = header[0] >> 2;
        let frame_type = FrameType::from(header[1]);

        fcs.update(&header);

        let mut len = (header[2] >> 1) as usize;
        if (header[2] & EA) != EA {
            let mut l2 = [0u8; 1];
            Self::read_exact(reader, &mut l2).await?;
            fcs.update(&l2);
            len |= (l2[0] as usize) << 7;
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
        assert!(self.len <= 24);

        let mut buf = [0u8; 24];
        Self::read_exact(self.reader, &mut buf[..self.len]).await?;

        if self.frame_type == FrameType::Ui {
            self.fcs.update(&buf[..self.len]);
        }

        let info = Information::parse(&buf[..self.len])?;

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

            // FIXME: This should be re-written in a way that allows us to set channel flowcontrol if `w` cannot receive more bytes
            let n = w.write(&buf[..n]).await.map_err(|e| Error::Write(e.kind()))?;
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
            warn!("Discarding {} bytes of data in {:?}", n, self.frame_type);
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
    fn pf(&self) -> u8;

    fn id(&self) -> u8;

    fn information(&self) -> Option<&Information> {
        None
    }

    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), Error> {
        let information_len = self.information().map_or(0, |i| i.wire_len());

        trace!(
            "TxHeader {{ id: {}, frame_type: {:?}, len: {}}}",
            self.id(),
            Self::FRAME_TYPE,
            information_len
        );

        let fcs = if information_len < 128 {
            let header = [
                FLAG,
                self.id() << 2 | EA | self.cr(),
                Self::FRAME_TYPE as u8 | self.pf(),
                (information_len as u8) << 1 | EA,
            ];

            writer.write_all(&header).await.map_err(|e| Error::Write(e.kind()))?;

            0xFF - FCS.checksum(&header[1..])
        } else {
            let [b1, b2] = ((information_len as u16) << 1).to_le_bytes();

            let header = [
                FLAG,
                self.id() << 2 | EA | self.cr(),
                Self::FRAME_TYPE as u8 | self.pf(),
                b1,
                b2,
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

    fn pf(&self) -> u8 {
        PF::Poll as u8
    }

    fn id(&self) -> u8 {
        self.id
    }
}

/// Unnumbered information with header check (UIH) command and response
pub struct Uih<'d> {
    pub id: u8,
    pub information: Information<'d>,
}

impl<'d> Frame for Uih<'d> {
    const FRAME_TYPE: FrameType = FrameType::Uih;

    fn cr(&self) -> u8 {
        CR::Command as u8
    }

    fn id(&self) -> u8 {
        self.id
    }

    fn pf(&self) -> u8 {
        PF::Final as u8
    }

    fn information(&self) -> Option<&Information> {
        Some(&self.information)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_ea_test() {
        let tests = [
            (vec![EA], 0),
            (vec![0x01 << 1, 0xFE | EA], 255),
            (vec![0x02 << 1, 0xFE | EA], 255 + 128),
        ];

        // assert_eq!((0xFE | EA as usize) << 7 | (((0x01 << 1 & !EA) >> 1) as usize), 255);

        for (data, exp) in tests {
            let mut buf = [0u8; 1024];
            buf[..data.len()].copy_from_slice(data.as_slice());
            assert_eq!(read_ea(&buf).len(), exp);

            let header = ((exp as u16) << 1).to_le_bytes();

            let mut len = (header[0] >> 1) as usize;
            if (header[0] & EA) != EA {
                len |= (header[1] as usize) << 7;
            };

            assert_eq!(len, exp);
        }
    }

    #[cfg(test)]
    #[tokio::test]
    async fn msc() {
        let buf = &mut [0u8; 32];
        let mut w = &mut buf[..];

        ModemStatusCommand {
            cr: CR::Command,
            dlci: 2,
            control: Control::new(),
            brk: Some(Break::new()),
        }
        .write(&mut w)
        .await
        .unwrap();

        assert_eq!(&buf[..5], &[0xE3, 0x07, 2 << 2 | 0x03, 0x01, 0x01][..]);
    }

    #[cfg(test)]
    #[tokio::test]
    async fn data_frame() {
        let buf = &mut [0u8; 32];
        let mut w = &mut buf[..];

        let data = b"Hello";

        let frame = Uih {
            id: 2,
            information: Information::Data(data),
        };

        frame.write(&mut w).await.unwrap();

        assert_eq!(
            &buf[..4],
            &[0xF9, 2 << 2 | CR | EA, 0xEF, (data.len() as u8) << 1 | 1][..]
        );
        assert_eq!(&buf[4..4 + data.len()], data);
        assert_eq!(&buf[4 + data.len()..4 + data.len() + 2], &[0x5D, 0xF9][..]);
    }
}
