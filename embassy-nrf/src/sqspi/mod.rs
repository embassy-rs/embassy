#![macro_use]

//! Soft Quad Serial Peripheral Interface (sQSPI) driver.
//!
//! The sQSPI is a soft peripheral that runs on the nRF54L's RISC-V core.
//! Unlike hardware peripherals, the register interface lives in shared RAM,
//! not at a fixed address.
//!
//! The register layout is defined in the [`regs`] submodule, mirroring the
//! C `NRF_SP_QSPI_Type` from the Nordic SDK header `nrf_sp_qspi.h`.

mod regs;

use core::future::poll_fn;
use core::marker::PhantomData;
use core::ptr;
use core::task::Poll;

use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;
use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};

use crate::gpio::{self, Pin as GpioPin, SealedPin};
use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::pac;
use crate::pac::gpio::vals as gpiovals;

// ============================================================================
// Firmware metadata
// ============================================================================

/// Expected sQSPI soft peripheral ID in firmware metadata.
///
/// See `softperipheral_meta.h`: `SOFTPERIPHERAL_META_SOFTPERIPHERAL_ID_SQSPI`.
const SOFTPERIPHERAL_ID_SQSPI: u16 = 0x45b1;

/// Parsed firmware metadata from the start of the sQSPI firmware binary.
///
/// The firmware binary begins with a `softperipheral_metadata_t` header
/// (defined in the Nordic SDK header `softperipheral_meta.h`). This struct
/// holds the parsed fields needed by the driver for firmware loading.
///
/// The header is 8 words (32 bytes) and supports two versions (v1 and v2)
/// which share the same field layout for the fields we use.
struct FirmwareMetadata {
    /// If true, the VPR boots directly from the firmware address (NVM).
    /// If false, the driver must copy the firmware code into RAM.
    self_boot: bool,
    /// Firmware code size in units of 16 bytes.
    fw_code_size: u16,
    /// Total RAM needed by the firmware in units of 16 bytes.
    /// Includes code region, execution RAM, and shared/register interface.
    #[allow(dead_code)]
    fw_ram_total_size: u16,
    /// Offset (in bytes) from the RAM base to the shared register interface.
    fw_shared_ram_addr_offset: u16,
}

/// Error type for sQSPI operations.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Firmware binary is too short to contain a valid metadata header.
    FirmwareTooShort,
    /// The soft peripheral ID in the firmware doesn't match sQSPI.
    InvalidPeripheralId,
    /// The provided RAM buffer is too small for the firmware.
    BufferTooSmall,
    /// Operation address was out of bounds.
    OutOfBounds,
}

impl FirmwareMetadata {
    /// Parse and validate metadata from the start of a firmware binary.
    ///
    /// The binary must begin with a `softperipheral_metadata_t` header
    /// (8 x u32 = 32 bytes). See `softperipheral_meta.h` for the layout.
    ///
    /// Both header version 1 and 2 are supported — the fields used by this
    /// driver have compatible layouts across both versions.
    fn parse(fw: &[u8]) -> Result<Self, Error> {
        if fw.len() < 32 {
            return Err(Error::FirmwareTooShort);
        }

        let w0 = u32::from_le_bytes([fw[0], fw[1], fw[2], fw[3]]);
        let w1 = u32::from_le_bytes([fw[4], fw[5], fw[6], fw[7]]);
        let w3 = u32::from_le_bytes([fw[12], fw[13], fw[14], fw[15]]);
        let w6 = u32::from_le_bytes([fw[24], fw[25], fw[26], fw[27]]);

        // w0: magic(16) | header_version(4) | comm_id(8) | reserved(3) | self_boot(1)
        let self_boot = (w0 >> 31) & 1 != 0;

        // w1: softperiph_id(16) | platform(16)
        let softperiph_id = (w1 & 0xFFFF) as u16;
        if softperiph_id != SOFTPERIPHERAL_ID_SQSPI {
            return Err(Error::InvalidPeripheralId);
        }

        // w3: fw_code_size(16) | fw_ram_total_size(16)
        let fw_code_size = (w3 & 0xFFFF) as u16;
        let fw_ram_total_size = ((w3 >> 16) & 0xFFFF) as u16;

        // w6: fw_shared_ram_size(16) | fw_shared_ram_addr_offset(16)
        let fw_shared_ram_addr_offset = ((w6 >> 16) & 0xFFFF) as u16;

        Ok(Self {
            self_boot,
            fw_code_size,
            fw_ram_total_size,
            fw_shared_ram_addr_offset,
        })
    }

    /// Firmware code size in bytes.
    fn code_size_bytes(&self) -> usize {
        (self.fw_code_size as usize) << 4
    }

    /// Total RAM needed in bytes.
    #[allow(dead_code)]
    fn ram_total_bytes(&self) -> usize {
        (self.fw_ram_total_size as usize) << 4
    }
}

