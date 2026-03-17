//! sQSPI virtual register interface.
//!
//! Register structs mirroring the C `NRF_SP_QSPI_Type` layout from the Nordic
//! SDK header `nrf_sp_qspi.h`. Since the soft peripheral registers live in RAM
//! rather than at a fixed MMIO address, they are defined here using
//! `nrf_pac::common::Reg` for type-safe volatile access.
//!
//! Synchronization barrier constants originate from `softperipheral_regif.h`.

use crate::pac::common::{Reg, R, RW, W};

// ============================================================================
// Top-level register block
// ============================================================================

/// sQSPI virtual register interface mapped in shared RAM.
///
/// This struct provides accessor methods for the full register set of the soft
/// QSPI peripheral. It mirrors the C type `NRF_SP_QSPI_Type` defined in the
/// Nordic SDK header `nrf_sp_qspi.h` (lines 2480-2500).
///
/// The memory layout from the base pointer is:
///
/// | Offset | Size  | Field         | C type                           |
/// |--------|-------|---------------|----------------------------------|
/// | 0x000  | 0x004 | TASKS_START   | `uint32_t`                       |
/// | 0x004  | 0x004 | TASKS_RESET   | `uint32_t`                       |
/// | 0x008  | 0x004 | EVENTS_CORE   | `uint32_t`                       |
/// | 0x00C  | 0x030 | EVENTS_DMA    | `NRF_QSPI_EVENTS_DMA_Type`      |
/// | 0x03C  | 0x004 | EVENTS_IDLE   | `uint32_t`                       |
/// | 0x040  | 0x004 | SHORTS        | `uint32_t`                       |
/// | 0x044  | 0x004 | INTEN         | `uint32_t`                       |
/// | 0x048  | 0x004 | INTENSET      | `uint32_t`                       |
/// | 0x04C  | 0x004 | INTENCLR      | `uint32_t`                       |
/// | 0x050  | 0x004 | INTPEND       | `uint32_t`                       |
/// | 0x054  | 0x004 | ENABLE        | `uint32_t`                       |
/// | 0x058  | 0x014 | CONFIG        | `NRF_QSPI_CONFIG_Type`           |
/// | 0x06C  | 0x014 | FORMAT        | `NRF_QSPI_FORMAT_Type`           |
/// | 0x080  | 0x028 | DMA           | `NRF_QSPI_DMA_Type`             |
/// | 0x0A8  | 0x100 | CORE          | `NRF_QSPI_CORE_Type`            |
/// | 0x1A8  | 0x010 | SPSYNC        | `NRF_QSPI_SPSYNC_Type`          |
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Regs {
    ptr: *mut u8,
}

unsafe impl Send for Regs {}
unsafe impl Sync for Regs {}

