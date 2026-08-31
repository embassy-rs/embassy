use super::Status;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SerialNorOptionTag {
    Config = 0x0C, // SDK vs. RM mismatch; TODO: confirm correct value and semantics.
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SerialNorOptionSize {
    Option0Only = 0,
    Option0AndOption1 = 1,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SerialNorDeviceType {
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
enum SerialNorOptionPadEncoding {
    // Encoded values for option0.query_pads / option0.cmd_pads.
    // These match the ROM option field encoding, not the literal kSerialFlash_*Pad values.
    One = 0,
    Four = 2,
    Eight = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SerialNorQuadModeSetting {
    NotConfigured = 0,
    StatusReg1Bit6 = 1,
    StatusReg2Bit1 = 2,
    StatusReg2Bit7 = 3,
    StatusReg2Bit1Via0x31 = 4,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SerialNorMiscMode {
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
enum FlexspiSerialClockFrequency {
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
enum SerialNorFlashConnection {
    SinglePortA = 0,
    Parallel = 1,
    SinglePortB = 2,
    BothPorts = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlexspiClockSource {
    // Table 60 selector values for the ROM set_clock_source API.
    NoClock = 0,
    Pll0 = 1,
    FroHf = 3,
    Pll1 = 5,
    UsbPll = 6,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlexspiClockConfigFrequency {
    // Table 61 values for the ROM config_clock API.
    MHz30 = 1,
    MHz50 = 2,
    MHz60 = 3,
    MHz75 = 4,
    MHz100 = 5,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlexspiClockConfigMode {
    Sdr = 0,
    Ddr = 1,
}

const fn pack_serial_nor_option0(
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

const fn pack_serial_nor_option1(
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
struct SerialNorConfigOption {
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
    option0: u32,
    option1: u32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlexspiOperationType {
    Command = 0,
    Config = 1,
    Write = 2,
    Read = 3,
}

impl FlexspiOperationType {
    const END: Self = Self::Read;
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
union FlashRunContext {
    B: FlashRunContextFields,
    U: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FlexspiLutSeq {
    seq_num: u8,
    seq_id: u8,
    reserved: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FlexspiDllTime {
    time_100ps: u8,
    delay_cells: u8,
}

#[repr(C)]
struct FlexspiMemConfig {
    tag: u32,
    version: u32,
    reserved0: u32,
    read_sample_clk_src: u8,
    cs_hold_time: u8,
    cs_setup_time: u8,
    column_address_width: u8,
    device_mode_cfg_enable: u8,
    device_mode_type: u8,
    wait_time_cfg_commands: u16,
    device_mode_seq: FlexspiLutSeq,
    device_mode_arg: u32,
    config_cmd_enable: u8,
    config_mode_type: [u8; 3],
    config_cmd_seqs: [FlexspiLutSeq; 3],
    reserved1: u32,
    config_cmd_args: [u32; 3],
    reserved2: u32,
    controller_misc_option: u32,
    device_type: u8,
    sflash_pad_type: u8,
    serial_clk_freq: u8,
    lut_custom_seq_enable: u8,
    reserved3: [u32; 2],
    sflash_a1_size: u32,
    sflash_a2_size: u32,
    sflash_b1_size: u32,
    sflash_b2_size: u32,
    cs_pad_setting_override: u32,
    sclk_pad_setting_override: u32,
    data_pad_setting_override: u32,
    dqs_pad_setting_override: u32,
    timeout_in_ms: u32,
    command_interval: u32,
    data_valid_time: [FlexspiDllTime; 2],
    busy_offset: u16,
    busy_bit_polarity: u16,
    lookup_table: [u32; 64],
    lut_custom_seq: [FlexspiLutSeq; 12],
    dll0_cr_val: u32,
    dll1_cr_val: u32,
    reserved4: [u32; 2],
}

#[repr(C)]
struct FlexspiNorConfig {
    mem_config: FlexspiMemConfig,
    page_size: u32,
    sector_size: u32,
    ipcmd_serial_clk_freq: u8,
    is_uniform_block_size: u8,
    is_data_order_swapped: u8,
    reserved0: [u8; 1],
    serial_nor_type: u8,
    need_exit_no_cmd_mode: u8,
    half_clk_for_non_read_cmd: u8,
    need_restore_no_cmd_mode: u8,
    block_size: u32,
    flash_state_ctx: FlashRunContext,
    reserved2: [u32; 10],
}

#[repr(C)]
struct FlexspiXfer {
    operation: FlexspiOperationType,
    base_address: u32,
    seq_id: u32,
    seq_num: u32,
    is_parallel_mode_enable: bool,
    tx_buffer: *const u32,
    tx_size: u32,
    rx_buffer: *mut u32,
    rx_size: u32,
}

#[repr(C)]
pub struct FlexspiNorFlashDriver {
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

impl FlexspiNorFlashDriver {
    fn version(&self) -> u32 {
        self.version
    }

    fn init(&self, instance: u32, cfg: *mut FlexspiNorConfig) -> FlexspiStatus {
        unsafe { (self.init)(instance, cfg) }.into()
    }

    fn page_program(&self, instance: u32, cfg: *mut FlexspiNorConfig, dst: u32, src: *const u32) -> FlexspiStatus {
        unsafe { (self.page_program)(instance, cfg, dst, src) }.into()
    }

    fn erase_all(&self, instance: u32, cfg: *mut FlexspiNorConfig) -> FlexspiStatus {
        unsafe { (self.erase_all)(instance, cfg) }.into()
    }

    fn erase(&self, instance: u32, cfg: *mut FlexspiNorConfig, start: u32, len: u32) -> FlexspiStatus {
        unsafe { (self.erase)(instance, cfg, start, len) }.into()
    }

    fn erase_sector(&self, instance: u32, cfg: *mut FlexspiNorConfig, addr: u32) -> FlexspiStatus {
        unsafe { (self.erase_sector)(instance, cfg, addr) }.into()
    }

    fn erase_block(&self, instance: u32, cfg: *mut FlexspiNorConfig, addr: u32) -> FlexspiStatus {
        unsafe { (self.erase_block)(instance, cfg, addr) }.into()
    }

    fn get_config(
        &self,
        instance: u32,
        cfg: *mut FlexspiNorConfig,
        opt: *mut SerialNorConfigOption,
    ) -> FlexspiStatus {
        unsafe { (self.get_config)(instance, cfg, opt) }.into()
    }

    fn read(
        &self,
        instance: u32,
        cfg: *mut FlexspiNorConfig,
        dst: *mut u32,
        start: u32,
        bytes: u32,
    ) -> FlexspiStatus {
        unsafe { (self.read)(instance, cfg, dst, start, bytes) }.into()
    }

    fn xfer(&self, instance: u32, xfer: *mut FlexspiXfer) -> FlexspiStatus {
        unsafe { (self.xfer)(instance, xfer) }.into()
    }

    fn update_lut(&self, instance: u32, seq_index: u32, lut_base: *const u32, num_seq: u32) -> FlexspiStatus {
        unsafe { (self.update_lut)(instance, seq_index, lut_base, num_seq) }.into()
    }

    fn set_clock_source(&self, clock_src: FlexspiClockSource) -> FlexspiStatus {
        unsafe { (self.set_clock_source)(clock_src as u32) }.into()
    }

    fn config_clock(
        &self,
        instance: u32,
        freq_option: FlexspiClockConfigFrequency,
        sample_clk_mode: FlexspiClockConfigMode,
    ) {
        unsafe { (self.config_clock)(instance, freq_option as u32, sample_clk_mode as u32) }
    }

    fn partial_program(
        &self,
        instance: u32,
        cfg: *mut FlexspiNorConfig,
        dst: u32,
        src: *const u32,
        len: u32,
    ) -> FlexspiStatus {
        unsafe { (self.partial_program)(instance, cfg, dst, src, len) }.into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlexspiStatus {
    Success,
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
}

impl From<u32> for FlexspiStatus {
    fn from(raw: u32) -> Self {
        match raw {
            super::KSTATUS_FLEXSPI_SUCCESS => Self::Success,
            super::KSTATUS_FLEXSPI_FAIL => Self::Fail,
            super::KSTATUS_FLEXSPI_INVALID_ARGUMENT => Self::InvalidArgument,
            super::KSTATUS_FLEXSPI_SEQUENCE_EXECUTION_TIMEOUT => Self::SequenceExecutionTimeout,
            super::KSTATUS_FLEXSPI_INVALID_SEQUENCE => Self::InvalidSequence,
            super::KSTATUS_FLEXSPI_DEVICE_TIMEOUT => Self::DeviceTimeout,
            super::KSTATUS_FLEXSPINOR_PROGRAM_FAIL => Self::ProgramFail,
            super::KSTATUS_FLEXSPINOR_ERASE_SECTOR_FAIL => Self::EraseSectorFail,
            super::KSTATUS_FLEXSPINOR_ERASE_ALL_FAIL => Self::EraseAllFail,
            super::KSTATUS_FLEXSPINOR_WAIT_TIMEOUT => Self::WaitTimeout,
            super::KSTATUS_FLEXSPINOR_WRITE_ALIGNMENT_ERROR => Self::WriteAlignmentError,
            super::KSTATUS_FLEXSPINOR_COMMAND_FAILURE => Self::CommandFailure,
            super::KSTATUS_FLEXSPINOR_SFDP_NOT_FOUND => Self::SfdpNotFound,
            super::KSTATUS_FLEXSPINOR_UNSUPPORTED_SFDP_VERSION => Self::UnsupportedSfdpVersion,
            super::KSTATUS_FLEXSPINOR_FLASH_NOT_FOUND => Self::FlashNotFound,
            super::KSTATUS_FLEXSPINOR_DTR_READ_DUMMY_PROBE_FAILED => Self::DtrReadDummyProbeFailed,
            other => Self::Unknown(other),
        }
    }
}