// ============================================================================
// Config
// ============================================================================

/// Multi-line SPI mode selection.
///
/// Configures the number of data lines used for command, address, and data
/// phases of SPI transactions. Maps to CTRLR0.SPI_FRF and SPICTRLR0.TRANSTYPE
/// in the sQSPI core registers.
///
/// See `nrf_sqspi.h`: `nrf_sqspi_spi_lines_t`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpiLines {
    /// Single-line SPI (standard MOSI/MISO).
    Single,
    /// Dual: single command, single address, dual data (1-1-2).
    Dual1_1_2,
    /// Dual: single command, dual address, dual data (1-2-2).
    Dual1_2_2,
    /// Quad: single command, single address, quad data (1-1-4).
    Quad1_1_4,
    /// Quad: single command, quad address, quad data (1-4-4).
    Quad1_4_4,
}

/// Address mode (24-bit or 32-bit).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddressMode {
    /// 24-bit addressing (3 bytes).
    _24Bit,
    /// 32-bit addressing (4 bytes).
    _32Bit,
}

/// SPI clock polarity.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Polarity {
    /// Clock idle low.
    IdleLow,
    /// Clock idle high.
    IdleHigh,
}

/// SPI clock phase.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Phase {
    /// Data captured on first clock edge.
    CaptureOnFirstTransition,
    /// Data captured on second clock edge.
    CaptureOnSecondTransition,
}

/// SPI mode (polarity + phase).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Mode {
    /// Clock polarity.
    pub polarity: Polarity,
    /// Clock phase.
    pub phase: Phase,
}

/// SPI Mode 0: CPOL=0, CPHA=0.
pub const MODE_0: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

/// SPI Mode 1: CPOL=0, CPHA=1.
pub const MODE_1: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnSecondTransition,
};

/// SPI Mode 2: CPOL=1, CPHA=0.
pub const MODE_2: Mode = Mode {
    polarity: Polarity::IdleHigh,
    phase: Phase::CaptureOnFirstTransition,
};

/// SPI Mode 3: CPOL=1, CPHA=1.
pub const MODE_3: Mode = Mode {
    polarity: Polarity::IdleHigh,
    phase: Phase::CaptureOnSecondTransition,
};

/// sQSPI driver configuration.
#[non_exhaustive]
pub struct Config {
    /// SCK clock frequency in kHz. The actual divider is `128 MHz / (sck_freq_khz * 1000)`.
    pub sck_freq_khz: u32,
    /// SPI clock polarity and phase.
    pub spi_mode: Mode,
    /// Multi-line mode configuration.
    pub lines: SpiLines,
    /// Address mode (24-bit or 32-bit).
    pub address_mode: AddressMode,
    /// Read command opcode (e.g. 0x6B for Quad Output Read).
    pub read_opcode: u8,
    /// Write/program command opcode (e.g. 0x32 for Quad Page Program).
    pub write_opcode: u8,
    /// Flash memory capacity in bytes (for bounds checking). 0 disables bounds checks.
    pub capacity: u32,
    /// Optional RX sample delay in clock cycles.
    pub sample_delay: Option<u8>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sck_freq_khz: 8000,
            spi_mode: MODE_0,
            lines: SpiLines::Quad1_1_4,
            address_mode: AddressMode::_24Bit,
            read_opcode: 0x6B,
            write_opcode: 0x32,
            capacity: 0,
            sample_delay: None,
        }
    }
}

// ============================================================================
// Transfer direction
// ============================================================================

/// Transfer direction for the SPI core CTRLR0.TMOD field.
///
/// See `nrf_sp_qspi.h`: `QSPI_CORE_CORE_CTRLR0_TMOD_*` defines.
#[repr(u32)]
#[derive(Copy, Clone)]
enum TransferDir {
    /// Transmit and receive.
    TxRx = 0,
    /// Transmit only.
    TxOnly = 1,
    /// Receive only.
    RxOnly = 2,
}

// ============================================================================
// Interrupt handler
// ============================================================================

/// Interrupt handler for the sQSPI driver.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // Check VPR EVENTS_TRIGGERED[20] for the soft peripheral completion event.
        // Index 20 = SP_VPR_EVENT_IDX from `softperipheral_regif.h`.
        let vpr = T::vpr_regs();
        let event_val = vpr.events_triggered(regs::SP_VPR_EVENT_IDX).read();
        trace!("IRQ: events_triggered[{}] = {}", regs::SP_VPR_EVENT_IDX, event_val);
        if event_val != 0 {
            vpr.events_triggered(regs::SP_VPR_EVENT_IDX).write_value(0);
            trace!("IRQ: waking waker");
            T::state().waker.wake();
        }
    }
}