impl Regs {
    /// Create a register block from a raw pointer to the base of the register region.
    ///
    /// # Safety
    /// The pointer must point to a valid, properly-aligned RAM region of at least
    /// [`SIZE`](Self::SIZE) bytes that will remain valid for the lifetime of this struct.
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }

    /// Returns the raw pointer to the base of the register block.
    pub const fn as_ptr(&self) -> *mut () {
        self.ptr as _
    }

    /// Total size of the `NRF_SP_QSPI_Type` register block in bytes.
    pub const SIZE: usize = 0x1B8;

    /// `TASKS_START` - Start operation (offset 0x000, write-only).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.TASKS_START`.
    pub const fn tasks_start(self) -> Reg<u32, W> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x000) as _) }
    }

    /// `TASKS_RESET` - Reset the QSPI (offset 0x004, write-only).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.TASKS_RESET`.
    pub const fn tasks_reset(self) -> Reg<u32, W> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x004) as _) }
    }

    /// `EVENTS_CORE` - Interrupt from the QSPI core (offset 0x008).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.EVENTS_CORE`.
    pub const fn events_core(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x008) as _) }
    }

    /// `EVENTS_DMA` - DMA peripheral events (offset 0x00C, size 0x30).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.EVENTS_DMA` / `NRF_QSPI_EVENTS_DMA_Type`.
    pub const fn events_dma(self) -> EventsDma {
        unsafe { EventsDma::from_ptr(self.ptr.wrapping_add(0x00C) as _) }
    }

    /// `EVENTS_IDLE` - QSPI core idle event (offset 0x03C).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.EVENTS_IDLE`.
    pub const fn events_idle(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x03C) as _) }
    }

    /// `SHORTS` - Shortcuts between local events and tasks (offset 0x040).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.SHORTS`.
    pub const fn shorts(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x040) as _) }
    }

    /// `INTEN` - Enable or disable interrupt (offset 0x044).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.INTEN`.
    pub const fn inten(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x044) as _) }
    }

    /// `INTENSET` - Enable interrupt (offset 0x048).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.INTENSET`.
    pub const fn intenset(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x048) as _) }
    }

    /// `INTENCLR` - Disable interrupt (offset 0x04C).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.INTENCLR`.
    pub const fn intenclr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x04C) as _) }
    }

    /// `INTPEND` - Pending interrupts (offset 0x050, read-only).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.INTPEND`.
    pub const fn intpend(self) -> Reg<u32, R> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x050) as _) }
    }

    /// `ENABLE` - Enable the QSPI / request clock for the IP core (offset 0x054).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.ENABLE`.
    pub const fn enable(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x054) as _) }
    }

    /// `CONFIG` - DMA configuration registers (offset 0x058, size 0x14).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.CONFIG` / `NRF_QSPI_CONFIG_Type`.
    pub const fn config(self) -> RegConfig {
        unsafe { RegConfig::from_ptr(self.ptr.wrapping_add(0x058) as _) }
    }

    /// `FORMAT` - Data format configuration registers (offset 0x06C, size 0x14).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.FORMAT` / `NRF_QSPI_FORMAT_Type`.
    pub const fn format(self) -> Format {
        unsafe { Format::from_ptr(self.ptr.wrapping_add(0x06C) as _) }
    }

    /// `DMA` - DMA status and configuration registers (offset 0x080, size 0x28).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.DMA` / `NRF_QSPI_DMA_Type`.
    pub const fn dma(self) -> Dma {
        unsafe { Dma::from_ptr(self.ptr.wrapping_add(0x080) as _) }
    }

    /// `CORE` - QSPI SPI controller core registers (offset 0x0A8, size 0x100).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.CORE` / `NRF_QSPI_CORE_CORE_Type`.
    pub const fn core(self) -> Core {
        unsafe { Core::from_ptr(self.ptr.wrapping_add(0x0A8) as _) }
    }

    /// `SPSYNC` - Handshake registers for synchronization barriers (offset 0x1A8, size 0x10).
    /// See `nrf_sp_qspi.h`: `NRF_SP_QSPI_Type.SPSYNC` / `NRF_QSPI_SPSYNC_Type`.
    pub const fn spsync(self) -> Spsync {
        unsafe { Spsync::from_ptr(self.ptr.wrapping_add(0x1A8) as _) }
    }
}

// ============================================================================
// EVENTS_DMA (offset 0x00C, size 0x30 = 48 bytes)
// ============================================================================

/// DMA peripheral events.
///
/// Mirrors `NRF_QSPI_EVENTS_DMA_Type` from `nrf_sp_qspi.h` (lines 140-163).
///
/// | Offset | Field              |
/// |--------|--------------------|
/// | 0x00   | EVENTS_DONE (5xu32)|
/// | 0x14   | ERROR              |
/// | 0x18   | PAUSED             |
/// | 0x1C   | RESET              |
/// | 0x20   | DONE               |
/// | 0x24   | TXUNEXPECTEDIDLE   |
/// | 0x28   | INTERNALBUSERROR   |
/// | 0x2C   | ABORTED            |
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct EventsDma {
    ptr: *mut u8,
}

