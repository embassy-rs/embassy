//! This module provides the FMC driver elements used for configuring the FMC's SDRAM banks.
//!
//! Most STM32 MCUs with an FMC support two SDRAM controllers,
//! for two banks of memory-mapped external SDRAM-based memory.
//!
//! Some STM32 models only support 12 bit addressing, and some only support 16 data bits,
//! so make sure to check the datasheet and reference manual to ensure that the memory
//! configuration you're trying to use is compatible!

use embedded_hal_async::delay::DelayNs;

use core::cmp;

use crate::{
    fmc::{self, Fmc, FmcSdramBank},
    time::Hertz,
};

// Shadow the metapac values to make them more convenient to access.
pub use crate::pac::fmc::vals;

pub mod devices;

/// Specifies how many cycles to delay for reading.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReadPipeDelayCycles {
    NoDelay,
    OneCycle,
    TwoCycles,
}

impl Into<vals::Rpipe> for ReadPipeDelayCycles {
    fn into(self) -> vals::Rpipe {
        match self {
            ReadPipeDelayCycles::NoDelay => vals::Rpipe::NO_DELAY,
            ReadPipeDelayCycles::OneCycle => vals::Rpipe::CLOCKS1,
            ReadPipeDelayCycles::TwoCycles => vals::Rpipe::CLOCKS2,
        }
    }
}

/// Specifies how bits width the SDRAM data bus is.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MemoryDataWidth {
    Bits8,
    Bits16,
    Bits32,
}

impl Into<vals::Mwid> for MemoryDataWidth {
    fn into(self) -> vals::Mwid {
        match self {
            MemoryDataWidth::Bits8 => vals::Mwid::BITS8,
            MemoryDataWidth::Bits16 => vals::Mwid::BITS16,
            MemoryDataWidth::Bits32 => vals::Mwid::BITS32,
        }
    }
}

/// FMC SDRAM controller configuration values that apply to both SDRAM banks globally.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdramGlobalConfiguration {
    /// Enables burst reading.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same value.
    pub read_burst: bool,

    /// Specifies the delay in system clock cycles for reading.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same value.
    pub read_pipe_delay_cycles: ReadPipeDelayCycles,
}

/// FMC SDRAM controller timing parameters that apply to both SDRAM banks globally.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdramGlobalTiming {
    /// The maximum SDRAM clock frequency.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same timing.
    ///
    /// Check your SDRAM datasheet to determine what this should be.
    ///
    /// Most SDRAM ICs support a few different frequencies that effect timing
    /// parameters, so you may need to adjust other parameters accordingly
    /// depending on the frequency you pick. This is typically defined in Mhz
    /// in a timing parameters table in the datasheet.
    pub max_sd_clock_hz: Hertz,

    /// Period between refresh cycles in nanoseconds.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same value.
    pub refresh_period_ns: u32,

    /// Command Period (REF to REF / ACT to ACT) parameter.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same value.
    ///
    /// Auto refresh command duration.
    ///
    /// Typically defined as the symbol `tRC` in the datasheet
    /// in nanoseconds. Convert to clock cycles by dividing the
    /// value in nanoseconds by `1000/(max_sd_clock_hz)` and
    /// rounding by.
    pub row_cycle: u8,

    /// Command Period (PRE to ACT) parameter.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same value.
    ///
    /// Delay between a `PRECHARGE` command and another command.
    ///
    /// Typically defined as the symbol `tRP` in the datasheet
    /// in nanoseconds. Convert to clock cycles by dividing the
    /// value in nanoseconds by `1000/(max_sd_clock_hz)` and
    /// rounding by.
    pub row_precharge_cycles: u8,
}

/// Specifies how many address bits are used for column selection.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnBits {
    Bits8 = 0x0,
    Bits9 = 0x01,
    Bits10 = 0x02,
    Bits11 = 0x03,
}

impl Into<vals::Nc> for ColumnBits {
    fn into(self) -> vals::Nc {
        match self {
            ColumnBits::Bits8 => vals::Nc::BITS8,
            ColumnBits::Bits9 => vals::Nc::BITS9,
            ColumnBits::Bits10 => vals::Nc::BITS10,
            ColumnBits::Bits11 => vals::Nc::BITS11,
        }
    }
}

