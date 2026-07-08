#![no_std]
#![no_main]

/// Brings up the S70KL1281 HyperRAM on OCTOSPI2 (memory-mapped, 16 MB window at
/// `0x7000_0000`) and exercises it with a CPU read/write test.
///
/// The whole bring-up uses only the public `embassy_stm32::ospi` API:
/// - the device `Config` (memory type, size, prescaler, refresh, chip-select boundary,
///   ...) and the `new_blocking_octospi_with_dqs` constructor program DCR1-4 / TCR / CR;
/// - `Ospi::configure_hyperbus` programs the HyperBus latency register (HLCR);
/// - `enable_memory_mapped_mode`, though written for NOR-flash commands, expresses the
///   HyperBus CCR/WCCR command frame correctly when given a `TransferConfig` with an
///   8-lane DTR address phase and no instruction phase.
///
/// Register values are the known-good ones validated by the bare-metal C++ bring-up on
/// this exact board: `docs/hal-oracle/octospi-hyperram.md` (HAL oracle) and
/// `docs/hal-oracle/octospi-hyperram-cmsis.md` (raw CMSIS recipe).
///
/// Note on caching: neither embassy-stm32 nor cortex-m-rt enables the Cortex-M7 I/D
/// caches on this chip, so they are off here too. That makes CPU access to the
/// HyperRAM window trivially coherent (no MPU/clean/invalidate needed for a CPU-only
/// test); MPU attributes only become load-bearing once a second AXI master (DMA2D,
/// LTDC) touches the same memory - see `docs/hal-oracle/octospi-hyperram.md` §3.
use core::ptr::{read_volatile, write_volatile};

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::ospi::{
    AddressSize, ChipSelectHighTime, Config, FIFOThresholdLevel, HyperbusConfig, HyperbusLatencyMode, MemorySize,
    MemoryType, Ospi, OspiWidth, TransferConfig, WrapSize,
};
use {defmt_rtt as _, panic_probe as _};

/// Base address of the OCTOSPI2 memory-mapped window (silicon-fixed, RM0468 memory map).
const HYPERRAM_BASE: *mut u32 = 0x7000_0000 as *mut u32;
/// Words tested by the bulk pattern pass (1 MiB). Well clear of the 8 MiB dual-die
/// boundary at `0x7080_0000` and of the 16 MiB device limit at `0x7100_0000`.
const TEST_WORDS: usize = 1024 * 1024 / 4;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START");

    let p = rcc_setup::stm32h735g_init();

    // HyperBus device config for the S70KL1281 (16 MB, dual-die, fixed latency).
    // Field-by-field derivation: docs/hal-oracle/octospi-hyperram.md §2.
    let ospi_config = Config {
        fifo_threshold: FIFOThresholdLevel::_4Bytes,
        memory_type: MemoryType::HyperBusMemory,
        device_size: MemorySize::_16MiB,
        chip_select_high_time: ChipSelectHighTime::_4Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        clock_prescaler: 1, // 200 MHz kernel / 2 = 100 MHz HyperBus clock
        sample_shifting: false,
        delay_hold_quarter_cycle: true,
        chip_select_boundary: 23, // NCS released every 2^23 = 8 MiB (die boundary)
        delay_block_bypass: false,
        max_transfer: 0,
        refresh: 400, // self-refresh window; mandatory since CR.TCEN stays 0 in HyperBus MM
    };

    // Pin map from docs/HARDWARE.md "OCTOSPI2 <-> HyperRAM pin map" (also the raw-CMSIS
    // recipe's §2 table): CLK=PF4, DQS=PF12, NCS=PG12, DQ0-3=PF0-3, DQ4-7=PG0,PG1,PG10,PG11.
    let mut ospi = Ospi::new_blocking_octospi_with_dqs(
        p.OCTOSPI2,
        p.PF4,  // CLK
        p.PF0,  // DQ0
        p.PF1,  // DQ1
        p.PF2,  // DQ2
        p.PF3,  // DQ3
        p.PG0,  // DQ4
        p.PG1,  // DQ5
        p.PG10, // DQ6
        p.PG11, // DQ7
        p.PG12, // NCS
        p.PF12, // DQS
        ospi_config,
    );

    // Program the HyperBus latency (HLCR). Values match the S70KL1281 power-on default
    // (fixed latency, initial latency 6) that the ST BSP relies on without ever touching
    // the device CR0 register. Fixed mode pays 2x TACC = 12 CLK once per burst.
    ospi.configure_hyperbus(HyperbusConfig {
        latency_mode: HyperbusLatencyMode::Fixed,
        access_time: 6,
        rw_recovery_time: 3,
        write_zero_latency: false,
    });

    // HyperBus memory-mapped read/write command. Unlike NOR flash there is no
    // instruction phase; the 8-lane DTR address phase is the entire command frame.
    // `enable_memory_mapped_mode` programs CCR (reads) and WCCR (writes) from these
    // two configs - using the same TransferConfig for both reproduces the oracle's
    // single composite value (`docs/hal-oracle/octospi-hyperram-cmsis.md` §5: CCR =
    // WCCR = 0x2C00_3C00) without any further raw register access.
    let hyperbus_command = TransferConfig {
        adwidth: OspiWidth::OCTO,
        address: Some(0),
        adsize: AddressSize::_32bit,
        addtr: true,
        dwidth: OspiWidth::OCTO,
        ddtr: true,
        dqse: true,
        ..Default::default()
    };
    ospi.enable_memory_mapped_mode(hyperbus_command, hyperbus_command)
        .expect("failed to enable HyperRAM memory-mapped mode");

    info!("HyperRAM memory-mapped at 0x{:08x}", HYPERRAM_BASE as u32);

    // Bulk pattern test: write a pseudo-random pattern derived from the word index,
    // then read it back. Exercises the linear burst path the LTDC scanner will later
    // depend on.
    let mismatches = bulk_pattern_test();
    info!("bulk pattern test: {} words, {} mismatches", TEST_WORDS, mismatches);

    // Walking-1s test at a fixed address: classic data-line stuck-at/short check. The
    // oracle flags that no delay-block calibration is done anywhere in this bring-up,
    // so this is the cheap extra confidence check it explicitly recommends.
    let walk_errors = walking_ones_test();
    info!("walking-1s data-line test: {} errors", walk_errors);

    if mismatches == 0 && walk_errors == 0 {
        info!("PASS");
    } else {
        error!("FAIL");
    }

    info!("END");
    loop {}
}