unsafe impl Send for EventsDma {}
unsafe impl Sync for EventsDma {}

impl EventsDma {
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }

    /// `EVENTS_DONE` - Granular DMA completion events (offset 0x00, size 0x14).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_EVENTS_DMA_EVENTS_DONE_Type`.
    pub const fn events_done(self) -> EventsDmaDone {
        unsafe { EventsDmaDone::from_ptr(self.ptr.wrapping_add(0x00) as _) }
    }

    /// `ERROR` - AXI bus error received (offset 0x14).
    pub const fn error(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x14) as _) }
    }

    /// `PAUSED` - DMA paused with task TASKS_PAUSE (offset 0x18).
    pub const fn paused(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x18) as _) }
    }

    /// `RESET` - DMA reset with task TASKS_RESET (offset 0x1C).
    pub const fn reset(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x1C) as _) }
    }

    /// `DONE` - DMA transfer done (offset 0x20).
    pub const fn done(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x20) as _) }
    }

    /// `TXUNEXPECTEDIDLE` - TX buffer underflow caused unexpected idle (offset 0x24).
    pub const fn txunexpectedidle(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x24) as _) }
    }

    /// `INTERNALBUSERROR` - Internal AHB bus error during transfer (offset 0x28).
    pub const fn internalbuserror(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x28) as _) }
    }

    /// `ABORTED` - DMA aborted due to error (offset 0x2C).
    pub const fn aborted(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x2C) as _) }
    }
}

/// Granular DMA completion events.
///
/// Mirrors `NRF_QSPI_EVENTS_DMA_EVENTS_DONE_Type` from `nrf_sp_qspi.h` (lines 22-33).
///
/// | Offset | Field     |
/// |--------|-----------|
/// | 0x00   | LIST      |
/// | 0x04   | LISTPART  |
/// | 0x08   | SELECTJOB |
/// | 0x0C   | DATA      |
/// | 0x10   | JOB       |
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct EventsDmaDone {
    ptr: *mut u8,
}

unsafe impl Send for EventsDmaDone {}
unsafe impl Sync for EventsDmaDone {}

impl EventsDmaDone {
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }

    /// `LIST` - Descriptor list is complete (offset 0x00).
    pub const fn list(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x00) as _) }
    }

    /// `LISTPART` - Descriptor list is partially complete (offset 0x04).
    pub const fn listpart(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x04) as _) }
    }

    /// `SELECTJOB` - Selected job is completed (offset 0x08).
    pub const fn selectjob(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x08) as _) }
    }

    /// `DATA` - Job data has been completely transferred (offset 0x0C).
    pub const fn data(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x0C) as _) }
    }

    /// `JOB` - A job has been fetched from the joblist (offset 0x10).
    pub const fn job(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x10) as _) }
    }
}

// ============================================================================
// CONFIG (offset 0x058, size 0x14 = 20 bytes)
// ============================================================================

/// DMA configuration registers.
///
/// Mirrors `NRF_QSPI_CONFIG_Type` from `nrf_sp_qspi.h` (lines 313-321).
///
/// | Offset | Field             |
/// |--------|-------------------|
/// | 0x00   | TXBURSTLENGTH     |
/// | 0x04   | RXBURSTLENGTH     |
/// | 0x08   | RXTRANSFERLENGTH  |
/// | 0x0C   | STOPON            |
/// | 0x10   | AXIMODE           |
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RegConfig {
    ptr: *mut u8,
}

unsafe impl Send for RegConfig {}
unsafe impl Sync for RegConfig {}

