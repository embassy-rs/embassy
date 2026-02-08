//! Config and timings for ili9341-based displays running on parallel mode.

use crate::fmc::nor_sram::{
    NorSramAccessMode, NorSramChip, NorSramConfiguration, NorSramDriver, NorSramMemoryDataWidth, NorSramMemoryType,
    NorSramPageSize, NorSramTiming, NorSramWaitSignalActive, NorSramWaitSignalPolarity,
};

/// Implements a device for driving an ILI9341-based display
/// controller running in Intel 8080 parallel mode.  
pub struct Ili9341 {}

/// Provides the timing and configuration parameters for the device.
impl NorSramChip for Ili9341 {
    const CONFIG: NorSramConfiguration = NorSramConfiguration {
        data_address_mux_enabled: false,
        memory_type: NorSramMemoryType::Sram,
        memory_data_width: NorSramMemoryDataWidth::Bits32, // TODO: this should be configurable
        burst_access_mode_enable: false,
        wait_signal_enable_polarity: NorSramWaitSignalPolarity::ActiveLow,
        wait_signal_enable_active: NorSramWaitSignalActive::BeforeWaitState,
        write_enable: true,
        wait_signal_enable: false, // ILI9341 doesn't use a wait signal.
        extended_mode: false,
        asynchronous_wait: false,
        write_burst_enable: false,
        // We only want write/read clock signals when actually
        // reading/writing, so disable the continuous clock.
        continuous_clock_enable: false,
        write_fifo_disable: false,
        page_size: NorSramPageSize::NoBurstSplit,
    };

    const TIMING: NorSramTiming = NorSramTiming {
        address_setup_time: 0,       // tast
        address_hold_time: 1,        // taht - should be 0, but FMC doesn't support 1?
        data_setup_time: 20,         // trod - can probably shorted to 10 (tdst)
        bus_turn_around_duration: 2, // unused by sram
        clock_division: 6,           // not used? should be a way to set clock..
        data_latency: 0,             // unused by sram
        access_mode: NorSramAccessMode::AccessModeA,
    };
}

/// Implements the driver for running ILI9341-based displays.
impl NorSramDriver for Ili9341 {}
