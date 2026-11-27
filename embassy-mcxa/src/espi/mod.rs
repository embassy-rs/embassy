//! eSPI target (device) driver, adapted from embassy PR #6516 for the MCXA577.
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
//!
//! # Electrical requirements
//! RM 52.7 requires VDDIO_0 at 1.8 V for eSPI. Check the board supply and
//! routing before connecting a host; software cannot configure that rail.

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Poll;

use cortex_m::asm;
use cortex_m::peripheral::SCB;
use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;

use crate::clocks::periph_helpers::EspiConfig;
use crate::clocks::{self, ClockError, WakeGuard};
use crate::gpio::SealedPin;
use crate::interrupt;
use crate::interrupt::typelevel::{Binding, Interrupt};
use crate::pac::espi as pac_espi;
use crate::pac::espi::Pstat as PortStat;

mod protocol;

struct Info {
    regs: pac_espi::Espi,
}

static INFO: Info = Info {
    regs: crate::pac::ESPI0,
};

// ---- DMA coherence ----
//
// The eSPI controller reads and writes [`EspiRam`] as an AHB master. The
// current MCX-A target has no data cache and its SRAM is non-cacheable, so the
// buffer needs no cache maintenance. Inline `dsb()` barriers preserve ordering
// between controller status reads and RAM reads, and complete CPU RAM writes
// before the controller is notified. `Espi::new` debug-asserts that the data
// cache is disabled; supporting cacheable RAM would require cache maintenance.

/// Debug-assert that the data cache is disabled (see the DMA coherence note above).
#[inline]
fn assert_dma_noncacheable() {
    debug_assert!(
        !SCB::dcache_enabled(),
        "eSPI mailbox RAM assumes non-cacheable SRAM, but the data cache is enabled; implement \
         cache maintenance before enabling the D-cache"
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
    /// Two-bit PnCFG.DIRECTION: 0 = both, 1 = host-write only,
    /// 2 = host-read only, 3 = neither (RM 52.8.1.22).
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
    /// A port's RAM window exceeds the [`EspiRam`] buffer or is misaligned.
    RamOverflow {
        /// Offending hardware port index.
        port: u8,
    },
    /// Two DMA windows overlap, or multiple ports claim a unique role.
    ConflictingPorts,
    /// A port setting is outside its hardware field or contradicts the capabilities.
    InvalidPortConfig,
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
            pac_espi::Wirero(self.0).$pac_field()
        }
    };
}

impl VWireIn {
    vwire_in_bit!(
        /// Sleep S3 (active low).
        slp_s3n,
        slp_s3n
    );
    vwire_in_bit!(
        /// Sleep S4 (active low).
        slp_s4n,
        slp_s4n
    );
    vwire_in_bit!(
        /// Sleep S5 (active low).
        slp_s5n,
        slp_s5n
    );
    vwire_in_bit!(
        /// Suspend status.
        sus_stat,
        sus_stat
    );
    vwire_in_bit!(
        /// Platform reset (active low).
        pltrstn,
        pltrstn
    );
    vwire_in_bit!(
        /// OOB reset warning.
        oob_rst_warn,
        oob_rst_warn
    );
    vwire_in_bit!(
        /// Host reset warning.
        host_rst_warn,
        host_rst_warn
    );
    vwire_in_bit!(
        /// Suspend warning.
        sus_warn,
        sus_warn
    );
    vwire_in_bit!(
        /// Suspend power-down acknowledge (active low).
        sus_pwrdn_ackn,
        sus_pwrdn_ackn
    );
    vwire_in_bit!(
        /// Sleep A (active low).
        slp_an,
        slp_an
    );
    vwire_in_bit!(
        /// Wired LAN sleep.
        slp_lan,
        slp_lan
    );
    vwire_in_bit!(
        /// Wireless LAN sleep.
        slp_wlan,
        slp_wlan
    );
    vwire_in_bit!(
        /// Host entering C10.
        host_c10n,
        host_c10n
    );

