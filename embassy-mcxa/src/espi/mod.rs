//! eSPI target (device) driver for the MCXA5xx.
//!
//! The MCXA577 integrates an eSPI target controller based on the Intel eSPI
//! v1.5 specification with an NXP-specific register interface. The controller
//! exposes five hardware "ports" that can be configured as ACPI-style
//! endpoints, index/data endpoints, mailboxes (including the OOB channel
//! mailbox), or bus-master/flash (SAF/MAF) ports, plus Port 80 POST-code
//! capture, virtual wires, and host IRQ injection.
//!
//! This driver is a port of the MCUX SDK `fsl_espi` driver to the embassy
//! model: the SDK's IRQ-callback architecture is replaced by an async
//! [`Espi::wait_event`] loop. All host-initiated activity (bus reset, virtual
//! wire changes, Port 80 writes, per-port reads/writes, SAF flash requests)
//! surfaces as [`Event`]s; responses are issued through the synchronous
//! methods on [`Espi`].
//!
//! # Clocking and pins
//! The peripheral clock is routed through the standard clock helpers
//! ([`crate::clocks::periph_helpers::EspiConfig`]); the FRDM-MCXA577 test
//! bench uses `FRO_HF` undivided. The eSPI signals (CLK, CSn, DATA0-3, RST,
//! NOTIFY) are claimed as typed pins; on the FRDM-MCXA577 they live on
//! `P4_6..P4_13` (ALT11).
//!
//! # Mailbox RAM
//! The controller DMAs mailbox/flash payloads to and from a 4 KiB-aligned
//! window in system RAM ([`EspiRam`]), provided by the application. Port
//! windows are carved out of it via [`PortConfig::ram_offset`].

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Poll;

use cortex_m::asm;
use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;

use crate::clocks::periph_helpers::EspiConfig;
use crate::clocks::{self, ClockError, Gate, WakeGuard};
use crate::gpio::{DriveStrength, Pull, SealedPin, SlewRate};
use crate::interrupt;
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::pac::espi as pac_espi;
use crate::pac::espi::STAT as PortStat;
use crate::pac::port::Mux;

#[inline]
fn regs() -> pac_espi::Espi {
    crate::pac::ESPI0
}

// ---- DMA coherence hooks ----
//
// The eSPI controller reads and writes [`EspiRam`] as an AHB master. The
// current MCX-A target has no data cache and its SRAM is non-cacheable, so the
// buffer stays coherent using only `dsb()` barriers. These two functions are
// the single porting hook: a part with a data cache must grow real
// clean/invalidate operations here. `Espi::new` debug-asserts that the data
// cache is disabled so the assumption cannot be silently violated.

/// Barrier after the CPU writes [`EspiRam`], before a doorbell register write
/// makes the data visible to the controller.
#[inline]
fn dma_clean() {
    asm::dsb();
}

/// Barrier before the CPU reads controller-written [`EspiRam`] contents.
#[inline]
fn dma_invalidate() {
    asm::dsb();
}

/// Architectural address of the SCB Configuration and Control Register, and
/// its data-cache-enable bit (DC).
const SCB_CCR_ADDR: *const u32 = 0xE000_ED14 as *const u32;
const SCB_CCR_DC: u32 = 1 << 16;

/// Debug-assert that the data cache is disabled (see the DMA coherence note
/// above; mirrors the USB driver's guard on its DMA structures).
#[inline]
fn assert_dma_noncacheable() {
    // SAFETY: architectural read-only access to the System Control Block CCR.
    let dcache_enabled = unsafe { core::ptr::read_volatile(SCB_CCR_ADDR) } & SCB_CCR_DC != 0;
    debug_assert!(
        !dcache_enabled,
        "eSPI mailbox RAM assumes non-cacheable SRAM, but the data cache is enabled; implement \
         cache maintenance in dma_clean/dma_invalidate before enabling the D-cache"
    );
}

/// Number of hardware ports exposed by the controller.
pub const PORT_COUNT: usize = 5;

/// Size in bytes of the eSPI mailbox RAM window.
pub const RAM_SIZE: usize = 4096;

/// Sentinel meaning "no port with this role is configured".
const INVALID_PORT: u8 = 0xFF;

// =========================================================================
// Configuration types (port of `espi_config_t` / `espi_port_config_t`)
// =========================================================================

/// Port function (PnCFG.TYPE).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PortType {
    /// Unconfigured (reset state).
    Unconfigured = 0x0,
    /// ACPI-style endpoint (single data byte, host IO).
    AcpiEndpoint = 0x1,
    /// ACPI-style index/data pair.
    AcpiIndexData = 0x2,
    /// Bus-master memory, single transaction (MAF-style memory move).
    BusMasterMemSingle = 0x4,
    /// Bus-master flash, single transaction (SAF request port).
    BusMasterFlashSingle = 0x5,
    /// Shared mailbox.
    MailboxShared = 0x8,
    /// Single-direction mailbox.
    MailboxSingle = 0x9,
    /// Split mailbox.
    MailboxSplit = 0xA,
    /// Split mailbox carrying the OOB channel.
    MailboxOobSplit = 0xB,
    /// OEM mailbox.
    MailboxOem = 0xC,
}

/// Mailbox RAM window size (PnRAMUSE.LEN encoding, bytes = `4 << n`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RamSize {
    /// 4 bytes.
    Size4B = 0,
    /// 8 bytes.
    Size8B = 1,
    /// 16 bytes.
    Size16B = 2,
    /// 32 bytes.
    Size32B = 3,
    /// 64 bytes.
    Size64B = 4,
    /// 128 bytes.
    Size128B = 5,
    /// 256 bytes.
    Size256B = 6,
    /// 512 bytes.
    Size512B = 7,
}

impl RamSize {
    /// Window size in bytes.
    pub const fn bytes(self) -> usize {
        4 << (self as usize)
    }
}

/// Address decode base for a port (PnADDR.BASE_ASZ).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddrBase {
    /// Direct addressing: the port address is `PnADDR.OFF`.
    Direct = 0,
    /// `MAPBASE.BASE0 << 16 | PnADDR.OFF`.
    Base0 = 1,
    /// `MAPBASE.BASE1 << 16 | PnADDR.OFF`.
    Base1 = 2,
}

/// SPI wire modes advertised to the host (ESPICAP.SPICAP).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpiMode {
    /// Single I/O only.
    SingleOnly = 0,
    /// Single and dual I/O.
    SingleAndDual = 1,
    /// Single and quad I/O.
    SingleAndQuad = 2,
    /// Single, dual and quad I/O.
    All = 3,
}

/// Maximum SPI clock speed advertised to the host (ESPICAP.MAXSPD).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MaxSpeed {
    /// Up to 20 MHz.
    Speed20MHz = 0,
    /// Up to 25 MHz.
    Speed25MHz = 1,
    /// Up to 33 MHz.
    Speed33MHz = 2,
    /// Up to 50 MHz.
    Speed50MHz = 3,
    /// Up to 66 MHz.
    Speed66MHz = 4,
}

/// eSPI/LPC enable mode (MCTRL.ENABLE).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EnableMode {
    /// Controller disabled.
    Disabled = 0,
    /// eSPI mode.
    Espi = 1,
    /// LPC mode.
    Lpc = 2,
}

/// SAF minimum erase sector size (ESPICAP.SAFERA).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SafEraseSize {
    /// 2 KiB sectors.
    Erase2KB = 0,
    /// 4 KiB sectors.
    Erase4KB = 1,
    /// 8 KiB sectors.
    Erase8KB = 2,
    /// 16 KiB sectors.
    Erase16KB = 3,
}

/// Maximum read-request size advertised for SAF (ESPICAP.TRGT_REQ_SIZE_SUPP).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadReqSize {
    /// 64 bytes.
    Req64B = 1,
    /// 128 bytes.
    Req128B = 2,
    /// 256 bytes.
    Req256B = 3,
    /// 512 bytes.
    Req512B = 4,
    /// 1024 bytes.
    Req1024B = 5,
    /// 2048 bytes.
    Req2048B = 6,
    /// 4096 bytes.
    Req4096B = 7,
}

/// Maximum memory-channel payload (ESPICAP.MEMMX).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MemMaxPayload {
    /// 64 bytes.
    Max64B = 1,
    /// 128 bytes.
    Max128B = 2,
    /// 256 bytes.
    Max256B = 3,
}