/// Specifies how many address bits are used for row selection.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RowBits {
    Bits11 = 0x0,
    Bits12 = 0x01,
    Bits13 = 0x02,
}

impl Into<vals::Nr> for RowBits {
    fn into(self) -> vals::Nr {
        match self {
            RowBits::Bits11 => vals::Nr::BITS11,
            RowBits::Bits12 => vals::Nr::BITS12,
            RowBits::Bits13 => vals::Nr::BITS13,
        }
    }
}

/// Specifies the CAS latency.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CasLatency {
    Cycle1 = 0x01,
    Cycle2 = 0x02,
    Cycle3 = 0x03,
}

impl Into<vals::Cas> for CasLatency {
    fn into(self) -> vals::Cas {
        match self {
            CasLatency::Cycle1 => vals::Cas::CLOCKS1,
            CasLatency::Cycle2 => vals::Cas::CLOCKS2,
            CasLatency::Cycle3 => vals::Cas::CLOCKS3,
        }
    }
}

/// Specifies the number of internal banks.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InternalBanks {
    TwoBanks = 0x0,
    FourBanks = 0x01,
}

impl Into<vals::Nb> for InternalBanks {
    fn into(self) -> vals::Nb {
        match self {
            InternalBanks::TwoBanks => vals::Nb::NB2,
            InternalBanks::FourBanks => vals::Nb::NB4,
        }
    }
}

/// Device configuration for an FMC SDRAM controller.
///
/// These settings need to match the settings defined in the datasheet for your
/// particular SDRAM, otherwise there will be problems!
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdramConfiguration {
    /// Number of address bits used for the column addresses.
    ///
    /// For common memory organizations this is typically:
    ///  - 16MxXX: 8 bits
    ///  - 32MxXX: 9 bits
    ///
    /// You'll typically see this written in the datasheets in
    /// the form of `A0-A8`. Memory is zero-indexed, so this
    /// example maps to 9 bits.
    pub column_bits: ColumnBits,

    /// Number of address bits used for row addresses.
    ///
    /// For most memory types this will be the full width
    /// of the address bus, typically 12 or 13 bits.
    ///
    /// You'll typically see this written in the datasheets in
    /// the form of `A0-A12`. Memory is zero-indexed, so this
    /// example maps to 13 bits.
    pub row_bits: RowBits,

    /// The bit width of the data for the SDRAM device.
    ///
    /// The STM32 FMC supports 8, 16, or 32 bits depending on the MCU.
    pub memory_data_width: MemoryDataWidth,

    /// Number of the device's internal banks.
    ///
    /// You will see this written in the SDMRAM datasheets under address parameters
    /// as something like `16Mx16x4`, where the `4` at the end is the bank count.
    pub internal_banks: InternalBanks,

    /// The CAS latency, specified in number of memory clock cycles.
    ///
    /// The value of this can depend on other memory timing parameters, and will
    /// typically be defined in a table near the begining of the datasheet.
    pub cas_latency: CasLatency,

    /// Enables write protection, disabling the FMC's ability to write data to the memory.
    pub write_protection: bool,

    /// Enables burst reading.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same value.
    pub read_burst: bool,

    /// Specifies the delay in system clock cycles for reading.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same value.
    pub read_pipe_delay_cycles: ReadPipeDelayCycles,
}

