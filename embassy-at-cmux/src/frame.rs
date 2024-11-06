//! CMUX Frame Encoding and Decoding

use bitfield_struct::bitfield;
use crc::CRC_8_ROHC;
use embedded_io_async::{BufRead, Error as IoError, ErrorKind as IoErrorKind, Write};

// --- Constants ---
const FLAG: u8 = 0xF9;
const EA: u8 = 0x01;
const CR: u8 = 0x02;
const PF: u8 = 0x10;
const FCS_GENERATOR: crc::Crc<u8> = crc::Crc::<u8>::new(&CRC_8_ROHC);
const GOOD_FCS: u8 = 0xCF;
// Maximum size of an Information field.
const MAX_INFO_LEN: usize = 254;

// --- Error Handling ---
#[derive(Debug, PartialEq)]
pub enum CmuxError {
    Io(IoErrorKind),
    CrcMismatch,
    InvalidFrameFormat,
    UnsupportedOperation, // For unimplemented features
    InvalidInformationField,
}

impl<E: IoError> From<E> for CmuxError {
    fn from(err: E) -> Self {
        CmuxError::Io(err.kind())
    }
}

// --- Helper Functions ---
async fn write_ea_length<W: Write>(writer: &mut W, length: usize) -> Result<(), CmuxError> {
    let mut len = length;
    let mut bytes = [0u8; 4]; // Max EA length can fit in 4 bytes
    let mut i = 3;

    while len > 0 {
        bytes[i] = ((len as u8 & 0x7F) << 1) | EA;
        len >>= 7;
        i -= 1;
    }

    Ok(writer.write_all(&bytes[i + 1..]).await?)
}

async fn read_ea_length<R: BufRead>(reader: &mut R) -> Result<usize, CmuxError> {
    let mut len = 0;
    let mut shift = 0;

    loop {
        let byte = reader.read_u8().await.map_err(|e| CmuxError::Io(e.kind()))?;

        len |= ((byte >> 1) as usize) << shift;
        shift += 7;

        if byte & EA == EA {
            break;
        }
    }
    Ok(len)
}

// --- CMUX Frame Components ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum CommandResponse {
    Response = 0x00,
    Command = CR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PollFinal {
    Final = 0x00,
    Poll = PF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InformationType {
    ParameterNegotiation = 0x80,
    // ... other InformationType variants ...
    PowerSavingControl = 0x40,
    MultiplexerCloseDown = 0xC0,
    TestCommand = 0x20,
    FlowControlOnCommand = 0xA0,
    FlowControlOffCommand = 0x60,
    ModemStatusCommand = 0xE0,
    NonSupportedCommandResponse = 0x10,
    RemotePortNegotiationCommand = 0x90,
    RemoteLineStatusCommand = 0x50,
    ServiceNegotiationCommand = 0xD0,
}

impl TryFrom<u8> for InformationType {
    type Error = CmuxError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & !(CR | EA) {
            0x80 => Ok(Self::ParameterNegotiation),
            // ... other InformationType matches ...
            0x40 => Ok(Self::PowerSavingControl),
            0xC0 => Ok(Self::MultiplexerCloseDown),
            0x20 => Ok(Self::TestCommand),
            0xA0 => Ok(Self::FlowControlOnCommand),
            0x60 => Ok(Self::FlowControlOffCommand),
            0xE0 => Ok(Self::ModemStatusCommand),
            0x10 => Ok(Self::NonSupportedCommandResponse),
            0x90 => Ok(Self::RemotePortNegotiationCommand),
            0x50 => Ok(Self::RemoteLineStatusCommand),
            0xD0 => Ok(Self::ServiceNegotiationCommand),
            _ => Err(CmuxError::InvalidFrameFormat),
        }
    }
}

// --- Information Field Implementations ---

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ParameterNegotiation {
    pub cr: CommandResponse,
    // ... PN-specific fields ...
}

impl ParameterNegotiation {
    pub fn new(cr: CommandResponse) -> Self {
        Self { cr }
    }
}

impl InformationField for ParameterNegotiation {
    fn information_type(&self) -> InformationType {
        InformationType::ParameterNegotiation
    }

    async fn write<W: Write>(&self, writer: &mut W) -> Result<(), CmuxError> {
        // Write PN-specific data here
        Ok(())
    }

