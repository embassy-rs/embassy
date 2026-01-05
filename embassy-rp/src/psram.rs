//! PSRAM driver for APS6404L and compatible devices
//!
//! This driver provides support for PSRAM (Pseudo-Static RAM) devices connected via QMI CS1.
//! It handles device verification, initialization, and memory-mapped access configuration.
//!
//! This driver is only available on RP235x chips as it requires the QMI CS1 peripheral.

// Credit: Initially based on https://github.com/Altaflux/gb-rp2350 (also licensed Apache 2.0 + MIT).
// Copyright (c) Altaflux

#![cfg(feature = "_rp235x")]

use critical_section::{CriticalSection, RestoreState, acquire, release};

use crate::pac;
use crate::qmi_cs1::QmiCs1;

/// PSRAM errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// PSRAM device is not detected or not supported
    DeviceNotFound,
    /// Invalid configuration
    InvalidConfig,
    /// Detected PSRAM size does not match the expected size
    SizeMismatch,
}

/// PSRAM device verification type.
#[derive(Clone, Copy)]
pub enum VerificationType {
    /// Skip device verification
    None,
    /// Verify as APS6404L device
    Aps6404l,
}

/// Memory configuration.
#[derive(Clone)]
pub struct Config {
    /// System clock frequency in Hz
    pub clock_hz: u32,
    /// Maximum memory operating frequency in Hz
    pub max_mem_freq: u32,
    /// Maximum CS assert time in microseconds (must be <= 8 us)
    pub max_select_us: u32,
    /// Minimum CS deassert time in nanoseconds (must be >= 18 ns)
    pub min_deselect_ns: u32,
    /// Cooldown period between operations (in SCLK cycles)
    pub cooldown: u8,
    /// Page break size for memory operations
    pub page_break: PageBreak,
    /// Clock divisor for direct mode operations during initialization
    pub init_clkdiv: u8,
    /// Enter Quad Mode command
    pub enter_quad_cmd: Option<u8>,
    /// Quad Read command (fast read with 4-bit data)
    pub quad_read_cmd: u8,
    /// Quad Write command (page program with 4-bit data)
    pub quad_write_cmd: Option<u8>,
    /// Number of dummy cycles for quad read operations
    pub dummy_cycles: u8,
    /// Read format configuration
    pub read_format: FormatConfig,
    /// Write format configuration
    pub write_format: Option<FormatConfig>,
    /// Expected memory size in bytes
    pub mem_size: usize,
    /// Device verification type
    pub verification_type: VerificationType,
    /// Whether the memory is writable via XIP (e.g., PSRAM vs. read-only flash)
    pub xip_writable: bool,
}

/// Page break configuration for memory window operations.
#[derive(Clone, Copy)]
pub enum PageBreak {
    /// No page breaks
    None,
    /// Break at 256-byte boundaries
    _256,
    /// Break at 1024-byte boundaries
    _1024,
    /// Break at 4096-byte boundaries
    _4096,
}

/// Format configuration for read/write operations.
#[derive(Clone)]
pub struct FormatConfig {
    /// Width of command prefix phase
    pub prefix_width: Width,
    /// Width of address phase
    pub addr_width: Width,
    /// Width of command suffix phase
    pub suffix_width: Width,
    /// Width of dummy/turnaround phase
    pub dummy_width: Width,
    /// Width of data phase
    pub data_width: Width,
    /// Length of prefix (None or 8 bits)
    pub prefix_len: bool,
    /// Length of suffix (None or 8 bits)
    pub suffix_len: bool,
}

/// Interface width for different phases of SPI transfer.
#[derive(Clone, Copy)]
pub enum Width {
    /// Single-bit (standard SPI)
    Single,
    /// Dual-bit (2 data lines)
    Dual,
    /// Quad-bit (4 data lines)
    Quad,
}

impl Default for Config {
    fn default() -> Self {
        Self::aps6404l()
    }
}