/// Device timing parameters for an FMC SDRAM controller bank.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdramTiming {
    /// Time in nanoseconds between applying a valid clock and
    /// any command other than `COMMAND INHIBIT` or `NOP`.
    pub startup_delay_ns: u32,

    /// The maximum SDRAM clock frequency.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same value.
    ///
    /// Check your SDRAM datasheet to determine what this should be.
    ///
    /// Most SDRAM ICs support a few different frequencies that effect timing
    /// parameters, so you may need to adjust other parameters accordingly
    /// depending on the frequency you pick. This is typically defined in Mhz
    /// in a timing parameters table in the datasheet.
    pub max_sd_clock_hz: Hertz,

    /// Period between refresh cycles in nanoseconds.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same value.
    pub refresh_period_ns: u32,

    /// Mode Register Program Time timing parameter.
    ///
    /// Delay in clock cycles between a `LOAD MODE`
    /// register command and an `ACTIVATE` command.
    ///
    /// Typically defined as the symbol `tMRD` in the datasheet
    /// in nanoseconds. Convert to clock cycles by dividing the
    /// value in nanoseconds by `1000/(max_sd_clock_hz)` and
    /// rounding by.
    pub mode_register_to_active_cycles: u8,

    /// Exit Self-Refresh to Active Time parameter.
    ///
    /// Delay from releasing self refresh to next command.
    ///
    /// Typically defined as the symbol `tXSR` in the datasheet
    /// in nanoseconds. Convert to clock cycles by dividing the
    /// value in nanoseconds by `1000/(max_sd_clock_hz)` and
    /// rounding by.
    pub exit_self_refresh_cycles: u8,

    /// Command Period (ACT to PRE) parameter.
    ///
    /// Delay between an `ACTIVATE` and a `PRECHARGE` command.
    ///
    /// Typically defined as the symbol `tRAS` in the datasheet
    /// in nanoseconds. Convert to clock cycles by dividing the
    /// value in nanoseconds by `1000/(max_sd_clock_hz)` and
    /// rounding by.
    pub active_to_precharge_cycles: u8,

    /// Command Period (REF to REF / ACT to ACT) parameter.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same value.
    ///
    /// Auto refresh command duration.
    ///
    /// Typically defined as the symbol `tRC` in the datasheet
    /// in nanoseconds. Convert to clock cycles by dividing the
    /// value in nanoseconds by `1000/(max_sd_clock_hz)` and
    /// rounding by.
    pub row_cycle: u8,

    /// Command Period (PRE to ACT) parameter.
    ///
    /// This setting is shared by both SDRAM banks, so when using
    /// 2 banks both SDRAM devices MUST use the same value.
    ///
    /// Delay between a `PRECHARGE` command and another command.
    ///
    /// Typically defined as the symbol `tRP` in the datasheet
    /// in nanoseconds. Convert to clock cycles by dividing the
    /// value in nanoseconds by `1000/(max_sd_clock_hz)` and
    /// rounding by.
    pub row_precharge_cycles: u8,

    /// Active Command To Read / Write Command Delay Time parameter.
    ///
    /// Delay between an ACTIVATE command and READ/WRITE command.
    ///
    /// Typically defined as the symbol `tRCD` in the datasheet
    /// in nanoseconds. Convert to clock cycles by dividing the
    /// value in nanoseconds by `1000/(max_sd_clock_hz)` and
    /// rounding by.
    pub row_to_column_cycles: u8,
}

/// Encapsulates the parameters required to initialize
/// one of the FMC SDRAM banks for a given chip.
pub trait SdramChip {
    /// Value of the FMC's SDRAM controller mode register.
    const MODE_REGISTER: u16;

    /// The SDRAM controller addressing and data configuration.
    ///
    /// These need to be derived from the SDRAM chip's datasheet.
    const CONFIG: SdramConfiguration;

    /// The SDRAM controller timing parameters.
    ///
    /// These need to be derived from the SDRAM chip's datasheet.
    const TIMING: SdramTiming;
}

/// Defines the commands that can be issued to the FMC's `SDCMR` register.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(unused)]
pub enum SdramCommand {
    /// Normal mode command.
    NormalMode,

    /// Clock Configuration Enable command.
    ClkEnable,

    /// PALL (“All Bank Precharge”) command.
    Pall,

    /// Auto-refresh command.
    ///
    /// Refresh one row of each bank.
    ///
    /// All banks must be precharged.
    AutoRefresh(u8),

    /// Load Mode Register command.
    ///
    /// Load A0 through A9 as a register to configure the DRAM chip.
    ///
    /// The u16 is used as the value of the mode register to load.
    LoadMode(u16),

    /// Self-refresh command.
    SelfRefresh,

    /// Power-down command.
    PowerDown,
}