/// Maximum flash-channel payload (ESPICAP.FLASHMX).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlashMaxPayload {
    /// 64 bytes.
    Max64B = 0,
    /// 128 bytes.
    Max128B = 1,
    /// 256 bytes.
    Max256B = 2,
    /// 512 bytes.
    Max512B = 3,
}

/// Maximum OOB-channel payload (ESPICAP.OOBMX).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OobMaxPayload {
    /// 64 bytes.
    Max64B = 1,
    /// 128 bytes.
    Max128B = 2,
    /// 256 bytes.
    Max256B = 3,
}

/// Configuration of one hardware port (PnCFG/PnRAMUSE/PnADDR).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PortConfig {
    /// Port function.
    pub port_type: PortType,
    /// PnCFG.DIRECTION for single-direction mailboxes.
    pub direction: u8,
    /// Byte offset of this port's window inside [`EspiRam`].
    pub ram_offset: u16,
    /// Size of this port's mailbox window.
    pub ram_size: RamSize,
    /// Port address offset within the selected base (PnADDR.OFF).
    pub addr_offset: u16,
    /// Address decode base selection.
    pub addr_base: AddrBase,
    /// Index register offset for index/data ports (PnADDR.IDXOFF).
    pub idx_offset: u8,
}

impl PortConfig {
    /// Bytes of [`EspiRam`] this port occupies. Split mailboxes (including the
    /// OOB mailbox) use two adjacent windows (receive + transmit).
    fn ram_bytes(&self) -> usize {
        match self.port_type {
            PortType::MailboxSplit | PortType::MailboxOobSplit => 2 * self.ram_size.bytes(),
            PortType::AcpiEndpoint | PortType::AcpiIndexData => 0,
            _ => self.ram_size.bytes(),
        }
    }
}

/// Location of the host-visible status block (STATADDR).
///
/// The controller mirrors its channel status into host address space at
/// `base`/`offset` when enabled via [`Config::status_block`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StatusBlockConfig {
    /// Address decode base (same selection as port addressing).
    pub base: AddrBase,
    /// Byte offset within the selected base. Must be 8-byte aligned
    /// (STATADDR.OFF holds address bits 15:3).
    pub offset: u16,
}

/// eSPI controller configuration (port of `espi_config_t`).
///
/// `Default` mirrors the SDK `ESPI_GetDefaultConfig`: eSPI mode, all SPI wire
/// modes, 66 MHz, all optional channels disabled.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct Config {
    /// Peripheral clock routing.
    pub clock: EspiConfig,
    /// MAPBASE.BASE0 (upper 16 address bits for [`AddrBase::Base0`] ports).
    pub base0_addr: u16,
    /// MAPBASE.BASE1 (upper 16 address bits for [`AddrBase::Base1`] ports).
    pub base1_addr: u16,
    /// Advertise Slave-Attached-Flash support.
    pub enable_saf: bool,
    /// Advertise the OOB channel.
    pub enable_oob: bool,
    /// Enable Port 80 POST-code capture.
    pub enable_p80: bool,
    /// Use the dedicated alert pin instead of signaling on DATA1.
    pub enable_alert_pin: bool,
    /// Improve timing margin at high bus frequencies.
    pub enable_early_sample: bool,
    /// Disable the internal `espi_fast_clk` auto-divider.
    pub disable_clk_div: bool,
    /// Host-visible status block. `Some` programs STATADDR with the given
    /// location and enables the block (MCTRL.SBLKENA); `None` leaves it
    /// disabled.
    pub status_block: Option<StatusBlockConfig>,
    /// SPI wire modes advertised to the host.
    pub spi_mode: SpiMode,
    /// Maximum bus speed advertised to the host.
    pub bus_speed: MaxSpeed,
    /// eSPI/LPC mode selection.
    pub enable_mode: EnableMode,
    /// SAF minimum erase sector size.
    pub saf_erase_size: SafEraseSize,
    /// Maximum SAF read-request size.
    pub max_saf_rx_req_size: ReadReqSize,
    /// Maximum memory-channel payload.
    pub max_payload_size: MemMaxPayload,
    /// Maximum flash-channel payload.
    pub max_flash_payload_size: FlashMaxPayload,
    /// Maximum OOB-channel payload.
    pub max_oob_payload_size: OobMaxPayload,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clock: EspiConfig::default(),
            base0_addr: 0,
            base1_addr: 0,
            enable_saf: false,
            enable_oob: false,
            enable_p80: false,
            enable_alert_pin: false,
            enable_early_sample: false,
            disable_clk_div: false,
            status_block: None,
            spi_mode: SpiMode::All,
            bus_speed: MaxSpeed::Speed66MHz,
            enable_mode: EnableMode::Espi,
            saf_erase_size: SafEraseSize::Erase2KB,
            max_saf_rx_req_size: ReadReqSize::Req4096B,
            max_payload_size: MemMaxPayload::Max256B,
            max_flash_payload_size: FlashMaxPayload::Max512B,
            max_oob_payload_size: OobMaxPayload::Max256B,
        }
    }
}

/// Error returned while creating the eSPI driver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum InitError {
    /// Clock configuration failed.
    Clock(ClockError),
    /// More ports configured than the hardware provides.
    TooManyPorts,
    /// A port's RAM window exceeds the [`EspiRam`] buffer.
    RamOverflow {
        /// Offending hardware port index.
        port: u8,
    },
    /// The status-block offset is not 8-byte aligned (STATADDR.OFF holds
    /// address bits 15:3).
    StatusBlockMisaligned,
}

impl From<ClockError> for InitError {
    fn from(err: ClockError) -> Self {
        Self::Clock(err)
    }
}

// =========================================================================
// Events
// =========================================================================

/// Snapshot of Port 80 POST-code state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Port80Status {
    /// Latest POST code.
    pub current: u8,
    /// Previous POST code.
    pub previous: u8,
    /// POST-code counter (wraps at 16).
    pub counter: u8,
}

/// Host-driven virtual wire states (WIRERO snapshot).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VWireIn(pub u32);

/// Forward each wire getter through the PAC's WIRERO field accessors so the
/// bit layout has a single source of truth.
macro_rules! vwire_in_bit {
    ($(#[$doc:meta])* $name:ident, $pac_field:ident) => {
        $(#[$doc])*
        #[inline]
        pub const fn $name(self) -> bool {
            pac_espi::WIRERO(self.0).$pac_field()
        }
    };
}

impl VWireIn {
    vwire_in_bit!(
        /// Sleep S3 (active low).
        slp_s3n,
        SLP_S3N
    );
    vwire_in_bit!(
        /// Sleep S4 (active low).
        slp_s4n,
        SLP_S4N
    );
    vwire_in_bit!(
        /// Sleep S5 (active low).
        slp_s5n,
        SLP_S5N
    );
    vwire_in_bit!(
        /// Suspend status.
        sus_stat,
        SUS_STAT
    );
    vwire_in_bit!(
        /// Platform reset (active low).
        pltrstn,
        PLTRSTN
    );
    vwire_in_bit!(
        /// OOB reset warning.
        oob_rst_warn,
        OOB_RST_WARN
    );
    vwire_in_bit!(
        /// Host reset warning.
        host_rst_warn,
        HOST_RST_WARN
    );
    vwire_in_bit!(
        /// Suspend warning.
        sus_warn,
        SUS_WARN
    );
    vwire_in_bit!(
        /// Suspend power-down acknowledge (active low).
        sus_pwrdn_ackn,
        SUS_PWRDN_ACKN
    );
    vwire_in_bit!(
        /// Sleep A (active low).
        slp_an,
        SLP_AN
    );
    vwire_in_bit!(
        /// Wired LAN sleep.
        slp_lan,
        SLP_LAN
    );
    vwire_in_bit!(
        /// Wireless LAN sleep.
        slp_wlan,
        SLP_WLAN
    );
    vwire_in_bit!(
        /// Host entering C10.
        host_c10n,
        HOST_C10N
    );

    /// PCIe-to-EC wire group.
    #[inline]
    pub const fn p2e(self) -> u8 {
        pac_espi::WIRERO(self.0).P2E()
    }
}

/// MCU-driven virtual wires (WIREWO write flags).
///
/// `E2p` is an 8-bit group; all other wires are single-bit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum VWireOut {
    /// OOB reset acknowledge.
    OobRstAck,
    /// Wake / sensor input.
    WakenScin,
    /// Power management event.
    Pmen,
    /// SCI interrupt wire.
    Scin,
    /// SMI interrupt wire.
    Smin,
    /// RCIN reset request wire.
    Rcinn,
    /// Host reset acknowledge.
    HostRstAck,
    /// Suspend acknowledge.
    SusAckN,
    /// EC-to-PCIe wire group (8 bits).
    E2p,
    /// Boot done.
    BootDone,
    /// Boot error (active low).
    BootErrn,
    /// Deep-sleep-well power OK / reset.
    DswPwrokRst,
}

/// GPIO virtual-wire input snapshot (WIREIN_GPIO).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GpioWire {
    /// Wire group index.
    pub index: u8,
    /// Valid mask for the four levels.
    pub valid: u8,
    /// Wire levels.
    pub level: u8,
}