impl RegConfig {
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }

    /// `TXBURSTLENGTH` - Transmit burst length, bits [0:4] = AMOUNT (offset 0x00).
    pub const fn txburstlength(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x00) as _) }
    }

    /// `RXBURSTLENGTH` - Receive burst length, bits [0:4] = AMOUNT (offset 0x04).
    pub const fn rxburstlength(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x04) as _) }
    }

    /// `RXTRANSFERLENGTH` - RX transfer length, bits [0:17] = AMOUNT (offset 0x08).
    pub const fn rxtransferlength(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x08) as _) }
    }

    /// `STOPON` - Stop conditions, bits [0:3] (offset 0x0C).
    pub const fn stopon(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x0C) as _) }
    }

    /// `AXIMODE` - AXI mode, bit 4 = AXIMODE, bit 5 = MODE (offset 0x10).
    pub const fn aximode(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x10) as _) }
    }
}

// ============================================================================
// FORMAT (offset 0x06C, size 0x14 = 20 bytes)
// ============================================================================

/// Data format configuration registers.
///
/// Mirrors `NRF_QSPI_FORMAT_Type` from `nrf_sp_qspi.h` (lines 448-455).
///
/// | Offset | Field    |
/// |--------|----------|
/// | 0x00   | DFS      |
/// | 0x04   | BPP      |
/// | 0x08   | PIXELS   |
/// | 0x0C   | CILEN    |
/// | 0x10   | BITORDER |
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Format {
    ptr: *mut u8,
}

unsafe impl Send for Format {}
unsafe impl Sync for Format {}

impl Format {
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }

    /// `DFS` - Data frame size, bits [0:5] (offset 0x00).
    pub const fn dfs(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x00) as _) }
    }

    /// `BPP` - Bits per pixel, bits [0:5] (offset 0x04).
    pub const fn bpp(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x04) as _) }
    }

    /// `PIXELS` - Number of pixels, bits [0:17] (offset 0x08).
    pub const fn pixels(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x08) as _) }
    }

    /// `CILEN` - Command/instruction length, bits [0:1] (offset 0x0C).
    pub const fn cilen(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x0C) as _) }
    }

    /// `BITORDER` - Bit order; bit 0 = COMMAND order, bit 1 = DATA order (offset 0x10).
    pub const fn bitorder(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x10) as _) }
    }
}

// ============================================================================
// DMA (offset 0x080, size 0x28 = 40 bytes)
// ============================================================================

/// DMA status and configuration registers.
///
/// Mirrors `NRF_QSPI_DMA_Type` from `nrf_sp_qspi.h` (lines 705-709).
/// Contains a `STATUS` sub-struct (7 x u32 = 0x1C) followed by a `CONFIG`
/// sub-struct (3 x u32 = 0x0C).
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Dma {
    ptr: *mut u8,
}

unsafe impl Send for Dma {}
unsafe impl Sync for Dma {}

impl Dma {
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }

    /// `STATUS` - DMA status registers (offset 0x00, size 0x1C).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_DMA_STATUS_Type`.
    pub const fn status(self) -> DmaStatus {
        unsafe { DmaStatus::from_ptr(self.ptr.wrapping_add(0x00) as _) }
    }

    /// `CONFIG` - DMA configuration registers (offset 0x1C, size 0x0C).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_DMA_CONFIG_Type`.
    pub const fn config(self) -> DmaConfig {
        unsafe { DmaConfig::from_ptr(self.ptr.wrapping_add(0x1C) as _) }
    }
}

/// DMA status registers (all read-only).
///
/// Mirrors `NRF_QSPI_DMA_STATUS_Type` from `nrf_sp_qspi.h` (lines 513-523).
///
/// | Offset | Field     |
/// |--------|-----------|
/// | 0x00   | BYTECOUNT |
/// | 0x04   | ATTRIBUTE |
/// | 0x08   | ADDRESS   |
/// | 0x0C   | JOBCOUNT  |
/// | 0x10   | BUSERROR  |
/// | 0x14   | FIFO      |
/// | 0x18   | ACTIVE    |
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DmaStatus {
    ptr: *mut u8,
}

unsafe impl Send for DmaStatus {}
unsafe impl Sync for DmaStatus {}