impl SdramCommand {
    /// Returns the metapac mode for the FMC's SDCMR register.
    pub fn mode(&self) -> vals::Mode {
        match self {
            SdramCommand::NormalMode => vals::Mode::NORMAL,
            SdramCommand::ClkEnable => vals::Mode::CLOCK_CONFIGURATION_ENABLE,
            SdramCommand::Pall => vals::Mode::PALL,
            SdramCommand::AutoRefresh(_) => vals::Mode::AUTO_REFRESH_COMMAND,
            SdramCommand::LoadMode(_) => vals::Mode::LOAD_MODE_REGISTER,
            SdramCommand::SelfRefresh => vals::Mode::SELF_REFRESH_COMMAND,
            SdramCommand::PowerDown => vals::Mode::POWER_DOWN_COMMAND,
        }
    }

    /// Sends the command to the specified SDRAM FMC bank.
    pub fn send_command<T: fmc::Instance>(&self, target: FmcSdramBank) {
        // Command
        let (number_refresh, mode_reg) = match self {
            SdramCommand::NormalMode => (1, 0),
            SdramCommand::ClkEnable => (1, 0),
            SdramCommand::Pall => (1, 0),
            SdramCommand::AutoRefresh(a) => (*a, 0u16), // Autorefresh
            SdramCommand::LoadMode(mr) => (1, *mr),     // Mode register
            SdramCommand::SelfRefresh => (1, 0),
            SdramCommand::PowerDown => (1, 0),
        };

        T::regs().sdcmr().modify(|reg| {
            reg.set_mrd(mode_reg);
            reg.set_nrfs(number_refresh);
            reg.set_ctb1(target == FmcSdramBank::Bank1);
            reg.set_ctb2(target == FmcSdramBank::Bank2);
            reg.set_mode(self.mode());

            *reg
        });
    }
}

/// Manages the two FMC SDRAM banks.
#[allow(missing_debug_implementations)]
pub struct Sdram<'a, 'd, T: fmc::Instance> {
    /// Reference to the Fmc driver that was used to initialize the SDRAM.
    fmc: &'a mut Fmc<'d, T>,

    /// Configuration values that are shared by both SDRAM banks.
    ///
    /// They're persisted here to validate them against individual bank configs.
    config: SdramGlobalConfiguration,

    /// Timing parameters that are shared by both SDRAM banks.
    ///
    /// They're persisted here to validate them against individual bank timings.
    timing: SdramGlobalTiming,

    // Placeholder used to ensure that only one SDRAM controller
    // can be created for a specific bank at a time by moving them
    // out of the Fmc instance when consumed.
    sdram1: u16,
    sdram2: u16,
}

impl<'a, 'd, T: fmc::Instance> Sdram<'a, 'd, T> {
    /// Creates a new Sdram controller using the global config and
    /// timing data supplied by the specified chip implementation.
    ///
    /// This is the recommended way of initializing the SDRAM driver
    /// to ensure that the config and timing parameters are kept in
    /// sync with the intended target chip.
    fn new_for_chip<Chip: SdramChip>(fmc: &'a mut Fmc<'d, T>, _chip: &Chip) -> Self {
        // Construct the driver using globals derrived from the supplied chip.
        Self::new(
            fmc,
            SdramGlobalConfiguration {
                read_burst: Chip::CONFIG.read_burst,
                read_pipe_delay_cycles: Chip::CONFIG.read_pipe_delay_cycles,
            },
            SdramGlobalTiming {
                max_sd_clock_hz: Chip::TIMING.max_sd_clock_hz,
                refresh_period_ns: Chip::TIMING.refresh_period_ns,
                row_cycle: Chip::TIMING.row_cycle,
                row_precharge_cycles: Chip::TIMING.row_precharge_cycles,
            },
        )
    }