// ============================================================================
// Driver struct
// ============================================================================

/// sQSPI flash driver.
pub struct Sqspi<'d> {
    regs: regs::Regs,
    vpr: pac::vpr::Vpr,
    state: &'static State,
    config: Config,
    task_count: u32,
    _phantom: PhantomData<&'d ()>,
}

impl<'d> Sqspi<'d> {
    /// Create a new sQSPI driver.
    ///
    /// This loads the FLPR firmware, starts the VPR co-processor, configures
    /// GPIO pins for VPR control, and sets up the SPI core.
    ///
    /// # Arguments
    /// - `sqspi`: The SQSPI peripheral singleton.
    /// - `_irq`: Interrupt binding for VPR00.
    /// - `firmware`: The sQSPI firmware binary (starts with `softperipheral_metadata_t`).
    /// - `ram`: A static mutable RAM buffer for the firmware + register interface.
    ///   Must be at least `fw_ram_total_size` bytes (typically 0x3D40 for nRF54L15).
    /// - `sck`, `csn`, `io0`..`io3`: GPIO pins.
    /// - `config`: Driver configuration.
    pub fn new<T: Instance>(
        _sqspi: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        firmware: &[u8],
        ram: &'d mut [u8],
        sck: Peri<'d, impl GpioPin>,
        csn: Peri<'d, impl GpioPin>,
        io0: Peri<'d, impl GpioPin>,
        io1: Peri<'d, impl GpioPin>,
        io2: Peri<'d, impl GpioPin>,
        io3: Peri<'d, impl GpioPin>,
        config: Config,
    ) -> Result<Self, Error> {
        let meta = FirmwareMetadata::parse(firmware)?;
        let vpr = T::vpr_regs();
        let state = T::state();

        // Validate RAM buffer is large enough.
        let shared_ram_offset = meta.fw_shared_ram_addr_offset as usize;
        let code_size = meta.code_size_bytes();
        let needed = shared_ram_offset + regs::Regs::SIZE + code_size;
        info!(
            "fw metadata: self_boot={}, code_size={}, shared_ram_offset={}, needed={}, ram_len={}",
            meta.self_boot, code_size, shared_ram_offset, needed, ram.len()
        );
        if ram.len() < needed {
            return Err(Error::BufferTooSmall);
        }

        // Compute register base address.
        // Layout: [firmware code | execution RAM | register interface]
        // reg_base = ram_base + shared_ram_offset + code_size
        // But per the C driver: vpr_init_pc = p_reg - fw_shared_ram_addr_offset - (fw_code_size << 4)
        // So: p_reg = ram_base + code_size + fw_shared_ram_addr_offset
        let ram_base = ram.as_mut_ptr() as usize;
        let reg_base = ram_base + code_size + shared_ram_offset;
        let reg_ptr = reg_base as *mut ();
        let sp_regs = unsafe { regs::Regs::from_ptr(reg_ptr) };
        info!(
            "ram_base=0x{:08x}, reg_base=0x{:08x}, vpr_base=0x{:08x}",
            ram_base, reg_base, vpr.as_ptr() as u32
        );

        // Grant secure access to the VPR00 peripheral via SPU00.
        // VPR00 is peripheral index 0xC in SPU00.
        // See `sqspi_nrf54L_series_porting_v1_2_1.rst` line 114.
        pac::SPU00.periph(0xC).perm().write(|w| {
            w.set_secattr(true);
            w.set_dmasec(true);
        });

        // ---- Phase 1: Load and start firmware (nrf_sqspi_init) ----

        // Zero the register region.
        // See `nrf_sqspi.c` line 153: memset(p_reg, 0, sizeof(NRF_SP_QSPI_Type))
        unsafe {
            ptr::write_bytes(reg_ptr as *mut u8, 0, regs::Regs::SIZE);
        }

        // Set ENABLE = 1, firmware will clear it when ready.
        // See `nrf_sqspi.c` line 156: nrf_qspi2_enable(p_qspi->p_reg)
        sp_regs.enable().write_value(1);

        // Compute VPR init PC.
        // See `nrf_sqspi.c` line 158-159.
        let vpr_init_pc = ram_base;

        // Copy firmware to RAM if not self-boot.
        // See `nrf_sqspi.c` lines 161-164.
        if !meta.self_boot {
            let copy_len = firmware.len().min(code_size);
            info!(
                "copying {} of {} bytes of firmware to 0x{:08x} (code_size={})",
                copy_len, firmware.len(), vpr_init_pc, code_size
            );
            unsafe {
                ptr::copy_nonoverlapping(firmware.as_ptr(), vpr_init_pc as *mut u8, copy_len);
            }
        }

        // Start the VPR co-processor.
        // See `nrf_sqspi.c` lines 166-173.
        vpr.initpc().write_value(vpr_init_pc as u32);
        vpr.cpurun().write(|w| w.set_en(pac::vpr::vals::CpurunEn::RUNNING));

        // Wait for firmware to become ready (ENABLE goes from 1 to 0).
        // See `nrf_sqspi.c` lines 175-178.
        while sp_regs.enable().read() != 0 {}
        info!("firmware ready (ENABLE cleared)");

        // Configure GPIO pins AFTER firmware init, matching C driver order.
        // See `nrf_sqspi.c` lines 184-241 (init) and 354-387 (dev_cfg).
        // SCK: output, no pull.
        Self::config_pin_output(&*sck, gpiovals::Pull::DISABLED);
        // IO0-IO3: output+input, pull-up.
        Self::config_pin_io(&*io0, gpiovals::Pull::PULLUP);
        Self::config_pin_io(&*io1, gpiovals::Pull::PULLUP);
        Self::config_pin_io(&*io2, gpiovals::Pull::PULLUP);
        Self::config_pin_io(&*io3, gpiovals::Pull::PULLUP);
        // CSN: output, no pull. Configured in dev_cfg() in C driver.
        Self::config_pin_output(&*csn, gpiovals::Pull::DISABLED);

        // Set up format for 8-bit flash frames (DFS=7, BPP=8, MSB-first, no padding).
        // See `nrf_sqspi.c` lines 243-247.
        sp_regs.format().dfs().write_value(7);
        sp_regs.format().bpp().write_value(8);
        sp_regs.format().bitorder().write_value(0);
        // DR[22] carries effective DFS for the firmware (32 - padding = 32).
        sp_regs.core().dr(22).write_value(32);

        // Enable sQSPI interrupt events (soft peripheral register, not VPR INTENSET).
        // See `nrf_sqspi.c` lines 251-255: enable DMA_DONE, DMA_ABORTED, DMA_DONEJOB.
        sp_regs.intenset().write_value((1 << 5) | (1 << 8) | (1 << 4));

        // ---- Phase 2: Configure device (nrf_sqspi_dev_cfg) ----

        // Configure baud rate.
        // See `nrf_sqspi.c` line 349: clkdiv = SP_VPR_BASE_FREQ_HZ / (sck_freq_khz * 1000)
        let clkdiv = if config.sck_freq_khz > 0 {
            regs::SP_VPR_BASE_FREQ_HZ / (config.sck_freq_khz * 1000)
        } else {
            0
        };
        sp_regs.core().baudr().write_value(clkdiv);
        info!("configured: clkdiv={}, sck_freq_khz={}", clkdiv, config.sck_freq_khz);

        // Configure RX sample delay if requested.
        // See `nrf_sqspi.c` line 404.
        if let Some(delay) = config.sample_delay {
            sp_regs.core().rxsampledelay().write_value(delay as u32);
        }

        // ---- Phase 3: Activate (nrf_sqspi_activate) ----

        // Enable the sQSPI and issue ASB.
        // See `nrf_sqspi.c` lines 507-515 (nrf_sqspi_activate).
        sp_regs.enable().write_value(1);
        info!("activating (ENABLE=1, issuing ASB)");

        let mut driver = Self {
            regs: sp_regs,
            vpr,
            state,
            config,
            task_count: 1,
            _phantom: PhantomData,
        };

        driver.asb();

        // Clear DMA events.
        // See `nrf_sqspi.c` lines 511-513.
        sp_regs.events_dma().done().write_value(0);
        sp_regs.events_dma().aborted().write_value(0);
        sp_regs.events_dma().events_done().job().write_value(0);

        // Enable VPR interrupt AFTER ASB, matching C driver order.
        // See `nrf_sqspi.c` line 515: NRFX_IRQ_ENABLE(SP_VPR_IRQn)
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        info!("init complete");

        Ok(driver)
    }