impl DmaStatus {
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }

    /// `BYTECOUNT` - Bytes sent/received (offset 0x00).
    pub const fn bytecount(self) -> Reg<u32, R> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x00) as _) }
    }

    /// `ATTRIBUTE` - Latest job attribute (offset 0x04).
    pub const fn attribute(self) -> Reg<u32, R> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x04) as _) }
    }

    /// `ADDRESS` - Latest address (offset 0x08).
    pub const fn address(self) -> Reg<u32, R> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x08) as _) }
    }

    /// `JOBCOUNT` - Completed job count (offset 0x0C).
    pub const fn jobcount(self) -> Reg<u32, R> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x0C) as _) }
    }

    /// `BUSERROR` - Bus error status (offset 0x10).
    pub const fn buserror(self) -> Reg<u32, R> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x10) as _) }
    }

    /// `FIFO` - FIFO status (offset 0x14).
    pub const fn fifo(self) -> Reg<u32, R> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x14) as _) }
    }

    /// `ACTIVE` - DMA activity state (offset 0x18).
    pub const fn active(self) -> Reg<u32, R> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x18) as _) }
    }
}

/// DMA configuration registers.
///
/// Mirrors `NRF_QSPI_DMA_CONFIG_Type` from `nrf_sp_qspi.h` (lines 656-664).
///
/// | Offset | Field           |
/// |--------|-----------------|
/// | 0x00   | BUFFERFILL      |
/// | 0x04   | LISTPTR         |
/// | 0x08   | LISTPARTTHRESH  |
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DmaConfig {
    ptr: *mut u8,
}

unsafe impl Send for DmaConfig {}
unsafe impl Sync for DmaConfig {}

impl DmaConfig {
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }

    /// `BUFFERFILL` - Data for BufferFill attribute (offset 0x00).
    pub const fn bufferfill(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x00) as _) }
    }

    /// `LISTPTR` - Descriptor list start address (offset 0x04).
    pub const fn listptr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x04) as _) }
    }

    /// `LISTPARTTHRESH` - Threshold for partial completion, bits [0:15] (offset 0x08).
    pub const fn listpartthresh(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x08) as _) }
    }
}

// ============================================================================
// CORE (offset 0x0A8, size 0x100 = 256 bytes)
// ============================================================================

/// QSPI SPI controller core registers.
///
/// Mirrors `NRF_QSPI_CORE_CORE_Type` from `nrf_sp_qspi.h` (lines 715-746).
/// Note: `NRF_QSPI_CORE_Type` (lines 2452-2455) is a single-field wrapper
/// around `NRF_QSPI_CORE_CORE_Type`; we flatten them into one struct.
///
/// | Offset | Field            | Description                              |
/// |--------|------------------|------------------------------------------|
/// | 0x00   | CTRLR0           | Control Register 0                       |
/// | 0x04   | CTRLR1           | Control Register 1 (NDF)                 |
/// | 0x08   | SQSPIENR         | QSPI Enable                              |
/// | 0x0C   | MWCR             | Microwire Control                        |
/// | 0x10   | SER              | Slave Enable                             |
/// | 0x14   | BAUDR            | Baud Rate Select                         |
/// | 0x18   | TXFTLR           | TX FIFO Threshold Level                  |
/// | 0x1C   | RXFTLR           | RX FIFO Threshold Level                  |
/// | 0x20   | TXFLR            | TX FIFO Level                            |
/// | 0x24   | RXFLR            | RX FIFO Level                            |
/// | 0x28   | SR               | Status Register                          |
/// | 0x2C   | IMR              | Interrupt Mask                           |
/// | 0x30   | ISR              | Interrupt Status                         |
/// | 0x34   | RISR             | Raw Interrupt Status                     |
/// | 0x38   | TXEICR           | TX Error Interrupt Clear                 |
/// | 0x3C   | RXOICR           | RX Overflow Interrupt Clear              |
/// | 0x40   | RXUICR           | RX Underflow Interrupt Clear             |
/// | 0x44   | MSTICR           | Multi-Controller Interrupt Clear         |
/// | 0x48   | ICR              | Interrupt Clear                          |
/// | 0x4C   | DMACR            | DMA Control                              |
/// | 0x50   | DMATDLR          | DMA TX Data Level                        |
/// | 0x54   | DMARDLR          | DMA RX Data Level                        |
/// | 0x58   | IDR              | Identification                           |
/// | 0x5C   | SQSPICVERSIONID  | Version ID                               |
/// | 0x60   | DR\[36\]         | Data Registers (36 x u32)                |
/// | 0xF0   | RXSAMPLEDELAY    | RX Sample Delay                          |
/// | 0xF4   | SPICTRLR0        | SPI Control Register 0                   |
/// | 0xF8   | SPICTRLR1        | SPI Control Register 1                   |
/// | 0xFC   | SPITECR          | SPI Transmit Error Interrupt Clear       |
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Core {
    ptr: *mut u8,
}

