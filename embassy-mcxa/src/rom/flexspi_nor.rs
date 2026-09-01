use core::sync::atomic::{AtomicBool, Ordering};

use super::Status;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SerialNorOptionTag {
    Config = 0x0C, // SDK vs. RM mismatch; TODO: confirm correct value and semantics.
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialNorOptionSize {
    Option0Only = 0,
    Option0AndOption1 = 1,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialNorDeviceType {
    ReadSfdpSdr = 0,
    ReadSfdpDdr = 1,
    Hyperflash1V8 = 2,
    Hyperflash3V0 = 3,
    MacronixOctalDdr = 4,
    MacronixOctalSdr = 5,
    MicronOctalDdr = 6,
    MicronOctalSdr = 7,
    AdestoOctalDdr = 8,
    AdestoOctalSdr = 9,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialNorOptionPadEncoding {
    // Encoded values for option0.query_pads / option0.cmd_pads.
    // These match the ROM option field encoding, not the literal kSerialFlash_*Pad values.
    One = 0,
    Four = 2,
    Eight = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialNorQuadModeSetting {
    NotConfigured = 0,
    StatusReg1Bit6 = 1,
    StatusReg2Bit1 = 2,
    StatusReg2Bit7 = 3,
    StatusReg2Bit1Via0x31 = 4,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialNorMiscMode {
    Disabled = 0,
    Mode0_4_4 = 1,
    Mode0_8_8 = 2,
    DataOrderSwapped = 3,
    SecondPinMux = 4,
    InternalLoopback = 5,
    SpiMode = 6,
    ExternalDqs = 8,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexspiSerialClockFrequency {
    NoChange = 0,
    MHz30 = 1,
    MHz50 = 2,
    MHz60 = 3,
    MHz75 = 4,
    MHz80 = 5,
    MHz100 = 6,
    MHz133 = 7,
    MHz166 = 8,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialNorFlashConnection {
    SinglePortA = 0,
    Parallel = 1,
    SinglePortB = 2,
    BothPorts = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexspiClockSource {
    // Table 60 selector values for the ROM set_clock_source API.
    NoClock = 0,
    Pll0 = 1,
    FroHf = 3,
    Pll1 = 5,
    UsbPll = 6,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexspiClockConfigFrequency {
    // Table 61 values for the ROM config_clock API.
    MHz30 = 1,
    MHz50 = 2,
    MHz60 = 3,
    MHz75 = 4,
    MHz100 = 5,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexspiClockConfigMode {
    Sdr = 0,
    Ddr = 1,
}

pub const fn pack_serial_nor_option0(
    option_size: SerialNorOptionSize,
    device_type: SerialNorDeviceType,
    query_pad: SerialNorOptionPadEncoding,
    cmd_pad: SerialNorOptionPadEncoding,
    quad_mode_setting: SerialNorQuadModeSetting,
    misc_mode: SerialNorMiscMode,
    max_freq: FlexspiSerialClockFrequency,
) -> u32 {
    ((SerialNorOptionTag::Config as u32) << 28)
        | ((option_size as u32) << 24)
        | ((device_type as u32) << 20)
        | ((query_pad as u32) << 16)
        | ((cmd_pad as u32) << 12)
        | ((quad_mode_setting as u32) << 8)
        | ((misc_mode as u32) << 4)
        | (max_freq as u32)
}

pub const fn pack_serial_nor_option1(
    flash_connection: SerialNorFlashConnection,
    dqs_pinmux_group: u32,
    pinmux_group: u32,
    status_override: u32,
    dummy_cycles: u32,
) -> u32 {
    ((flash_connection as u32) << 28)
        | ((dqs_pinmux_group & 0xF) << 20)
        | ((pinmux_group & 0xF) << 16)
        | ((status_override & 0xFF) << 8)
        | (dummy_cycles & 0xFF)
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SerialNorConfigOption {
    // Packed ROM ABI input for `flexspiNorDriver->get_config(...)`.
    //
    // Typical MCXA ROM examples:
    // - Quad NOR, Quad SDR read @ 75 MHz:  option0 = 0xC000_0004, option1 = 0
    // - Quad NOR, Quad DDR read @ 60 MHz:  option0 = 0xC010_0003, option1 = 0
    //
    // Build these words with `pack_serial_nor_option0(...)` and
    // `pack_serial_nor_option1(...)`, keeping the transport struct itself raw.
    //
    // Example call pattern from the ROM docs:
    //   let mut option = SerialNorConfigOption { option0: 0xC000_0001, option1: 0 };
    //   flexspi_nor().get_config(instance, &mut cfg, &mut option);
    /// For convenience, see [pack_serial_nor_option0]
    pub option0: u32,
    /// For convenience, see [pack_serial_nor_option1]
    pub option1: u32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexspiOperationType {
    Command = 0,
    Config = 1,
    Write = 2,
    Read = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FlashRunContextFields {
    por_mode: u8,
    current_mode: u8,
    exit_no_cmd_sequence: u8,
    restore_sequence: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union FlashRunContext {
    b: FlashRunContextFields,
    u: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FlexspiLutSeq {
    pub seq_num: u8,
    pub seq_id: u8,
    pub reserved: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FlexspiDllTime {
    pub time_100ps: u8,
    pub delay_cells: u8,
}

#[repr(C)]
pub struct FlexspiMemConfig {
    pub tag: u32,
    pub version: u32,
    pub reserved0: u32,
    pub read_sample_clk_src: u8,
    pub cs_hold_time: u8,
    pub cs_setup_time: u8,
    pub column_address_width: u8,
    pub device_mode_cfg_enable: u8,
    pub device_mode_type: u8,
    pub wait_time_cfg_commands: u16,
    pub device_mode_seq: FlexspiLutSeq,
    pub device_mode_arg: u32,
    pub config_cmd_enable: u8,
    pub config_mode_type: [u8; 3],
    pub config_cmd_seqs: [FlexspiLutSeq; 3],
    pub reserved1: u32,
    pub config_cmd_args: [u32; 3],
    pub reserved2: u32,
    pub controller_misc_option: u32,
    pub device_type: u8,
    pub sflash_pad_type: u8,
    pub serial_clk_freq: u8,
    pub lut_custom_seq_enable: u8,
    pub reserved3: [u32; 2],
    pub sflash_a1_size: u32,
    pub sflash_a2_size: u32,
    pub sflash_b1_size: u32,
    pub sflash_b2_size: u32,
    pub cs_pad_setting_override: u32,
    pub sclk_pad_setting_override: u32,
    pub data_pad_setting_override: u32,
    pub dqs_pad_setting_override: u32,
    pub timeout_in_ms: u32,
    pub command_interval: u32,
    pub data_valid_time: [FlexspiDllTime; 2],
    pub busy_offset: u16,
    pub busy_bit_polarity: u16,
    pub lookup_table: [u32; 64],
    pub lut_custom_seq: [FlexspiLutSeq; 12],
    pub dll0_cr_val: u32,
    pub dll1_cr_val: u32,
    pub reserved4: [u32; 2],
}

#[repr(C)]
pub struct FlexspiNorConfig {
    pub mem_config: FlexspiMemConfig,
    pub page_size: u32,
    pub sector_size: u32,
    pub ipcmd_serial_clk_freq: u8,
    pub is_uniform_block_size: u8,
    pub is_data_order_swapped: u8,
    pub reserved0: [u8; 1],
    pub serial_nor_type: u8,
    pub need_exit_no_cmd_mode: u8,
    pub half_clk_for_non_read_cmd: u8,
    pub need_restore_no_cmd_mode: u8,
    pub block_size: u32,
    pub flash_state_ctx: FlashRunContext,
    pub reserved2: [u32; 10],
}

#[repr(C)]
pub struct FlexspiXfer {
    pub operation: FlexspiOperationType,
    pub base_address: u32,
    pub seq_id: u32,
    pub seq_num: u32,
    pub is_parallel_mode_enable: bool,
    pub tx_buffer: *const u32,
    pub tx_size: u32,
    pub rx_buffer: *mut u32,
    pub rx_size: u32,
}

#[repr(C)]
pub struct FlexspiNorVtable {
    version: u32,
    init: unsafe extern "C" fn(instance: u32, cfg: *mut FlexspiNorConfig) -> Status,
    page_program: unsafe extern "C" fn(instance: u32, cfg: *mut FlexspiNorConfig, dst: u32, src: *const u32) -> Status,
    erase_all: unsafe extern "C" fn(instance: u32, cfg: *mut FlexspiNorConfig) -> Status,
    erase: unsafe extern "C" fn(instance: u32, cfg: *mut FlexspiNorConfig, start: u32, len: u32) -> Status,
    erase_sector: unsafe extern "C" fn(instance: u32, cfg: *mut FlexspiNorConfig, addr: u32) -> Status,
    erase_block: unsafe extern "C" fn(instance: u32, cfg: *mut FlexspiNorConfig, addr: u32) -> Status,
    get_config:
        unsafe extern "C" fn(instance: u32, cfg: *mut FlexspiNorConfig, opt: *mut SerialNorConfigOption) -> Status,
    read: unsafe extern "C" fn(
        instance: u32,
        cfg: *mut FlexspiNorConfig,
        dst: *mut u32,
        start: u32,
        bytes: u32,
    ) -> Status,
    xfer: unsafe extern "C" fn(instance: u32, xfer: *mut FlexspiXfer) -> Status,
    update_lut: unsafe extern "C" fn(instance: u32, seq_index: u32, lut_base: *const u32, num_seq: u32) -> Status,
    set_clock_source: unsafe extern "C" fn(clock_src: u32) -> Status,
    config_clock: unsafe extern "C" fn(instance: u32, freq_option: u32, sample_clk_mode: u32),
    partial_program:
        unsafe extern "C" fn(instance: u32, cfg: *mut FlexspiNorConfig, dst: u32, src: *const u32, len: u32) -> Status,
}

pub struct FlexspiNor {
    vtable: &'static FlexspiNorVtable,
    instance: u32,
    config: FlexspiNorConfig,
}

static TAKEN: AtomicBool = AtomicBool::new(false);

impl FlexspiNor {
    pub(super) fn new(
        vtable: &'static FlexspiNorVtable,
        instance: u32,
        mut config: FlexspiNorConfig,
    ) -> Result<Self, FlexspiError> {
        if TAKEN
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(FlexspiError::Unavailable);
        }

        Result::from(unsafe { (vtable.init)(instance, &raw mut config) })?;
        Ok(Self {
            vtable,
            instance,
            config,
        })
    }

    pub fn version(&self) -> u32 {
        self.vtable.version
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref, reason = "ROM will check the pointer for validity")]
    pub fn page_program(&mut self, dst: u32, src: *const u32) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.page_program)(self.instance, &raw mut self.config, dst, src) }.into()
    }

    pub fn erase_all(&mut self) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.erase_all)(self.instance, &raw mut self.config) }.into()
    }

    pub fn erase(&mut self, start: u32, len: u32) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.erase)(self.instance, &raw mut self.config, start, len) }.into()
    }

    pub fn erase_sector(&mut self, addr: u32) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.erase_sector)(self.instance, &raw mut self.config, addr) }.into()
    }

    pub fn erase_block(&mut self, addr: u32) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.erase_block)(self.instance, &raw mut self.config, addr) }.into()
    }

    pub fn get_config(&mut self) -> Result<SerialNorConfigOption, FlexspiError> {
        let mut opt = SerialNorConfigOption::default();
        Result::from(unsafe { (self.vtable.get_config)(self.instance, &raw mut self.config, &raw mut opt) })
            .map(|()| opt)
    }

    pub fn read(&mut self, dst: &mut [u32], start: u32) -> Result<(), FlexspiError> {
        unsafe {
            (self.vtable.read)(
                self.instance,
                &raw mut self.config,
                dst.as_mut_ptr(),
                start,
                dst.len() as u32,
            )
        }
        .into()
    }

    pub fn xfer(&mut self, xfer: &mut FlexspiXfer) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.xfer)(self.instance, xfer) }.into()
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref, reason = "ROM will check the pointer for validity")]
    pub fn update_lut(&mut self, seq_index: u32, lut_base: *const u32, num_seq: u32) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.update_lut)(self.instance, seq_index, lut_base, num_seq) }.into()
    }

    pub fn set_clock_source(&mut self, clock_src: FlexspiClockSource) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.set_clock_source)(clock_src as u32) }.into()
    }

    pub fn config_clock(&mut self, freq_option: FlexspiClockConfigFrequency, sample_clk_mode: FlexspiClockConfigMode) {
        unsafe { (self.vtable.config_clock)(self.instance, freq_option as u32, sample_clk_mode as u32) }
    }

    pub fn partial_program(&mut self, dst: u32, src: &[u32]) -> Result<(), FlexspiError> {
        unsafe {
            (self.vtable.partial_program)(self.instance, &raw mut self.config, dst, src.as_ptr(), src.len() as u32)
        }
        .into()
    }
}