/// Boot outcome reported to the host via the BOOT_ERR# virtual wire
/// (see [`Espi::boot_status`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BootStatus {
    /// Boot completed successfully.
    Success,
    /// Boot failed.
    Failure,
}

/// Port error condition, decoded from PnSTAT.ERR0-3 based on the port type
/// (port of `espi_port_error_t`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PortError {
    /// Host wrote an endpoint while write-ready was still set.
    EndpointWriteOverrun,
    /// Host read an endpoint with no read data pending.
    EndpointReadEmpty,
    /// Endpoint transfer larger than one byte.
    EndpointInvalidSize,
    /// Invalid host access to a mailbox.
    MailboxInvalidAccess,
    /// Mailbox write overrun or read underrun.
    MailboxOverrunUnderrun,
    /// Host request exceeds the mailbox window.
    MailboxSizeOverflow,
    /// AHB/RAM access error during a mailbox transfer.
    MailboxRamBusError,
    /// From-host bus-master transfer failed.
    MasterFromHostFailed,
    /// Bus-master transfer overrun/underrun.
    MasterOverrunUnderrun,
    /// Flash erase failed.
    MasterEraseFailed,
    /// Bus-master AHB access error.
    MasterBusError,
}

/// Per-port event flags, as latched from PnSTAT by the interrupt handler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PortEvent {
    /// Hardware port index.
    pub port: u8,
    /// Host read event (INTRD).
    pub read: bool,
    /// Host write event (INTWR).
    pub write: bool,
    /// Special event 0 (INTSPC0; e.g. index-data write, SAF completion pull).
    pub spec0: bool,
    /// Special event 1 (INTSPC1; e.g. mailbox read started).
    pub spec1: bool,
    /// Special event 2 (INTSPC2).
    pub spec2: bool,
    /// Special event 3 (INTSPC3; e.g. mailbox read done).
    pub spec3: bool,
    /// Decoded error condition, if any error bit was set.
    pub error: Option<PortError>,
    /// WRSTAT field at interrupt time (flash request type for SAF ports).
    pub wrstat: u8,
}

/// Bus- and port-level events delivered by [`Espi::wait_event`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    /// eSPI bus reset (also delivered on the host re-configuring the link).
    BusReset,
    /// CRC error on the bus.
    CrcError,
    /// Host stall detected.
    HostStall,
    /// Host-to-MCU virtual wires changed; carries the new WIRERO snapshot.
    WireChange(VWireIn),
    /// GPIO virtual-wire activity; carries the WIREIN_GPIO snapshot.
    GpioWire(GpioWire),
    /// A previously pushed IRQ (see [`Espi::push_irq`]) completed.
    IrqPushDone,
    /// Port 80 POST code received.
    Port80(Port80Status),
    /// Activity on a hardware port.
    Port(PortEvent),
}

/// SAF (slave-attached flash) request kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlashRequestKind {
    /// Host flash read.
    Read,
    /// Host flash write.
    Write,
    /// Host flash erase.
    Erase,
}

/// Decoded SAF flash request (port of `espi_flash_request_t`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FlashRequest {
    /// Request kind.
    pub kind: FlashRequestKind,
    /// Flash address, relative to the port's configured address offset.
    pub addr: u32,
    /// Request length in bytes.
    pub length: u32,
    /// Transaction tag to echo in completions.
    pub tag: u8,
    /// `true` for the initial read request; `false` for a continuation pull
    /// (the host fetching the next split completion).
    pub read_start: bool,
}

/// SAF read-completion split position (port of `espi_saf_rx_completion_type_t`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SafCompletionType {
    /// Middle completion of a split sequence.
    Middle = 0,
    /// First completion of a split sequence.
    First = 1,
    /// Last completion of a split sequence.
    Last = 2,
    /// Only completion of a single-shot transaction.
    Only = 3,
}

/// PnIRULESTAT.SSTCL status controls (port of `espi_sstcl_t`, mailbox subset).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MailboxStatus {
    /// MCU started writing data (host-read direction).
    RdStarted = 0x1,
    /// MCU completed writing data (host-read direction).
    RdCompleted = 0x2,
    /// Host-read direction empty.
    RdEmpty = 0x3,
    /// MCU started reading (host-write direction).
    WrStarted = 0x4,
    /// Host-write direction empty (message consumed).
    WrEmpty = 0xC,
    /// Both directions empty.
    BothEmpty = 0xF,
}

// =========================================================================
// Interrupt handler and shared state
// =========================================================================

/// Bus-level pending event bits, derived from the PAC's MSTAT field setters
/// so the register layout has a single source of truth.
const PENDING_P80: u32 = {
    let mut m = pac_espi::MSTAT(0);
    m.set_P80INT(true);
    m.0
};
const PENDING_BUSRST: u32 = {
    let mut m = pac_espi::MSTAT(0);
    m.set_BUSRST(pac_espi::MSTAT_BUSRST::from_bits(1));
    m.0
};
const PENDING_IRQUPD: u32 = {
    let mut m = pac_espi::MSTAT(0);
    m.set_IRQUPD(pac_espi::MSTAT_IRQUPD::from_bits(1));
    m.0
};
const PENDING_WIRECHG: u32 = {
    let mut m = pac_espi::MSTAT(0);
    m.set_WIRECHG(pac_espi::MSTAT_WIRECHG::from_bits(1));
    m.0
};
const PENDING_HSTALL: u32 = {
    let mut m = pac_espi::MSTAT(0);
    m.set_HSTALL(pac_espi::MSTAT_HSTALL::from_bits(1));
    m.0
};
const PENDING_CRCERR: u32 = {
    let mut m = pac_espi::MSTAT(0);
    m.set_CRCERR(pac_espi::MSTAT_CRCERR::from_bits(1));
    m.0
};
const PENDING_GPIO: u32 = {
    let mut m = pac_espi::MSTAT(0);
    m.set_GPIO(pac_espi::MSTAT_GPIO::from_bits(1));
    m.0
};

/// PnSTAT bits that are independent single-bit event flags and therefore safe
/// to OR-accumulate across coalesced interrupts: the INT* flags and ERR0-ERR3,
/// derived from the PAC's STAT field setters. The RDSTAT/WRSTAT fields are
/// multi-bit state encodings that must never be ORed across snapshots; the
/// latest snapshot is kept separately in [`PORT_LAST_STAT`].
const PORT_EVENT_MASK: u32 = {
    let mut s = PortStat(0);
    s.set_INTERR(true);
    s.set_INTRD(true);
    s.set_INTWR(true);
    s.set_INTSPC0(true);
    s.set_INTSPC1(true);
    s.set_INTSPC2(true);
    s.set_INSTSPC3(true);
    s.set_ERR0(true);
    s.set_ERR1(true);
    s.set_ERR2(true);
    s.set_ERR3(true);
    s.0
};

static BUS_PENDING: AtomicU32 = AtomicU32::new(0);
static PORT_PENDING: [AtomicU32; PORT_COUNT] = [const { AtomicU32::new(0) }; PORT_COUNT];
/// Latest raw PnSTAT snapshot per port (overwritten, never ORed).
static PORT_LAST_STAT: [AtomicU32; PORT_COUNT] = [const { AtomicU32::new(0) }; PORT_COUNT];
static WAKER: AtomicWaker = AtomicWaker::new();