unsafe impl Send for Core {}
unsafe impl Sync for Core {}

impl Core {
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }

    /// `CTRLR0` - Control Register 0 (offset 0x00).
    ///
    /// Contains: DFS (bits 0-4), FRF (bits 6-7), SCPH (bit 8), SCPOL (bit 9),
    /// TMOD (bits 10-11), SLV_OE (bit 12), SRL (bit 13), SSTE (bit 14),
    /// CFS (bits 16-19), DFS_32 (bits 21-25), SPI_FRF (bits 22-23),
    /// SPI_HYPERBUS_EN (bit 24).
    /// See `nrf_sp_qspi.h`: `QSPI_CORE_CORE_CTRLR0_*` defines.
    pub const fn ctrlr0(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x00) as _) }
    }

    /// `CTRLR1` - Control Register 1 (offset 0x04).
    ///
    /// Contains: NDF (bits 0-15) - Number of Data Frames.
    /// See `nrf_sp_qspi.h`: `QSPI_CORE_CORE_CTRLR1_NDF_*` defines.
    pub const fn ctrlr1(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x04) as _) }
    }

    /// `SQSPIENR` - QSPI Enable Register (offset 0x08).
    ///
    /// Bit 0: SQSPIENR - enable/disable the SPI controller.
    /// See `nrf_sp_qspi.h`: `QSPI_CORE_CORE_SQSPIENR_*` defines.
    pub const fn sqspienr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x08) as _) }
    }

    /// `MWCR` - Microwire Control Register (offset 0x0C).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.MWCR`.
    pub const fn mwcr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x0C) as _) }
    }

    /// `SER` - Slave Enable Register (offset 0x10).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.SER`.
    pub const fn ser(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x10) as _) }
    }

    /// `BAUDR` - Baud Rate Select (offset 0x14).
    ///
    /// Bits [0:15] = SCKDV. The clock divider: `SCK = FLPR_CLK / SCKDV`.
    /// See `nrf_sp_qspi.h`: `QSPI_CORE_CORE_BAUDR_SCKDV_*` defines.
    pub const fn baudr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x14) as _) }
    }

    /// `TXFTLR` - Transmit FIFO Threshold Level (offset 0x18).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.TXFTLR`.
    pub const fn txftlr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x18) as _) }
    }

    /// `RXFTLR` - Receive FIFO Threshold Level (offset 0x1C).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.RXFTLR`.
    pub const fn rxftlr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x1C) as _) }
    }

    /// `TXFLR` - Transmit FIFO Level Register (offset 0x20).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.TXFLR`.
    pub const fn txflr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x20) as _) }
    }

    /// `RXFLR` - Receive FIFO Level Register (offset 0x24).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.RXFLR`.
    pub const fn rxflr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x24) as _) }
    }

    /// `SR` - Status Register (offset 0x28).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.SR`.
    pub const fn sr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x28) as _) }
    }

    /// `IMR` - Interrupt Mask Register (offset 0x2C).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.IMR`.
    pub const fn imr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x2C) as _) }
    }

    /// `ISR` - Interrupt Status Register (offset 0x30).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.ISR`.
    pub const fn isr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x30) as _) }
    }

    /// `RISR` - Raw Interrupt Status Register (offset 0x34).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.RISR`.
    pub const fn risr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x34) as _) }
    }

    /// `TXEICR` - TX Error Interrupt Clear Register (offset 0x38).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.TXEICR`.
    pub const fn txeicr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x38) as _) }
    }

    /// `RXOICR` - RX Overflow Interrupt Clear Register (offset 0x3C).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.RXOICR`.
    pub const fn rxoicr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x3C) as _) }
    }

    /// `RXUICR` - RX Underflow Interrupt Clear Register (offset 0x40).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.RXUICR`.
    pub const fn rxuicr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x40) as _) }
    }

    /// `MSTICR` - Multi-Controller Interrupt Clear Register (offset 0x44).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.MSTICR`.
    pub const fn msticr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x44) as _) }
    }

    /// `ICR` - Interrupt Clear Register (offset 0x48).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.ICR`.
    pub const fn icr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x48) as _) }
    }

    /// `DMACR` - DMA Control Register (offset 0x4C).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.DMACR`.
    pub const fn dmacr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x4C) as _) }
    }

    /// `DMATDLR` - DMA Transmit Data Level (offset 0x50).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.DMATDLR`.
    pub const fn dmatdlr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x50) as _) }
    }

    /// `DMARDLR` - DMA Receive Data Level (offset 0x54).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.DMARDLR`.
    pub const fn dmardlr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x54) as _) }
    }

    /// `IDR` - Identification Register (offset 0x58).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.IDR`.
    pub const fn idr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x58) as _) }
    }

    /// `SQSPICVERSIONID` - Version ID Register (offset 0x5C).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.SQSPICVERSIONID`.
    pub const fn sqspicversionid(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x5C) as _) }
    }

    /// `DR[n]` - Data Register array (36 entries at offset 0x60, used for
    /// command/address/data passing).
    ///
    /// In the transfer protocol, specific DR indices carry specific data:
    /// - DR\[0\]: command opcode
    /// - DR\[1..2\]: address (split across two registers for 32-bit addresses)
    /// - DR\[3\]: data pointer
    /// - DR\[4\]: data length
    /// - DR\[22\], DR\[23\]: debug registers
    ///
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.DR[36]`.
    pub const fn dr(self, index: usize) -> Reg<u32, RW> {
        core::assert!(index < 36);
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0x60 + index * 4) as _) }
    }

    /// `RXSAMPLEDELAY` - RX Sample Delay Register (offset 0xF0).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.RXSAMPLEDELAY`.
    pub const fn rxsampledelay(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0xF0) as _) }
    }

    /// `SPICTRLR0` - SPI Control Register 0 (offset 0xF4).
    ///
    /// Contains: TRANSTYPE (bits 0-1), ADDRL (bits 2-5), INSTL (bits 8-9),
    /// WAITCYCLES (bits 11-15), SPIDDRENB (bit 16), INSTDDRENB (bit 17),
    /// SPIFRF (bits 22-23), etc.
    /// See `nrf_sp_qspi.h`: `QSPI_CORE_CORE_SPICTRLR0_*` defines.
    pub const fn spictrlr0(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0xF4) as _) }
    }

    /// `SPICTRLR1` - SPI Control Register 1 (offset 0xF8).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.SPICTRLR1`.
    pub const fn spictrlr1(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0xF8) as _) }
    }

    /// `SPITECR` - SPI Transmit Error Interrupt Clear Register (offset 0xFC).
    /// See `nrf_sp_qspi.h`: `NRF_QSPI_CORE_CORE_Type.SPITECR`.
    pub const fn spitecr(self) -> Reg<u32, RW> {
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(0xFC) as _) }
    }
}

