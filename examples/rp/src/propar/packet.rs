use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, Default, Clone, Copy)]
pub struct ProcessData {
    pub chained: Chained,
    pub process_number: u8,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ParamData {
    pub chained: Chained,
    pub parameter_type: ParamType,
    pub parameter_number: u8,
}

pub const PACKET_DATA_VALUE_LEN: u8 = 30;

#[derive(Debug, Default, Clone, Copy)]
pub struct PacketData {
    pub process_data: ProcessData,
    pub parameter_data: ParamData,
    pub value: [u8; PACKET_DATA_VALUE_LEN as usize],
    pub len: u8,
}

impl PacketData {
    pub fn to_parsed_value(self) -> ParamTypeValue {
        let len = match self.parameter_data.parameter_type {
            ParamType::StringP => {
                core::cmp::Ord::clamp(self.value[0], 0, PACKET_DATA_VALUE_LEN - 1) as usize
            }
            p => p.to_length(),
        };

        match self.parameter_data.parameter_type {
            ParamType::CharacterP => ParamTypeValue::CharacterP(u8::from_be_bytes(
                self.value[0..len].try_into().unwrap(),
            )),
            ParamType::InetegerP => ParamTypeValue::InetegerP(u16::from_be_bytes(
                self.value[0..len].try_into().unwrap(),
            )),
            ParamType::FloatOrLongP => ParamTypeValue::FloatOrLongP(u32::from_be_bytes(
                self.value[0..len].try_into().unwrap(),
            )),
            ParamType::StringP => {
                let mut c: [char; 29] = [0x00 as char; 29];
                for i in 0..len {
                    c[i] = self.value[i + 1] as char;
                }
                ParamTypeValue::StringP(c)
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Packet {
    pub node: u8,
    pub len: u8,
    pub command: CommandType,
    pub status: Option<Status>,
}

pub struct Param {
    pub process_number: u8,
    pub fbnr: u8,
    pub parameter_type: ParamType,
}

#[derive(Debug, Default, Clone, FromPrimitive)]
#[repr(u8)]
pub enum Status {
    #[default]
    NoError = 0x00,
    ProcessClaimed = 0x01,
    CommandError = 0x02,
    ProcessError = 0x03,
    ParameterError = 0x04,
    ParametertypeError = 0x05,
    ParameterValueError = 0x06,
    NetworkNotActive = 0x07,
    TimeOutstartCharacter = 0x08,
    TimeOutSerialLine = 0x09,
    HardwareMemoryError = 0x0A,
    NodeNumberError = 0x0B,
    GeneralCommunicationError = 0x0C,
    ReadOnlyParameter = 0x0D,
    ErrorPcCommunication = 0x0E,
    NoRs232Connection = 0x0F,
    PCOutOfMemory = 0x10,
    WriteOnlyParameter = 0x11,
    SystemConfigurationUnknown = 0x12,
    NoFreeNodeAddress = 0x13,
    WrongInterfaceType = 0x14,
    ErrorSerialPortConnection = 0x15,
    Erroropeningcommunication = 0x16,
    CommunicationError = 0x17,
    ErrorInterfaceBusMaster = 0x18,
    TimeoutAnswer = 0x19,
    NoStartCharacter = 0x1A,
    ErrorFirstDigit = 0x1B,
    BufferOverflowInHost = 0x1C,
    BufferOverflow = 0x1D,
    NoAnswerFound = 0x1E,
    Errorclosingcommunication = 0x1F,
    SynchronisationError = 0x20,
    SendError = 0x21,
    ProtocolError = 0x22,
    BufferOverflowInModule = 0x23,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum CommandType {
    #[default]
    Status = 0x00,
    WriteWithStatusAnswer = 0x01,
    WriteWithoutStatusAnswer = 0x02,
    WriteWithSourceAddress = 0x03,
    RequestParameter = 0x04,
}

#[derive(Debug, Default, Clone, Copy, FromPrimitive, PartialEq)]
#[repr(u8)]
pub enum Chained {
    Is = 0x80,
    #[default]
    Not = 0x00,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum ParamType {
    #[default]
    CharacterP = 0x00,
    InetegerP = 0x20,
    FloatOrLongP = 0x40,
    StringP = 0x60,
}

impl ParamType {
    pub fn to_length(&self) -> usize {
        match self {
            Self::CharacterP => 1,
            Self::InetegerP => 2,
            Self::FloatOrLongP => 4,
            Self::StringP => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamTypeValue {
    CharacterP(u8),
    InetegerP(u16),
    FloatOrLongP(u32),
    StringP([char; 29]),
}

pub const BRONK_MEASURE_PROCESS: u8 = 1;
pub const BRONK_MEASURE_FBNR: u8 = 0;

pub const BRONK_SETPOINT_PROCESS: u8 = 1;
pub const BRONK_SETPOINT_FBNR: u8 = 1;

pub const BRONK_SETPOINT_SLOPE_PROCESS: u8 = 1;
pub const BRONK_SETPOINT_SLOPE_FBNR: u8 = 2;

pub const BRONK_ANALOG_INPUT_PROCESS: u8 = 1;
pub const BRONK_ANALOG_INPUT_FBNR: u8 = 3;

pub const BRONK_CONTROL_MODE_PROCESS: u8 = 1;
pub const BRONK_CONTROL_MODE_FBNR: u8 = 4;

// Parametername Group0 Group1 Group2 Processnumber FBnr(par) VarType VarLength
// Minvalue Maxvalue Read Write Poll Secured HighlySecured DefaultValue
// Description

// Measure 2 0 i -23593 41942 Yes Yes Yes No No 0 measure measured value (100% =
// 32000)
pub const BRONK_MEASURE_P: Param = Param {
    process_number: BRONK_MEASURE_PROCESS,
    fbnr: BRONK_MEASURE_FBNR,
    parameter_type: ParamType::InetegerP,
};

// Setpoint 2 18 1 i 0 32767 Yes Yes Yes No No 0 setpoint setpoint: wanted value
// (100% = 32000)
pub const BRONK_SETPOINT_P: Param = Param {
    process_number: BRONK_SETPOINT_PROCESS,
    fbnr: BRONK_SETPOINT_FBNR,
    parameter_type: ParamType::InetegerP,
};

// Setpoint slope 18 2 i 0 30000 Yes Yes No No No 0 setpslope setpoint ramp
// signal 0..100 % in up to slope x 0.1 sec
pub const BRONK_SETPOINT_SLOPE_P: Param = Param {
    process_number: BRONK_SETPOINT_SLOPE_PROCESS,
    fbnr: BRONK_SETPOINT_SLOPE_FBNR,
    parameter_type: ParamType::InetegerP,
};

// Analog input 2 18 3 i -23593 41942 Yes No Yes No No 0 analoginp analog input
// signal, normally used for ext. setp. (100% = 32000)
pub const BRONK_ANALOG_INPUT_P: Param = Param {
    process_number: BRONK_ANALOG_INPUT_PROCESS,
    fbnr: BRONK_ANALOG_INPUT_FBNR,
    parameter_type: ParamType::InetegerP,
};

// Control mode 18 4 c 0 255 Yes Yes No No No 0 cntrlmode control mode selection
// for instrument or module
pub const BRONK_CONTROL_MODE_P: Param = Param {
    process_number: BRONK_CONTROL_MODE_PROCESS,
    fbnr: BRONK_CONTROL_MODE_FBNR,
    parameter_type: ParamType::CharacterP,
};