impl Config {
    /// Create configuration for APS6404L PSRAM.
    pub fn aps6404l() -> Self {
        Self {
            clock_hz: 125_000_000,        // Default to 125MHz
            max_mem_freq: 133_000_000,    // APS6404L max frequency
            max_select_us: 8,             // 8 microseconds max CS assert
            min_deselect_ns: 18,          // 18 nanoseconds min CS deassert
            cooldown: 1,                  // 1 SCLK cycle cooldown
            page_break: PageBreak::_1024, // 1024-byte page boundaries
            init_clkdiv: 10,              // Medium clock for initialization
            enter_quad_cmd: Some(0x35),   // Enter Quad Mode
            quad_read_cmd: 0xEB,          // Fast Quad Read
            quad_write_cmd: Some(0x38),   // Quad Page Program
            dummy_cycles: 24,             // 24 dummy cycles for quad read
            read_format: FormatConfig {
                prefix_width: Width::Quad,
                addr_width: Width::Quad,
                suffix_width: Width::Quad,
                dummy_width: Width::Quad,
                data_width: Width::Quad,
                prefix_len: true,  // 8-bit prefix
                suffix_len: false, // No suffix
            },
            write_format: Some(FormatConfig {
                prefix_width: Width::Quad,
                addr_width: Width::Quad,
                suffix_width: Width::Quad,
                dummy_width: Width::Quad,
                data_width: Width::Quad,
                prefix_len: true,  // 8-bit prefix
                suffix_len: false, // No suffix
            }),
            mem_size: 8 * 1024 * 1024, // 8MB for APS6404L
            verification_type: VerificationType::Aps6404l,
            xip_writable: true, // PSRAM is writable
        }
    }

    /// Create a custom memory configuration.
    pub fn custom(
        clock_hz: u32,
        max_mem_freq: u32,
        max_select_us: u32,
        min_deselect_ns: u32,
        cooldown: u8,
        page_break: PageBreak,
        init_clkdiv: u8,
        enter_quad_cmd: Option<u8>,
        quad_read_cmd: u8,
        quad_write_cmd: Option<u8>,
        dummy_cycles: u8,
        read_format: FormatConfig,
        write_format: Option<FormatConfig>,
        mem_size: usize,
        verification_type: VerificationType,
        xip_writable: bool,
    ) -> Self {
        Self {
            clock_hz,
            max_mem_freq,
            max_select_us,
            min_deselect_ns,
            cooldown,
            page_break,
            init_clkdiv,
            enter_quad_cmd,
            quad_read_cmd,
            quad_write_cmd,
            dummy_cycles,
            read_format,
            write_format,
            mem_size,
            verification_type,
            xip_writable,
        }
    }
}

/// PSRAM driver.
pub struct Psram<'d> {
    #[allow(dead_code)]
    qmi_cs1: QmiCs1<'d>,
    size: usize,
}

impl<'d> Psram<'d> {
    /// Create a new PSRAM driver instance.
    ///
    /// This will detect the PSRAM device and configure it for memory-mapped access.
    pub fn new(qmi_cs1: QmiCs1<'d>, config: Config) -> Result<Self, Error> {
        let qmi = pac::QMI;
        let xip = pac::XIP_CTRL;

        // Verify PSRAM device if requested
        match config.verification_type {
            VerificationType::Aps6404l => {
                Self::verify_aps6404l(&qmi, config.mem_size)?;
                debug!("APS6404L PSRAM verified, size: {:#x}", config.mem_size);
            }
            VerificationType::None => {
                debug!("Skipping PSRAM verification, assuming size: {:#x}", config.mem_size);
            }
        }

        // Initialize PSRAM with proper timing
        Self::init_psram(&qmi, &xip, &config)?;

        Ok(Self {
            qmi_cs1,
            size: config.mem_size,
        })
    }

    /// Get the detected PSRAM size in bytes.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the base address for memory-mapped access.
    ///
    /// After initialization, PSRAM can be accessed directly through memory mapping.
    /// The base address for CS1 is typically 0x11000000.
    pub fn base_address(&self) -> *mut u8 {
        0x1100_0000 as *mut u8
    }