fn reset_static_state() {
    BUS_PENDING.store(0, Ordering::Relaxed);
    for pending in &PORT_PENDING {
        pending.store(0, Ordering::Relaxed);
    }
    for stat in &PORT_LAST_STAT {
        stat.store(0, Ordering::Relaxed);
    }
    WAKER.wake();
}

type EspiInterrupt = crate::interrupt::typelevel::ESPI;

/// Interrupt handler for the eSPI controller.
///
/// Bind this with [`crate::bind_interrupts!`] to the `ESPI` interrupt.
pub struct InterruptHandler;

impl interrupt::typelevel::Handler<EspiInterrupt> for InterruptHandler {
    unsafe fn on_interrupt() {
        let r = regs();
        let status = r.MSTAT().read();
        if status.0 == 0 {
            return;
        }
        // Acknowledge everything we observed (write-1-to-clear; read-only
        // state bits ignore the write).
        r.MSTAT().write_value(status);

        // Latch per-port status. Only independent single-bit flags (INT*,
        // ERR*) are OR-accumulated; the RDSTAT/WRSTAT state fields are
        // multi-bit encodings, so ORing two snapshots would fabricate values
        // (e.g. Read|Write reading as Erase). The latest snapshot is stored
        // by overwrite for the state fields instead.
        let portint = status.PORTINT();
        for (port, pending) in PORT_PENDING.iter().enumerate() {
            if portint & (1 << port) != 0 {
                let pstat = r.PORT(port).STAT().read();
                let mut clear = PortStat(0);
                clear.set_INTERR(pstat.INTERR());
                clear.set_INTRD(pstat.INTRD());
                clear.set_INTWR(pstat.INTWR());
                clear.set_INTSPC0(pstat.INTSPC0());
                clear.set_INTSPC1(pstat.INTSPC1());
                clear.set_INTSPC2(pstat.INTSPC2());
                clear.set_INSTSPC3(pstat.INSTSPC3());
                r.PORT(port).STAT().write_value(clear);
                PORT_LAST_STAT[port].store(pstat.0, Ordering::Relaxed);
                pending.fetch_or(pstat.0 & PORT_EVENT_MASK, Ordering::Relaxed);
            }
        }

        let bus_bits = status.0
            & (PENDING_P80
                | PENDING_BUSRST
                | PENDING_IRQUPD
                | PENDING_WIRECHG
                | PENDING_HSTALL
                | PENDING_CRCERR
                | PENDING_GPIO);
        if bus_bits != 0 {
            BUS_PENDING.fetch_or(bus_bits, Ordering::Relaxed);
        }

        WAKER.wake();
    }
}

// =========================================================================
// Clock gate
// =========================================================================

impl Gate for crate::peripherals::ESPI0 {
    type MrccPeriphConfig = EspiConfig;

    #[inline]
    unsafe fn enable_clock() {
        crate::pac::MRCC0.mrcc_glb_cc2().modify(|w| w.set_espi0(true));
    }

    #[inline]
    unsafe fn disable_clock() {
        crate::pac::MRCC0.mrcc_glb_cc2().modify(|w| w.set_espi0(false));
    }

    #[inline]
    unsafe fn assert_reset() {
        crate::pac::MRCC0.mrcc_glb_rst2().modify(|w| w.set_espi0(false));
        while Self::is_reset_released() {}
    }

    #[inline]
    unsafe fn release_reset() {
        crate::pac::MRCC0.mrcc_glb_rst2().modify(|w| w.set_espi0(true));
        while !Self::is_reset_released() {}
    }

    #[inline]
    fn is_clock_enabled() -> bool {
        crate::pac::MRCC0.mrcc_glb_cc2().read().espi0()
    }

    #[inline]
    fn is_reset_released() -> bool {
        crate::pac::MRCC0.mrcc_glb_rst2().read().espi0()
    }
}

// =========================================================================
// Pins
// =========================================================================

mod sealed {
    pub trait Sealed {}
}

impl<T: SealedPin> sealed::Sealed for T {}

macro_rules! espi_pin_trait {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        pub trait $name: sealed::Sealed + PeripheralType + 'static {
            /// Apply the eSPI pin profile (mux, no pulls, fast slew, normal
            /// drive, push-pull, digital input enabled) to this pin.
            fn configure(&self);
        }
    };
}

use embassy_hal_internal::PeripheralType;

espi_pin_trait!(
    /// eSPI clock input pin.
    ClkPin
);
espi_pin_trait!(
    /// eSPI chip-select input pin.
    CsPin
);
espi_pin_trait!(
    /// eSPI DATA0 pin.
    Data0Pin
);
espi_pin_trait!(
    /// eSPI DATA1 pin.
    Data1Pin
);
espi_pin_trait!(
    /// eSPI DATA2 pin.
    Data2Pin
);
espi_pin_trait!(
    /// eSPI DATA3 pin.
    Data3Pin
);
espi_pin_trait!(
    /// eSPI platform-reset input pin.
    RstPin
);
espi_pin_trait!(
    /// eSPI notify/alert output pin.
    NotifyPin
);

macro_rules! impl_espi_pin {
    ($pin:ident, $mux:ident, $trait:ident) => {
        impl $trait for crate::peripherals::$pin {
            fn configure(&self) {
                // Matches the SDK test-bench pin profile.
                self.set_pull(Pull::Disabled);
                self.set_slew_rate(SlewRate::Fast.into());
                self.set_drive_strength(DriveStrength::Normal.into());
                self.set_function(Mux::$mux);
                self.set_enable_input_buffer(true);
            }
        }
    };
}

// FRDM-MCXA577 primary pin set (PORT4, ALT11).
impl_espi_pin!(P4_10, Mux11, ClkPin);
impl_espi_pin!(P4_11, Mux11, CsPin);
impl_espi_pin!(P4_9, Mux11, Data0Pin);
impl_espi_pin!(P4_8, Mux11, Data1Pin);
impl_espi_pin!(P4_13, Mux11, Data2Pin);
impl_espi_pin!(P4_12, Mux11, Data3Pin);
impl_espi_pin!(P4_6, Mux11, RstPin);
impl_espi_pin!(P4_7, Mux11, NotifyPin);

// Alternate pin set (PORT3, ALT5).
impl_espi_pin!(P3_16, Mux5, ClkPin);
impl_espi_pin!(P3_17, Mux5, CsPin);
impl_espi_pin!(P3_15, Mux5, Data0Pin);
impl_espi_pin!(P3_14, Mux5, Data1Pin);
impl_espi_pin!(P3_13, Mux5, Data2Pin);
impl_espi_pin!(P3_12, Mux5, Data3Pin);
impl_espi_pin!(P3_21, Mux5, RstPin);
impl_espi_pin!(P3_20, Mux5, NotifyPin);

// =========================================================================
// Mailbox RAM
// =========================================================================

/// Mailbox RAM window shared between the CPU and the eSPI controller.
///
/// The controller addresses this buffer as an AHB master; RAMBASE requires
/// 4 KiB alignment and PnRAMUSE offsets address up to 4 KiB, so the window is
/// a full page. Allocate it statically (e.g. via `static_cell::StaticCell`)
/// and hand it to [`Espi::new`].
#[repr(C, align(4096))]
pub struct EspiRam(pub [u8; RAM_SIZE]);

impl EspiRam {
    /// Create a zeroed RAM window.
    pub const fn new() -> Self {
        Self([0; RAM_SIZE])
    }
}

impl Default for EspiRam {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// Driver
// =========================================================================

/// eSPI target driver.
pub struct Espi<'d> {
    _phantom: PhantomData<&'d mut crate::peripherals::ESPI0>,
    _wake_guard: Option<WakeGuard>,
    ram: *mut u8,
    /// Cached PnADDR.OFF per port (mirrors the SDK handle's `addrOffset`).
    addr_offset: [u16; PORT_COUNT],
    /// Cached PnRAMUSE per port.
    ram_offset: [u16; PORT_COUNT],
    ram_window: [usize; PORT_COUNT],
    port_type: [Option<PortType>; PORT_COUNT],
    oob_port: u8,
    saf_port: u8,
}