    /// PCIe-to-EC wire group.
    #[inline]
    pub const fn p2e(self) -> u8 {
        pac_espi::Wirero(self.0).p2e()
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
    generation: u32,
    data_in: u32,
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

impl PortEvent {
    /// Input metadata captured with this event, rather than a later live read.
    pub fn endpoint_data(&self) -> (u16, u8) {
        let data = pac_espi::Pdatain(self.data_in);
        (data.idx(), data.data_len())
    }
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
    /// Physical Alert/Reset GPIO input changed (ESPIMISC.GPIO_IN), not a VW.
    GpioInput(bool),
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
    /// Decoded SAF count (DATAIN[11:0] + 1).
    ///
    /// The reference manual's SAF examples and generic DATAIN table disagree
    /// about inclusion of the four address bytes (52.6.1/52.6.3 versus
    /// 52.8.1.27). This preserves the PR's raw-count interface; it does not
    /// normalize that conflict or translate erase counts to flash geometry.
    pub length: u32,
    /// Transaction tag to echo in completions.
    pub tag: u8,
    /// `true` for the initial read request; `false` for a continuation pull
    /// (the host fetching the next split completion).
    pub read_start: bool,
}

impl FlashRequest {
    /// Entire requested range within an application-provided backing store.
    pub fn range(&self, capacity: usize) -> Option<core::ops::Range<usize>> {
        let start = self.addr as usize;
        let end = start.checked_add(self.length as usize)?;
        (self.length != 0 && end <= capacity).then_some(start..end)
    }
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
    let mut m = pac_espi::Mstat(0);
    m.set_p80int(true);
    m.0
};
const PENDING_BUSRST: u32 = {
    let mut m = pac_espi::Mstat(0);
    m.set_busrst((1) != 0);
    m.0
};
const PENDING_IRQUPD: u32 = {
    let mut m = pac_espi::Mstat(0);
    m.set_irqupd((1) != 0);
    m.0
};
const PENDING_WIRECHG: u32 = {
    let mut m = pac_espi::Mstat(0);
    m.set_wirechg((1) != 0);
    m.0
};
const PENDING_HSTALL: u32 = {
    let mut m = pac_espi::Mstat(0);
    m.set_hstall((1) != 0);
    m.0
};
const PENDING_CRCERR: u32 = {
    let mut m = pac_espi::Mstat(0);
    m.set_crcerr((1) != 0);
    m.0
};
const PENDING_GPIO: u32 = {
    let mut m = pac_espi::Mstat(0);
    m.set_gpio((1) != 0);
    m.0
};

/// PnSTAT bits that are independent single-bit event flags and therefore safe
/// to OR-accumulate across coalesced interrupts: the INT* flags and ERR0-ERR3,
/// derived from the PAC's STAT field setters. The RDSTAT/WRSTAT fields are
/// multi-bit state encodings that must never be ORed across snapshots; the
/// latest snapshot is kept separately in [`PORT_LAST_STAT`].
const PORT_EVENT_MASK: u32 = {
    let mut s = PortStat(0);
    s.set_interr(true);
    s.set_intrd(true);
    s.set_intwr(true);
    s.set_intspc0(true);
    s.set_intspc1(true);
    s.set_intspc2(true);
    s.set_instspc3(true);
    s.set_err0(true);
    s.set_err1(true);
    s.set_err2(true);
    s.set_err3(true);
    s.0
};

static BUS_PENDING: AtomicU32 = AtomicU32::new(0);
static BUS_GENERATION: AtomicU32 = AtomicU32::new(0);
static WIRE_INPUT: AtomicU32 = AtomicU32::new(0);
static GPIO_WIRE_INPUT: AtomicU32 = AtomicU32::new(0);
static GPIO_PIN_INPUT: AtomicU32 = AtomicU32::new(0);
// Software-only pending cause: WIREIN_GPIO is captured on WIRECHG, not GPIO.
const PENDING_GPIO_WIRE: u32 = 1 << 31;
static PORT_PENDING: [AtomicU32; PORT_COUNT] = [const { AtomicU32::new(0) }; PORT_COUNT];
static PORT_LAST_DATA: [AtomicU32; PORT_COUNT] = [const { AtomicU32::new(0) }; PORT_COUNT];
/// Latest raw PnSTAT snapshot per port (overwritten, never ORed).
static PORT_LAST_STAT: [AtomicU32; PORT_COUNT] = [const { AtomicU32::new(0) }; PORT_COUNT];
static WAKER: AtomicWaker = AtomicWaker::new();

fn reset_static_state() {
    BUS_PENDING.store(0, Ordering::Relaxed);
    BUS_GENERATION.store(0, Ordering::Relaxed);
    WIRE_INPUT.store(0, Ordering::Relaxed);
    GPIO_WIRE_INPUT.store(0, Ordering::Relaxed);
    GPIO_PIN_INPUT.store(0, Ordering::Relaxed);
    for pending in &PORT_PENDING {
        pending.store(0, Ordering::Relaxed);
    }
    for stat in &PORT_LAST_STAT {
        stat.store(0, Ordering::Relaxed);
    }
    for data in &PORT_LAST_DATA {
        data.store(0, Ordering::Relaxed);
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
        let r = INFO.regs;
        let status = pac_espi::Mstat(r.intstat().read().0);
        if status.0 == 0 {
            return;
        }
        // RM 52.8.1.3: allow at least 50 PCLK cycles before acknowledging.
        // Each volatile APB read takes at least one PCLK cycle, independently
        // of the CPU/functional clock ratio.
        for _ in 0..50 {
            let _ = r.mstat().read();
        }
        if status.busrst() {
            // Invalidate queued software observations here, not when the task
            // eventually consumes BusReset. Later IRQs retain new requests.
            BUS_GENERATION.fetch_add(1, Ordering::Relaxed);
            BUS_PENDING.store(0, Ordering::Relaxed);
            // Its notification was canceled too. An identical write in the
            // next generation must not be deduplicated against this snapshot.
            GPIO_WIRE_INPUT.store(0, Ordering::Relaxed);
            for pending in &PORT_PENDING {
                pending.store(0, Ordering::Relaxed);
            }
            for data in &PORT_LAST_DATA {
                data.store(0, Ordering::Relaxed);
            }
            for stat in &PORT_LAST_STAT {
                stat.store(0, Ordering::Relaxed);
            }
        }

        // Latch per-port status. Only independent single-bit flags (INT*,
        // ERR*) are OR-accumulated; the RDSTAT/WRSTAT state fields are
        // multi-bit encodings, so ORing two snapshots would fabricate values
        // (e.g. Read|Write reading as Erase). The latest snapshot is stored
        // by overwrite for the state fields instead.
        let portint = status.portint();
        for (port, pending) in PORT_PENDING.iter().enumerate() {
            if portint & (1 << port) != 0 {
                let pstat = r.pstat(port).read();
                let mut clear = PortStat(0);
                clear.set_interr(pstat.interr());
                clear.set_intrd(pstat.intrd());
                clear.set_intwr(pstat.intwr());
                clear.set_intspc0(pstat.intspc0());
                clear.set_intspc1(pstat.intspc1());
                clear.set_intspc2(pstat.intspc2());
                clear.set_instspc3(pstat.instspc3());
                r.pstat(port).write_value(clear);
                if !r.mstat().read().inrst() {
                    // Hardware observations after the reset boundary belong
                    // to the current generation. Ignore requests during reset.
                    PORT_LAST_DATA[port].store(r.pdatain(port).read().0, Ordering::Relaxed);
                    PORT_LAST_STAT[port].store(pstat.0, Ordering::Relaxed);
                    pending.fetch_or(pstat.0 & PORT_EVENT_MASK, Ordering::Relaxed);
                }
            }
        }
        if status.wirechg() {
            WIRE_INPUT.store(r.wirero().read().0, Ordering::Relaxed);
            let gpio = r.wirein_gpio().read().0;
            if GPIO_WIRE_INPUT.swap(gpio, Ordering::Relaxed) != gpio {
                BUS_PENDING.fetch_or(PENDING_GPIO_WIRE, Ordering::Relaxed);
            }
        }
        if status.gpio() {
            GPIO_PIN_INPUT.store(r.espimisc().read().gpio_in() as u32, Ordering::Relaxed);
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
            r.mstat().write_value(pac_espi::Mstat(bus_bits));
        }

        WAKER.wake();
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

#[doc(hidden)]
#[macro_export]
macro_rules! impl_espi_pin {
    ($pin:ident, $mux:ident, $trait:ident) => {
        impl $crate::espi::$trait for $crate::peripherals::$pin {
            fn configure(&self) {
                use $crate::gpio::{DriveStrength, Pull, SealedPin, SlewRate};
                use $crate::pac::port::Mux;
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
    info: &'static Info,
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
    generation: u32,
}

impl<'d> Espi<'d> {
    /// Create and start the eSPI target controller.
    ///
    /// Enables the peripheral clock through the standard clock helpers,
    /// configures the pins, programs capabilities and per-port windows, and
    /// enables the controller in the mode selected by
    /// [`Config::enable_mode`]. Host interaction is reported via
    /// [`Self::wait_event`] once the controller is enabled.
    ///
    /// RAM is consumed for the program's lifetime: forgetting this driver must
    /// not release a live AHB target. It is not returned on drop. This
    /// constructor claims eSPI pins and rejects LPC mode.
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
        ram: &'static mut EspiRam,
        ports: &[PortConfig],
        config: Config,
    ) -> Result<Self, InitError> {
        if ports.len() > PORT_COUNT {
            return Err(InitError::TooManyPorts);
        }
        if config.enable_mode == EnableMode::Lpc {
            // This constructor claims eSPI pins, not the LPC pin interface.
            return Err(InitError::InvalidPortConfig);
        }
        let mut ranges = [(0usize, 0usize); PORT_COUNT];
        let mut oob_count = 0;
        let mut flash_count = 0;
        let mut master_count = 0;
        for (i, port) in ports.iter().enumerate() {
            if port.direction > 3 || port.idx_offset > 15 {
                return Err(InitError::InvalidPortConfig);
            }
            let mut size = port.ram_bytes();
            let mut alignment = size.max(1);
            match port.port_type {
                PortType::Unconfigured => {
                    size = 0;
                    alignment = 1;
                }
                PortType::MailboxOobSplit => {
                    oob_count += 1;
                    if !config.enable_oob {
                        return Err(InitError::InvalidPortConfig);
                    }
                }
                PortType::BusMasterFlashSingle => {
                    flash_count += 1;
                    if !config.enable_saf {
                        return Err(InitError::InvalidPortConfig);
                    }
                    // SAF ignores RAMUSE.LEN: reserve address plus advertised payload.
                    size = 4 + (64usize << config.max_flash_payload_size as usize);
                    alignment = 4;
                }
                PortType::BusMasterMemSingle => {
                    master_count += 1;
                    size = size.max(8 + (32usize << config.max_payload_size as usize));
                    alignment = 4;
                }
                _ => {}
            }
            let start = port.ram_offset as usize;
            if !protocol::ram_window_valid(start, size, alignment) {
                return Err(InitError::RamOverflow { port: i as u8 });
            }
            if size != 0 {
                let end = start + size;
                if ranges[..i].iter().any(|&(a, b)| a < end && start < b) {
                    return Err(InitError::ConflictingPorts);
                }
                ranges[i] = (start, end);
            }
        }
        if oob_count > 1 || flash_count > 1 || master_count > 1 {
            return Err(InitError::ConflictingPorts);
        }
        if config.enable_oob != (oob_count == 1) || config.enable_saf != (flash_count == 1) {
            return Err(InitError::InvalidPortConfig);
        }
        if config.status_block.is_some_and(|sb| sb.offset & 0x7 != 0) {
            return Err(InitError::StatusBlockMisaligned);
        }

        // A previous driver instance may have left latched events behind;
        // exclusivity itself is guaranteed by ownership of the `ESPI0` Peri.
        assert_dma_noncacheable();
        EspiInterrupt::disable();
        reset_static_state();

        // SAFETY: we own the eSPI peripheral and bring up its clock once.
        let parts = unsafe { clocks::enable_and_reset::<crate::peripherals::ESPI0>(&config.clock)? };
        let max_bus_hz = match config.bus_speed {
            MaxSpeed::Speed20MHz => 20_000_000,
            MaxSpeed::Speed25MHz => 25_000_000,
            MaxSpeed::Speed33MHz => 33_000_000,
            MaxSpeed::Speed50MHz => 50_000_000,
            MaxSpeed::Speed66MHz => 66_000_000,
        };
        if parts.freq < 2 * max_bus_hz {
            return Err(ClockError::BadConfig {
                clock: "espi fast clock",
                reason: "must be at least twice the advertised serial clock",
            }
            .into());
        }

        clk.configure();
        cs.configure();
        io0.configure();
        io1.configure();
        io2.configure();
        io3.configure();
        rst.configure();
        notify.configure();

        let info = &INFO;
        let r = info.regs;
        let ram_ptr = ram.0.as_mut_ptr();

        // RAM base and mapped bases.
        r.rambase().write(|w| w.set_ram(ram_ptr as u32 >> 12));
        r.mapbase().write(|w| {
            w.set_base0(config.base0_addr);
            w.set_base1(config.base1_addr);
        });

        // Host-visible status block location (enabled via MCTRL.SBLKENA
        // below; alignment validated up front). STATADDR.OFF holds address
        // bits 15:3.
        if let Some(sb) = config.status_block {
            r.stataddr().write(|w| {
                w.set_off(sb.offset >> 3);
                w.set_base(sb.base as u8);
            });
        }

        // Advertised capabilities.
        r.espicap().write(|w| {
            w.set_spicap(pac_espi::EspicapSpicap::from_bits(config.spi_mode as u8));
            w.set_maxspd(config.bus_speed as u8);
            w.set_trgt_req_size_supp(config.max_saf_rx_req_size as u8);
            w.set_memmx(config.max_payload_size as u8);
            w.set_alpin(config.enable_alert_pin);
            if config.enable_saf {
                w.set_saf(true);
                w.set_safera(config.saf_erase_size as u8);
                w.set_flashmx(config.max_flash_payload_size as u8);
            }
            if config.enable_oob {
                w.set_oobok(true);
                w.set_oobmx(config.max_oob_payload_size as u8);
            }
        });

        // Per-port configuration.
        let mut state = Self {
            info,
            _phantom: PhantomData,
            _wake_guard: parts.wake_guard,
            ram: ram_ptr,
            addr_offset: [0; PORT_COUNT],
            ram_offset: [0; PORT_COUNT],
            ram_window: [0; PORT_COUNT],
            port_type: [None; PORT_COUNT],
            oob_port: INVALID_PORT,
            saf_port: INVALID_PORT,
            generation: BUS_GENERATION.load(Ordering::Relaxed),
        };

        let mut pena = 0u8;
        for (i, port) in ports.iter().enumerate() {
            let p = i;
            r.pcfg(p).write(|w| {
                w.set_type_(pac_espi::PcfgType::from_bits(port.port_type as u8));
                w.set_direction(pac_espi::PcfgDirection::from_bits(port.direction));
            });
            r.pramuse(p).write(|w| {
                w.set_off(port.ram_offset);
                w.set_len(port.ram_size as u8);
            });
            r.paddr(p).write(|w| {
                w.set_off(port.addr_offset);
                w.set_base_asz(pac_espi::PaddrBaseAsz::from_bits(port.addr_base as u8));
                w.set_idxoff(port.idx_offset);
            });
            if port.port_type != PortType::Unconfigured {
                pena |= 1 << i;
            }

            state.addr_offset[i] = port.addr_offset;
            state.ram_offset[i] = port.ram_offset;
            // Derive capacity from the validated reservation, including zero
            // capacity for endpoints/unconfigured ports. Never reconstruct it
            // from RAMUSE.LEN, which does not apply to those port types.
            let reserved = ranges[i].1 - ranges[i].0;
            state.ram_window[i] = match port.port_type {
                PortType::MailboxSplit | PortType::MailboxOobSplit => reserved / 2,
                _ => reserved,
            };
            state.port_type[i] = Some(port.port_type);
            match port.port_type {
                PortType::MailboxOobSplit => state.oob_port = i as u8,
                PortType::BusMasterFlashSingle => state.saf_port = i as u8,
                _ => {}
            }
        }

        // Master control: enable ports and the selected mode.
        r.mctrl().write(|w| {
            w.set_pena(pena);
            w.set_enable(pac_espi::MctrlEnable::from_bits(config.enable_mode as u8));
            w.set_p80ena(config.enable_p80);
            w.set_clk_div_disable(config.disable_clk_div);
            w.set_early_sample(config.enable_early_sample);
            w.set_sblkena(config.status_block.is_some());
        });

        // Enable event interrupts (everything the event model reports) and
        // all per-port interrupt rules.
        r.mstat().write_value(pac_espi::Mstat(0xFFFF_FFFF));
        r.intenset().write(|w| {
            w.set_portint(pena);
            w.set_p80int(config.enable_p80);
            w.set_busrst(true);
            w.set_irqupd(true);
            w.set_wirechg(true);
            w.set_hstall(true);
            w.set_crcerr(true);
            w.set_gpio(true);
        });
        for i in 0..ports.len() {
            r.pirulestat(i).modify(|w| {
                w.set_interr(true);
                w.set_intrd(true);
                w.set_intwr(true);
                w.set_intspc(0xF);
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
    /// This is a coalescing status interface, not a lossless transaction queue.
    /// Service each event before waiting again. Reset invalidates pending old
    /// events in the ISR; response methods return `BusReset` if reset interrupts
    /// application work. Repeated resets can coalesce into one notification.
    pub async fn wait_event(&mut self) -> Event {
        poll_fn(|cx| {
            WAKER.register(cx.waker());
            critical_section::with(|_| {
                if let Some(event) = self.take_bus_event() {
                    return Poll::Ready(event);
                }
                let status = self.info.regs.mstat().read();
                if status.busrst() || status.inrst() {
                    return Poll::Pending;
                }
                for (port, pending) in PORT_PENDING.iter().enumerate() {
                    let (flags, state) = critical_section::with(|_| {
                        // Keep the flag batch paired with its state snapshot; an IRQ
                        // between these reads must not replace the request kind.
                        (
                            PortStat(pending.swap(0, Ordering::Relaxed)),
                            PortStat(PORT_LAST_STAT[port].load(Ordering::Relaxed)),
                        )
                    });
                    if flags.0 != 0 {
                        return Poll::Ready(Event::Port(self.decode_port_event(port as u8, flags, state)));
                    }
                }
                Poll::Pending
            })
        })
        .await
    }

    fn take_bus_event(&mut self) -> Option<Event> {
        let r = self.info.regs;
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
            } else if pending & PENDING_GPIO_WIRE != 0 {
                PENDING_GPIO_WIRE
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
                PENDING_BUSRST => {
                    self.generation = BUS_GENERATION.load(Ordering::Relaxed);
                    Event::BusReset
                }
                PENDING_CRCERR => Event::CrcError,
                PENDING_HSTALL => Event::HostStall,
                PENDING_WIRECHG => Event::WireChange(VWireIn(WIRE_INPUT.load(Ordering::Relaxed))),
                PENDING_GPIO_WIRE => {
                    let gpio = pac_espi::WireinGpio(GPIO_WIRE_INPUT.load(Ordering::Relaxed));
                    Event::GpioWire(GpioWire {
                        index: gpio.index(),
                        valid: gpio.valid(),
                        level: gpio.level(),
                    })
                }
                PENDING_GPIO => Event::GpioInput(GPIO_PIN_INPUT.load(Ordering::Relaxed) != 0),
                PENDING_P80 => {
                    let p80 = r.p80stat().read();
                    Event::Port80(Port80Status {
                        current: p80.curr(),
                        previous: p80.prev(),
                        counter: p80.cnt(),
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
            generation: self.generation,
            data_in: PORT_LAST_DATA[port as usize].load(Ordering::Relaxed),
            port,
            read: flags.intrd(),
            write: flags.intwr(),
            spec0: flags.intspc0(),
            spec1: flags.intspc1(),
            spec2: flags.intspc2(),
            spec3: flags.instspc3(),
            error: flags.interr().then(|| self.decode_port_error(port, flags)).flatten(),
            wrstat: state.wrstat(),
        }
    }

    /// Port of `ESPI_GetPortErrorStatus`: ERR0-3 meaning depends on port type.
    fn decode_port_error(&self, port: u8, pstat: PortStat) -> Option<PortError> {
        use PortError::*;
        let ty = self.port_type[port as usize]?;
        match ty {
            PortType::AcpiEndpoint | PortType::AcpiIndexData => {
                if pstat.err0() {
                    Some(EndpointWriteOverrun)
                } else if pstat.err1() {
                    Some(EndpointReadEmpty)
                } else if pstat.err2() {
                    Some(EndpointInvalidSize)
                } else {
                    None
                }
            }
            PortType::MailboxShared | PortType::MailboxSingle | PortType::MailboxSplit | PortType::MailboxOobSplit => {
                if pstat.err0() {
                    Some(MailboxInvalidAccess)
                } else if pstat.err1() {
                    Some(MailboxOverrunUnderrun)
                } else if pstat.err2() {
                    Some(MailboxSizeOverflow)
                } else if pstat.err3() {
                    Some(MailboxRamBusError)
                } else {
                    None
                }
            }
            PortType::BusMasterMemSingle | PortType::BusMasterFlashSingle => {
                if pstat.err0() {
                    Some(MasterFromHostFailed)
                } else if pstat.err1() {
                    Some(MasterOverrunUnderrun)
                } else if pstat.err2() {
                    Some(MasterEraseFailed)
                } else if pstat.err3() {
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
        self.info.regs.mstat().read().0
    }

    /// Whether a delivered event belongs to the current bus generation.
    /// Recheck after awaiting application work. Events must not be reused after
    /// the next `wait_event` call; reset cancels any in-progress response.
    pub fn event_is_current(&self, event: &PortEvent) -> bool {
        event.generation == self.generation && self.on_current_bus(|| ()).is_ok()
    }

    fn on_current_bus<R>(&self, operation: impl FnOnce() -> R) -> Result<R, BusReset> {
        critical_section::with(|_| {
            let status = self.info.regs.mstat().read();
            if self.generation != BUS_GENERATION.load(Ordering::Relaxed) || status.busrst() || status.inrst() {
                Err(BusReset)
            } else {
                let result = operation();
                // Interrupt masking cannot stop the external reset pin. If it
                // changed during the operation, do not report a valid response.
                let status = self.info.regs.mstat().read();
                if status.busrst() || status.inrst() {
                    Err(BusReset)
                } else {
                    Ok(result)
                }
            }
        })
    }

    /// Raw advertised-capabilities register (ESPICAP).
    pub fn capabilities(&self) -> u32 {
        self.info.regs.espicap().read().0
    }

    /// Raw host-negotiated configuration register (ESPICFG).
    pub fn host_config(&self) -> u32 {
        self.info.regs.espicfg().read().0
    }

    /// Current host-driven virtual wire states.
    pub fn vwires(&self) -> VWireIn {
        VWireIn(self.info.regs.wirero().read().0)
    }

    /// Current GPIO virtual-wire input snapshot (WIREIN_GPIO).
    pub fn gpio_wires(&self) -> GpioWire {
        let gpio = self.info.regs.wirein_gpio().read();
        GpioWire {
            index: gpio.index(),
            valid: gpio.valid(),
            level: gpio.level(),
        }
    }

    /// Raw MCU-to-host virtual wire register (WIREWO), including DONE.
    pub fn vwires_out_raw(&self) -> u32 {
        self.info.regs.wirewo().read().0
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
        let r = self.info.regs;
        let mut next = r.wirewo().read();
        if !next.done() {
            return Err(VWireBusy);
        }
        // RM 52.8.1.11: WIREWO holds wire levels. A single-wire update must
        // preserve unrelated levels, particularly BOOT_DONE and BOOT_ERRN.
        next.set_done(false);
        let w = &mut next;
        match wire {
            VWireOut::OobRstAck => w.set_oob_rst_ack(value != 0),
            VWireOut::WakenScin => w.set_waken_scin(value != 0),
            VWireOut::Pmen => w.set_pmen(value != 0),
            VWireOut::Scin => w.set_scin(value != 0),
            VWireOut::Smin => w.set_smin(value != 0),
            VWireOut::Rcinn => w.set_rcinn(value != 0),
            VWireOut::HostRstAck => w.set_host_rst_ack(value != 0),
            VWireOut::SusAckN => w.set_susackn(value != 0),
            VWireOut::E2p => w.set_e2p(value),
            VWireOut::BootDone => w.set_boot_done(value != 0),
            VWireOut::BootErrn => w.set_boot_errn((value & 1) != 0),
            VWireOut::DswPwrokRst => w.set_dsw_pwrok_rst(value != 0),
        }
        r.wirewo().write_value(next);
        Ok(())
    }

    /// Replace all writable wire levels. Unlike `send_vwire`, this does not
    /// preserve other levels. Returns busy until the previous update completes.
    pub fn send_vwire_mask(&mut self, mask: u32) -> Result<(), VWireBusy> {
        if !self.info.regs.wirewo().read().done() {
            return Err(VWireBusy);
        }
        self.info.regs.wirewo().write_value(pac_espi::Wirewo(mask & 0x7_ffff));
        Ok(())
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
    /// Only one push may be outstanding: wait for `is_irq_push_done()` and
    /// consume the completion event before pushing another IRQ.
    pub fn push_irq(&mut self, irq: u8) {
        self.info.regs.irqpush().write(|w| w.set_irq(irq));
    }

    /// Whether the last pushed IRQ has been delivered.
    pub fn is_irq_push_done(&self) -> bool {
        self.info.regs.irqpush().read().done()
    }

    // ---------------------------------------------------------------------
    // Port 80
    // ---------------------------------------------------------------------

    /// Current Port 80 status, if Port 80 capture is enabled.
    pub fn port80_status(&self) -> Option<Port80Status> {
        let r = self.info.regs;
        if !r.mctrl().read().p80ena() {
            return None;
        }
        let p80 = r.p80stat().read();
        Some(Port80Status {
            current: p80.curr(),
            previous: p80.prev(),
            counter: p80.cnt(),
        })
    }

    /// Reset the Port 80 POST-code counter.
    pub fn reset_port80_counter(&mut self) {
        self.info.regs.p80stat().modify(|w| w.set_rst(true));
    }

    // ---------------------------------------------------------------------
    // Endpoint / mailbox data
    // ---------------------------------------------------------------------

    /// Read PnDATAIN: `(index, data-or-length)` (port of `ESPI_GetEndpointData`).
    /// Prefer `PortEvent::endpoint_data` when handling a delivered event.
    pub fn endpoint_data(&self, port: u8) -> (u16, u8) {
        let datain = self.info.regs.pdatain(port as usize).read();
        (datain.idx(), datain.data_len())
    }

    /// Write a response byte to PnDATAOUT (port of `ESPI_WritePortData`).
    pub fn write_endpoint_data(&mut self, port: u8, data: u8) -> Result<(), BusReset> {
        self.on_current_bus(|| self.info.regs.pdataout(port as usize).write(|w| w.set_data(data)))
    }

    /// Length in bytes of the mailbox message currently in the port window.
    pub fn mailbox_msg_len(&self, port: u8) -> usize {
        self.info.regs.pdatain(port as usize).read().data_len() as usize + 1
    }

    /// Size in bytes of the port's mailbox window.
    pub fn mailbox_size(&self, port: u8) -> usize {
        self.ram_window[port as usize]
    }

    /// Copy the current mailbox message out of the port's RAM window.
    ///
    /// Uses the last live DATAIN offset and length, clamped to `buf` and the
    /// reservation. Returns zero for non-mailbox ports. For queued events, use
    /// `read_mailbox_event` to select the associated captured access metadata.
    pub fn read_mailbox(&mut self, port: u8, buf: &mut [u8]) -> usize {
        if !self.is_mailbox(port) || self.on_current_bus(|| ()).is_err() {
            return 0;
        }
        let access = self.info.regs.pdatain(port as usize).read();
        let len = (access.data_len() as usize + 1).min(buf.len());
        self.read_mailbox_at(port, access.idx() as usize, &mut buf[..len])
    }

    fn is_mailbox(&self, port: u8) -> bool {
        matches!(
            self.port_type.get(port as usize),
            Some(Some(
                PortType::MailboxShared | PortType::MailboxSingle | PortType::MailboxSplit | PortType::MailboxOem
            ))
        )
    }

    /// Copy mailbox bytes at a host-visible offset. For split mailboxes the
    /// second half starts at `mailbox_size(port)`, matching DATAIN.IDX.
    /// Returns zero for a non-mailbox port or an out-of-range offset.
    pub fn read_mailbox_at(&self, port: u8, offset: usize, buf: &mut [u8]) -> usize {
        if !self.is_mailbox(port) {
            return 0;
        }
        let window = self.ram_window[port as usize];
        let capacity = if self.port_type[port as usize] == Some(PortType::MailboxSplit) {
            2 * window
        } else {
            window
        };
        let len = capacity.saturating_sub(offset).min(buf.len());
        if len == 0 {
            return 0;
        }
        self.on_current_bus(|| {
            // Order the controller status read before reading its RAM writes.
            asm::dsb();
            for (i, byte) in buf[..len].iter_mut().enumerate() {
                // SAFETY: the port window was validated against RAM_SIZE at init.
                *byte = unsafe { self.port_ram_ptr(port).add(offset + i).read_volatile() };
            }
            len
        })
        .unwrap_or(0)
    }

    /// Copy the mailbox access described by an event's captured DATAIN fields.
    /// A reset between event delivery and this call cancels the copy.
    pub fn read_mailbox_event(&mut self, event: &PortEvent, buf: &mut [u8]) -> Result<usize, BusReset> {
        if !self.event_is_current(event) {
            return Err(BusReset);
        }
        let (offset, length) = event.endpoint_data();
        let length = (length as usize + 1).min(buf.len());
        self.on_current_bus(|| self.read_mailbox_at(event.port, offset as usize, &mut buf[..length]))
    }

    /// Copy bytes into a mailbox's host-read window. `offset` is relative to
    /// that direction's window; split mailboxes select their transmit half.
    /// Announce readiness separately with `set_mailbox_status` after copying.
    pub fn write_mailbox(&mut self, port: u8, offset: usize, data: &[u8]) -> Result<(), MailboxError> {
        if !self.is_mailbox(port) {
            return Err(MailboxError::NotMailbox);
        }
        let window = self.ram_window[port as usize];
        if !offset.checked_add(data.len()).is_some_and(|end| end <= window) {
            return Err(MailboxError::InvalidRange);
        }
        let base = if self.port_type[port as usize] == Some(PortType::MailboxSplit) {
            window
        } else {
            0
        };
        self.on_current_bus(|| {
            for (i, &byte) in data.iter().enumerate() {
                // SAFETY: the range lies within the validated transmit reservation.
                unsafe { self.port_ram_ptr(port).add(base + offset + i).write_volatile(byte) };
            }
            // Complete RAM writes before a subsequent readiness announcement.
            asm::dsb();
        })
        .map_err(|_| MailboxError::BusReset)
    }

    /// Update the port's SSTCL status field (mailbox handshakes).
    pub fn set_mailbox_status(&mut self, port: u8, status: MailboxStatus) -> Result<(), BusReset> {
        self.on_current_bus(|| {
            self.info
                .regs
                .pirulestat(port as usize)
                .modify(|w| w.set_sstcl(status as u8))
        })
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
    /// Messages must fit both the OOB transmit window and the 124-byte
    /// hardware transfer limit (RM 52.8.1.26), as well as the negotiated OOB
    /// payload plus its nine framing bytes (RM 52.3.1.4).
    pub fn send_oob(&mut self, data: &[u8], announce: bool) -> Result<(), OobError> {
        self.on_current_bus(|| {
            if self.oob_port == INVALID_PORT {
                return Err(OobError::NotConfigured);
            }
            let port = self.oob_port;
            let mb_size = self.ram_window[port as usize];
            let host = self.info.regs.espicfg().read();
            if !host.oobok() || !(1..=3).contains(&host.oobsz()) {
                return Err(OobError::NotReady);
            }
            // RM 52.3.1.4: a 64-byte negotiated payload permits 73 bytes of
            // complete OOB packet data. Include those nine framing bytes once.
            let host_packet_limit = (32usize << host.oobsz()) + 9;
            let length =
                protocol::oob_len(data.len(), mb_size.min(host_packet_limit)).ok_or(OobError::InvalidLength)?;
            if self.info.regs.pstat(port as usize).read().rdstat() != 0 {
                return Err(OobError::Busy);
            }

            let p = port as usize;
            if announce {
                self.info.regs.pirulestat(p).modify(|w| w.set_sstcl(0x1));
            }

            // Transmit half of the split OOB window sits above the receive half.
            for (i, &byte) in data.iter().enumerate() {
                // SAFETY: the doubled window was validated against RAM_SIZE at init.
                unsafe { self.port_ram_ptr(port).add(mb_size + i).write_volatile(byte) };
            }
            // Complete RAM writes before announcing the OOB message.
            asm::dsb();

            self.info.regs.pomflen(p).write(|w| w.set_len(length));
            self.info.regs.pirulestat(p).modify(|w| w.set_sstcl(0x2));
            Ok(())
        })
        .map_err(|_| OobError::BusReset)?
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
        // Order DATAIN before reading the controller-written message.
        asm::dsb();
        for (i, byte) in buf[..len].iter_mut().enumerate() {
            // SAFETY: the window was validated against RAM_SIZE at init.
            *byte = unsafe { self.port_ram_ptr(port).add(i).read_volatile() };
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
        self.on_current_bus(|| {
            if event.generation != self.generation {
                return None;
            }
            let kind = match event.wrstat {
                1 => FlashRequestKind::Read,
                2 => FlashRequestKind::Write,
                3 => FlashRequestKind::Erase,
                _ => return None,
            };

            let (length, tag) = protocol::saf_input(event.data_in);

            if event.read || event.write {
                // Flash address is big-endian in the first four window bytes.
                let mut addr_bytes = [0u8; 4];
                // Order request status reads before reading the DMA address.
                asm::dsb();
                for (i, byte) in addr_bytes.iter_mut().enumerate() {
                    // SAFETY: the SAF address and payload reservation was validated at init.
                    *byte = unsafe { self.port_ram_ptr(self.saf_port).add(i).read_volatile() };
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
        })
        .ok()
        .flatten()
    }

    /// Copy bytes from the SAF RAM window. Write-request data starts at offset 4.
    ///
    /// Unlike a mutable slice, volatile copies do not lend DMA-owned memory to
    /// safe Rust. Call before accepting another request, which may overwrite it.
    /// Panics if SAF is not configured or the range exceeds its reserved window.
    pub fn read_flash_data(&mut self, offset: usize, buf: &mut [u8]) -> Result<(), BusReset> {
        assert!(self.saf_port != INVALID_PORT);
        assert!(
            offset
                .checked_add(buf.len())
                .is_some_and(|end| end <= self.ram_window[self.saf_port as usize])
        );
        self.on_current_bus(|| {
            // Order the controller status read before reading its RAM writes.
            asm::dsb();
            for (i, byte) in buf.iter_mut().enumerate() {
                // SAFETY: the requested range and the SAF reservation are checked.
                *byte = unsafe { self.port_ram_ptr(self.saf_port).add(offset + i).read_volatile() };
            }
        })
    }

    /// Copy completion bytes into the SAF RAM window, starting at offset zero.
    ///
    /// Call before announcing the completion, not while a transfer is in flight.
    /// Panics if SAF is not configured or the data exceeds its reserved window.
    pub fn write_flash_data(&mut self, data: &[u8]) -> Result<(), BusReset> {
        assert!(self.saf_port != INVALID_PORT);
        assert!(data.len() <= self.ram_window[self.saf_port as usize]);
        self.on_current_bus(|| {
            for (i, &byte) in data.iter().enumerate() {
                // SAFETY: the requested range and the SAF reservation are checked.
                unsafe { self.port_ram_ptr(self.saf_port).add(i).write_volatile(byte) };
            }
            // Complete RAM writes before a subsequent completion announcement.
            asm::dsb();
        })
    }

    /// Program the flash completion transaction type and length
    /// (port of `ESPI_SetFlashOpLen`).
    ///
    /// Data completions must be nonempty and fit the hardware's 124-byte limit
    /// and the host-negotiated payload. Invalid fixed arguments panic rather
    /// than silently wrapping the seven-bit length encoding. A reset or a
    /// renegotiated payload that no longer fits cancels the response.
    pub fn set_flash_op_len(&mut self, response: SafResponse, length: u32) -> Result<(), BusReset> {
        assert!(self.saf_port != INVALID_PORT);
        if response == SafResponse::WithData {
            assert!(length > 0 && length <= 124);
        }
        self.on_current_bus(|| {
            // The host can renegotiate while application work is suspended.
            // Check cancellation before using that new limit, and never panic
            // based on mutable host configuration from a different generation.
            if response == SafResponse::WithData && length > self.flash_max_payload() {
                return Err(BusReset);
            }
            // Complete payload writes before programming completion metadata.
            asm::dsb();
            self.info.regs.pomflen(self.saf_port as usize).write(|w| {
                w.set_trans(response as u8);
                w.set_len(if response == SafResponse::WithData {
                    (length - 1) as u8
                } else {
                    0
                });
            });
            Ok(())
        })?
    }

    /// Update the SAF completion status fields
    /// (port of `ESPI_SetFlashCompletion`).
    pub fn set_flash_completion(
        &mut self,
        tag: u8,
        command: SafCommand,
        completion: SafCompletionType,
    ) -> Result<(), BusReset> {
        assert!(self.saf_port != INVALID_PORT);
        assert!(tag <= 15);
        // Ensure completion data written to the flash window is visible to
        // the controller before the completion is announced.
        asm::dsb();
        self.on_current_bus(|| {
            // The tag must be programmed before the completion command (RM 52.6.1).
            self.info.regs.pirulestat(self.saf_port as usize).modify(|w| {
                w.set_sstcl(0);
                w.set_srst(false);
                w.set_cpu_tag(tag);
                w.set_flash_completion_type(pac_espi::PirulestatFlashCompletionType::from_bits(completion as u8));
            });
            self.info
                .regs
                .pirulestat(self.saf_port as usize)
                .modify(|w| w.set_sstcl(command as u8));
        })
    }

    /// Maximum host-negotiated flash payload in bytes
    /// (port of `ESPI_GetFlashMaxPayload`).
    pub fn flash_max_payload(&self) -> u32 {
        64 << self.info.regs.espicfg().read().flashsz()
    }
}

impl Drop for Espi<'_> {
    fn drop(&mut self) {
        let r = self.info.regs;
        // Mask interrupts before stopping the controller. RAM is static, so even
        // forgetting this driver cannot release storage under an AHB transfer.
        EspiInterrupt::disable();
        r.mctrl()
            .modify(|w| w.set_enable(pac_espi::MctrlEnable::from_bits(EnableMode::Disabled as u8)));
        r.intenclr().write_value(pac_espi::Intenclr(0xFFFF_FFFF));
    }
}

/// A previous virtual-wire update is still in flight (WIREWO.DONE clear).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VWireBusy;

/// A reset canceled the response. Consume `Event::BusReset` before handling
/// more traffic; do not retry an old transaction in the new generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BusReset;

/// Errors accessing an ordinary mailbox's RAM window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MailboxError {
    /// A reset canceled this access.
    BusReset,
    /// The selected port does not have ordinary mailbox RAM.
    NotMailbox,
    /// The requested range exceeds its direction's window.
    InvalidRange,
}

/// OOB channel errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OobError {
    /// A reset canceled this access.
    BusReset,
    /// No port is configured as [`PortType::MailboxOobSplit`].
    NotConfigured,
    /// The host has not enabled/configured the OOB channel.
    NotReady,
    /// A previous outgoing message is still pending.
    Busy,
    /// Message empty or larger than the OOB window.
    InvalidLength,
    /// Received message was longer than the provided buffer.
    Truncated,
}

/// SAF status command written to PIRULESTAT.SSTCL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SafCommand {
    /// Accept the host's flash request.
    AcceptRequest = 0x2,
    /// Announce the prepared flash completion.
    SendCompletion = 0x1,
}

/// SAF completion response encoded in POMFLEN.TRANS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SafResponse {
    /// Unsuccessful completion.
    Failure = 0x0,
    /// Successful completion carrying data.
    WithData = 0x1,
    /// Successful completion without data.
    WithoutData = 0x2,
}