    /// Verify APS6404L PSRAM device matches expected configuration.
    #[unsafe(link_section = ".data.ram_func")]
    #[inline(never)]
    fn verify_aps6404l(qmi: &pac::qmi::Qmi, expected_size: usize) -> Result<(), Error> {
        // APS6404L-specific constants
        const EXPECTED_KGD: u8 = 0x5D;
        crate::multicore::pause_core1();
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        {
            // Helper for making sure `release` is called even if `f` panics.
            struct Guard {
                state: RestoreState,
            }

            impl Drop for Guard {
                #[inline(always)]
                fn drop(&mut self) {
                    unsafe { release(self.state) }
                }
            }

            let state = unsafe { acquire() };
            let _guard = Guard { state };

            let _cs = unsafe { CriticalSection::new() };

            let (kgd, eid) = unsafe { Self::read_aps6404l_kgd_eid(qmi) };

            let mut detected_size: u32 = 0;
            if kgd == EXPECTED_KGD as u32 {
                detected_size = 1024 * 1024;
                let size_id = eid >> 5;
                if eid == 0x26 || size_id == 2 {
                    // APS6404L-3SQR-SN or 8MB variants
                    detected_size *= 8;
                } else if size_id == 0 {
                    detected_size *= 2;
                } else if size_id == 1 {
                    detected_size *= 4;
                }
            }

            // Verify the detected size matches the expected size
            if detected_size as usize != expected_size {
                return Err(Error::SizeMismatch);
            }

            Ok(())
        }?;

        crate::multicore::resume_core1();

        Ok(())
    }

    #[unsafe(link_section = ".data.ram_func")]
    #[inline(never)]
    unsafe fn read_aps6404l_kgd_eid(qmi: &pac::qmi::Qmi) -> (u32, u32) {
        const RESET_ENABLE_CMD: u8 = 0xf5;
        const READ_ID_CMD: u8 = 0x9f;

        #[allow(unused_assignments)]
        let mut kgd: u32 = 0;
        #[allow(unused_assignments)]
        let mut eid: u32 = 0;

        let qmi_base = qmi.as_ptr() as usize;

        #[cfg(target_arch = "arm")]
        core::arch::asm!(
        // Configure DIRECT_CSR: shift clkdiv (30) to bits 29:22 and set EN (bit 0)
        "movs {temp}, #30",
        "lsls {temp}, {temp}, #22",
        "orr {temp}, {temp}, #1",        // Set EN bit
        "str {temp}, [{qmi_base}]",

        // Poll for BUSY to clear before first operation
        "1:",
        "ldr {temp}, [{qmi_base}]",
        "lsls {temp}, {temp}, #30",      // Shift BUSY bit to sign position
        "bmi 1b",                        // Branch if negative (BUSY = 1)

        // Assert CS1N (bit 3)
        "ldr {temp}, [{qmi_base}]",
        "orr {temp}, {temp}, #8",        // Set ASSERT_CS1N bit (bit 3)
        "str {temp}, [{qmi_base}]",

        // Transmit RESET_ENABLE_CMD as quad
        // DIRECT_TX: OE=1 (bit 19), IWIDTH=2 (bits 17:16), DATA=RESET_ENABLE_CMD
        "movs {temp}, {reset_enable_cmd}",
        "orr {temp}, {temp}, #0x80000",  // Set OE (bit 19)
        "orr {temp}, {temp}, #0x20000",  // Set IWIDTH=2 (quad, bits 17:16)
        "str {temp}, [{qmi_base}, #4]",  // Store to DIRECT_TX

        // Wait for BUSY to clear
        "2:",
        "ldr {temp}, [{qmi_base}]",
        "lsls {temp}, {temp}, #30",
        "bmi 2b",

        // Read and discard RX data
        "ldr {temp}, [{qmi_base}, #8]",

        // Deassert CS1N
        "ldr {temp}, [{qmi_base}]",
        "bic {temp}, {temp}, #8",        // Clear ASSERT_CS1N bit
        "str {temp}, [{qmi_base}]",

        // Assert CS1N again
        "ldr {temp}, [{qmi_base}]",
        "orr {temp}, {temp}, #8",        // Set ASSERT_CS1N bit
        "str {temp}, [{qmi_base}]",

        // Read ID loop (7 iterations)
        "movs {counter}, #0",            // Initialize counter

        "3:",                            // Loop start
        "cmp {counter}, #0",
        "bne 4f",                        // If not first iteration, send 0xFF

        // First iteration: send READ_ID_CMD
        "movs {temp}, {read_id_cmd}",
        "b 5f",
        "4:",                            // Other iterations: send 0xFF
        "movs {temp}, #0xFF",
        "5:",
        "str {temp}, [{qmi_base}, #4]",  // Store to DIRECT_TX

        // Wait for TXEMPTY
        "6:",
        "ldr {temp}, [{qmi_base}]",
        "lsls {temp}, {temp}, #20",      // Shift TXEMPTY (bit 11) to bit 31
        "bpl 6b",                        // Branch if positive (TXEMPTY = 0)

        // Wait for BUSY to clear
        "7:",
        "ldr {temp}, [{qmi_base}]",
        "lsls {temp}, {temp}, #30",      // Shift BUSY bit to sign position
        "bmi 7b",                        // Branch if negative (BUSY = 1)

        // Read RX data
        "ldr {temp}, [{qmi_base}, #8]",
        "uxth {temp}, {temp}",           // Extract lower 16 bits

        // Store KGD or EID based on iteration
        "cmp {counter}, #5",
        "bne 8f",
        "mov {kgd}, {temp}",             // Store KGD
        "b 9f",
        "8:",
        "cmp {counter}, #6",
        "bne 9f",
        "mov {eid}, {temp}",             // Store EID

        "9:",
        "adds {counter}, #1",
        "cmp {counter}, #7",
        "blt 3b",                        // Continue loop if counter < 7

        // Disable direct mode: clear EN and ASSERT_CS1N
        "movs {temp}, #0",
        "str {temp}, [{qmi_base}]",

        // Memory barriers
        "dmb",
        "dsb",
        "isb",
        qmi_base = in(reg) qmi_base,
        temp = out(reg) _,
        counter = out(reg) _,
        kgd = out(reg) kgd,
        eid = out(reg) eid,
        reset_enable_cmd = const RESET_ENABLE_CMD as u32,
        read_id_cmd = const READ_ID_CMD as u32,
        options(nostack),
        );

        #[cfg(target_arch = "riscv32")]
        unimplemented!("APS6404L PSRAM verification not implemented for RISC-V");

        (kgd, eid)
    }