impl<'d> Espi<'d> {
    /// Create and start the eSPI target controller.
    ///
    /// Enables the peripheral clock through the standard clock helpers,
    /// configures the pins, programs capabilities and per-port windows, and
    /// enables the controller in the mode selected by
    /// [`Config::enable_mode`]. Host interaction is reported via
    /// [`Self::wait_event`] once the controller is enabled.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        _peri: Peri<'d, crate::peripherals::ESPI0>,
        _irq: impl Binding<EspiInterrupt, InterruptHandler>,
        clk: Peri<'d, impl ClkPin>,
        cs: Peri<'d, impl CsPin>,
        io0: Peri<'d, impl Data0Pin>,
        io1: Peri<'d, impl Data1Pin>,
        io2: Peri<'d, impl Data2Pin>,
        io3: Peri<'d, impl Data3Pin>,
        rst: Peri<'d, impl RstPin>,
        notify: Peri<'d, impl NotifyPin>,
        ram: &'d mut EspiRam,
        ports: &[PortConfig],
        config: Config,
    ) -> Result<Self, InitError> {
        if ports.len() > PORT_COUNT {
            return Err(InitError::TooManyPorts);
        }
        for (i, port) in ports.iter().enumerate() {
            let end = port.ram_offset as usize + port.ram_bytes();
            if end > RAM_SIZE {
                return Err(InitError::RamOverflow { port: i as u8 });
            }
        }
        if config.status_block.is_some_and(|sb| sb.offset & 0x7 != 0) {
            return Err(InitError::StatusBlockMisaligned);
        }

        // A previous driver instance may have left latched events behind;
        // exclusivity itself is guaranteed by ownership of the `ESPI0` Peri.
        assert_dma_noncacheable();
        reset_static_state();

        // SAFETY: we own the eSPI peripheral and bring up its clock once.
        let parts = unsafe { clocks::enable_and_reset::<crate::peripherals::ESPI0>(&config.clock)? };

        clk.configure();
        cs.configure();
        io0.configure();
        io1.configure();
        io2.configure();
        io3.configure();
        rst.configure();
        notify.configure();

        let r = regs();
        let ram_ptr = ram.0.as_mut_ptr();

        // RAM base and mapped bases.
        r.RAMBASE().write(|w| w.set_RAM(ram_ptr as u32 >> 12));
        r.MAPBASE().write(|w| {
            w.set_BASE0(config.base0_addr);
            w.set_BASE1(config.base1_addr);
        });

        // Host-visible status block location (enabled via MCTRL.SBLKENA
        // below; alignment validated up front). STATADDR.OFF holds address
        // bits 15:3.
        if let Some(sb) = config.status_block {
            r.STATADDR().write(|w| {
                w.set_OFF(sb.offset >> 3);
                w.set_BASE(pac_espi::BASE::from_bits(sb.base as u8));
            });
        }

        // Advertised capabilities.
        r.ESPICAP().write(|w| {
            w.set_SPICAP(pac_espi::SPICAP::from_bits(config.spi_mode as u8));
            w.set_MAXSPD(pac_espi::MAXSPD::from_bits(config.bus_speed as u8));
            w.set_TRGT_REQ_SIZE_SUPP(pac_espi::TRGT_REQ_SIZE_SUPP::from_bits(
                config.max_saf_rx_req_size as u8,
            ));
            w.set_MEMMX(pac_espi::MEMMX::from_bits(config.max_payload_size as u8));
            w.set_ALPIN(config.enable_alert_pin);
            if config.enable_saf {
                w.set_SAF(pac_espi::ESPICAP_SAF::from_bits(1));
                w.set_SAFERA(pac_espi::SAFERA::from_bits(config.saf_erase_size as u8));
                w.set_FLASHMX(pac_espi::FLASHMX::from_bits(config.max_flash_payload_size as u8));
            }
            if config.enable_oob {
                w.set_OOBOK(pac_espi::OOBOK::from_bits(1));
                w.set_OOBMX(pac_espi::OOBMX::from_bits(config.max_oob_payload_size as u8));
            }
        });

        // Per-port configuration.
        let mut state = Self {
            _phantom: PhantomData,
            _wake_guard: parts.wake_guard,
            ram: ram_ptr,
            addr_offset: [0; PORT_COUNT],
            ram_offset: [0; PORT_COUNT],
            ram_window: [0; PORT_COUNT],
            port_type: [None; PORT_COUNT],
            oob_port: INVALID_PORT,
            saf_port: INVALID_PORT,
        };

        let mut pena = 0u8;
        for (i, port) in ports.iter().enumerate() {
            let p = r.PORT(i);
            p.CFG().write(|w| {
                w.set_TYPE(pac_espi::CFG_TYPE::from_bits(port.port_type as u8));
                w.set_DIRECTION(pac_espi::CFG_DIRECTION::from_bits(port.direction));
            });
            p.RAMUSE().write(|w| {
                w.set_OFF(port.ram_offset);
                w.set_LEN(port.ram_size as u8);
            });
            p.ADDR().write(|w| {
                w.set_OFF(port.addr_offset);
                w.set_BASE_ASZ(pac_espi::ADDR_BASE_ASZ::from_bits(port.addr_base as u8));
                w.set_IDXOFF(port.idx_offset);
            });
            pena |= 1 << i;

            state.addr_offset[i] = port.addr_offset;
            state.ram_offset[i] = port.ram_offset;
            state.ram_window[i] = port.ram_size.bytes();
            state.port_type[i] = Some(port.port_type);
            match port.port_type {
                PortType::MailboxOobSplit => state.oob_port = i as u8,
                PortType::BusMasterFlashSingle => state.saf_port = i as u8,
                _ => {}
            }
        }

        // Master control: enable ports and the selected mode.
        r.MCTRL().write(|w| {
            w.set_PENA(pena);
            w.set_ENABLE(pac_espi::ENABLE::from_bits(config.enable_mode as u8));
            w.set_P80ENA(config.enable_p80);
            w.set_CLK_DIV_DISABLE(pac_espi::CLK_DIV_DISABLE::from_bits(config.disable_clk_div as u8));
            w.set_EARLY_SAMPLE(config.enable_early_sample);
            w.set_SBLKENA(config.status_block.is_some());
        });

        // Enable event interrupts (everything the event model reports) and
        // all per-port interrupt rules.
        r.MSTAT().write_value(pac_espi::MSTAT(0xFFFF_FFFF));
        r.INTENSET().write(|w| {
            w.set_PORTINT(pac_espi::INTENSET_PORTINT::from_bits(pena));
            w.set_P80INT(pac_espi::INTENSET_P80INT::from_bits(config.enable_p80 as u8));
            w.set_BUSRST(pac_espi::INTENSET_BUSRST::from_bits(1));
            w.set_IRQUPD(pac_espi::INTENSET_IRQUPD::from_bits(1));
            w.set_WIRECHG(pac_espi::INTENSET_WIRECHG::from_bits(1));
            w.set_HSTALL(pac_espi::INTENSET_HSTALL::from_bits(1));
            w.set_CRCERR(pac_espi::INTENSET_CRCERR::from_bits(1));
            w.set_GPIO(pac_espi::INTENSET_GPIO::from_bits(1));
        });
        for i in 0..ports.len() {
            r.PORT(i).IRULESTAT().modify(|w| {
                w.set_INTERR(true);
                w.set_INTRD(true);
                w.set_INTWR(true);
                w.set_INTSPC(0xF);
            });
        }

        // SAFETY: enabling the controller interrupt.
        unsafe {
            EspiInterrupt::unpend();
            EspiInterrupt::enable();
        }

        Ok(state)
    }

    // ---------------------------------------------------------------------
    // Events
    // ---------------------------------------------------------------------

    /// Wait for the next controller event.
    ///
    /// Events latched by the interrupt handler are drained one per call, bus
    /// events first, then port events in ascending port order.
    pub async fn wait_event(&mut self) -> Event {
        poll_fn(|cx| {
            WAKER.register(cx.waker());

            if let Some(event) = self.take_bus_event() {
                return Poll::Ready(event);
            }
            for (port, pending) in PORT_PENDING.iter().enumerate() {
                let flags = PortStat(pending.swap(0, Ordering::Relaxed));
                if flags.0 != 0 {
                    let state = PortStat(PORT_LAST_STAT[port].load(Ordering::Relaxed));
                    return Poll::Ready(Event::Port(self.decode_port_event(port as u8, flags, state)));
                }
            }
            Poll::Pending
        })
        .await
    }

    fn take_bus_event(&mut self) -> Option<Event> {
        let r = regs();
        loop {
            let pending = BUS_PENDING.load(Ordering::Relaxed);
            if pending == 0 {
                return None;
            }
            // Select the highest-priority pending bus event.
            let bit = if pending & PENDING_BUSRST != 0 {
                PENDING_BUSRST
            } else if pending & PENDING_CRCERR != 0 {
                PENDING_CRCERR
            } else if pending & PENDING_HSTALL != 0 {
                PENDING_HSTALL
            } else if pending & PENDING_WIRECHG != 0 {
                PENDING_WIRECHG
            } else if pending & PENDING_GPIO != 0 {
                PENDING_GPIO
            } else if pending & PENDING_P80 != 0 {
                PENDING_P80
            } else if pending & PENDING_IRQUPD != 0 {
                PENDING_IRQUPD
            } else {
                // Unknown latched bits; discard them.
                BUS_PENDING.fetch_and(!pending, Ordering::Relaxed);
                continue;
            };

            // Consume the bit BEFORE reading the payload registers: if a new
            // event lands in between, it re-latches the bit and is delivered
            // again with a fresh snapshot, instead of being lost with a stale
            // one.
            BUS_PENDING.fetch_and(!bit, Ordering::Relaxed);

            let event = match bit {
                PENDING_BUSRST => Event::BusReset,
                PENDING_CRCERR => Event::CrcError,
                PENDING_HSTALL => Event::HostStall,
                PENDING_WIRECHG => Event::WireChange(VWireIn(r.WIRERO().read().0)),
                PENDING_GPIO => {
                    let gpio = r.WIREIN_GPIO().read();
                    Event::GpioWire(GpioWire {
                        index: gpio.INDEX(),
                        valid: gpio.VALID().to_bits(),
                        level: gpio.LEVEL().to_bits(),
                    })
                }
                PENDING_P80 => {
                    let p80 = r.P80STAT().read();
                    Event::Port80(Port80Status {
                        current: p80.CURR(),
                        previous: p80.PREV(),
                        counter: p80.CNT(),
                    })
                }
                _ => Event::IrqPushDone,
            };
            return Some(event);
        }
    }

    /// `flags` carries the OR-accumulated single-bit event flags; `state` is
    /// the latest raw PnSTAT snapshot for the multi-bit RDSTAT/WRSTAT fields.
    fn decode_port_event(&self, port: u8, flags: PortStat, state: PortStat) -> PortEvent {
        PortEvent {
            port,
            read: flags.INTRD(),
            write: flags.INTWR(),
            spec0: flags.INTSPC0(),
            spec1: flags.INTSPC1(),
            spec2: flags.INTSPC2(),
            spec3: flags.INSTSPC3(),
            error: flags.INTERR().then(|| self.decode_port_error(port, flags)).flatten(),
            wrstat: state.WRSTAT().to_bits(),
        }
    }

    /// Port of `ESPI_GetPortErrorStatus`: ERR0-3 meaning depends on port type.
    fn decode_port_error(&self, port: u8, pstat: PortStat) -> Option<PortError> {
        use PortError::*;
        let ty = self.port_type[port as usize]?;
        match ty {
            PortType::AcpiEndpoint | PortType::AcpiIndexData => {
                if pstat.ERR0() {
                    Some(EndpointWriteOverrun)
                } else if pstat.ERR1() {
                    Some(EndpointReadEmpty)
                } else if pstat.ERR2() {
                    Some(EndpointInvalidSize)
                } else {
                    None
                }
            }
            PortType::MailboxShared | PortType::MailboxSingle | PortType::MailboxSplit | PortType::MailboxOobSplit => {
                if pstat.ERR0() {
                    Some(MailboxInvalidAccess)
                } else if pstat.ERR1() {
                    Some(MailboxOverrunUnderrun)
                } else if pstat.ERR2() {
                    Some(MailboxSizeOverflow)
                } else if pstat.ERR3() {
                    Some(MailboxRamBusError)
                } else {
                    None
                }
            }
            PortType::BusMasterMemSingle | PortType::BusMasterFlashSingle => {
                if pstat.ERR0() {
                    Some(MasterFromHostFailed)
                } else if pstat.ERR1() {
                    Some(MasterOverrunUnderrun)
                } else if pstat.ERR2() {
                    Some(MasterEraseFailed)
                } else if pstat.ERR3() {
                    Some(MasterBusError)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // ---------------------------------------------------------------------
    // Status snapshots
    // ---------------------------------------------------------------------

    /// Raw MSTAT snapshot (busy/in-reset/pending state bits).
    pub fn status(&self) -> u32 {
        regs().MSTAT().read().0
    }

    /// Raw advertised-capabilities register (ESPICAP).
    pub fn capabilities(&self) -> u32 {
        regs().ESPICAP().read().0
    }

    /// Raw host-negotiated configuration register (ESPICFG).
    pub fn host_config(&self) -> u32 {
        regs().ESPICFG().read().0
    }

    /// Current host-driven virtual wire states.
    pub fn vwires(&self) -> VWireIn {
        VWireIn(regs().WIRERO().read().0)
    }

    /// Current GPIO virtual-wire input snapshot (WIREIN_GPIO).
    pub fn gpio_wires(&self) -> GpioWire {
        let gpio = regs().WIREIN_GPIO().read();
        GpioWire {
            index: gpio.INDEX(),
            valid: gpio.VALID().to_bits(),
            level: gpio.LEVEL().to_bits(),
        }
    }

    /// Raw MCU-to-host virtual wire register (WIREWO), including DONE.
    pub fn vwires_out_raw(&self) -> u32 {
        regs().WIREWO().read().0
    }

    // ---------------------------------------------------------------------
    // Virtual wires
    // ---------------------------------------------------------------------

    /// Send a virtual wire to the host (port of `ESPI_SendVWire`).
    ///
    /// For single-bit wires `value` must be 0 or 1; for [`VWireOut::E2p`] it
    /// is the full 8-bit group. Returns [`VWireBusy`] while a previous
    /// virtual wire update is still in flight (WIREWO.DONE clear), matching
    /// the SDK's busy semantics; poll and retry.
    pub fn send_vwire(&mut self, wire: VWireOut, value: u8) -> Result<(), VWireBusy> {
        let r = regs();
        if r.WIREWO().read().DONE() != pac_espi::DONE::DONE {
            return Err(VWireBusy);
        }
        // Deliberately a fresh write, not read-modify-write: `ESPI_SendVWire`
        // in the SDK builds the value from zero (`uint32_t reg = 0U`) and
        // assigns it. WIREWO is a message-push register - the write triggers
        // a virtual-wire update for the written group - not level state that
        // must be preserved across writes.
        r.WIREWO().write(|w| match wire {
            VWireOut::OobRstAck => w.set_OOB_RST_ACK(value != 0),
            VWireOut::WakenScin => w.set_WAKEN_SCIN(value != 0),
            VWireOut::Pmen => w.set_PMEN(value != 0),
            VWireOut::Scin => w.set_SCIN(value != 0),
            VWireOut::Smin => w.set_SMIN(value != 0),
            VWireOut::Rcinn => w.set_RCINN(value != 0),
            VWireOut::HostRstAck => w.set_HOST_RST_ACK(value != 0),
            VWireOut::SusAckN => w.set_SUSACKN(value != 0),
            VWireOut::E2p => w.set_E2P(value),
            VWireOut::BootDone => w.set_BOOT_DONE(value != 0),
            VWireOut::BootErrn => w.set_BOOT_ERRN(pac_espi::BOOT_ERRN::from_bits(value & 1)),
            VWireOut::DswPwrokRst => w.set_DSW_PWROK_RST(value != 0),
        });
        Ok(())
    }

    /// Send a raw WIREWO mask (test-bench `send_vw_mask` command).
    pub fn send_vwire_mask(&mut self, mask: u32) {
        regs().WIREWO().write_value(pac_espi::WIREWO(mask));
    }

    // ---------------------------------------------------------------------
    // Semantic virtual-wire helpers
    //
    // Thin wrappers over [`Self::send_vwire`] with the platform meaning and
    // active-low polarity folded in, mirroring the embassy-imxrt eSPI API.
    // All of them share `send_vwire`'s non-blocking semantics: they return
    // [`VWireBusy`] while a previous wire update is still in flight instead
    // of busy-waiting on WIREWO.DONE.
    // ---------------------------------------------------------------------

    /// Drive the WAKE# wire to wake the host from Sx (lid switch, AC
    /// insertion, or any general wake event). If the host is in S0, an SCI
    /// is generated instead.
    ///
    /// WAKE# is active low: `wake(true)` asserts the event; the wire does
    /// not auto-clear, so call `wake(false)` afterwards.
    pub fn wake(&mut self, set: bool) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::WakenScin, !set as u8)
    }

    /// Drive the PME# wire to wake the host from Sx through PCI power
    /// management.
    ///
    /// PME# is active low: `pme(true)` asserts the event; call `pme(false)`
    /// to clear it.
    pub fn pme(&mut self, set: bool) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::Pmen, !set as u8)
    }

    /// Drive the SCI# wire, causing the OS to invoke an ACPI method.
    ///
    /// SCI# is active low: `sci(true)` asserts the interrupt; call
    /// `sci(false)` once the host has handled it.
    pub fn sci(&mut self, set: bool) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::Scin, !set as u8)
    }

    /// Drive the SMI# wire, causing the BIOS to invoke its system
    /// management handler.
    ///
    /// SMI# is active low: `smi(true)` asserts the interrupt; call
    /// `smi(false)` once the host has handled it.
    pub fn smi(&mut self, set: bool) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::Smin, !set as u8)
    }

    /// Drive the RCIN# wire to request a host CPU reset.
    ///
    /// RCIN# is active low: `rcin(true)` asserts the request. Normally the
    /// platform resets in response, so clearing with `rcin(false)` is rarely
    /// needed.
    pub fn rcin(&mut self, set: bool) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::Rcinn, !set as u8)
    }

    /// Acknowledge SUS_WARN# with SUS_ACK# (active low: asserts by driving
    /// the wire low).
    pub fn suspend_ack(&mut self) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::SusAckN, 0)
    }

    /// Acknowledge HOST_RST_WARN with HOST_RST_ACK (active high).
    pub fn host_reset_ack(&mut self) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::HostRstAck, 1)
    }

    /// Acknowledge OOB_RST_WARN with OOB_RST_ACK (active high).
    pub fn oob_reset_ack(&mut self) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::OobRstAck, 1)
    }

    /// Signal SLAVE_BOOT_LOAD_DONE: the EC has finished booting and the
    /// host may continue its G3 to S0 exit (active high).
    pub fn boot_done(&mut self) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::BootDone, 1)
    }

    /// Report the boot outcome on BOOT_ERR# (active low: `Success` leaves
    /// the wire deasserted/high, `Failure` drives it low).
    pub fn boot_status(&mut self, status: BootStatus) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::BootErrn, (status == BootStatus::Success) as u8)
    }

    /// Send the EC-to-PCH (E2P) byte group.
    pub fn e2p(&mut self, data: u8) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::E2p, data)
    }

    /// Signal DSW_PWROK reset, to be sent when the host enters G3
    /// (active high).
    pub fn dsw_pwrok_reset(&mut self) -> Result<(), VWireBusy> {
        self.send_vwire(VWireOut::DswPwrokRst, 1)
    }

    // ---------------------------------------------------------------------
    // Host IRQ injection
    // ---------------------------------------------------------------------

    /// Push an IRQ number to the host (port of `ESPI_PushIrq`).
    ///
    /// Completion is reported via [`Event::IrqPushDone`].
    pub fn push_irq(&mut self, irq: u8) {
        regs().IRQPUSH().write(|w| w.set_IRQ(irq));
    }

    /// Whether the last pushed IRQ has been delivered.
    pub fn is_irq_push_done(&self) -> bool {
        regs().IRQPUSH().read().DONE()
    }

    // ---------------------------------------------------------------------
    // Port 80
    // ---------------------------------------------------------------------

    /// Current Port 80 status, if Port 80 capture is enabled.
    pub fn port80_status(&self) -> Option<Port80Status> {
        let r = regs();
        if !r.MCTRL().read().P80ENA() {
            return None;
        }
        let p80 = r.P80STAT().read();
        Some(Port80Status {
            current: p80.CURR(),
            previous: p80.PREV(),
            counter: p80.CNT(),
        })
    }

    /// Reset the Port 80 POST-code counter.
    pub fn reset_port80_counter(&mut self) {
        regs().P80STAT().modify(|w| w.set_RST(true));
    }

    // ---------------------------------------------------------------------
    // Endpoint / mailbox data
    // ---------------------------------------------------------------------

    /// Read PnDATAIN: `(index, data-or-length)` (port of `ESPI_GetEndpointData`).
    pub fn endpoint_data(&self, port: u8) -> (u16, u8) {
        let datain = regs().PORT(port as usize).DATAIN().read();
        (datain.IDX(), datain.DATA_LEN())
    }

    /// Write a response byte to PnDATAOUT (port of `ESPI_WritePortData`).
    pub fn write_endpoint_data(&mut self, port: u8, data: u8) {
        regs().PORT(port as usize).DATAOUT().write(|w| w.set_DATA(data));
    }

    /// Length in bytes of the mailbox message currently in the port window.
    pub fn mailbox_msg_len(&self, port: u8) -> usize {
        regs().PORT(port as usize).DATAIN().read().DATA_LEN() as usize + 1
    }

    /// Size in bytes of the port's mailbox window.
    pub fn mailbox_size(&self, port: u8) -> usize {
        self.ram_window[port as usize]
    }

    /// Copy the current mailbox message out of the port's RAM window.
    ///
    /// Returns the number of bytes copied (clamped to `buf` and the window).
    pub fn read_mailbox(&mut self, port: u8, buf: &mut [u8]) -> usize {
        let window = self.ram_window[port as usize];
        let len = self.mailbox_msg_len(port).min(window).min(buf.len());
        dma_invalidate();
        // SAFETY: the port window was validated against RAM_SIZE at init.
        unsafe {
            core::ptr::copy_nonoverlapping(self.port_ram_ptr(port), buf.as_mut_ptr(), len);
        }
        len
    }

    /// Update the port's SSTCL status field (mailbox handshakes).
    pub fn set_mailbox_status(&mut self, port: u8, status: MailboxStatus) {
        regs()
            .PORT(port as usize)
            .IRULESTAT()
            .modify(|w| w.set_SSTCL(status as u8));
    }

    fn port_ram_ptr(&self, port: u8) -> *mut u8 {
        // SAFETY: offset validated against RAM_SIZE at init.
        unsafe { self.ram.add(self.ram_offset[port as usize] as usize) }
    }

    // ---------------------------------------------------------------------
    // OOB channel
    // ---------------------------------------------------------------------

    /// Send an out-of-band message to the host (port of `ESPI_SendOOB`).
    ///
    /// Copies `data` into the OOB transmit window and triggers the message.
    /// With `announce`, the "started by MCU" status is set first so hosts
    /// that react to it can prepare. Completion is reported by a
    /// [`Event::Port`] read event on the OOB port.
    ///
    /// OMFLEN.LEN is a 7-bit field; like the SDK, message lengths above 128
    /// bytes are truncated by the hardware encoding even though the OOB
    /// window itself may be larger.
    pub fn send_oob(&mut self, data: &[u8], announce: bool) -> Result<(), OobError> {
        if self.oob_port == INVALID_PORT {
            return Err(OobError::NotConfigured);
        }
        if data.is_empty() {
            return Err(OobError::InvalidLength);
        }
        let port = self.oob_port;
        let mb_size = self.ram_window[port as usize];
        if data.len() > mb_size {
            return Err(OobError::InvalidLength);
        }

        let p = regs().PORT(port as usize);
        if announce {
            p.IRULESTAT().modify(|w| w.set_SSTCL(0x1));
        }

        // Transmit half of the split OOB window sits above the receive half.
        // SAFETY: the doubled window was validated against RAM_SIZE at init.
        unsafe {
            core::ptr::copy_nonoverlapping(data.as_ptr(), self.port_ram_ptr(port).add(mb_size), data.len());
        }
        dma_clean();

        // Port of `ESPI_TriggerOOBMsg`. Subtract before narrowing: OMFLEN.LEN
        // is a 7-bit field, so like the SDK (which computes `length - 1` in
        // u32 arithmetic and lets the field mask it) lengths above 128 bytes
        // are truncated by the hardware encoding.
        p.OMFLEN().modify(|w| w.set_LEN((data.len() - 1) as u8));
        p.IRULESTAT().modify(|w| w.set_SSTCL(0x2));
        Ok(())
    }

    /// Read the received out-of-band message (port of `ESPI_ReadOOB`).
    ///
    /// Returns the number of bytes copied, or [`OobError::Truncated`] with
    /// the buffer filled if the message was longer than `buf`.
    pub fn read_oob(&mut self, buf: &mut [u8]) -> Result<usize, OobError> {
        if self.oob_port == INVALID_PORT {
            return Err(OobError::NotConfigured);
        }
        let port = self.oob_port;
        let window = self.ram_window[port as usize];
        let msg_len = self.mailbox_msg_len(port).min(window);
        let len = msg_len.min(buf.len());
        dma_invalidate();
        // SAFETY: the window was validated against RAM_SIZE at init.
        unsafe {
            core::ptr::copy_nonoverlapping(self.port_ram_ptr(port), buf.as_mut_ptr(), len);
        }
        if len < msg_len {
            Err(OobError::Truncated)
        } else {
            Ok(len)
        }
    }

    /// Hardware port index carrying the OOB channel, if configured.
    pub fn oob_port(&self) -> Option<u8> {
        (self.oob_port != INVALID_PORT).then_some(self.oob_port)
    }

    // ---------------------------------------------------------------------
    // SAF (slave-attached flash)
    // ---------------------------------------------------------------------

    /// Hardware port index configured as the SAF request port, if any.
    pub fn saf_port(&self) -> Option<u8> {
        (self.saf_port != INVALID_PORT).then_some(self.saf_port)
    }

    /// Decode a SAF flash request from a port event on the SAF port
    /// (port of the request decode in `ESPI_HandleSAFIRQ`).
    ///
    /// Returns `None` if the event carries no flash request. The returned
    /// address is relative to the SAF port's configured address offset
    /// (wrapping on underflow); range-checking against the backing flash size
    /// and issuing the failure completion is the application's job, as in the
    /// SDK example.
    ///
    /// An initial request (host read/write/erase) takes priority over a
    /// completion pull (`spec0`) if both flags coalesced into one event. The
    /// eSPI flash channel serializes transactions (the host waits for each
    /// completion), so a coalesced pull can only be the tail of an already
    /// fully-served read sequence, where the SDK reference drops it too.
    /// Continuation pulls carry `length == 0`, matching the zero-initialized
    /// request struct the SDK driver hands to the flash callback.
    pub fn flash_request(&self, event: &PortEvent) -> Option<FlashRequest> {
        if self.saf_port == INVALID_PORT || event.port != self.saf_port {
            return None;
        }
        let kind = match event.wrstat {
            1 => FlashRequestKind::Read,
            2 => FlashRequestKind::Write,
            3 => FlashRequestKind::Erase,
            _ => return None,
        };

        let p = regs().PORT(self.saf_port as usize);
        let datain = p.DATAIN().read();
        let length = datain.DATA_LEN() as u32 + 1;
        let tag = datain.TAG();

        if event.read || event.write {
            // Flash address is big-endian in the first four window bytes.
            let mut addr_bytes = [0u8; 4];
            dma_invalidate();
            // SAFETY: the window was validated against RAM_SIZE at init.
            unsafe {
                core::ptr::copy_nonoverlapping(self.port_ram_ptr(self.saf_port), addr_bytes.as_mut_ptr(), 4);
            }
            let addr = u32::from_be_bytes(addr_bytes).wrapping_sub(self.addr_offset[self.saf_port as usize] as u32);
            Some(FlashRequest {
                kind,
                addr,
                length,
                tag,
                read_start: true,
            })
        } else if event.spec0 && kind == FlashRequestKind::Read {
            // Host pulled a split completion; the app should push the next
            // one. Like the SDK's zero-initialized request, no address or
            // length accompanies a pull.
            Some(FlashRequest {
                kind,
                addr: 0,
                length: 0,
                tag,
                read_start: false,
            })
        } else {
            None
        }
    }

    /// Slice of the SAF port RAM region where request/completion payloads
    /// live. For writes the payload starts at byte 4 (after the address).
    ///
    /// Unlike mailbox ports, flash ports are not bounded by PnRAMUSE.LEN: the
    /// controller stores 4 address bytes plus up to the negotiated flash
    /// payload (see [`Self::flash_max_payload`]), so the slice extends from
    /// the port's RAM offset to the end of [`EspiRam`]. Lay out the RAM so no
    /// other port window sits within `4 +` max payload above the SAF offset.
    pub fn flash_window(&mut self) -> &mut [u8] {
        assert!(self.saf_port != INVALID_PORT);
        let offset = self.ram_offset[self.saf_port as usize] as usize;
        dma_invalidate();
        // SAFETY: the offset was validated against RAM_SIZE at init, the
        // slice stays within the `EspiRam` borrowed for 'd, and the borrow is
        // tied to `&mut self`.
        unsafe { core::slice::from_raw_parts_mut(self.port_ram_ptr(self.saf_port), RAM_SIZE - offset) }
    }

    /// Program the flash completion transaction type and length
    /// (port of `ESPI_SetFlashOpLen`).
    ///
    /// OMFLEN.LEN is a 7-bit field; like the SDK, lengths above 128 bytes are
    /// truncated by the hardware encoding.
    pub fn set_flash_op_len(&mut self, trans: u8, length: u32) {
        assert!(self.saf_port != INVALID_PORT);
        // Ensure completion data written to the flash window is visible to
        // the controller before the length/completion doorbells.
        dma_clean();
        regs().PORT(self.saf_port as usize).OMFLEN().write(|w| {
            w.set_TRANS(pac_espi::OMFLEN_TRANS::from_bits(trans));
            w.set_LEN(length.saturating_sub(1) as u8);
        });
    }

    /// Update the SAF completion status fields
    /// (port of `ESPI_SetFlashCompletion`).
    pub fn set_flash_completion(&mut self, tag: u8, state: u8, completion: SafCompletionType) {
        assert!(self.saf_port != INVALID_PORT);
        // Ensure completion data written to the flash window is visible to
        // the controller before the completion is announced.
        dma_clean();
        regs().PORT(self.saf_port as usize).IRULESTAT().modify(|w| {
            w.set_SSTCL(state);
            w.set_CPU_TAG(tag);
            w.set_FLASH_COMPLETION_TYPE(pac_espi::IRULESTAT_FLASH_COMPLETION_TYPE::from_bits(completion as u8));
        });
    }

    /// Maximum host-negotiated flash payload in bytes
    /// (port of `ESPI_GetFlashMaxPayload`).
    pub fn flash_max_payload(&self) -> u32 {
        64 << regs().ESPICFG().read().FLASHSZ().to_bits()
    }
}