    // ========================================================================
    // GPIO pin configuration helpers
    // ========================================================================

    /// Configure a pin as output (no input connect) with S0S1 drive and VPR control.
    fn config_pin_output(pin: &impl SealedPin, pull: gpiovals::Pull) {
        pin.set_high();
        pin.conf().write(|w| {
            w.set_dir(gpiovals::Dir::OUTPUT);
            w.set_input(gpiovals::Input::DISCONNECT);
            w.set_pull(pull);
            w.set_drive0(gpiovals::Drive::S);
            w.set_drive1(gpiovals::Drive::S);
            w.set_ctrlsel(gpiovals::Ctrlsel::VPR);
        });
    }

    /// Configure a pin as output+input with S0S1 drive and VPR control (data lines).
    fn config_pin_io(pin: &impl SealedPin, pull: gpiovals::Pull) {
        pin.set_high();
        pin.conf().write(|w| {
            w.set_dir(gpiovals::Dir::OUTPUT);
            w.set_input(gpiovals::Input::CONNECT);
            w.set_pull(pull);
            w.set_drive0(gpiovals::Drive::S);
            w.set_drive1(gpiovals::Drive::S);
            w.set_ctrlsel(gpiovals::Ctrlsel::VPR);
        });
    }

    // ========================================================================
    // VPR task trigger helper
    // ========================================================================