    async fn read<R: BufRead>(reader: &mut R, len: usize) -> Result<Self, CmuxError> {
        if len != 0 {
            return Err(CmuxError::InvalidInformationField);
        }
        // Read and parse PN-specific data from 'reader' here
        Ok(Self {
            cr: CommandResponse::Command,
            // ... parsed fields ...
        })
    }

    fn wire_len(&self) -> usize {
        // Return the length of the serialized PN data
        0
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
    /// The res bits are set to zero for the sender and ignored by the receiver.
    #[bits(4, access = None)]
    reserved: u8,
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MultiplexerCloseDown {
    pub cr: CommandResponse,
}

impl MultiplexerCloseDown {
    pub fn new(cr: CommandResponse) -> Self {
        Self { cr }
    }
}

impl InformationField for MultiplexerCloseDown {
    fn information_type(&self) -> InformationType {
        InformationType::MultiplexerCloseDown
    }

    async fn write<W: Write>(&self, writer: &mut W) -> Result<(), CmuxError> {
        // Write CLD-specific data here
        Ok(())
    }

    async fn read<R: BufRead>(reader: &mut R, len: usize) -> Result<Self, CmuxError> {
        if len != 0 {
            return Err(CmuxError::InvalidInformationField);
        }
        Ok(Self {
            cr: CommandResponse::Command,
            // ... parsed fields ...
        })
    }

    fn wire_len(&self) -> usize {
        // Return the length of the serialized CLD data
        0
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlowControlOffCommand {
    pub cr: CommandResponse,
}

impl FlowControlOffCommand {
    pub fn new(cr: CommandResponse) -> Self {
        Self { cr }
    }
}

impl InformationField for FlowControlOffCommand {
    fn information_type(&self) -> InformationType {
        InformationType::FlowControlOffCommand
    }

    async fn write<W: Write>(&self, writer: &mut W) -> Result<(), CmuxError> {
        Ok(())
    }

    async fn read<R: BufRead>(reader: &mut R, len: usize) -> Result<Self, CmuxError> {
        if len != 0 {
            return Err(CmuxError::InvalidInformationField);
        }
        Ok(Self {
            cr: CommandResponse::Command,
        })
    }

    fn wire_len(&self) -> usize {
        0
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlowControlOnCommand {
    pub cr: CommandResponse,
}

impl FlowControlOnCommand {
    pub fn new(cr: CommandResponse) -> Self {
        Self { cr }
    }
}

impl InformationField for FlowControlOnCommand {
    fn information_type(&self) -> InformationType {
        InformationType::FlowControlOnCommand
    }

    async fn write<W: Write>(&self, writer: &mut W) -> Result<(), CmuxError> {
        // Write FCon-specific data here
        Ok(())
    }

    async fn read<R: BufRead>(reader: &mut R, len: usize) -> Result<Self, CmuxError> {
        if len != 0 {
            return Err(CmuxError::InvalidInformationField);
        }
        Ok(Self {
            cr: CommandResponse::Command,
        })
    }

    fn wire_len(&self) -> usize {
        // Return the length of the serialized FCon data
        0
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModemStatusCommand {
    /// Command/Response flag for the frame.
    pub cr: CommandResponse,
    /// Data Link Connection Identifier (DLCI).
    pub dlci: u8,
    /// Control signals for the frame.
    pub control: Control,
    /// Optional break signal for the frame.
    pub brk: Option<Break>,
}

impl ModemStatusCommand {
    pub fn new(cr: CommandResponse, dlci: u8, control: Control, brk: Option<Break>) -> Self {
        Self { cr, dlci, control, brk }
    }
}

impl InformationField for ModemStatusCommand {
    fn information_type(&self) -> InformationType {
        InformationType::ModemStatusCommand
    }

    async fn write<W: Write>(&self, writer: &mut W) -> Result<(), CmuxError> {
        writer.write_all(&[(self.dlci << 2) | (self.cr as u8)]).await?;
        writer.write_all(&[self.control.with_ea(true).into_bits()]).await?;
        if let Some(brk) = self.brk {
            writer.write_all(&[brk.with_ea(true).into_bits()]).await?;
        }
        Ok(())
    }

    async fn read<R: BufRead>(reader: &mut R, len: usize) -> Result<Self, CmuxError> {
        if len < 2 || len > 3 {
            return Err(CmuxError::InvalidInformationField);
        }
        let dlci_cr = reader.read_u8().await?;
        let control = Control::from_bits(reader.read_u8().await?);
        let brk = if len == 3 {
            Some(Break::from_bits(reader.read_u8().await?))
        } else {
            None
        };
        Ok(Self {
            cr: (dlci_cr & 0x03).into(),
            dlci: dlci_cr >> 2,
            control,
            brk,
        })
    }

    fn wire_len(&self) -> usize {
        // Return the length of the serialized FCon data
        2 + if self.brk.is_some() { 1 } else { 0 }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NonSupportedCommandResponse {
    /// Command/Response flag for the frame.
    pub cr: CommandResponse,
    /// Type of the command that was not supported.
    pub command_type: InformationType,
}

impl NonSupportedCommandResponse {
    pub fn new(cr: CommandResponse, command_type: InformationType) -> Self {
        Self { cr, command_type }
    }
}

impl InformationField for NonSupportedCommandResponse {
    fn information_type(&self) -> InformationType {
        InformationType::NonSupportedCommandResponse
    }

    async fn write<W: Write>(&self, writer: &mut W) -> Result<(), CmuxError> {
        writer.write_all(&[self.command_type as u8 | (self.cr as u8)]).await?;
        Ok(())
    }

    async fn read<R: BufRead>(reader: &mut R, len: usize) -> Result<Self, CmuxError> {
        if len != 1 {
            return Err(CmuxError::InvalidInformationField);
        }
        let command_type_cr = reader.read_u8().await?;
        Ok(Self {
            cr: (command_type_cr & 0x03).into(),
            command_type: (command_type_cr & !0x03).try_into()?,
        })
    }

    fn wire_len(&self) -> usize {
        1
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RemotePortNegotiationCommand {
    pub cr: CommandResponse,
    // pub port_parameters: Vec<PortParameter>, // Add port parameters as needed
}

impl RemotePortNegotiationCommand {
    pub fn new(cr: CommandResponse) -> Self {
        Self { cr }
    }
}

impl InformationField for RemotePortNegotiationCommand {
    fn information_type(&self) -> InformationType {
        InformationType::RemotePortNegotiationCommand
    }

    async fn write<W: Write>(&self, writer: &mut W) -> Result<(), CmuxError> {
        // Write RPN-specific data here, including port parameters
        Ok(())
    }

    async fn read<R: BufRead>(reader: &mut R, len: usize) -> Result<Self, CmuxError> {
        if len == 0 {
            return Err(CmuxError::InvalidInformationField);
        }
        // Read and parse RPN-specific data from 'reader' here
        Ok(Self {
            cr: CommandResponse::Command,
        })
    }

    fn wire_len(&self) -> usize {
        // Return the length of the serialized RPN data,
        // including the length of serialized port parameters
        0
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RemoteLineStatusCommand {
    /// Command/Response flag for the frame.
    pub cr: CommandResponse,
    /// Data Link Connection Identifier (DLCI).
    pub dlci: u8,
    /// Remote line status information.
    pub remote_line_status: RemoteLineStatus,
}

impl RemoteLineStatusCommand {
    pub fn new(cr: CommandResponse, dlci: u8, remote_line_status: RemoteLineStatus) -> Self {
        Self {
            cr,
            dlci,
            remote_line_status,
        }
    }
}

impl InformationField for RemoteLineStatusCommand {
    fn information_type(&self) -> InformationType {
        InformationType::RemoteLineStatusCommand
    }

    async fn write<W: Write>(&self, writer: &mut W) -> Result<(), CmuxError> {
        writer.write_all(&[(self.dlci << 2) | (self.cr as u8)]).await?;
        writer.write_all(&[self.remote_line_status.into_bits()]).await?;
        Ok(())
    }

    async fn read<R: BufRead>(reader: &mut R, len: usize) -> Result<Self, CmuxError> {
        if len != 2 {
            return Err(CmuxError::InvalidInformationField);
        }
        let dlci_cr = reader.read_u8().await?;
        Ok(Self {
            cr: (dlci_cr & 0x03).into(),
            dlci: dlci_cr >> 2,
            remote_line_status: RemoteLineStatus::from_bits(reader.read_u8().await?),
        })
    }

    fn wire_len(&self) -> usize {
        2
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ServiceNegotiationCommand {
    pub cr: CommandResponse,
    // pub services: Vec<Service>, // Add service definitions as needed
}

impl ServiceNegotiationCommand {
    pub fn new(cr: CommandResponse) -> Self {
        Self { cr }
    }
}

impl InformationField for ServiceNegotiationCommand {
    fn information_type(&self) -> InformationType {
        InformationType::ServiceNegotiationCommand
    }

    async fn write<W: Write>(&self, writer: &mut W) -> Result<(), CmuxError> {
        // Write SNC-specific data here, including service definitions
        Ok(())
    }

    async fn read<R: BufRead>(reader: &mut R, len: usize) -> Result<Self, CmuxError> {
        if len == 0 {
            return Err(CmuxError::InvalidInformationField);
        }
        // Read and parse SNC-specific data from 'reader' here,
        // including service parameters.
        Ok(Self {
            cr: CommandResponse::Command,
        })
    }

    fn wire_len(&self) -> usize {
        // Return the length of the serialized SNC data,
        // including the length of any serialized service parameters.
        0
    }
}

// Trait for a consistent interface if desired
trait InformationField {
    fn information_type(&self) -> InformationType;
    async fn write<W: Write>(&self, writer: &mut W) -> Result<(), CmuxError>;
    async fn read<R: BufRead>(reader: &mut R, len: usize) -> Result<Self, CmuxError>
    where
        Self: Sized;
    fn wire_len(&self) -> usize;
}

// --- CMUX Frame Types ---

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

impl TryFrom<u8> for FrameType {
    type Error = CmuxError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & !PF {
            0x2F => Ok(Self::Sabm),
            0x63 => Ok(Self::Ua),
            0x0F => Ok(Self::Dm),
            0x43 => Ok(Self::Disc),
            0xEF => Ok(Self::Uih),
            0x03 => Ok(Self::Ui),
            // ... other FrameType matches ...
            _ => Err(CmuxError::InvalidFrameFormat),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Frame<I: InformationField> {
    Sabm(SabmFrame),
    Uih(UihFrame<I>),
    // ... other Frame variants, potentially with associated data ...
}

impl<I: InformationField> Frame<I> {
    pub async fn read<R: BufRead>(reader: &mut R) -> Result<Self, CmuxError> {
        // 1. Find FLAG (handling potential byte stuffing)
        loop {
            let buf = reader.fill_buf().await?;
            if !buf.is_empty() && buf[0] == FLAG {
                reader.consume(1); // Consume the FLAG byte
                break;
            } else if !buf.is_empty() {
                reader.consume(1); // Consume the non-FLAG byte
            } else {
                // Handle potential EOF or buffer underrun (if applicable)
                // You might want to return a WouldBlock error or similar
                // depending on your requirements
                return Err(CmuxError::Io(IoErrorKind::BrokenPipe));
            }
        }

        // Helper function to read a single byte
        async fn read_byte<R: BufRead>(reader: &mut R) -> Result<u8, CmuxError> {
            let buf = reader.fill_buf().await?;
            if !buf.is_empty() {
                reader.consume(1);
                Ok(buf[0])
            } else {
                Err(CmuxError::Io(IoErrorKind::BrokenPipe))
            }
        }

        // 2. Read address, control, and length (with EA handling)
        let address_control = read_byte(reader).await?;
        let frame_type_byte = read_byte(reader).await?;
        let mut length = read_byte(reader).await? as usize;
        if length as u8 & EA == 0 {
            length = (length << 8) | read_byte(reader).await? as usize;
            length >>= 1; // Shift out the EA bit
        } else {
            length >>= 1; // Shift out the EA bit
        }

        // 3. Calculate CRC on the fly
        let mut fcs = FCS_GENERATOR.digest();
        fcs.update(&[
            address_control,
            frame_type_byte,
            (length >> 8) as u8,
            (length & 0xFF) as u8,
        ]);

        let id = address_control >> 2;
        let cr = CommandResponse::from(address_control);

        let frame_type = FrameType::try_from(frame_type_byte)?;
        // 4. Parse based on FrameType
        let frame = match frame_type {
            FrameType::Sabm => {
                if length != 0 {
                    return Err(CmuxError::InvalidInformationField);
                }
                Frame::Sabm(SabmFrame {
                    id,
                    pf: (frame_type_byte & PF).into(),
                })
            }
            FrameType::Uih => {
                if length > MAX_INFO_LEN {
                    return Err(CmuxError::InvalidFrameFormat);
                }
                let mut info_bytes = [0u8; MAX_INFO_LEN];

                reader.read_exact(&mut info_bytes[0..length]).await?;
                fcs.update(&info_bytes[0..length]);
                let info_type = InformationType::try_from(info_bytes[0])?;
                let information: I = match info_type {
                    InformationType::ParameterNegotiation => {
                        ParameterNegotiation::read(&mut &info_bytes[1..length], length - 1).await?
                    }
                    InformationType::MultiplexerCloseDown => {
                        MultiplexerCloseDown::read(&mut &info_bytes[1..length], length - 1).await?
                    }
                    InformationType::FlowControlOffCommand => {
                        FlowControlOffCommand::read(&mut &info_bytes[1..length], length - 1).await?
                    }
                    InformationType::FlowControlOnCommand => {
                        FlowControlOnCommand::read(&mut &info_bytes[1..length], length - 1).await?
                    }
                    InformationType::ModemStatusCommand => {
                        ModemStatusCommand::read(&mut &info_bytes[1..length], length - 1).await?
                    }
                    InformationType::NonSupportedCommandResponse => {
                        NonSupportedCommandResponse::read(&mut &info_bytes[1..length], length - 1).await?
                    }
                    InformationType::RemotePortNegotiationCommand => {
                        RemotePortNegotiationCommand::read(&mut &info_bytes[1..length], length - 1).await?
                    }
                    InformationType::RemoteLineStatusCommand => {
                        RemoteLineStatusCommand::read(&mut &info_bytes[1..length], length - 1).await?
                    }
                    InformationType::ServiceNegotiationCommand => {
                        ServiceNegotiationCommand::read(&mut &info_bytes[1..length], length - 1).await?
                    }
                    _ => return Err(CmuxError::UnsupportedOperation), // Or handle as needed
                };

                Frame::Uih(UihFrame {
                    id,
                    information,
                    cr,
                    pf: (frame_type_byte & PF).into(),
                })
            }
            _ => return Err(CmuxError::UnsupportedOperation), // Or handle as needed
        };

        // 6. Finalize and check FCS
        let received_fcs = read_byte(reader).await?;
        if received_fcs != fcs.finalize() {
            return Err(CmuxError::CrcMismatch);
        }

        // 7. Read trailing FLAG
        if read_byte(reader).await? != FLAG {
            return Err(CmuxError::InvalidFrameFormat);
        }

        Ok(frame)
    }

    pub async fn write<W: Write>(&self, writer: &mut W) -> Result<(), CmuxError> {
        // 1. Write FLAG
        writer.write_all(&[FLAG]).await?;

        match self {
            Frame::Sabm(frame) => {
                // 2. Write address and control
                writer.write_all(&[(frame.id << 2) | (frame.pf as u8)]).await?;

                // 3. Write frame type
                writer.write_all(&[FrameType::Sabm as u8 | (frame.pf as u8)]).await?;

                // 4. Write length (using EA if necessary)
                // No information field for SABM

                // 5. Write information field
                // No information field for SABM

                // 6. Calculate and write FCS
                let fcs = 0xFF
                    - FCS_GENERATOR.checksum(&[
                        (frame.id << 2) | (frame.pf as u8),
                        FrameType::Sabm as u8 | (frame.pf as u8),
                        (0 as u8) << 1 | EA,
                    ]);
                writer.write_all(&[fcs]).await?;

                // 7. Write trailing FLAG
                writer.write_all(&[FLAG]).await?;
            }
            Frame::Uih(frame) => {
                // 2. Write address and control
                writer.write_all(&[(frame.id << 2) | (frame.cr as u8)]).await?;

                // 3. Write frame type
                writer.write_all(&[FrameType::Uih as u8 | (frame.pf as u8)]).await?;

                // 4. Write length (using EA if necessary)
                let info_len = frame.information.wire_len();
                write_ea_length(writer, info_len).await?;

                // 5. Write information field
                frame.information.write(writer).await?;

                // 6. Calculate and write FCS
                let mut fcs_buf = [0u8; 255];
                let mut fcs_writer = &mut fcs_buf[..];
                fcs_writer.write_all(&[(frame.id << 2) | (frame.cr as u8)]).await?;
                fcs_writer.write_all(&[FrameType::Uih as u8 | (frame.pf as u8)]).await?;

                let info_len = frame.information.wire_len();
                if info_len > 127 {
                    // Extended length
                    fcs_writer.write_all(&[((info_len >> 7) as u8) << 1]).await?;
                }
                fcs_writer.write_all(&[((info_len as u8) << 1) | EA]).await?;
                frame.information.write(fcs_writer)?;

                let fcs = 0xFF - FCS_GENERATOR.checksum(&fcs_buf[0..fcs_writer.len()]);
                writer.write_all(&[fcs]).await?;

                // 7. Write trailing FLAG
                writer.write_all(&[FLAG]).await?;
            } // Handle other frame types...
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct SabmFrame {
    pub id: u8,
    pub pf: PollFinal, // Assuming SABM uses PF
}

impl SabmFrame {
    pub fn new(id: u8, pf: PollFinal) -> Self {
        Self { id, pf }
    }
}

// ... Implementations for other frame types: UihFrame, etc. ...

#[derive(Debug, PartialEq)]
pub struct UihFrame<I: InformationField> {
    pub id: u8,
    pub information: I,
    pub cr: CommandResponse,
    pub pf: PollFinal,
}

impl<I: InformationField> UihFrame<I> {
    pub fn new(id: u8, information: I, cr: CommandResponse, pf: PollFinal) -> Self {
        Self {
            id,
            information,
            cr,
            pf,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    // --- Test Utilities ---
    // (These structs and implementations are helpful for testing)

    struct TestReader {
        data: Vec<u8>,
        pos: usize,
    }

    impl TestReader {
        fn new(data: Vec<u8>) -> Self {
            Self { data, pos: 0 }
        }
    }

    impl BufRead for TestReader {
        async fn fill_buf(&mut self) -> Result<&[u8], IoError> {
            if self.pos >= self.data.len() {
                Ok(&[])
            } else {
                Ok(&self.data[self.pos..])
            }
        }

        fn consume(&mut self, amt: usize) {
            self.pos += amt;
        }
    }

    struct TestWriter {
        data: Vec<u8>,
    }

    impl TestWriter {
        fn new() -> Self {
            Self { data: Vec::new() }
        }

        fn into_inner(self) -> Vec<u8> {
            self.data
        }
    }

    impl Write for TestWriter {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
            self.data.extend_from_slice(buf);
            Ok(buf.len())
        }

        async fn flush(&mut self) -> Result<(), IoError> {
            Ok(())
        }
    }

    // --- Unit Tests ---

    #[tokio::test]
    async fn test_sabm_frame() {
        let mut writer = TestWriter::new();
        let frame = Frame::Sabm(SabmFrame::new(2, PollFinal::Poll));

        frame.write(&mut writer).await.unwrap();
        let written_data = writer.into_inner();

        assert_eq!(
            written_data,
            vec![0xF9, 0x0C, 0x3F, 0x9B, 0xF9] // Expected byte sequence for SABM
        );

        // Now, test reading
        let mut reader = TestReader::new(written_data);
        let read_frame = Frame::read(&mut reader).await.unwrap();

        assert_eq!(frame, read_frame);
    }

    #[tokio::test]
    async fn test_uih_frame_with_parameter_negotiation() {
        let mut writer = TestWriter::new();
        let info_field = ParameterNegotiation::new(CommandResponse::Command);
        let frame = Frame::Uih(UihFrame::new(
            1,
            &info_field,
            CommandResponse::Command,
            PollFinal::Final,
        ));

        frame.write(&mut writer).await.unwrap();
        let written_data = writer.into_inner();

        // Expected byte sequence for UIH with empty PN
        let expected_data = vec![0xF9, 0x05, 0xEF, 0x01, 0x3E, 0xF9];
        assert_eq!(written_data, expected_data);

        let mut reader = TestReader::new(written_data);
        let read_frame = Frame::read(&mut reader).await.unwrap();

        assert_eq!(frame, read_frame);
    }

    #[tokio::test]
    async fn test_uih_frame_with_modem_status_command() {
        let mut writer = TestWriter::new();
        let info_field = ModemStatusCommand::new(CommandResponse::Command, 2, Control::new(), Some(Break::new()));
        let frame = Frame::Uih(UihFrame::new(
            1,
            &info_field,
            CommandResponse::Command,
            PollFinal::Final,
        ));

        frame.write(&mut writer).await.unwrap();
        let written_data = writer.into_inner();

        // Expected byte sequence for UIH with empty PN
        let expected_data = vec![0xF9, 0x05, 0xEF, 0x05, 0x0c, 0x01, 0x01, 0x0a, 0xF9];
        assert_eq!(written_data, expected_data);

        let mut reader = TestReader::new(written_data);
        let read_frame = Frame::read(&mut reader).await.unwrap();

        assert_eq!(frame, read_frame);
    }

    // ... Add more tests for different frame types and information fields ...

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