    /// Creates a new Sdram controller using the specified FMC peripheral.
    ///
    /// Prefer using one of the `_chip` methods instead which will help ensure
    /// that your config and timing parameter are correctly syncronized across
    /// both SDRAM banks.
    fn new(fmc: &'a mut Fmc<'d, T>, config: SdramGlobalConfiguration, timing: SdramGlobalTiming) -> Self {
        // Calcuate the required SDRAM clock.
        let (sd_clock_hz, sd_clock_divide) = {
            let fmc_source_ck_hz = <T as crate::rcc::SealedRccPeripheral>::frequency().0;
            let sd_clock_wanted = timing.max_sd_clock_hz.0;

            // Divider, round up. At least 2
            let divide: u32 = cmp::max((fmc_source_ck_hz + sd_clock_wanted - 1) / sd_clock_wanted, 2);

            let sd_clock_hz = fmc_source_ck_hz / divide;
            (
                sd_clock_hz,
                match divide {
                    1 => vals::Sdclk::DISABLED,
                    2 => vals::Sdclk::DIV2,
                    3 => vals::Sdclk::DIV3,
                    _ => panic!("Source clock too fast for required SD_CLOCK. The maximum division ratio is 3"),
                },
            )
        };

        // Configure settings shared by both SDRAM banks.
        T::regs().sdcr(0).modify(|reg| {
            reg.set_rpipe(config.read_pipe_delay_cycles.into());
            reg.set_rburst(config.read_burst);
            reg.set_sdclk(sd_clock_divide)
        });

        // Configure timing parameters shared by both SDRAM banks.
        T::regs().sdtr(0).modify(|reg| {
            reg.set_trc(timing.row_cycle - 1);
            reg.set_trp(timing.row_precharge_cycles - 1)
        });

        // Set the refresh rate counter
        // period (ns) * frequency (hz) / 10^9 = count
        let refresh_counter_top = ((timing.refresh_period_ns as u64 * sd_clock_hz as u64) / 1_000_000_000) - 20;

        assert!(
            refresh_counter_top >= 41 && refresh_counter_top < (1 << 13),
            "Impossible configuration for H7 FMC Controller"
        );

        T::regs()
            .sdrtr()
            .modify(|reg| reg.set_count(refresh_counter_top as u16));

        Self {
            fmc,
            config,
            timing,
            sdram1: 0,
            sdram2: 0,
        }
    }

    /// Consumes the SDRAM driver to create a single bank driver for the specified chip.
    pub async fn init_bank_for_chip<Chip: SdramChip, Delay: DelayNs>(
        fmc: &'a mut Fmc<'d, T>,
        bank: FmcSdramBank,
        chip: &Chip,
        delay: &mut Delay,
    ) -> *mut u32 {
        Self::new_for_chip(fmc, chip)
            .internal_init_bank_for_chip(bank, chip, delay)
            .await
    }

    /// Initializes a single SDRAM bank.
    ///
    /// Prefer using `new_bank_for_chip` to help ensure
    /// your timing and config parameters are correct.
    pub async fn init_bank<Delay: DelayNs>(
        fmc: &'a mut Fmc<'d, T>,
        bank: FmcSdramBank,
        config: SdramConfiguration,
        timing: SdramTiming,
        mode_register: u16,
        delay: &mut Delay,
    ) -> *mut u32 {
        Self::new(
            fmc,
            SdramGlobalConfiguration {
                read_burst: config.read_burst,
                read_pipe_delay_cycles: config.read_pipe_delay_cycles,
            },
            SdramGlobalTiming {
                max_sd_clock_hz: timing.max_sd_clock_hz,
                refresh_period_ns: timing.refresh_period_ns,
                row_cycle: timing.row_cycle,
                row_precharge_cycles: timing.row_precharge_cycles,
            },
        )
        .internal_init_bank(bank, config, timing, mode_register, delay)
        .await
    }

    /// Consumes the SDRAM driver to create dual SDRAM bank drivers for the specified chip.
    pub async fn init_banks_for_chip<Chip: SdramChip, Delay: DelayNs>(
        fmc: &'a mut Fmc<'d, T>,
        _chip: Chip,
        delay: &mut Delay,
    ) -> (*mut u32, *mut u32) {
        Self::init_banks(fmc, Chip::CONFIG, Chip::TIMING, Chip::MODE_REGISTER, delay).await
    }