    /// Trigger a VPR task by index.
    ///
    /// The nRF54L VPR has `TASKS_TRIGGER[23]` at offset 0x000 from the VPR base,
    /// with software-usable indices 16..22 (see `nrf54l15_types.h` line 42967:
    /// `VPR_TASKS_TRIGGER_MinIndex=16`, `VPR_TASKS_TRIGGER_MaxIndex=22`).
    fn vpr_trigger_task(&self, task_idx: usize) {
        let pac_ptr = self.vpr.tasks_trigger(task_idx).as_ptr();
        let expected_ptr = unsafe { (self.vpr.as_ptr() as *const u32).add(task_idx) };
        trace!(
            "vpr_trigger_task({}): pac_ptr=0x{:08x}, expected_ptr=0x{:08x}",
            task_idx, pac_ptr as u32, expected_ptr as u32
        );
        if pac_ptr as u32 != expected_ptr as u32 {
            warn!(
                "vpr_trigger_task: ADDRESS MISMATCH! pac=0x{:08x} vs expected=0x{:08x} (diff={})",
                pac_ptr as u32, expected_ptr as u32,
                pac_ptr as i32 - expected_ptr as i32
            );
        }
        self.vpr.tasks_trigger(task_idx).write_value(1);
    }

    // ========================================================================
    // Synchronization barriers
    // ========================================================================

    /// Extended Synchronization Barrier (XSB) implementation.
    ///
    /// Ports the `__XSBx` macro from `softperipheral_regif.h` (lines 45-55):
    /// 1. Write task counter to SPSYNC.AUX[0]
    /// 2. Trigger VPR TASKS_TRIGGER[task_idx]
    /// 3. Poll until AUX[0] == AUX[1] (FLPR acknowledgement)
    /// 4. Increment task counter
    fn xsb(&mut self, task_idx: usize) {
        let spsync = self.regs.spsync();
        trace!("xsb: task_idx={}, task_count={}", task_idx, self.task_count);
        spsync.aux(0).write_value(self.task_count);
        // DMB ensures the AUX[0] write is visible to the VPR core before we
        // trigger the task.  The C driver does this via sp_handshake_set().
        cortex_m::asm::dmb();
        self.vpr_trigger_task(task_idx);
        let mut spin_count: u32 = 0;
        loop {
            let a0 = spsync.aux(0).read();
            let a1 = spsync.aux(1).read();
            if a0 == a1 {
                break;
            }
            cortex_m::asm::nop();
            cortex_m::asm::nop();
            cortex_m::asm::nop();
            spin_count += 1;
            if spin_count % 1_000_000 == 0 {
                let cpurun_ptr = self.vpr.cpurun().as_ptr() as *const u32;
                let cpurun_val = unsafe { cpurun_ptr.read_volatile() };
                let dmstatus_ptr = self.vpr.debugif().dmstatus().as_ptr() as *const u32;
                let dmstatus_val = unsafe { dmstatus_ptr.read_volatile() };
                // Read EVENTS_TRIGGERED for indices 16..21 to check VPR event state.
                let mut evts = [0u32; 6];
                for i in 0..6 {
                    evts[i] = self.vpr.events_triggered(16 + i).read();
                }
                warn!(
                    "xsb: STUCK task_idx={}, task_count={}, aux[0]={}, aux[1]={}, spins={}, enable={}, cpurun=0x{:08x}, dmstatus=0x{:08x}",
                    task_idx,
                    self.task_count,
                    a0,
                    a1,
                    spin_count,
                    self.regs.enable().read(),
                    cpurun_val,
                    dmstatus_val,
                );
                warn!(
                    "xsb: events_triggered[16..21] = [{}, {}, {}, {}, {}, {}]",
                    evts[0], evts[1], evts[2], evts[3], evts[4], evts[5]
                );
            }
        }
        trace!(
            "xsb: done, aux[0]={}, aux[1]={}, spins={}",
            spsync.aux(0).read(),
            spsync.aux(1).read(),
            spin_count
        );
        self.task_count = self.task_count.wrapping_add(1);
    }

    /// Config Synchronization Barrier.
    /// See `softperipheral_regif.h`: `__CSB(R)`.
    fn csb(&mut self) {
        trace!("CSB");
        self.xsb(regs::SP_VPR_TASK_CONFIG_IDX);
    }

    /// Action Synchronization Barrier.
    /// See `softperipheral_regif.h`: `__ASB(R)`.
    fn asb(&mut self) {
        trace!("ASB");
        self.xsb(regs::SP_VPR_TASK_ACTION_IDX);
    }