// ============================================================================
// SPSYNC (offset 0x1A8, size 0x10 = 16 bytes)
// ============================================================================

/// Synchronization barrier handshake registers.
///
/// Mirrors `NRF_QSPI_SPSYNC_Type` from `nrf_sp_qspi.h` (lines 2461-2464).
///
/// These auxiliary registers are used by the XSB (Extended Synchronization
/// Barrier) protocol defined in `softperipheral_regif.h`. The host writes a
/// task counter to `AUX[0]`, triggers the corresponding VPR task, then polls
/// until `AUX[0] == AUX[1]` (indicating the FLPR firmware has acknowledged).
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Spsync {
    ptr: *mut u8,
}

unsafe impl Send for Spsync {}
unsafe impl Sync for Spsync {}

impl Spsync {
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }

    /// `AUX[n]` - Auxiliary handshake registers (4 entries at offset 0x00).
    ///
    /// Used by the `__CSB`, `__ASB`, `__SSB` synchronization barrier macros in
    /// `softperipheral_regif.h`:
    /// - `AUX[0]`: host writes task counter here via `sp_handshake_set()`
    /// - `AUX[1]`: FLPR firmware writes acknowledgement here
    /// - Host polls until `AUX[0] == AUX[1]` to confirm barrier completion
    pub const fn aux(self, index: usize) -> Reg<u32, RW> {
        core::assert!(index < 4);
        unsafe { Reg::from_ptr(self.ptr.wrapping_add(index * 4) as _) }
    }
}