    /// Consumes the SDRAM driver to create a dual SDRAM bank drivers.
    ///
    /// Prefer using `new_banks_for_chip` to help ensure
    /// your timing and config parameters are correct.
    pub async fn init_banks<Delay: DelayNs>(
        fmc: &'a mut Fmc<'d, T>,
        config: SdramConfiguration,
        timing: SdramTiming,
        mode_register: u16,
        delay: &mut Delay,
    ) -> (*mut u32, *mut u32) {
        let mut sdram = Self::new(
            fmc,
            SdramGlobalConfiguration {
                read_burst: config.read_burst,
                read_pipe_delay_cycles: config.read_pipe_delay_cycles,
            },
            SdramGlobalTiming {
                max_sd_clock_hz: timing.max_sd_clock_hz,
                refresh_period_ns: timing.refresh_period_ns,
                row_cycle: timing.row_cycle,
                row_precharge_cycles: timing.row_precharge_cycles,
            },
        );

        (
            sdram
                .internal_init_bank(FmcSdramBank::Bank1, config, timing, mode_register, delay)
                .await,
            sdram
                .internal_init_bank(FmcSdramBank::Bank2, config, timing, mode_register, delay)
                .await,
        )
    }

    /// Creates a bank using the config and timing
    /// parameters supplied by a chip implementation.
    async fn internal_init_bank_for_chip<Chip: SdramChip, Delay: DelayNs>(
        &mut self,
        bank: FmcSdramBank,
        _chip: &Chip,
        delay: &mut Delay,
    ) -> *mut u32 {
        // Construct the bank driver using the config and
        // timing information from the chip implementation.
        self.internal_init_bank(bank, Chip::CONFIG, Chip::TIMING, Chip::MODE_REGISTER, delay)
            .await
    }

    /// Create a driver for the specified SDRAM bank.
    async fn internal_init_bank<Delay: DelayNs>(
        &mut self,
        bank: FmcSdramBank,
        config: SdramConfiguration,
        timing: SdramTiming,
        mode_register: u16,
        delay: &mut Delay,
    ) -> *mut u32 {
        // Compare the global config and timing to the individual bank config.
        //
        // These parameters are shared by both SDRAM banks, so we do an extra
        // check here to make sure that the globals where set to the values
        // expected by the individual SDRAM banks.
        assert!(
            self.config.read_burst == config.read_burst,
            "SDRAM read_burst must be the same for both SDRAM banks"
        );
        assert!(
            self.config.read_pipe_delay_cycles == config.read_pipe_delay_cycles,
            "SDRAM read_pipe_delay_cycles must be the same for both SDRAM banks"
        );
        assert!(
            self.timing.max_sd_clock_hz == timing.max_sd_clock_hz,
            "SDRAM max_sd_clock_hz must be the same for both SDRAM banks"
        );
        assert!(
            self.timing.refresh_period_ns == timing.refresh_period_ns,
            "SDRAM refresh_period_ns must be the same for both SDRAM banks"
        );
        assert!(
            self.timing.row_cycle == timing.row_cycle,
            "SDRAM row_cycle must be the same for both SDRAM banks"
        );
        assert!(
            self.timing.row_precharge_cycles == timing.row_precharge_cycles,
            "SDRAM row_precharge_cycles must be the same for both SDRAM banks"
        );

        SdramBank::new(self.fmc, bank, config, timing, mode_register)
            .init(delay)
            .await
    }
}

/// Internal type to allow for clearly operating on a specific bank to initialize it.
#[allow(missing_debug_implementations)]
pub struct SdramBank<'a, 'd, T: fmc::Instance> {
    // Placeholder moved out of the Fmc driver instance to ensure
    // that the same bank cannot be initialized multiple times.
    bank_ref: u16,

    /// Reference to the Fmc driver that was used to initialize the SDRAM.
    fmc: &'a mut Fmc<'d, T>,

    bank: FmcSdramBank,

    config: SdramConfiguration,
    timing: SdramTiming,
    mode_register: u16,
}

impl<'a, 'd, T: fmc::Instance> SdramBank<'a, 'd, T> {
    /// Creates a new Sdram controller using the specified
    /// FMC peripheral and the specified SDRAM bank.
    fn new(
        fmc: &'a mut Fmc<'d, T>,
        bank: FmcSdramBank,
        config: SdramConfiguration,
        timing: SdramTiming,
        mode_register: u16,
    ) -> Self {
        // TODO: make a call to Fmc to get the mapped bank address in the case of re-mapped bank addressing

        // We move a value out of the FMC handle to make sure that multiple
        // SDRAM instances using the same bank can't be created.
        let bank_handle = match bank {
            FmcSdramBank::Bank1 => fmc.sdram1,
            FmcSdramBank::Bank2 => fmc.sdram2,
        };

        Self {
            bank_ref: bank_handle,
            fmc,
            bank,
            config,
            timing,
            mode_register,
        }
    }