    /// Stop Synchronization Barrier.
    /// See `softperipheral_regif.h`: `__SSB(R)`.
    #[allow(dead_code)]
    fn ssb(&mut self) {
        trace!("SSB");
        self.xsb(regs::SP_VPR_TASK_STOP_IDX);
    }

    // ========================================================================
    // Transfer implementation
    // ========================================================================

    /// Set up and start a flash transfer.
    ///
    /// Set up and start a flash transfer.
    ///
    /// Simplified from the C driver's `xfer_common` + `nrf_sqspi_xfer`
    /// (`nrf_sqspi.c` lines 553-706) for byte-oriented flash operations.
    fn start_transfer(&mut self, opcode: u8, address: u32, data_ptr: *mut u8, data_len: usize, dir: TransferDir) {
        info!(
            "start_transfer: opcode=0x{:02x}, addr=0x{:08x}, data_ptr=0x{:08x}, len={}, dir={}",
            opcode, address, data_ptr as u32, data_len, dir as u32
        );

        let sp = self.regs;
        let core = sp.core();
        let format = sp.format();

        // Number of data frames and bytes to transfer.
        let ndf = data_len as u32;
        format.pixels().write_value(ndf);
        core.ctrlr1().write_value(ndf);
        core.dr(23).write_value(ndf);
        // 8-bit command => cilen = 1 (one 32-bit word).
        format.cilen().write_value(1);

        // CTRLR0: 8-bit frames, SPI mode, clock polarity/phase, transfer direction.
        // See `nrf_sp_qspi.h`: `QSPI_CORE_CORE_CTRLR0_*` defines.
        let scph: u32 = match self.config.spi_mode.phase {
            Phase::CaptureOnFirstTransition => 0,
            Phase::CaptureOnSecondTransition => 1,
        };
        let scpol: u32 = match self.config.spi_mode.polarity {
            Polarity::IdleLow => 1,  // INACTIVEHIGH
            Polarity::IdleHigh => 0, // INACTIVELOW
        };
        let spi_frf: u32 = match self.config.lines {
            SpiLines::Single => 0,
            SpiLines::Dual1_1_2 | SpiLines::Dual1_2_2 => 1,
            SpiLines::Quad1_1_4 | SpiLines::Quad1_4_4 => 2,
        };
        let ctrlr0 = 7u32              // DFS = 7 (8-bit frames)
            | (scph << 8)
            | (scpol << 9)
            | ((dir as u32) << 10)      // TMOD
            | (spi_frf << 22)           // SPI_FRF
            | (1 << 31); // SQSPIISMST = controller
        core.ctrlr0().write_value(ctrlr0);
        trace!("CTRLR0=0x{:08x}", ctrlr0);

        // SPICTRLR0: multi-line mode, address length, 8-bit instruction, dummy cycles.
        // See `nrf_sp_qspi.h`: `QSPI_CORE_CORE_SPICTRLR0_*` defines.
        let transtype: u32 = match self.config.lines {
            SpiLines::Single | SpiLines::Dual1_1_2 | SpiLines::Quad1_1_4 => 0,
            SpiLines::Dual1_2_2 | SpiLines::Quad1_4_4 => 1,
        };
        let addr_len_bits: u32 = match self.config.address_mode {
            AddressMode::_24Bit => 24,
            AddressMode::_32Bit => 32,
        };
        let wait_cycles: u32 = match self.config.lines {
            SpiLines::Single => 0,
            _ => 8,
        };
        let spictrlr0 = transtype
            | (((addr_len_bits / 4) & 0xF) << 2)   // ADDRL
            | (2 << 8)                               // INSTL = 8-bit
            | ((wait_cycles & 0x1F) << 11); // WAITCYCLES
        core.spictrlr0().write_value(spictrlr0);
        trace!("SPICTRLR0=0x{:08x}", spictrlr0);

        // Write command, address, data pointer, data length to DR registers.
        core.dr(0).write_value(opcode as u32);
        core.dr(1).write_value(address);
        core.dr(2).write_value(0); // Upper address bits (always 0 for 24/32-bit).
        core.dr(3).write_value(data_ptr as u32);
        core.dr(4).write_value(data_len as u32);
        trace!(
            "DR[0..4]: cmd=0x{:02x}, addr=0x{:08x}, upper=0, ptr=0x{:08x}, len={}",
            opcode, address, data_ptr as u32, data_len
        );

        // Synchronize config, enable core, synchronize action, trigger transfer.
        self.csb();
        core.sqspienr().write_value(1);
        trace!("SQSPIENR=1");
        self.asb();
        info!("triggering DPPI_0 (task_idx={})", regs::SP_VPR_TASK_DPPI_0_IDX);
        self.vpr_trigger_task(regs::SP_VPR_TASK_DPPI_0_IDX);
    }