// ============================================================================
// VPR task/event index constants for the soft peripheral register interface.
//
// These are platform-specific indices into the VPR TASKS_TRIGGER[] and
// EVENTS_TRIGGERED[] arrays. Values below are for the nRF54L series (VPR00).
//
// Source: Nordic SDK `softperipheral_regif.h`, lines 14-21.
// ============================================================================

/// FLPR base clock frequency (128 MHz for nRF54L series).
///
/// See `softperipheral_regif.h`: `SP_VPR_BASE_FREQ_HZ`.
pub const SP_VPR_BASE_FREQ_HZ: u32 = 128_000_000;

/// VPR event index used by the soft peripheral to signal completion.
/// Maps to `NRF_VPR00->EVENTS_TRIGGERED[20]`.
///
/// See `softperipheral_regif.h`: `SP_VPR_EVENT_IDX`.
pub const SP_VPR_EVENT_IDX: usize = 20;

/// VPR task index to trigger a DPPI-connected transfer start.
/// Maps to `NRF_VPR00->TASKS_TRIGGER[16]`.
///
/// See `softperipheral_regif.h`: `SP_VPR_TASK_DPPI_0_IDX`.
pub const SP_VPR_TASK_DPPI_0_IDX: usize = 16;

/// VPR task index for Config Synchronization Barrier (`__CSB`).
/// Maps to `NRF_VPR00->TASKS_TRIGGER[17]`.
///
/// See `softperipheral_regif.h`: `SP_VPR_TASK_CONFIG_IDX`.
pub const SP_VPR_TASK_CONFIG_IDX: usize = 17;

/// VPR task index for Action Synchronization Barrier (`__ASB`).
/// Maps to `NRF_VPR00->TASKS_TRIGGER[18]`.
///
/// See `softperipheral_regif.h`: `SP_VPR_TASK_ACTION_IDX`.
pub const SP_VPR_TASK_ACTION_IDX: usize = 18;

/// VPR task index for Stop Synchronization Barrier (`__SSB`).
/// Maps to `NRF_VPR00->TASKS_TRIGGER[19]`.
///
/// See `softperipheral_regif.h`: `SP_VPR_TASK_STOP_IDX`.
pub const SP_VPR_TASK_STOP_IDX: usize = 19;
