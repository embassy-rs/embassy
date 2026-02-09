//! Management of external SDRAM through the STM32 FMC peripheral
//!
//! Most STM32 MCUs with an FMC support two SDRAM controllers,
//! for two banks of memory-mapped external SDRAM-based memory.
//!
//! Some STM32 models only support 12 bit addressing, and some only support 16 data bits,
//! so make sure to check the datasheet and reference manual to ensure that the memory
//! configuration you're trying to use is compatible!

// Originally implemented by the `stm32-fmc` crate by Richard Meadows in 2019 under the
// MIT license, improved and rolled into Embassy by Kat Mitchell (northernpaws).

// Convenience SDRAM chip definitions with pre-populated
// config values and timing parameters.
pub mod devices;

use embassy_time::Timer;

use core::cmp;

use crate::{
    fmc::{self, Fmc, FmcSdramBank},
    time::Hertz,
};

// Shadow the metapac values to make them more convenient to access.
pub use crate::pac::fmc::vals;

/// Specifies how many cycles to delay for reading.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReadPipeDelayCycles {
    /// No delay on reading.
    NoDelay,
    /// One clock cycle delay on reading.
    OneCycle,
    /// Two clock cycles delay on reading.
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
    /// 8-bit wide data bus.
    Bits8,
    /// 16-bit wide data bus.
    Bits16,
    /// 32-bit wide data bus.
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
    /// 8 column address bits.
    Bits8 = 0x0,
    /// 9 column address bits.
    Bits9 = 0x01,
    /// 10 column address bits.
    Bits10 = 0x02,
    /// 11 column address bits.
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
    /// 11 row address bits.
    Bits11 = 0x0,
    /// 12 row address bits.
    Bits12 = 0x01,
    /// 13 row address bits.
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
    /// 1 clock cycle latency.
    Cycle1 = 0x01,
    /// 2 clock cycle latency.
    Cycle2 = 0x02,
    /// 3 clock cycle latency.
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
    /// Two internal banks.
    TwoBanks = 0x0,
    /// Four internal banks.
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

        #[cfg(fmc_v4)]
        T::regs().sdram().sdcmr().modify(|reg| {
            reg.set_mrd(mode_reg);
            reg.set_nrfs(number_refresh);
            reg.set_ctb(0, target == FmcSdramBank::Bank1);
            reg.set_ctb(1, target == FmcSdramBank::Bank2);
            reg.set_mode(self.mode())
        });

        #[cfg(any(fmc_v1x3, fmc_v2x1, fmc_v3x1))]
        T::regs().sdcmr().modify(|reg| {
            reg.set_mrd(mode_reg);
            reg.set_nrfs(number_refresh);
            reg.set_ctb1(target == FmcSdramBank::Bank1);
            reg.set_ctb2(target == FmcSdramBank::Bank2);
            reg.set_mode(self.mode())
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

        #[cfg(any(fmc_v1x3, fmc_v2x1, fmc_v3x1))]
        let regs = T::regs();
        #[cfg(fmc_v4)]
        let regs = T::regs().sdram();

        // Configure settings shared by both SDRAM banks.
        regs.sdcr(0).modify(|reg| {
            reg.set_rpipe(config.read_pipe_delay_cycles.into());
            reg.set_rburst(config.read_burst);
            reg.set_sdclk(sd_clock_divide)
        });

        // Configure timing parameters shared by both SDRAM banks.
        regs.sdtr(0).modify(|reg| {
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

        regs.sdrtr().modify(|reg| reg.set_count(refresh_counter_top as u16));

        Self { fmc, config, timing }
    }

    /// Consumes the SDRAM driver to create a single bank driver for the specified chip.
    async fn init_bank_for_chip<Chip: SdramChip>(fmc: &'a mut Fmc<'d, T>, bank: FmcSdramBank, chip: &Chip) -> *mut u32 {
        Self::new_for_chip(fmc, chip)
            .internal_init_bank_for_chip(bank, chip)
            .await
    }

    /// Initializes a single SDRAM bank.
    ///
    /// Prefer using `new_bank_for_chip` to help ensure
    /// your timing and config parameters are correct.
    async fn init_bank(
        fmc: &'a mut Fmc<'d, T>,
        bank: FmcSdramBank,
        config: SdramConfiguration,
        timing: SdramTiming,
        mode_register: u16,
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
        .internal_init_bank(bank, config, timing, mode_register)
        .await
    }

    /// Consumes the SDRAM driver to create dual SDRAM bank drivers for the specified chip.
    async fn init_banks_for_chip<Chip: SdramChip>(fmc: &'a mut Fmc<'d, T>, _chip: Chip) -> (*mut u32, *mut u32) {
        Self::init_banks(fmc, Chip::CONFIG, Chip::TIMING, Chip::MODE_REGISTER).await
    }

    /// Consumes the SDRAM driver to create a dual SDRAM bank drivers.
    ///
    /// Prefer using `new_banks_for_chip` to help ensure
    /// your timing and config parameters are correct.
    pub async fn init_banks(
        fmc: &'a mut Fmc<'d, T>,
        config: SdramConfiguration,
        timing: SdramTiming,
        mode_register: u16,
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
                .internal_init_bank(FmcSdramBank::Bank1, config, timing, mode_register)
                .await,
            sdram
                .internal_init_bank(FmcSdramBank::Bank2, config, timing, mode_register)
                .await,
        )
    }

    /// Creates a bank using the config and timing
    /// parameters supplied by a chip implementation.
    async fn internal_init_bank_for_chip<Chip: SdramChip>(&mut self, bank: FmcSdramBank, _chip: &Chip) -> *mut u32 {
        // Construct the bank driver using the config and
        // timing information from the chip implementation.
        self.internal_init_bank(bank, Chip::CONFIG, Chip::TIMING, Chip::MODE_REGISTER)
            .await
    }

    /// Create a driver for the specified SDRAM bank.
    async fn internal_init_bank(
        &mut self,
        bank: FmcSdramBank,
        config: SdramConfiguration,
        timing: SdramTiming,
        mode_register: u16,
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
            .init()
            .await
    }
}

/// Internal type to allow for clearly operating on a specific bank to initialize it.
#[allow(missing_debug_implementations)]
pub struct SdramBank<'a, 'd, T: fmc::Instance> {
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
        Self {
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
    pub async fn init(&mut self) -> *mut u32 {
        unsafe {
            // Ensure that the FMC clock is disabled
            // before adjusting the timing registers.
            self.fmc.memory_controller_disable();

            // Program the configured device features and timing.
            self.set_features_timings();

            // Enable memory controller/FMC clock.
            self.fmc.memory_controller_enable();

            // Step 1: Send a clock configuration enable command
            self.send_command(SdramCommand::ClkEnable);

            // Step 2: SDRAM powerup delay
            let startup_delay_us = (self.timing.startup_delay_ns + 999) / 1000;
            Timer::after_micros(startup_delay_us as u64).await;

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
        self.fmc.sdram_ptr(self.bank)
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
    unsafe fn set_features_timings(&mut self) {
        #[cfg(any(fmc_v1x3, fmc_v2x1, fmc_v3x1))]
        let regs = T::regs();
        #[cfg(fmc_v4)]
        let regs = T::regs().sdram();

        // Set the configuration values for the bank.
        regs.sdcr(match self.bank {
            FmcSdramBank::Bank1 => 0,
            FmcSdramBank::Bank2 => 1,
        })
        .modify(|reg| {
            reg.set_wp(self.config.write_protection);
            reg.set_cas(self.config.cas_latency.into());
            reg.set_nb(self.config.internal_banks.into());
            reg.set_mwid(self.config.memory_data_width.into());
            reg.set_nr(self.config.row_bits.into());
            reg.set_nc(self.config.column_bits.into())
        });

        // Self refresh >= ACTIVE to PRECHARGE
        let minimum_self_refresh = self.timing.active_to_precharge_cycles;

        // Write recovery - Self refresh
        let write_recovery_self_refresh = minimum_self_refresh - self.timing.row_to_column_cycles;
        // Write recovery - WRITE command to PRECHARGE command
        let write_recovery_row_cycle =
            self.timing.row_cycle - self.timing.row_to_column_cycles - self.timing.row_precharge_cycles;
        let write_recovery = cmp::max(write_recovery_self_refresh, write_recovery_row_cycle);

        // Set the timing parameters for the bank.
        regs.sdtr(match self.bank {
            FmcSdramBank::Bank1 => 0,
            FmcSdramBank::Bank2 => 1,
        })
        .modify(|reg| {
            reg.set_trcd(self.timing.row_to_column_cycles - 1);
            reg.set_twr(write_recovery - 1);
            reg.set_tras(minimum_self_refresh - 1);
            reg.set_txsr(self.timing.exit_self_refresh_cycles - 1);
            reg.set_tmrd(self.timing.mode_register_to_active_cycles - 1)
        });
    }

    /// Send a command to the SDRAM bank being driven by this driver instance.
    fn send_command(&mut self, command: SdramCommand) {
        command.send_command::<T>(self.bank);
    }
}

macro_rules! config_pins {
    ($($pin:ident),*) => {
                $(
            set_as_af!($pin, AfType::output_pull(OutputType::PushPull, Speed::VeryHigh, Pull::Up));
        )*
    };
}

macro_rules! fmc_sdram_init {
    ($name:ident: (
        bank: $bank:expr,
        addr: [$(($addr_pin_name:ident: $addr_signal:ident)),*],
        ba: [$(($ba_pin_name:ident: $ba_signal:ident)),*],
        d: [$(($d_pin_name:ident: $d_signal:ident)),*],
        nbl: [$(($nbl_pin_name:ident: $nbl_signal:ident)),*],
        ctrl: [$(($ctrl_pin_name:ident: $ctrl_signal:ident)),*]
    )) => {
        /// Create a new FMC SDRAM driver for the specified chip and bank.
        pub async fn $name<CHIP: SdramChip>(
            fmc: &mut super::Fmc<'d, T>,
            $($addr_pin_name: Peri<'d, impl $addr_signal<T>>),*,
            $($ba_pin_name: Peri<'d, impl $ba_signal<T>>),*,
            $($d_pin_name: Peri<'d, impl $d_signal<T>>),*,
            $($nbl_pin_name: Peri<'d, impl $nbl_signal<T>>),*,
            $($ctrl_pin_name: Peri<'d, impl $ctrl_signal<T>>),*,
            chip: &CHIP,
        ) -> *mut u32 {
            // Ensure that the pins being used are configured
            // in their alternate function for FMC use.
            critical_section::with(|_| {
                config_pins!(
                    $($addr_pin_name),*,
                    $($ba_pin_name),*,
                    $($d_pin_name),*,
                    $($nbl_pin_name),*,
                    $($ctrl_pin_name),*
                );
            });

            Sdram::init_bank_for_chip(fmc, $bank, chip).await
        }
    };
}

macro_rules! fmc_sdram_init_dual {
    ($name:ident: (
        addr: [$(($addr_pin_name:ident: $addr_signal:ident)),*],
        ba: [$(($ba_pin_name:ident: $ba_signal:ident)),*],
        d: [$(($d_pin_name:ident: $d_signal:ident)),*],
        nbl: [$(($nbl_pin_name:ident: $nbl_signal:ident)),*],
        ctrl: [$(($ctrl_pin_name:ident: $ctrl_signal:ident)),*]
    )) => {
        /// Create a new FMC SDRAM driver for the specified chip and bank.
        pub async fn $name<CHIP: SdramChip>(
            fmc: &mut super::Fmc<'d, T>,
            $($addr_pin_name: Peri<'d, impl $addr_signal<T>>),*,
            $($ba_pin_name: Peri<'d, impl $ba_signal<T>>),*,
            $($d_pin_name: Peri<'d, impl $d_signal<T>>),*,
            $($nbl_pin_name: Peri<'d, impl $nbl_signal<T>>),*,
            $($ctrl_pin_name: Peri<'d, impl $ctrl_signal<T>>),*,
            chip: CHIP,
        ) -> (*mut u32, *mut u32) {
            // Ensure that the pins being used are configured
            // in their alternate function for FMC use.
            critical_section::with(|_| {
                config_pins!(
                    $($addr_pin_name),*,
                    $($ba_pin_name),*,
                    $($d_pin_name),*,
                    $($nbl_pin_name),*,
                    $($ctrl_pin_name),*
                );
            });

            Sdram::init_banks_for_chip(fmc, chip).await
        }
    };
}

use super::{
    A0Pin, A1Pin, A2Pin, A3Pin, A4Pin, A5Pin, A6Pin, A7Pin, A8Pin, A9Pin, A10Pin, A11Pin, A12Pin, BA0Pin, BA1Pin,
    D0Pin, D1Pin, D2Pin, D3Pin, D4Pin, D5Pin, D6Pin, D7Pin, D8Pin, D9Pin, D10Pin, D11Pin, D12Pin, D13Pin, D14Pin,
    D15Pin, D16Pin, D17Pin, D18Pin, D19Pin, D20Pin, D21Pin, D22Pin, D23Pin, D24Pin, D25Pin, D26Pin, D27Pin, D28Pin,
    D29Pin, D30Pin, D31Pin, NBL0Pin, NBL1Pin, NBL2Pin, NBL3Pin, SDCKE0Pin, SDCKE1Pin, SDCLKPin, SDNCASPin, SDNE0Pin,
    SDNE1Pin, SDNRASPin, SDNWEPin,
};

use crate::{
    Peri,
    gpio::{AfType, OutputType, Pull, Speed},
};

/// Initializers and constructors for settings pins to their required
/// alternate functions for FMC use, and returning a pointer to the
/// initialized SDRAM memory address.
impl<'a, 'd, T: super::Instance> Sdram<'a, 'd, T> {
    fmc_sdram_init!(init_sdram_a12bits_d16bits_4banks_bank1: (
        bank: FmcSdramBank::Bank1,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin)
        ],
        ctrl: [
            (sdcke: SDCKE0Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE0Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_init!(init_sdram_a12bits_d16bits_4banks_bank2: (
        bank: FmcSdramBank::Bank2,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin)
        ],
        ctrl: [
            (sdcke: SDCKE1Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE1Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_init_dual!(init_sdram_a12bits_d16bits_4banks_dual: (
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin)
        ],
        ctrl: [
            (sdcke0: SDCKE0Pin), (sdcke1: SDCKE1Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne0: SDNE0Pin), (sdne1: SDNE1Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_init!(init_sdram_a12bits_d32bits_4banks_bank1: (
        bank: FmcSdramBank::Bank1,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin),
            (d16: D16Pin), (d17: D17Pin), (d18: D18Pin), (d19: D19Pin), (d20: D20Pin), (d21: D21Pin), (d22: D22Pin), (d23: D23Pin),
            (d24: D24Pin), (d25: D25Pin), (d26: D26Pin), (d27: D27Pin), (d28: D28Pin), (d29: D29Pin), (d30: D30Pin), (d31: D31Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin), (nbl2: NBL2Pin), (nbl3: NBL3Pin)
        ],
        ctrl: [
            (sdcke: SDCKE0Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE0Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_init!(init_sdram_a12bits_d32bits_4banks_bank2: (
        bank: FmcSdramBank::Bank2,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin),
            (d16: D16Pin), (d17: D17Pin), (d18: D18Pin), (d19: D19Pin), (d20: D20Pin), (d21: D21Pin), (d22: D22Pin), (d23: D23Pin),
            (d24: D24Pin), (d25: D25Pin), (d26: D26Pin), (d27: D27Pin), (d28: D28Pin), (d29: D29Pin), (d30: D30Pin), (d31: D31Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin), (nbl2: NBL2Pin), (nbl3: NBL3Pin)
        ],
        ctrl: [
            (sdcke: SDCKE1Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE1Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_init_dual!(init_sdram_a12bits_d32bits_4banks_dual: (
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin),
            (d16: D16Pin), (d17: D17Pin), (d18: D18Pin), (d19: D19Pin), (d20: D20Pin), (d21: D21Pin), (d22: D22Pin), (d23: D23Pin),
            (d24: D24Pin), (d25: D25Pin), (d26: D26Pin), (d27: D27Pin), (d28: D28Pin), (d29: D29Pin), (d30: D30Pin), (d31: D31Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin), (nbl2: NBL2Pin), (nbl3: NBL3Pin)
        ],
        ctrl: [
            (sdcke0: SDCKE0Pin), (sdcke1: SDCKE1Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne0: SDNE0Pin), (sdne1: SDNE1Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_init!(init_sdram_a13bits_d32bits_4banks_bank1: (
        bank: FmcSdramBank::Bank1,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin), (a12: A12Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin),
            (d16: D16Pin), (d17: D17Pin), (d18: D18Pin), (d19: D19Pin), (d20: D20Pin), (d21: D21Pin), (d22: D22Pin), (d23: D23Pin),
            (d24: D24Pin), (d25: D25Pin), (d26: D26Pin), (d27: D27Pin), (d28: D28Pin), (d29: D29Pin), (d30: D30Pin), (d31: D31Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin), (nbl2: NBL2Pin), (nbl3: NBL3Pin)
        ],
        ctrl: [
            (sdcke: SDCKE0Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE0Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_init!(init_sdram_a13bits_d32bits_4banks_bank2: (
        bank: FmcSdramBank::Bank2,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin), (a12: A12Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin),
            (d16: D16Pin), (d17: D17Pin), (d18: D18Pin), (d19: D19Pin), (d20: D20Pin), (d21: D21Pin), (d22: D22Pin), (d23: D23Pin),
            (d24: D24Pin), (d25: D25Pin), (d26: D26Pin), (d27: D27Pin), (d28: D28Pin), (d29: D29Pin), (d30: D30Pin), (d31: D31Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin), (nbl2: NBL2Pin), (nbl3: NBL3Pin)
        ],
        ctrl: [
            (sdcke: SDCKE1Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE1Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_init_dual!(init_sdram_a13bits_d32bits_4banks_dual: (
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin), (a12: A12Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin),
            (d16: D16Pin), (d17: D17Pin), (d18: D18Pin), (d19: D19Pin), (d20: D20Pin), (d21: D21Pin), (d22: D22Pin), (d23: D23Pin),
            (d24: D24Pin), (d25: D25Pin), (d26: D26Pin), (d27: D27Pin), (d28: D28Pin), (d29: D29Pin), (d30: D30Pin), (d31: D31Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin), (nbl2: NBL2Pin), (nbl3: NBL3Pin)
        ],
        ctrl: [
            (sdcke0: SDCKE0Pin), (sdcke1: SDCKE1Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne0: SDNE0Pin), (sdne1: SDNE1Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_init!(init_sdram_a13bits_d16bits_4banks_bank1: (
        bank: FmcSdramBank::Bank1,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin), (a12: A12Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin)
        ],
        ctrl: [
            (sdcke: SDCKE0Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE0Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_init!(init_sdram_a13bits_d16bits_4banks_bank2: (
        bank: FmcSdramBank::Bank2,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin), (a12: A12Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin)
        ],
        ctrl: [
            (sdcke: SDCKE1Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE1Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_init_dual!(init_sdram_a13bits_d16bits_4banks_dual: (
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin), (a12: A12Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin)
        ],
        ctrl: [
            (sdcke0: SDCKE0Pin), (sdcke1: SDCKE1Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne0: SDNE0Pin), (sdne1: SDNE1Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));
}