    /// Wait for the DMA DONE event, then disable the core.
    async fn wait_done(&mut self) {
        trace!("wait_done: waiting for DMA DONE event");
        poll_fn(|cx| {
            self.state.waker.register(cx.waker());
            let done = self.regs.events_dma().done().read();
            let aborted = self.regs.events_dma().aborted().read();
            let donejob = self.regs.events_dma().events_done().job().read();
            trace!(
                "wait_done poll: done={}, aborted={}, donejob={}",
                done, aborted, donejob
            );
            if done != 0 {
                // Clear the event and disable the core.
                // See `nrf_sqspi.c` lines 761-764.
                self.regs.events_dma().done().write_value(0);
                self.regs.core().sqspienr().write_value(0);
                self.asb();
                info!("wait_done: DMA DONE received");
                Poll::Ready(())
            } else {
                if aborted != 0 {
                    warn!("wait_done: DMA ABORTED event detected!");
                }
                trace!("wait_done: pending");
                Poll::Pending
            }
        })
        .await
    }

    /// Blocking wait for DMA DONE.
    fn blocking_wait_done(&mut self) {
        trace!("blocking_wait_done: waiting for DMA DONE event");
        let mut spin_count: u32 = 0;
        while self.regs.events_dma().done().read() == 0 {
            spin_count += 1;
            if spin_count % 100_000 == 0 {
                let aborted = self.regs.events_dma().aborted().read();
                let donejob = self.regs.events_dma().events_done().job().read();
                warn!(
                    "blocking_wait_done: still waiting after {} spins, aborted={}, donejob={}",
                    spin_count, aborted, donejob
                );
            }
        }
        self.regs.events_dma().done().write_value(0);
        self.regs.core().sqspienr().write_value(0);
        self.asb();
        info!("blocking_wait_done: DMA DONE received after {} spins", spin_count);
    }

    // ========================================================================
    // Public API
    // ========================================================================

    /// Read data from the flash memory.
    pub async fn read(&mut self, address: u32, data: &mut [u8]) -> Result<(), Error> {
        info!("read: addr=0x{:08x}, len={}", address, data.len());
        if data.is_empty() {
            return Ok(());
        }
        self.bounds_check(address, data.len())?;
        self.start_transfer(
            self.config.read_opcode,
            address,
            data.as_mut_ptr(),
            data.len(),
            TransferDir::RxOnly,
        );
        self.wait_done().await;
        Ok(())
    }

    /// Write data to the flash memory.
    pub async fn write(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        info!("write: addr=0x{:08x}, len={}", address, data.len());
        if data.is_empty() {
            return Ok(());
        }
        self.bounds_check(address, data.len())?;
        self.start_transfer(
            self.config.write_opcode,
            address,
            data.as_ptr() as *mut u8,
            data.len(),
            TransferDir::TxOnly,
        );
        self.wait_done().await;
        Ok(())
    }

    /// Erase a 4KB sector at the given address.
    pub async fn erase(&mut self, address: u32) -> Result<(), Error> {
        info!("erase: addr=0x{:08x}", address);
        if self.config.capacity > 0 && address >= self.config.capacity {
            return Err(Error::OutOfBounds);
        }
        // Sector erase command (0x20) with no data.
        self.start_transfer(0x20, address, ptr::null_mut(), 0, TransferDir::TxOnly);
        self.wait_done().await;
        Ok(())
    }

    /// Execute a custom SPI instruction.
    pub async fn custom_instruction(&mut self, opcode: u8, req: &[u8], resp: &mut [u8]) -> Result<(), Error> {
        info!("custom_instruction: opcode=0x{:02x}, req_len={}, resp_len={}", opcode, req.len(), resp.len());
        let dir = if !resp.is_empty() {
            TransferDir::RxOnly
        } else {
            TransferDir::TxOnly
        };
        let (data_ptr, data_len) = if !resp.is_empty() {
            (resp.as_mut_ptr(), resp.len())
        } else {
            (req.as_ptr() as *mut u8, req.len())
        };
        self.start_transfer(opcode, 0, data_ptr, data_len, dir);
        self.wait_done().await;
        Ok(())
    }

    /// Read data from the flash memory, blocking version.
    pub fn blocking_read(&mut self, address: u32, data: &mut [u8]) -> Result<(), Error> {
        info!("blocking_read: addr=0x{:08x}, len={}", address, data.len());
        if data.is_empty() {
            return Ok(());
        }
        self.bounds_check(address, data.len())?;
        self.start_transfer(
            self.config.read_opcode,
            address,
            data.as_mut_ptr(),
            data.len(),
            TransferDir::RxOnly,
        );
        self.blocking_wait_done();
        Ok(())
    }

