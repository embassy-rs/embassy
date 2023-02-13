use core::fmt::Debug;

use lorawan_device::async_device::radio as device;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RadioError<BUS> {
    SPI(BUS),
    CS,
    Reset,
    AntRx,
    AntTx,
    Busy,
    DIO1,
    PayloadSizeMismatch(usize, usize),
    RetentionListExceeded,
    InvalidBandwidth,
    ModulationParamsMissing,
    PacketParamsMissing,
    HeaderError,
    CRCErrorUnexpected,
    CRCErrorOnReceive,
    TransmitTimeout,
    ReceiveTimeout,
    TimeoutUnexpected,
    TransmitDoneUnexpected,
    ReceiveDoneUnexpected,
    CADUnexpected,
}

pub struct RadioSystemError {
    pub rc_64khz_calibration: bool,
    pub rc_13mhz_calibration: bool,
    pub pll_calibration: bool,
    pub adc_calibration: bool,
    pub image_calibration: bool,
    pub xosc_start: bool,
    pub pll_lock: bool,
    pub pa_ramp: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PacketType {
    GFSK = 0x00,
    LoRa = 0x01,
    None = 0x0F,
}

impl PacketType {
    pub const fn value(self) -> u8 {
        self as u8
    }
    pub fn to_enum(value: u8) -> Self {
        if value == 0x00 {
            PacketType::GFSK
        } else if value == 0x01 {
            PacketType::LoRa
        } else {
            PacketType::None
        }
    }
}

#[derive(Clone, Copy)]
pub struct PacketStatus {
    pub rssi: i8,
    pub snr: i8,
    pub signal_rssi: i8,
    pub freq_error: u32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RadioType {
    SX1261,
    SX1262,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RadioMode {
    Sleep = 0x00,                    // sleep mode
    StandbyRC = 0x01,                // standby mode with RC oscillator
    StandbyXOSC = 0x02,              // standby mode with XOSC oscillator
    FrequencySynthesis = 0x03,       // frequency synthesis mode
    Transmit = 0x04,                 // transmit mode
    Receive = 0x05,                  // receive mode
    ReceiveDutyCycle = 0x06,         // receive duty cycle mode
    ChannelActivityDetection = 0x07, // channel activity detection mode
}

impl RadioMode {
    /// Returns the value of the mode.
    pub const fn value(self) -> u8 {
        self as u8
    }
    pub fn to_enum(value: u8) -> Self {
        if value == 0x00 {
            RadioMode::Sleep
        } else if value == 0x01 {
            RadioMode::StandbyRC
        } else if value == 0x02 {
            RadioMode::StandbyXOSC
        } else if value == 0x03 {
            RadioMode::FrequencySynthesis
        } else if value == 0x04 {
            RadioMode::Transmit
        } else if value == 0x05 {
            RadioMode::Receive
        } else if value == 0x06 {
            RadioMode::ReceiveDutyCycle
        } else if value == 0x07 {
            RadioMode::ChannelActivityDetection
        } else {
            RadioMode::Sleep
        }
    }
}

pub enum RadioState {
    Idle = 0x00,
    RxRunning = 0x01,
    TxRunning = 0x02,
    ChannelActivityDetecting = 0x03,
}

impl RadioState {
    /// Returns the value of the state.
    pub fn value(self) -> u8 {
        self as u8
    }
}

pub struct RadioStatus {
    pub cmd_status: u8,
    pub chip_mode: u8,
}

impl RadioStatus {
    pub fn value(self) -> u8 {
        (self.chip_mode << 4) | (self.cmd_status << 1)
    }
}

#[derive(Clone, Copy)]
pub enum IrqMask {
    None = 0x0000,
    TxDone = 0x0001,
    RxDone = 0x0002,
    PreambleDetected = 0x0004,
    SyncwordValid = 0x0008,
    HeaderValid = 0x0010,
    HeaderError = 0x0020,
    CRCError = 0x0040,
    CADDone = 0x0080,
    CADActivityDetected = 0x0100,
    RxTxTimeout = 0x0200,
    All = 0xFFFF,
}

impl IrqMask {
    pub fn value(self) -> u16 {
        self as u16
    }
}

#[derive(Clone, Copy)]
pub enum Register {
    PacketParams = 0x0704,          // packet configuration
    PayloadLength = 0x0702,         // payload size
    SynchTimeout = 0x0706,          // recalculated number of symbols
    Syncword = 0x06C0,              // Syncword values
    LoRaSyncword = 0x0740,          // LoRa Syncword value
    GeneratedRandomNumber = 0x0819, //32-bit generated random number
    AnaLNA = 0x08E2,                // disable the LNA
    AnaMixer = 0x08E5,              // disable the mixer
    RxGain = 0x08AC,                // RX gain (0x94: power saving, 0x96: rx boosted)
    XTATrim = 0x0911,               // device internal trimming capacitor
    OCP = 0x08E7,                   // over current protection max value
    RetentionList = 0x029F,         // retention list
    IQPolarity = 0x0736,            // optimize the inverted IQ operation (see DS_SX1261-2_V1.2 datasheet chapter 15.4)
    TxModulation = 0x0889, // modulation quality with 500 kHz LoRa Bandwidth (see DS_SX1261-2_V1.2 datasheet chapter 15.1)
    TxClampCfg = 0x08D8,   // better resistance to antenna mismatch (see DS_SX1261-2_V1.2 datasheet chapter 15.2)
    RTCCtrl = 0x0902,      // RTC control
    EvtClr = 0x0944,       // event clear
}

impl Register {
    pub fn addr(self) -> u16 {
        self as u16
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum OpCode {
    GetStatus = 0xC0,
    WriteRegister = 0x0D,
    ReadRegister = 0x1D,
    WriteBuffer = 0x0E,
    ReadBuffer = 0x1E,
    SetSleep = 0x84,
    SetStandby = 0x80,
    SetFS = 0xC1,
    SetTx = 0x83,
    SetRx = 0x82,
    SetRxDutyCycle = 0x94,
    SetCAD = 0xC5,
    SetTxContinuousWave = 0xD1,
    SetTxContinuousPremable = 0xD2,
    SetPacketType = 0x8A,
    GetPacketType = 0x11,
    SetRFFrequency = 0x86,
    SetTxParams = 0x8E,
    SetPAConfig = 0x95,
    SetCADParams = 0x88,
    SetBufferBaseAddress = 0x8F,
    SetModulationParams = 0x8B,
    SetPacketParams = 0x8C,
    GetRxBufferStatus = 0x13,
    GetPacketStatus = 0x14,
    GetRSSIInst = 0x15,
    GetStats = 0x10,
    ResetStats = 0x00,
    CfgDIOIrq = 0x08,
    GetIrqStatus = 0x12,
    ClrIrqStatus = 0x02,
    Calibrate = 0x89,
    CalibrateImage = 0x98,
    SetRegulatorMode = 0x96,
    GetErrors = 0x17,
    ClrErrors = 0x07,
    SetTCXOMode = 0x97,
    SetTxFallbackMode = 0x93,
    SetRFSwitchMode = 0x9D,
    SetStopRxTimerOnPreamble = 0x9F,
    SetLoRaSymbTimeout = 0xA0,
}

impl OpCode {
    pub fn value(self) -> u8 {
        self as u8
    }
}

pub struct SleepParams {
    pub wakeup_rtc: bool, // get out of sleep mode if wakeup signal received from RTC
    pub reset: bool,
    pub warm_start: bool,
}

impl SleepParams {
    pub fn value(self) -> u8 {
        ((self.warm_start as u8) << 2) | ((self.reset as u8) << 1) | (self.wakeup_rtc as u8)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum StandbyMode {
    RC = 0x00,
    XOSC = 0x01,
}

impl StandbyMode {
    pub fn value(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy)]
pub enum RegulatorMode {
    UseLDO = 0x00,
    UseDCDC = 0x01,
}

impl RegulatorMode {
    pub fn value(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy)]
pub struct CalibrationParams {
    pub rc64k_enable: bool,     // calibrate RC64K clock
    pub rc13m_enable: bool,     // calibrate RC13M clock
    pub pll_enable: bool,       // calibrate PLL
    pub adc_pulse_enable: bool, // calibrate ADC Pulse
    pub adc_bulkn_enable: bool, // calibrate ADC bulkN
    pub adc_bulkp_enable: bool, // calibrate ADC bulkP
    pub img_enable: bool,
}

impl CalibrationParams {
    pub fn value(self) -> u8 {
        ((self.img_enable as u8) << 6)
            | ((self.adc_bulkp_enable as u8) << 5)
            | ((self.adc_bulkn_enable as u8) << 4)
            | ((self.adc_pulse_enable as u8) << 3)
            | ((self.pll_enable as u8) << 2)
            | ((self.rc13m_enable as u8) << 1)
            | ((self.rc64k_enable as u8) << 0)
    }
}

#[derive(Clone, Copy)]
pub enum TcxoCtrlVoltage {
    Ctrl1V6 = 0x00,
    Ctrl1V7 = 0x01,
    Ctrl1V8 = 0x02,
    Ctrl2V2 = 0x03,
    Ctrl2V4 = 0x04,
    Ctrl2V7 = 0x05,
    Ctrl3V0 = 0x06,
    Ctrl3V3 = 0x07,
}

impl TcxoCtrlVoltage {
    pub fn value(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy)]
pub enum RampTime {
    Ramp10Us = 0x00,
    Ramp20Us = 0x01,
    Ramp40Us = 0x02,
    Ramp80Us = 0x03,
    Ramp200Us = 0x04,
    Ramp800Us = 0x05,
    Ramp1700Us = 0x06,
    Ramp3400Us = 0x07,
}

impl RampTime {
    pub fn value(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SpreadingFactor {
    _5 = 0x05,
    _6 = 0x06,
    _7 = 0x07,
    _8 = 0x08,
    _9 = 0x09,
    _10 = 0x0A,
    _11 = 0x0B,
    _12 = 0x0C,
}

impl SpreadingFactor {
    pub fn value(self) -> u8 {
        self as u8
    }
}

impl From<device::SpreadingFactor> for SpreadingFactor {
    fn from(sf: device::SpreadingFactor) -> Self {
        match sf {
            device::SpreadingFactor::_7 => SpreadingFactor::_7,
            device::SpreadingFactor::_8 => SpreadingFactor::_8,
            device::SpreadingFactor::_9 => SpreadingFactor::_9,
            device::SpreadingFactor::_10 => SpreadingFactor::_10,
            device::SpreadingFactor::_11 => SpreadingFactor::_11,
            device::SpreadingFactor::_12 => SpreadingFactor::_12,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Bandwidth {
    _500KHz = 0x06,
    _250KHz = 0x05,
    _125KHz = 0x04,
}

impl Bandwidth {
    pub fn value(self) -> u8 {
        self as u8
    }

    pub fn value_in_hz(self) -> u32 {
        match self {
            Bandwidth::_125KHz => 125000u32,
            Bandwidth::_250KHz => 250000u32,
            Bandwidth::_500KHz => 500000u32,
        }
    }
}

impl From<device::Bandwidth> for Bandwidth {
    fn from(bw: device::Bandwidth) -> Self {
        match bw {
            device::Bandwidth::_500KHz => Bandwidth::_500KHz,
            device::Bandwidth::_250KHz => Bandwidth::_250KHz,
            device::Bandwidth::_125KHz => Bandwidth::_125KHz,
        }
    }
}

#[derive(Clone, Copy)]
pub enum CodingRate {
    _4_5 = 0x01,
    _4_6 = 0x02,
    _4_7 = 0x03,
    _4_8 = 0x04,
}

impl CodingRate {
    pub fn value(self) -> u8 {
        self as u8
    }
}

impl From<device::CodingRate> for CodingRate {
    fn from(cr: device::CodingRate) -> Self {
        match cr {
            device::CodingRate::_4_5 => CodingRate::_4_5,
            device::CodingRate::_4_6 => CodingRate::_4_6,
            device::CodingRate::_4_7 => CodingRate::_4_7,
            device::CodingRate::_4_8 => CodingRate::_4_8,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ModulationParams {
    pub spreading_factor: SpreadingFactor,
    pub bandwidth: Bandwidth,
    pub coding_rate: CodingRate,
    pub low_data_rate_optimize: u8,
}

#[derive(Clone, Copy)]
pub struct PacketParams {
    pub preamble_length: u16,  // number of LoRa symbols in the preamble
    pub implicit_header: bool, // if the header is explicit, it will be transmitted in the LoRa packet, but is not transmitted if the header is implicit (known fixed length)
    pub payload_length: u8,
    pub crc_on: bool,
    pub iq_inverted: bool,
}

#[derive(Clone, Copy)]
pub enum CADSymbols {
    _1 = 0x00,
    _2 = 0x01,
    _4 = 0x02,
    _8 = 0x03,
    _16 = 0x04,
}

impl CADSymbols {
    pub fn value(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy)]
pub enum CADExitMode {
    CADOnly = 0x00,
    CADRx = 0x01,
    CADLBT = 0x10,
}

impl CADExitMode {
    pub fn value(self) -> u8 {
        self as u8
    }
}