    /// Initialize PSRAM with proper timing.
    #[unsafe(link_section = ".data.ram_func")]
    #[inline(never)]
    fn init_psram(qmi: &pac::qmi::Qmi, xip_ctrl: &pac::xip_ctrl::XipCtrl, config: &Config) -> Result<(), Error> {
        // Set PSRAM timing for APS6404
        //
        // Using an rxdelay equal to the divisor isn't enough when running the APS6404 close to 133 MHz.
        // So: don't allow running at divisor 1 above 100 MHz (because delay of 2 would be too late),
        // and add an extra 1 to the rxdelay if the divided clock is > 100 MHz (i.e., sys clock > 200 MHz).
        let clock_hz = config.clock_hz;
        let max_psram_freq = config.max_mem_freq;

        let mut divisor: u32 = (clock_hz + max_psram_freq - 1) / max_psram_freq;
        if divisor == 1 && clock_hz > 100_000_000 {
            divisor = 2;
        }
        let mut rxdelay: u32 = divisor;
        if clock_hz / divisor > 100_000_000 {
            rxdelay += 1;
        }

        // - Max select must be <= 8 us. The value is given in multiples of 64 system clocks.
        // - Min deselect must be >= 18ns. The value is given in system clock cycles - ceil(divisor / 2).
        let clock_period_fs: u64 = 1_000_000_000_000_000_u64 / u64::from(clock_hz);
        let max_select: u8 = ((config.max_select_us as u64 * 1_000_000) / clock_period_fs) as u8;
        let min_deselect: u32 = ((config.min_deselect_ns as u64 * 1_000_000 + (clock_period_fs - 1)) / clock_period_fs
            - u64::from(divisor + 1) / 2) as u32;

        crate::multicore::pause_core1();
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        if let Some(enter_quad_cmd) = config.enter_quad_cmd {
            // Helper for making sure `release` is called even if `f` panics.
            struct Guard {
                state: RestoreState,
            }

            impl Drop for Guard {
                #[inline(always)]
                fn drop(&mut self) {
                    unsafe { release(self.state) }
                }
            }

            let state = unsafe { acquire() };
            let _guard = Guard { state };

            let _cs = unsafe { CriticalSection::new() };

            unsafe { Self::direct_csr_send_init_command(config, enter_quad_cmd) };

            qmi.mem(1).timing().write(|w| {
                w.set_cooldown(config.cooldown);
                w.set_pagebreak(match config.page_break {
                    PageBreak::None => pac::qmi::vals::Pagebreak::NONE,
                    PageBreak::_256 => pac::qmi::vals::Pagebreak::_256,
                    PageBreak::_1024 => pac::qmi::vals::Pagebreak::_1024,
                    PageBreak::_4096 => pac::qmi::vals::Pagebreak::_4096,
                });
                w.set_max_select(max_select);
                w.set_min_deselect(min_deselect as u8);
                w.set_rxdelay(rxdelay as u8);
                w.set_clkdiv(divisor as u8);
            });

            // Set PSRAM commands and formats
            qmi.mem(1).rfmt().write(|w| {
                let width_to_pac = |w: Width| match w {
                    Width::Single => pac::qmi::vals::PrefixWidth::S,
                    Width::Dual => pac::qmi::vals::PrefixWidth::D,
                    Width::Quad => pac::qmi::vals::PrefixWidth::Q,
                };

                w.set_prefix_width(width_to_pac(config.read_format.prefix_width));
                w.set_addr_width(match config.read_format.addr_width {
                    Width::Single => pac::qmi::vals::AddrWidth::S,
                    Width::Dual => pac::qmi::vals::AddrWidth::D,
                    Width::Quad => pac::qmi::vals::AddrWidth::Q,
                });
                w.set_suffix_width(match config.read_format.suffix_width {
                    Width::Single => pac::qmi::vals::SuffixWidth::S,
                    Width::Dual => pac::qmi::vals::SuffixWidth::D,
                    Width::Quad => pac::qmi::vals::SuffixWidth::Q,
                });
                w.set_dummy_width(match config.read_format.dummy_width {
                    Width::Single => pac::qmi::vals::DummyWidth::S,
                    Width::Dual => pac::qmi::vals::DummyWidth::D,
                    Width::Quad => pac::qmi::vals::DummyWidth::Q,
                });
                w.set_data_width(match config.read_format.data_width {
                    Width::Single => pac::qmi::vals::DataWidth::S,
                    Width::Dual => pac::qmi::vals::DataWidth::D,
                    Width::Quad => pac::qmi::vals::DataWidth::Q,
                });
                w.set_prefix_len(if config.read_format.prefix_len {
                    pac::qmi::vals::PrefixLen::_8
                } else {
                    pac::qmi::vals::PrefixLen::NONE
                });
                w.set_suffix_len(if config.read_format.suffix_len {
                    pac::qmi::vals::SuffixLen::_8
                } else {
                    pac::qmi::vals::SuffixLen::NONE
                });
                w.set_dummy_len(match config.dummy_cycles {
                    0 => pac::qmi::vals::DummyLen::NONE,
                    4 => pac::qmi::vals::DummyLen::_4,
                    8 => pac::qmi::vals::DummyLen::_8,
                    12 => pac::qmi::vals::DummyLen::_12,
                    16 => pac::qmi::vals::DummyLen::_16,
                    20 => pac::qmi::vals::DummyLen::_20,
                    24 => pac::qmi::vals::DummyLen::_24,
                    28 => pac::qmi::vals::DummyLen::_28,
                    _ => pac::qmi::vals::DummyLen::_24, // Default to 24
                });
            });

            qmi.mem(1).rcmd().write(|w| w.set_prefix(config.quad_read_cmd));

            if let Some(ref write_format) = config.write_format {
                qmi.mem(1).wfmt().write(|w| {
                    w.set_prefix_width(match write_format.prefix_width {
                        Width::Single => pac::qmi::vals::PrefixWidth::S,
                        Width::Dual => pac::qmi::vals::PrefixWidth::D,
                        Width::Quad => pac::qmi::vals::PrefixWidth::Q,
                    });
                    w.set_addr_width(match write_format.addr_width {
                        Width::Single => pac::qmi::vals::AddrWidth::S,
                        Width::Dual => pac::qmi::vals::AddrWidth::D,
                        Width::Quad => pac::qmi::vals::AddrWidth::Q,
                    });
                    w.set_suffix_width(match write_format.suffix_width {
                        Width::Single => pac::qmi::vals::SuffixWidth::S,
                        Width::Dual => pac::qmi::vals::SuffixWidth::D,
                        Width::Quad => pac::qmi::vals::SuffixWidth::Q,
                    });
                    w.set_dummy_width(match write_format.dummy_width {
                        Width::Single => pac::qmi::vals::DummyWidth::S,
                        Width::Dual => pac::qmi::vals::DummyWidth::D,
                        Width::Quad => pac::qmi::vals::DummyWidth::Q,
                    });
                    w.set_data_width(match write_format.data_width {
                        Width::Single => pac::qmi::vals::DataWidth::S,
                        Width::Dual => pac::qmi::vals::DataWidth::D,
                        Width::Quad => pac::qmi::vals::DataWidth::Q,
                    });
                    w.set_prefix_len(if write_format.prefix_len {
                        pac::qmi::vals::PrefixLen::_8
                    } else {
                        pac::qmi::vals::PrefixLen::NONE
                    });
                    w.set_suffix_len(if write_format.suffix_len {
                        pac::qmi::vals::SuffixLen::_8
                    } else {
                        pac::qmi::vals::SuffixLen::NONE
                    });
                });
            }

            if let Some(quad_write_cmd) = config.quad_write_cmd {
                qmi.mem(1).wcmd().write(|w| w.set_prefix(quad_write_cmd));
            }

            if config.xip_writable {
                // Enable XIP writable mode for PSRAM
                xip_ctrl.ctrl().modify(|w| w.set_writable_m1(true));
            } else {
                // Disable XIP writable mode
                xip_ctrl.ctrl().modify(|w| w.set_writable_m1(false));
            }
        }
        crate::multicore::resume_core1();

        Ok(())
    }