    /// Write data to the flash memory, blocking version.
    pub fn blocking_write(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        info!("blocking_write: addr=0x{:08x}, len={}", address, data.len());
        if data.is_empty() {
            return Ok(());
        }
        self.bounds_check(address, data.len())?;
        self.start_transfer(
            self.config.write_opcode,
            address,
            data.as_ptr() as *mut u8,
            data.len(),
            TransferDir::TxOnly,
        );
        self.blocking_wait_done();
        Ok(())
    }

    /// Erase a 4KB sector, blocking version.
    pub fn blocking_erase(&mut self, address: u32) -> Result<(), Error> {
        info!("blocking_erase: addr=0x{:08x}", address);
        if self.config.capacity > 0 && address >= self.config.capacity {
            return Err(Error::OutOfBounds);
        }
        self.start_transfer(0x20, address, ptr::null_mut(), 0, TransferDir::TxOnly);
        self.blocking_wait_done();
        Ok(())
    }

    fn bounds_check(&self, address: u32, len: usize) -> Result<(), Error> {
        if self.config.capacity == 0 {
            return Ok(());
        }
        let len_u32: u32 = len.try_into().map_err(|_| Error::OutOfBounds)?;
        let end_address = address.checked_add(len_u32).ok_or(Error::OutOfBounds)?;
        if end_address > self.config.capacity {
            return Err(Error::OutOfBounds);
        }
        Ok(())
    }
}

// ============================================================================
// Drop
// ============================================================================

impl<'d> Drop for Sqspi<'d> {
    fn drop(&mut self) {
        // Deactivate: disable sQSPI and stop VPR.
        // See `nrf_sqspi.c` lines 276-320 (nrf_sqspi_uninit).

        // Disable the sQSPI core.
        self.regs.core().sqspienr().write_value(0);
        self.regs.enable().write_value(0);
        self.asb();

        // Stop VPR.
        self.vpr.cpurun().write(|w| w.set_en(pac::vpr::vals::CpurunEn::STOPPED));

        // Reset VPR via DEBUGIF.DMCONTROL.
        // See `nrf_sqspi.c` lines 309-318.
        self.vpr.debugif().dmcontrol().write(|w| {
            w.set_ndmreset(true);
            w.set_dmactive(true);
        });
        self.vpr.debugif().dmcontrol().write(|w| {
            w.set_ndmreset(false);
            w.set_dmactive(false);
        });
    }
}

// ============================================================================
// embedded-storage traits
// ============================================================================

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        NorFlashErrorKind::Other
    }
}

impl<'d> ErrorType for Sqspi<'d> {
    type Error = Error;
}

impl<'d> ReadNorFlash for Sqspi<'d> {
    const READ_SIZE: usize = 4;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.config.capacity as usize
    }
}

impl<'d> NorFlash for Sqspi<'d> {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = 4096;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        for address in (from..to).step_by(<Self as NorFlash>::ERASE_SIZE) {
            self.blocking_erase(address)?;
        }
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(offset, bytes)
    }
}

// ============================================================================
// embedded-storage-async traits
// ============================================================================

mod _async {
    use embedded_storage_async::nor_flash::{NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash};

    use super::*;

    impl<'d> AsyncReadNorFlash for Sqspi<'d> {
        const READ_SIZE: usize = 4;

        async fn read(&mut self, address: u32, data: &mut [u8]) -> Result<(), Self::Error> {
            self.read(address, data).await
        }

        fn capacity(&self) -> usize {
            self.config.capacity as usize
        }
    }

    impl<'d> AsyncNorFlash for Sqspi<'d> {
        const WRITE_SIZE: usize = 4;
        const ERASE_SIZE: usize = 4096;

        async fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
            self.write(offset, data).await
        }

        async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
            for address in (from..to).step_by(<Self as AsyncNorFlash>::ERASE_SIZE) {
                self.erase(address).await?;
            }
            Ok(())
        }
    }
}

// ============================================================================
// Instance traits and state
// ============================================================================

/// Peripheral static state.
pub(crate) struct State {
    waker: AtomicWaker,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

pub(crate) trait SealedInstance {
    fn vpr_regs() -> pac::vpr::Vpr;
    fn state() -> &'static State;
}

/// sQSPI peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for the VPR co-processor driving this sQSPI instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! impl_sqspi {
    ($type:ident, $pac_vpr:ident, $irq:ident) => {
        impl crate::sqspi::SealedInstance for peripherals::$type {
            fn vpr_regs() -> pac::vpr::Vpr {
                pac::$pac_vpr
            }
            fn state() -> &'static crate::sqspi::State {
                static STATE: crate::sqspi::State = crate::sqspi::State::new();
                &STATE
            }
        }
        impl crate::sqspi::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }
    };
}