impl Drop for Espi<'_> {
    fn drop(&mut self) {
        let r = regs();
        // Stop the controller and mask its interrupt; the clock gate stays
        // with the peripheral lifetime (matching the USB driver).
        r.MCTRL()
            .modify(|w| w.set_ENABLE(pac_espi::ENABLE::from_bits(EnableMode::Disabled as u8)));
        r.INTENCLR().write_value(pac_espi::INTENCLR(0xFFFF_FFFF));
        EspiInterrupt::disable();
    }
}

/// A previous virtual-wire update is still in flight (WIREWO.DONE clear).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VWireBusy;

/// OOB channel errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OobError {
    /// No port is configured as [`PortType::MailboxOobSplit`].
    NotConfigured,
    /// Message empty or larger than the OOB window.
    InvalidLength,
    /// Received message was longer than the provided buffer.
    Truncated,
}

/// SSTCL value: SAF request accepted (port of `kESPI_SSTCL_SAFReqAccepted`).
pub const SSTCL_SAF_REQ_ACCEPTED: u8 = 0x2;
/// SSTCL value: send SAF completion (port of `kESPI_SSTCL_SAFCompletion`).
pub const SSTCL_SAF_COMPLETION: u8 = 0x1;
/// OMFLEN.TRANS: SAF completion carrying data.
pub const OMFLEN_SAF_COMPLETION_WITH_DATA: u8 = 0x1;
/// OMFLEN.TRANS: SAF completion without data.
pub const OMFLEN_SAF_COMPLETION_NO_DATA: u8 = 0x2;
/// OMFLEN.TRANS: SAF completion failure.
pub const OMFLEN_SAF_COMPLETION_FAIL: u8 = 0x0;