    #[unsafe(link_section = ".data.ram_func")]
    #[inline(never)]
    unsafe fn direct_csr_send_init_command(config: &Config, init_cmd: u8) {
        #[cfg(target_arch = "arm")]
        core::arch::asm!(
        // Full memory barrier
        "dmb",
        "dsb",
        "isb",

        // Configure QMI Direct CSR register
        // Load base address of QMI (0x400D0000)
        "movw {base}, #0x0000",
        "movt {base}, #0x400D",

        // Load init_clkdiv and shift to bits 29:22
        "lsl {temp}, {clkdiv}, #22",

        // OR with EN (bit 0) and AUTO_CS1N (bit 7)
        "orr {temp}, {temp}, #0x81",

        // Store to DIRECT_CSR register
        "str {temp}, [{base}, #0]",

        // Memory barrier
        "dmb",

        // First busy wait loop
        "1:",
        "ldr {temp}, [{base}, #0]",      // Load DIRECT_CSR
        "tst {temp}, #0x2",              // Test BUSY bit (bit 1)
        "bne 1b",                        // Branch if busy

        // Write to Direct TX register
        "mov {temp}, {enter_quad_cmd}",

        // OR with NOPUSH (bit 20)
        "orr {temp}, {temp}, #0x100000",

        // Store to DIRECT_TX register (offset 0x4)
        "str {temp}, [{base}, #4]",

        // Memory barrier
        "dmb",

        // Second busy wait loop
        "2:",
        "ldr {temp}, [{base}, #0]",      // Load DIRECT_CSR
        "tst {temp}, #0x2",              // Test BUSY bit (bit 1)
        "bne 2b",                        // Branch if busy

        // Disable Direct CSR
        "mov {temp}, #0",
        "str {temp}, [{base}, #0]",      // Clear DIRECT_CSR register

        // Full memory barrier to ensure no prefetching
        "dmb",
        "dsb",
        "isb",
        base = out(reg) _,
        temp = out(reg) _,
        clkdiv = in(reg) config.init_clkdiv as u32,
        enter_quad_cmd = in(reg) u32::from(init_cmd),
        options(nostack),
        );

        #[cfg(target_arch = "riscv32")]
        unimplemented!("Direct CSR command sending is not implemented for RISC-V yet");
    }
}