/// Writes then reads back a per-word pattern over `TEST_WORDS` sequential 32-bit words
/// starting at `HYPERRAM_BASE`. Returns the number of mismatching words.
fn bulk_pattern_test() -> u32 {
    fn pattern_for(index: usize) -> u32 {
        (index as u32).wrapping_mul(0x9E37_79B1) ^ 0xA5A5_A5A5
    }

    unsafe {
        for i in 0..TEST_WORDS {
            write_volatile(HYPERRAM_BASE.add(i), pattern_for(i));
        }
    }

    let mut mismatches = 0u32;
    unsafe {
        for i in 0..TEST_WORDS {
            let expected = pattern_for(i);
            let actual = read_volatile(HYPERRAM_BASE.add(i));
            if actual != expected {
                mismatches += 1;
                if mismatches <= 8 {
                    error!(
                        "mismatch at word {}: expected 0x{:08x}, got 0x{:08x}",
                        i, expected, actual
                    );
                }
            }
        }
    }
    mismatches
}

/// Writes a single set bit at a time (0..32) to a fixed address and reads it back.
/// Returns the number of bits that did not round-trip.
fn walking_ones_test() -> u32 {
    let mut errors = 0u32;
    unsafe {
        for bit in 0..32 {
            let pattern = 1u32 << bit;
            write_volatile(HYPERRAM_BASE, pattern);
            let readback = read_volatile(HYPERRAM_BASE);
            if readback != pattern {
                errors += 1;
                error!(
                    "walking-1 bit {}: wrote 0x{:08x}, read 0x{:08x}",
                    bit, pattern, readback
                );
            }
        }
    }
    errors
}

mod rcc_setup {
    use embassy_stm32::rcc::mux::Fmcsel;
    use embassy_stm32::rcc::{Hse, HseMode, *};
    use embassy_stm32::time::Hertz;
    use embassy_stm32::{Config, Peripherals};

    /// Clocks for the STM32H735G-DK: SYSCLK 520 MHz (PLL1), OCTOSPI kernel clock
    /// 200 MHz (PLL2_R, routed via `mux.octospisel`). Numbers match
    /// `docs/hal-oracle/octospi-hyperram.md` §1 / `docs/HARDWARE.md`.
    pub fn stm32h735g_init() -> Peripherals {
        let mut config = Config::default();
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(25),
            mode: HseMode::Oscillator,
        });
        config.rcc.hsi = None;
        config.rcc.csi = false;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5, // PLL_M
            mul: PllMul::Mul104,     // PLL_N
            divp: Some(PllDiv::Div1),
            divq: Some(PllDiv::Div4),
            divr: Some(PllDiv::Div2),
        });
        // numbers adapted from Drivers/BSP/STM32H735G-DK/stm32h735g_discovery_ospi.c
        // MX_OSPI_ClockConfig
        config.rcc.pll2 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5, // PLL_M
            mul: PllMul::Mul80,      // PLL_N
            divp: Some(PllDiv::Div5),
            divq: Some(PllDiv::Div2),
            divr: Some(PllDiv::Div2), // pll2_r = 200 MHz
        });
        config.rcc.voltage_scale = VoltageScale::Scale0;
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div2;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.apb3_pre = APBPrescaler::Div2;
        config.rcc.apb4_pre = APBPrescaler::Div2;
        config.rcc.mux.octospisel = Fmcsel::Pll2R;
        embassy_stm32::init(config)
    }
}