    /// Initializes the FMC SDRAM controller corrosponding to the configured bank.
    ///
    /// Returns a raw pointer to the memory-mapped address of the SDRAM block.
    pub async fn init<Delay: DelayNs>(&mut self, delay: &mut Delay) -> *mut u32 {
        unsafe {
            // Enable memory controller AHB register access
            self.fmc.enable();

            // Program device features and timing
            self.set_features_timings(self.config, self.timing);

            // Enable memory controller
            self.fmc.memory_controller_enable();

            // Step 1: Send a clock configuration enable command
            self.send_command(SdramCommand::ClkEnable);

            // Step 2: SDRAM powerup delay
            let startup_delay_us = (self.timing.startup_delay_ns + 999) / 1000;

            delay.delay_us(startup_delay_us.try_into().unwrap()).await;

            // Step 3: Send a PALL (precharge all) command
            self.send_command(SdramCommand::Pall);

            // Step 4: Send eight auto refresh commands
            self.send_command(SdramCommand::AutoRefresh(8));

            // Step 5: Program the SDRAM's mode register
            self.send_command(SdramCommand::LoadMode(self.mode_register));
        }

        // Memory now initialised.
        //
        // Return base address of the configured FMC bank.
        //
        // This takes in the FMC bank mapping configuration
        // to ensure we're calculating the correct base address
        // in the event that the SDRAM banks have been swapped.
        self.bank.ptr(self.fmc.mapping())
    }

    /// Configure the memory device features and timings based
    /// on the provided configuration and timing parameters.
    ///
    /// # Safety
    ///
    /// Some settings are common between both banks. Calling this function
    /// mutliple times with different banks and different configurations is
    /// unsafe.
    ///
    /// For example, see RM0433 rev 7 Section 22.9.3
    async unsafe fn set_features_timings(&mut self, config: SdramConfiguration, timing: SdramTiming) {
        // Set the configuration values for the bank.
        T::regs()
            .sdcr(match self.bank {
                FmcSdramBank::Bank1 => 0,
                FmcSdramBank::Bank2 => 1,
            })
            .modify(|reg| {
                reg.set_wp(config.write_protection);
                reg.set_cas(config.cas_latency.into());
                reg.set_nb(config.internal_banks.into());
                reg.set_mwid(config.memory_data_width.into());
                reg.set_nr(config.row_bits.into());
                reg.set_nc(config.column_bits.into())
            });

        // Self refresh >= ACTIVE to PRECHARGE
        let minimum_self_refresh = timing.active_to_precharge_cycles;

        // Write recovery - Self refresh
        let write_recovery_self_refresh = minimum_self_refresh - timing.row_to_column_cycles;
        // Write recovery - WRITE command to PRECHARGE command
        let write_recovery_row_cycle = timing.row_cycle - timing.row_to_column_cycles - timing.row_precharge_cycles;
        let write_recovery = cmp::max(write_recovery_self_refresh, write_recovery_row_cycle);

        // Set the timing parameters for the bank.
        T::regs()
            .sdtr(match self.bank {
                FmcSdramBank::Bank1 => 0,
                FmcSdramBank::Bank2 => 1,
            })
            .modify(|reg| {
                reg.set_trcd(timing.row_to_column_cycles - 1);
                reg.set_twr(write_recovery - 1);
                reg.set_tras(minimum_self_refresh - 1);
                reg.set_txsr(timing.exit_self_refresh_cycles - 1);
                reg.set_tmrd(timing.mode_register_to_active_cycles - 1)
            });
    }

    /// Send a command to the SDRAM bank being driven by this driver instance.
    fn send_command(&mut self, command: SdramCommand) {
        command.send_command::<T>(self.bank);
    }
}