impl Drop for FlexspiNor {
    fn drop(&mut self) {
        TAKEN.store(false, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexspiError {
    Fail,
    InvalidArgument,
    SequenceExecutionTimeout,
    InvalidSequence,
    DeviceTimeout,
    ProgramFail,
    EraseSectorFail,
    EraseAllFail,
    WaitTimeout,
    WriteAlignmentError,
    CommandFailure,
    SfdpNotFound,
    UnsupportedSfdpVersion,
    FlashNotFound,
    DtrReadDummyProbeFailed,
    Unknown(u32),
    Unavailable,
}

impl From<Status> for Result<(), FlexspiError> {
    fn from(raw: Status) -> Self {
        match raw.0 {
            KSTATUS_FLEXSPI_SUCCESS => Ok(()),
            KSTATUS_FLEXSPI_FAIL => Err(FlexspiError::Fail),
            KSTATUS_FLEXSPI_INVALID_ARGUMENT => Err(FlexspiError::InvalidArgument),
            KSTATUS_FLEXSPI_SEQUENCE_EXECUTION_TIMEOUT => Err(FlexspiError::SequenceExecutionTimeout),
            KSTATUS_FLEXSPI_INVALID_SEQUENCE => Err(FlexspiError::InvalidSequence),
            KSTATUS_FLEXSPI_DEVICE_TIMEOUT => Err(FlexspiError::DeviceTimeout),
            KSTATUS_FLEXSPINOR_PROGRAM_FAIL => Err(FlexspiError::ProgramFail),
            KSTATUS_FLEXSPINOR_ERASE_SECTOR_FAIL => Err(FlexspiError::EraseSectorFail),
            KSTATUS_FLEXSPINOR_ERASE_ALL_FAIL => Err(FlexspiError::EraseAllFail),
            KSTATUS_FLEXSPINOR_WAIT_TIMEOUT => Err(FlexspiError::WaitTimeout),
            KSTATUS_FLEXSPINOR_WRITE_ALIGNMENT_ERROR => Err(FlexspiError::WriteAlignmentError),
            KSTATUS_FLEXSPINOR_COMMAND_FAILURE => Err(FlexspiError::CommandFailure),
            KSTATUS_FLEXSPINOR_SFDP_NOT_FOUND => Err(FlexspiError::SfdpNotFound),
            KSTATUS_FLEXSPINOR_UNSUPPORTED_SFDP_VERSION => Err(FlexspiError::UnsupportedSfdpVersion),
            KSTATUS_FLEXSPINOR_FLASH_NOT_FOUND => Err(FlexspiError::FlashNotFound),
            KSTATUS_FLEXSPINOR_DTR_READ_DUMMY_PROBE_FAILED => Err(FlexspiError::DtrReadDummyProbeFailed),
            other => Err(FlexspiError::Unknown(other)),
        }
    }
}

// FlexSPI flash driver status codes
const KSTATUS_FLEXSPI_SUCCESS: u32 = 0;
const KSTATUS_FLEXSPI_FAIL: u32 = 1;
const KSTATUS_FLEXSPI_INVALID_ARGUMENT: u32 = 4;
const KSTATUS_FLEXSPI_SEQUENCE_EXECUTION_TIMEOUT: u32 = 6000;
const KSTATUS_FLEXSPI_INVALID_SEQUENCE: u32 = 6001;
const KSTATUS_FLEXSPI_DEVICE_TIMEOUT: u32 = 6002;

const KSTATUS_FLEXSPINOR_PROGRAM_FAIL: u32 = 20100;
const KSTATUS_FLEXSPINOR_ERASE_SECTOR_FAIL: u32 = 20101;
const KSTATUS_FLEXSPINOR_ERASE_ALL_FAIL: u32 = 20102;
const KSTATUS_FLEXSPINOR_WAIT_TIMEOUT: u32 = 20103;
const KSTATUS_FLEXSPINOR_WRITE_ALIGNMENT_ERROR: u32 = 20105;
const KSTATUS_FLEXSPINOR_COMMAND_FAILURE: u32 = 20106;
const KSTATUS_FLEXSPINOR_SFDP_NOT_FOUND: u32 = 20107;
const KSTATUS_FLEXSPINOR_UNSUPPORTED_SFDP_VERSION: u32 = 20108;
const KSTATUS_FLEXSPINOR_FLASH_NOT_FOUND: u32 = 20109;
const KSTATUS_FLEXSPINOR_DTR_READ_DUMMY_PROBE_FAILED: u32 = 20110;
