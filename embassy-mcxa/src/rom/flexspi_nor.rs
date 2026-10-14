use core::sync::atomic::{AtomicBool, Ordering};

use super::Status;
use crate::Peri;
use crate::flexspi::{Data0Pin, Data1Pin, Data2Pin, Data3Pin, DqsPin, Port, SclkPin, Ss0Pin, SsPin};
use crate::gpio::AnyPin;
use crate::peripherals::FLEXSPI0;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum SerialNorOptionTag {
    Config = 0x0C, // SDK vs. RM mismatch; TODO: confirm correct value and semantics.
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SerialNorOptionSize {
    Option0Only = 0,
    Option0AndOption1 = 1,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SerialNorOptionPadEncoding {
    // Encoded values for option0.query_pads / option0.cmd_pads.
    // These match the ROM option field encoding, not the literal kSerialFlash_*Pad values.
    One = 0,
    Four = 2,
    Eight = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SerialNorQuadModeSetting {
    NotConfigured = 0,
    StatusReg1Bit6 = 1,
    StatusReg2Bit1 = 2,
    StatusReg2Bit7 = 3,
    StatusReg2Bit1Via0x31 = 4,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SerialNorFlashConnection {
    SinglePortA = 0,
    Parallel = 1,
    SinglePortB = 2,
    BothPorts = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// MCXA5xx functional-clock mux encodings used by the ROM clock selector.
///
/// MCXA577 ROM 3.2.0 writes the argument directly to `MRCC_FLEXSPI0_CLKSEL`;
/// use the encodings in MCXA5xx RM Rev. 1, section 22.5.2.44. Table 60 in
/// section 5.8.12 lists incompatible generic selectors: its FRO_HF value 3
/// was observed selecting unsupported mux 3 and stopping the FlexSPI clock.
/// Verify this ROM behavior when targeting another revision.
pub enum FlexspiClockSource {
    /// Disable the functional clock. Not valid for options-based construction.
    NoClock = 7,
    /// Gated high-frequency FRO.
    FroHf = 1,
    /// USB PLL phase-fractional-divider clock, not the undivided USB PLL.
    UsbPfd = 4,
    /// Undivided PLL1 clock.
    Pll1 = 6,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

/// Pack the raw ROM option1 ABI fields, truncating each integer to its field width.
///
/// The MCXA5xx reference manual, Rev. 1, Table 56, identifies bits 27:24
/// (`drive_strength`) as reserved and ineffective on this device. Keep this
/// argument zero unless a vendor correction for your device/ROM says otherwise;
/// packing it does not establish control of the pads' electrical drive strength.
pub const fn pack_serial_nor_option1(
    flash_connection: SerialNorFlashConnection,
    drive_strength: u32,
    dqs_pinmux_group: u32,
    pinmux_group: u32,
    status_override: u32,
    dummy_cycles: u32,
) -> u32 {
    ((flash_connection as u32) << 28)
        | ((drive_strength & 0xF) << 24)
        | ((dqs_pinmux_group & 0xF) << 20)
        | ((pinmux_group & 0xF) << 16)
        | ((status_override & 0xFF) << 8)
        | (dummy_cycles & 0xFF)
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlexspiOperationType {
    Command = 0,
    Config = 1,
    Write = 2,
    Read = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlexspiLutSeq {
    pub seq_num: u8,
    pub seq_id: u8,
    pub reserved: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlexspiDllTime {
    pub time_100ps: u8,
    pub delay_cells: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

/// Exclusive borrows of the seven pins used by one quad NOR flash port.
///
/// Pass reborrowed pins to [`Self::new`] to recover their use after dropping the
/// ROM handle. Constructing this bundle does not configure or mux the pins;
/// ROM does that during probing/initialization. Constructors verify the selected
/// port and default pin group before probing and validate the final FCB before
/// initialization. The safe wrapper supports one CS0, at-most-quad device on
/// port A or B. Alternate pin groups, wider/multiple-port configurations and
/// additional chip selects require pins this bundle does not own and are rejected
/// with [`FlexspiError::InvalidArgument`]. Other GPIO/native-driver use is excluded
/// by the borrows for the lifetime of the returned ROM handle.
pub struct FlexspiNorPins<'d> {
    _pins: [Peri<'d, AnyPin>; 7],
    port: FlexspiNorPort,
}

impl<'d> FlexspiNorPins<'d> {
    /// Borrow the CS0, clock, DQS and four data pins of a single FlexSPI port.
    pub fn new<P: Port>(
        ss: Peri<'d, impl Ss0Pin<FLEXSPI0, P> + SsPin<FLEXSPI0, P>>,
        sclk: Peri<'d, impl SclkPin<FLEXSPI0, P>>,
        dqs: Peri<'d, impl DqsPin<FLEXSPI0, P>>,
        data0: Peri<'d, impl Data0Pin<FLEXSPI0, P>>,
        data1: Peri<'d, impl Data1Pin<FLEXSPI0, P>>,
        data2: Peri<'d, impl Data2Pin<FLEXSPI0, P>>,
        data3: Peri<'d, impl Data3Pin<FLEXSPI0, P>>,
    ) -> Self {
        let port = if ss.chip_index() == 0 {
            FlexspiNorPort::A
        } else {
            FlexspiNorPort::B
        };
        Self {
            _pins: [
                ss.into(),
                sclk.into(),
                dqs.into(),
                data0.into(),
                data1.into(),
                data2.into(),
                data3.into(),
            ],
            port,
        }
    }

    fn validate_mapping(&self) -> Result<(), FlexspiError> {
        let expected = match self.port {
            FlexspiNorPort::A => [0, 7, 6, 8, 9, 10, 11],
            FlexspiNorPort::B => [17, 16, 1, 15, 14, 13, 12],
        };
        if self
            ._pins
            .iter()
            .zip(expected)
            .any(|(pin, index)| pin.port_index() != 3 || pin.pin_index() != index)
        {
            return Err(FlexspiError::InvalidArgument);
        }
        Ok(())
    }
}

/// ROM NOR driver retaining exclusive controller and pin borrows.
///
/// Drop releases the software claim and borrows, not the controller/flash state.
/// Unsafe code creating mapped-flash references or XIP code must ensure those
/// objects are not invalidated by any probe, erase or program operation.
pub struct FlexspiNor<'d> {
    vtable: &'static FlexspiNorVtable,
    config: FlexspiNorConfig,
    _claim: FlexspiClaim,
    _peri: Peri<'d, FLEXSPI0>,
    _pins: FlexspiNorPins<'d>,
}

const FLEXSPI_INSTANCE: u32 = 0;
static TAKEN: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, PartialEq, Eq)]
enum FlexspiNorPort {
    A,
    B,
}

impl FlexspiNorPort {
    fn validate_options(self, option: SerialNorConfigOption) -> Result<(), FlexspiError> {
        let connection = if (option.option0 >> 24) & 0xf == 0 {
            0
        } else {
            // Only the default pin group is owned by this wrapper.
            if option.option1 & 0x00ff_0000 != 0 {
                return Err(FlexspiError::InvalidArgument);
            }
            option.option1 >> 28
        };
        let expected = match self {
            Self::A => 0,
            Self::B => 2,
        };
        // A seven-pin owner cannot cover HyperFlash/Octal or multiple ports.
        if option.option0 >> 28 != 0xc
            || ((option.option0 >> 24) & 0xf) > 1
            || ((option.option0 >> 20) & 0xf) > 1
            || !matches!((option.option0 >> 16) & 0xf, 0 | 2)
            || !matches!((option.option0 >> 12) & 0xf, 0 | 2)
            || !matches!((option.option0 >> 4) & 0xf, 0 | 1 | 5 | 6 | 8)
            || connection != expected
        {
            return Err(FlexspiError::InvalidArgument);
        }
        Ok(())
    }

    fn validate_config(self, config: &FlexspiNorConfig) -> Result<(), FlexspiError> {
        let mem = &config.mem_config;
        let valid_geometry = match self {
            Self::A => mem.sflash_a1_size != 0 && mem.sflash_b1_size == 0,
            Self::B => mem.sflash_b1_size != 0 && mem.sflash_a1_size == 0,
        };
        if !valid_geometry
            || mem.sflash_a2_size != 0
            || mem.sflash_b2_size != 0
            || mem.device_type != 1
            || !matches!(mem.sflash_pad_type, 1 | 2 | 4)
            // Differential/second clocks and parallel mode need extra pins.
            || mem.controller_misc_option & 0x7 != 0
        {
            return Err(FlexspiError::InvalidArgument);
        }
        Ok(())
    }
}

// Own the claim from acquisition onward, including failed initialization and
// unwinding caller customization. A successful constructor moves it into the handle.
struct FlexspiClaim;

impl FlexspiClaim {
    fn acquire() -> Result<Self, FlexspiError> {
        TAKEN
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| FlexspiError::Unavailable)?;
        Ok(Self)
    }
}

impl Drop for FlexspiClaim {
    fn drop(&mut self) {
        TAKEN.store(false, Ordering::Release);
    }
}

fn initialization_result(status: Status, _phase: &str) -> Result<(), FlexspiError> {
    #[cfg(feature = "defmt")]
    if status.0 != KSTATUS_FLEXSPI_SUCCESS {
        defmt::debug!("FlexSPI ROM {} failed: status={=u32}", _phase, status.0);
    }
    status.into()
}

impl<'d> FlexspiNor<'d> {
    pub(super) fn new(
        vtable: &'static FlexspiNorVtable,
        peri: Peri<'d, FLEXSPI0>,
        pins: FlexspiNorPins<'d>,
        mut config: FlexspiNorConfig,
    ) -> Result<Self, FlexspiError> {
        pins.validate_mapping()?;
        pins.port.validate_config(&config)?;
        let claim = FlexspiClaim::acquire()?;

        initialization_result(unsafe { (vtable.init)(FLEXSPI_INSTANCE, &raw mut config) }, "init")?;
        Ok(Self {
            vtable,
            config,
            _claim: claim,
            _peri: peri,
            _pins: pins,
        })
    }

    /// Probe the attached serial NOR with `option`, let the caller override a
    /// few board-specific fields on the ROM-confirmed config, then init.
    ///
    /// DDR/DTR dummy-cycle auto-probing can write test patterns at
    /// flash offset 0x200, first 0x1000 bytes are usually kept
    /// reserved.
    ///
    /// This mirrors the real ROM flow: `get_config` fills in a working
    /// [`FlexspiNorConfig`] (read LUT, dummy cycles, page/sector sizes, …)
    /// derived from the device's SFDP tables. The `custom_fcb` closure then runs on
    /// that confirmed config so the caller can override only what the ROM can't
    /// infer from the part itself ( e.g. the port geometry (`sflash_a/b_size`) or
    /// the read sample-clock source) before the config is written back via
    /// `init`.
    ///
    /// Use this over [`FlexspiNor::new`] when you need the SFDP-derived
    /// config but must adjust connection/clock fields: `new` takes a
    /// fully-formed config and inits immediately, with no modification in between.
    pub(super) fn new_with_options(
        vtable: &'static FlexspiNorVtable,
        peri: Peri<'d, FLEXSPI0>,
        pins: FlexspiNorPins<'d>,
        option: SerialNorConfigOption,
        custom_fcb: impl FnOnce(&mut FlexspiNorConfig),
    ) -> Result<Self, FlexspiError> {
        Self::new_with_options_and_clock(vtable, peri, pins, option, None, custom_fcb)
    }

    pub(super) fn new_with_options_and_clock(
        vtable: &'static FlexspiNorVtable,
        peri: Peri<'d, FLEXSPI0>,
        pins: FlexspiNorPins<'d>,
        mut option: SerialNorConfigOption,
        clock_source: Option<FlexspiClockSource>,
        custom_fcb: impl FnOnce(&mut FlexspiNorConfig),
    ) -> Result<Self, FlexspiError> {
        pins.validate_mapping()?;
        pins.port.validate_options(option)?;
        if clock_source == Some(FlexspiClockSource::NoClock) {
            return Err(FlexspiError::InvalidArgument);
        }
        let claim = FlexspiClaim::acquire()?;

        // The caller has prepared a running upstream source and exclusive, idle
        // controller access. Select it before the probe, within the same claim.
        if let Some(source) = clock_source {
            #[cfg(feature = "defmt")]
            defmt::trace!("FlexSPI ROM set_clock_source begin: source={=u32}", source as u32);
            initialization_result(unsafe { (vtable.set_clock_source)(source as u32) }, "set_clock_source")?;
            #[cfg(feature = "defmt")]
            defmt::trace!("FlexSPI ROM set_clock_source complete");
        }

        // SAFETY: Every field is an integer, an array/struct of integers, or an
        // integer-backed union, so the all-zero representation is valid. ROM
        // populates the configuration before it is used for initialization.
        let mut config: FlexspiNorConfig = unsafe { core::mem::zeroed() };

        // Step 1: SFDP probe where the ROM populates `config` (read LUT, sizes, …).
        #[cfg(feature = "defmt")]
        defmt::trace!("FlexSPI ROM get_config begin");
        initialization_result(
            unsafe { (vtable.get_config)(FLEXSPI_INSTANCE, &raw mut config, &raw mut option) },
            "get_config",
        )?;
        #[cfg(feature = "defmt")]
        defmt::trace!("FlexSPI ROM get_config complete");

        // Step 2: let the caller override board-specific fields in place.
        custom_fcb(&mut config);
        pins.port.validate_config(&config)?;

        // Step 3: write the tweaked config back via init.
        #[cfg(feature = "defmt")]
        defmt::trace!("FlexSPI ROM init begin");
        initialization_result(unsafe { (vtable.init)(FLEXSPI_INSTANCE, &raw mut config) }, "init")?;
        #[cfg(feature = "defmt")]
        defmt::trace!("FlexSPI ROM init complete");

        Ok(Self {
            vtable,
            config,
            _claim: claim,
            _peri: peri,
            _pins: pins,
        })
    }

    /// ROM NOR driver version.
    pub fn version(&self) -> u32 {
        self.vtable.version
    }

    /// Program one configured flash page from a raw, word-aligned source.
    ///
    /// # Safety
    ///
    /// `src` must be valid and initialized for reads of the configured page size
    /// for the entire synchronous call. It must be 32-bit aligned and must not
    /// refer to flash affected by the operation. `dst` must satisfy the device's
    /// page-program address requirements. The configuration/LUT must still
    /// describe a page program, not an operation reading a larger source buffer.
    pub unsafe fn page_program(&mut self, dst: u32, src: *const u32) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.page_program)(FLEXSPI_INSTANCE, &raw mut self.config, dst, src) }.into()
    }

    /// Erase the entire configured flash.
    pub fn erase_all(&mut self) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.erase_all)(FLEXSPI_INSTANCE, &raw mut self.config) }.into()
    }

    /// Erase a flash region, using ROM/device alignment rules.
    pub fn erase(&mut self, start: u32, len: u32) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.erase)(FLEXSPI_INSTANCE, &raw mut self.config, start, len) }.into()
    }

    /// Erase the sector at the given flash offset.
    pub fn erase_sector(&mut self, addr: u32) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.erase_sector)(FLEXSPI_INSTANCE, &raw mut self.config, addr) }.into()
    }

    /// Erase the block at the given flash offset.
    pub fn erase_block(&mut self, addr: u32) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.erase_block)(FLEXSPI_INSTANCE, &raw mut self.config, addr) }.into()
    }

    /// Read `byte_len` bytes at the flash offset `start` into the prefix of `dst`.
    ///
    /// Word storage supplies the ROM's 32-bit destination alignment. Returns
    /// [`FlexspiError::BufferTooSmall`] before entering ROM if the byte count
    /// exceeds its capacity. Unrequested bytes are not part of the transfer.
    ///
    /// MCXA577 ROM 3.2.0 accepts non-word/non-page-aligned counts, as verified on
    /// hardware. This differs from the page-alignment statement in MCXA5xx RM
    /// Rev. 1, Table 57; verify support when targeting a different ROM revision.
    /// Zero-length reads are passed to ROM (3.2.0 returns `InvalidArgument`).
    pub fn read(&mut self, dst: &mut [u32], start: u32, byte_len: u32) -> Result<(), FlexspiError> {
        if byte_len as usize > core::mem::size_of_val(dst) {
            return Err(FlexspiError::BufferTooSmall);
        }
        unsafe {
            (self.vtable.read)(
                FLEXSPI_INSTANCE,
                &raw mut self.config,
                dst.as_mut_ptr(),
                start,
                byte_len,
            )
        }
        .into()
    }

    /// Execute a raw ROM transfer using the installed LUT.
    ///
    /// # Safety
    ///
    /// All buffers accessed by the operation and its LUT must be 32-bit aligned
    /// and valid for the full accessed extent throughout the synchronous call.
    /// TX storage must be initialized and readable; RX storage must be writable
    /// and exclusively accessible. Pointer/size pairs must stay within their
    /// allocations and must not alias storage/flash changed by the operation.
    /// The sequence index, count, and LUT must describe the requested operation;
    /// descriptor sizes alone do not bound a configuration command's LUT access.
    pub unsafe fn xfer(&mut self, xfer: &mut FlexspiXfer) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.xfer)(FLEXSPI_INSTANCE, xfer) }.into()
    }

    /// Install raw lookup-table sequences.
    ///
    /// # Safety
    ///
    /// `lut_base` must be 32-bit aligned and valid for initialized reads of all
    /// entries consumed by ROM throughout this call. Providing `4 * num_seq`
    /// words covers the complete four-word sequences. The destination range
    /// must fit the controller's 16-sequence LUT. Installed sequences must keep
    /// the buffer-access assumptions of subsequent safe reads/programs valid.
    pub unsafe fn update_lut(
        &mut self,
        seq_index: u32,
        lut_base: *const u32,
        num_seq: u32,
    ) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.update_lut)(FLEXSPI_INSTANCE, seq_index, lut_base, num_seq) }.into()
    }

    /// Select a prepared upstream clock after initialization.
    ///
    /// This does not start the source or adjust divisors. For selection before
    /// SFDP probing, use [`super::RomApi::flex_spi_with_options_and_clock`].
    pub fn set_clock_source(&mut self, clock_src: FlexspiClockSource) -> Result<(), FlexspiError> {
        unsafe { (self.vtable.set_clock_source)(clock_src as u32) }.into()
    }

    /// Configure the ROM serial-clock frequency and sample mode.
    pub fn config_clock(&mut self, freq_option: FlexspiClockConfigFrequency, sample_clk_mode: FlexspiClockConfigMode) {
        unsafe { (self.vtable.config_clock)(FLEXSPI_INSTANCE, freq_option as u32, sample_clk_mode as u32) }
    }

    /// Program `byte_len` bytes from the prefix of `src` at flash offset `dst`.
    ///
    /// Returns [`FlexspiError::BufferTooSmall`] before entering ROM if the count
    /// exceeds the source's byte capacity. Word storage supplies alignment; the
    /// count need not be a whole number of words on MCXA577 ROM 3.2.0. Observe the
    /// flash device's programming/page restrictions and erase requirements.
    /// Zero-length programs are passed to ROM (3.2.0 returns success).
    pub fn partial_program(&mut self, dst: u32, src: &[u32], byte_len: u32) -> Result<(), FlexspiError> {
        if byte_len as usize > core::mem::size_of_val(src) {
            return Err(FlexspiError::BufferTooSmall);
        }
        unsafe { (self.vtable.partial_program)(FLEXSPI_INSTANCE, &raw mut self.config, dst, src.as_ptr(), byte_len) }
            .into()
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
    BufferTooSmall,
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
