//! I3C Target Support

use core::future::poll_fn;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU8, AtomicU32, AtomicUsize, Ordering, fence};
use core::task::Poll;

use bbqueue::BBQueue;
use bbqueue::prod_cons::stream::StreamGrantW;
use bbqueue::traits::coordination::cas::AtomicCoord;
use bbqueue::traits::notifier::maitake::MaiNotSpsc;
use bbqueue::traits::storage::Storage;
use embassy_hal_internal::Peri;
use embassy_hal_internal::drop::OnDrop;
use grounded::uninit::GroundedCell;

use super::{Info, Instance, SclPin, SdaPin};
pub use crate::clocks::periph_helpers::{Div4, I3cClockSel, I3cConfig};
use crate::clocks::{ClockError, PoweredClock, WakeGuard, enable_and_reset};
use crate::dma::{Channel, DmaChannel, DmaRequest, TransferOptions};
use crate::gpio::{AnyPin, SealedPin};
use crate::interrupt::typelevel;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::i3c::Sstatus;
use crate::pac::i3c::{
    Evdet, Ibidis, Mstena, SctrlEvent, SdatactrlTxtrig, SdmactrlDmafb, SdmactrlDmatb, SdmactrlDmawidth, SstatusStart,
    SstatusTxnotfull, Stnotstop, Streqrd, Type,
};

/// Setup Errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Clock configuration error.
    ClockSetup(ClockError),
    /// User provided an invalid configuration
    InvalidConfiguration,
    /// Invalid Vendor ID
    InvalidVendorId,
    /// Invalid Part Number
    InvalidPartNumber,
    /// Other internal errors or unexpected state.
    Other,
}

/// I/O Errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum IOError {
    /// Overrun error
    Overrun,
    /// Underrun error
    Underrun,
    /// Underrun and NACK error
    UnderrunNack,
    /// Terminated error
    Terminated,
    /// Invalid start error.
    ///
    /// Reported when the target hardware latches `SERRWARN.INVSTART` —
    /// it observed a START condition in a state where it should not
    /// have (e.g. immediately after a prior transaction without a
    /// clean idle window). This is a recoverable, transient bus
    /// glitch: the hardware will re-synchronize on the next valid
    /// START. Applications should typically log it and resume calling
    /// [`I3c::listen`] rather than treating it as a hard failure.
    InvalidStart,
    /// SDR parity error.
    ///
    /// The target's hardware parity check failed on an SDR byte —
    /// typically a transient single-bit glitch on SDA. The current
    /// transaction is lost, but the bus recovers on the next valid
    /// START. Applications should generally log it and resume listening
    /// rather than treating it as fatal.
    SdrParity,
    /// HDR parity error
    HdrParity,
    /// HDR-DDR CRC error
    HdrDdrCrc,
    /// TE0 or TE1 Error
    TE0TE1,
    /// Overread error
    Overread,
    /// Overwrite
    Overwrite,
    /// IBI request was NACKed by the controller and won't be retried.
    IbiNacked,
    /// IBIs are disabled by the controller (via DISEC CCC).
    IbiDisabled,
    /// Other internal errors or unexpected state.
    Other,
}

impl From<crate::dma::InvalidParameters> for IOError {
    fn from(_value: crate::dma::InvalidParameters) -> Self {
        Self::Other
    }
}

/// Bus transfer type.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(dead_code)]
pub enum BusType {
    /// I3C SDR
    #[default]
    I3cSdr,
    /// Legacy I2C
    I2c,
    /// I3C DDR
    I3cDdr,
}

impl From<BusType> for Type {
    fn from(value: BusType) -> Self {
        match value {
            BusType::I3cSdr => Self::I3c,
            BusType::I2c => Self::I2c,
            BusType::I3cDdr => Self::Ddr,
        }
    }
}

/// I3C target configuration
#[non_exhaustive]
pub struct Config {
    /// 7-bit target address.
    pub address: Option<u8>,

    /// Vendor ID
    pub vendor_id: Option<u16>,

    /// Part number
    pub partno: Option<u32>,

    /// Max write length.
    ///
    /// Must be within the range 8..=4095. Values outside this range
    /// will be clamped accordingly.
    pub max_write_len: u16,

    /// Max read length
    ///
    /// Must be within the range 16..=4095. Values outside this range
    /// will be clamped accordingly.
    pub max_read_len: u16,

    /// Advertise IBI request capability (BCR\[1\]).
    ///
    /// When `true`, the controller can see via `GETBCR` that this target
    /// is capable of generating In-Band Interrupts.  Set this before
    /// calling [`I3c::async_send_ibi`].
    pub ibi_capable: bool,

    /// Advertise that IBIs are followed by a mandatory data byte (BCR\[2\]).
    ///
    /// When `true`, the controller will expect one MDB byte to follow the
    /// IBI address header.  Set `SCTRL.IBIDATA` before asserting the event.
    pub ibi_has_payload: bool,

    /// Clock configuration
    pub clock_config: ClockConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: None,
            vendor_id: None,
            partno: None,
            max_write_len: 256,
            max_read_len: 256,
            ibi_capable: false,
            ibi_has_payload: false,
            clock_config: ClockConfig::default(),
        }
    }
}

/// I3C controller clock configuration
#[derive(Clone)]
#[non_exhaustive]
pub struct ClockConfig {
    /// Powered clock configuration
    pub power: PoweredClock,
    /// I3C clock source
    pub source: I3cClockSel,
    /// I3C pre-divider
    pub div: Div4,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: I3cClockSel::FroLfDiv,
            div: const { Div4::no_div() },
        }
    }
}

/// Possible completitions of a read transaction.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadStatus {
    /// Transaction Complete, but controller stopped reading bytes before we ran out
    EarlyStop(usize),
    /// Transaction Complete, controller naked our last byte
    Complete(usize),
    /// Transaction Incomplete, controller trying to read more bytes than were provided
    Incomplete(usize),
}

/// Bus event observed by [`I3c::listen`].
///
/// `listen()` arms the I3C target interrupt for every kind below, sleeps until
/// one (or more) of them fires, and then returns the highest-priority pending
/// event after acknowledging it (W1C) on `SSTATUS`.
///
/// Multiple events can be pending after a single wakeup. To drain them, call
/// `listen()` repeatedly: each call returns the next pending event without
/// sleeping (because the relevant `SSTATUS` bit is still set until the call
/// acknowledges it).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Event {
    /// STOP detected on the bus — the current transaction has ended.
    Stop,
    /// (Repeated) START detected on the bus — a new transaction is starting.
    Start,
    /// Our static or dynamic address was matched in SDR mode.
    Matched,
    /// Our address was matched while in HDR-DDR mode.
    HdrMatched,
    /// A Common Command Code (CCC) was received from the controller.
    Ccc,
    /// A previously dispatched CCC has been handled by the hardware.
    CccHandled,
    /// The controller assigned or changed our Dynamic Address (e.g. via ENTDAA
    /// or SETNEWDA).
    DynamicAddressChanged,
    /// IBI / mastership / hot-join event status changed; inspect
    /// `SSTATUS.EVDET` for the precise outcome.
    EventStatusChanged,
    /// The RX FIFO contains data ready to be read.
    RxPending,
    /// The controller has addressed us for a read (SSTATUS.STREQRD == Busy)
    /// and the TX FIFO has space — the caller should service the read via
    /// [`I3c::async_respond_to_read`]. Mirrors the NXP SDK
    /// `kI3C_SlaveTransmitEvent`.
    TxPending,
}

/// I3C target driver (DMA-only).
///
/// At I3C-SDR speeds (12.5 MHz, 80 ns/bit) the post-IBI Sr→addr window is
/// ~640 ns and a full TX FIFO empties in ~5.9 µs — far below the ~58 µs
/// embassy wake latency. The only viable way to keep the slave TX FIFO
/// fed is to hand the work to DMA. This driver therefore exposes a single
/// `Dma`-backed mode; all read/write/IBI primitives are prefixed `dma_`.
pub struct I3c<'d> {
    info: &'static Info,
    bbq_state: &'static BbqState,
    _scl: Peri<'d, AnyPin>,
    _sda: Peri<'d, AnyPin>,
    tx_dma: DmaChannel<'d>,
    tx_request: DmaRequest,
    freq: u32,
    _wg: Option<WakeGuard>,
}

impl<'d> I3c<'d> {
    fn new_inner<T: Instance>(
        _peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        tx_dma: DmaChannel<'d>,
        config: Config,
    ) -> Result<Self, SetupError> {
        let ClockConfig { power, source, div } = config.clock_config;

        // Enable clocks
        let conf = I3cConfig { power, source, div };

        let parts = unsafe { enable_and_reset::<T>(&conf).map_err(SetupError::ClockSetup)? };

        scl.mux();
        sda.mux();

        let _scl = scl.into();
        let _sda = sda.into();

        let inst = Self {
            info: T::info(),
            bbq_state: T::bbq_state(),
            _scl,
            _sda,
            tx_dma,
            tx_request: T::TX_DMA_REQUEST,
            freq: parts.freq,
            _wg: parts.wake_guard,
        };

        inst.set_configuration(&config)?;

        Ok(inst)
    }

    fn check_status(&self) -> Result<(), IOError> {
        let status = self.info.regs().sstatus().read();
        let errwarn = self.info.regs().serrwarn().read();

        if status.errwarn() {
            // Clear all set error/warning flags (W1C register)
            self.info.regs().serrwarn().write(|w| w.0 = errwarn.0);

            if errwarn.orun() {
                Err(IOError::Overrun)
            } else if errwarn.urun() {
                Err(IOError::Underrun)
            } else if errwarn.urunnack() {
                Err(IOError::UnderrunNack)
            } else if errwarn.term() {
                Err(IOError::Terminated)
            } else if errwarn.invstart() {
                Err(IOError::InvalidStart)
            } else if errwarn.spar() {
                Err(IOError::SdrParity)
            } else if errwarn.hpar() {
                Err(IOError::HdrParity)
            } else if errwarn.hcrc() {
                Err(IOError::HdrDdrCrc)
            } else if errwarn.s0s1() {
                Err(IOError::TE0TE1)
            } else if errwarn.oread() {
                Err(IOError::Overread)
            } else if errwarn.owrite() {
                Err(IOError::Overwrite)
            } else {
                Err(IOError::Other)
            }
        } else {
            Ok(())
        }
    }

    fn clear_status(&self) -> Sstatus {
        let status = self.info.regs().sstatus().read();
        self.info.regs().sstatus().write(|w| w.0 = status.0);
        // Also clear any sticky error/warning flags (SERRWARN is W1C). If we
        // leave them set, the next call to `check_status()` will spuriously
        // report an error left over from a previous transaction (e.g. a
        // `term` flag from a controller-aborted read).
        let errwarn = self.info.regs().serrwarn().read();
        self.info.regs().serrwarn().write(|w| w.0 = errwarn.0);
        status
    }

    fn flush_fifos(&self) {
        // Use modify so we don't accidentally reset SDATACTRL.TXTRIG/RXTRIG —
        // those control when TXNOTFULL/RXPEND fire and writing the whole
        // register would zero them (worst-case: trigger only when empty,
        // which guarantees underrun on a fast controller read).
        self.info.regs().sdatactrl().modify(|w| {
            w.set_flushfb(true);
            w.set_flushtb(true);
        });
    }

    fn set_configuration(&self, config: &Config) -> Result<(), SetupError> {
        self.info.regs().mconfig().write(|w| w.set_mstena(Mstena::MasterOff));

        // Defensive wipe of all interrupt enables and W1C error/status flags
        // on **both** the controller (M*) and target (S*) sides — the I3C
        // peripheral routes both to the same NVIC line, so any stale M-side
        // enable from a previous owner would assert the line forever and
        // starve our handler.
        self.info.regs().mintclr().write(|w| w.0 = u32::MAX);
        self.info.regs().merrwarn().write(|w| w.0 = u32::MAX);
        self.info.regs().sintclr().write(|w| w.0 = u32::MAX);
        self.info.regs().serrwarn().write(|w| w.0 = u32::MAX);

        // Disable target
        self.info.regs().sconfig().write(|w| {
            w.set_slvena(false);
            w.set_saddr(config.address.unwrap_or(0));
            // MATCHSS=false matches the NXP SDK default (`I3C_SlaveGetDefaultConfig`
            // in fsl_i3c.c sets `matchSlaveStartStop = false`). With it enabled the
            // IP filters START/STOP interrupts to transactions that addressed us,
            // which breaks broadcast-CCC framing.
            w.set_matchss(false);
            w.set_s0ignore(true);
            w.set_ddrok(true);
            // SDK formula (fsl_i3c.c:2882): `matchCount = slowClock_Hz / 1e6 - 1;
            // if (matchCount == 0) matchCount = 1;`. We were one tick high which
            // shifts the bus-available detect window.
            let mhz = self.freq / 1_000_000;
            let bamatch = mhz.saturating_sub(1).max(1).min(63) as u8;
            w.set_bamatch(bamatch);
        });

        // Note: SETDASA is supposed to auto-populate the slave's dynamic
        // address register (SDYNADDR, offset 0x64 — NOT exposed in the
        // embassy nxp-pac). The read-only SMAPCTRL0 mirror reads 0 on
        // this part even after SETDASA appears to succeed on the wire,
        // which leaves the slave unable to address-match directed
        // I3C-SDR traffic (e.g. the read that follows our IBI ACK).
        // Diagnostic logging below dumps SDYNADDR via a raw pointer so
        // we can tell whether the HW absorbed SETDASA at all.

        if config.partno.is_some() {
            let partno = config.partno.unwrap();

            if partno == 0 {
                return Err(SetupError::InvalidPartNumber);
            }

            self.info.regs().sidpartno().write(|w| w.set_partno(partno));
        }

        if config.vendor_id.is_some() {
            let vendor_id = config.vendor_id.unwrap();

            if vendor_id == 0 {
                return Err(SetupError::InvalidVendorId);
            }

            self.info.regs().svendorid().write(|w| w.set_vid(vendor_id));
        }

        self.info.regs().smaxlimits().write(|w| {
            w.set_maxwr(config.max_write_len.clamp(8, 4095));
            w.set_maxrd(config.max_read_len.clamp(16, 4095));
        });

        // Configure BCR (Bus Characteristics Register) — visible to the controller via GETBCR.
        // BCR[1]: IBI Request Capable; BCR[2]: IBI Payload (mandatory data byte follows).
        let bcr: u8 = if config.ibi_capable { 0x02 } else { 0 } | if config.ibi_has_payload { 0x04 } else { 0 };
        self.info.regs().sidext().modify(|w| w.set_bcr(bcr));

        self.clear_status();
        self.flush_fifos();

        // Configure FIFO trigger thresholds for an early refill request.
        // `Triggroneless` (the chip default, but make it explicit) asks for
        // TXNOTFULL as soon as the FIFO has any space, giving software the
        // earliest opportunity to refill before underrun on a target read.
        self.info.regs().sdatactrl().modify(|w| {
            w.set_txtrig(SdatactrlTxtrig::Triggroneless);
        });

        // NOTE: the SDK does NOT pre-load SDYNADDR; it relies on the HW
        // SETDASA handler to do it. Pre-loading DAVALID|MAPSA before SETDASA
        // makes the slave behave as if it already had a dynamic address,
        // which causes the directed-phase `Sr 0x0a+W` (sent in I2C mode by
        // the master during SETDASA) to be ignored.

        // Enable target
        self.info.regs().sconfig().modify(|w| w.set_slvena(true));

        Ok(())
    }

}

impl<'d> I3c<'d> {
    /// Create a new DMA-backed I3C target driver.
    ///
    /// Configures the I3C peripheral as a slave with DMA used for both TX
    /// (read responses) and RX (write reception). RX is **mandatory
    /// BBQ-backed**: the caller passes a `&'static mut [u8]` backing
    /// buffer, the constructor wires it into a `bbqueue::BBQueue`,
    /// opens the first grant, and programs RX DMA to fill it
    /// continuously. The interrupt handler commits committed bytes on
    /// every Stop and immediately re-arms DMA into the next grant.
    ///
    /// At I3C-SDR speeds this is the only way to keep the bus fed
    /// without underrun — see the type docs on [`I3c`] for the latency
    /// budget that drove the design.
    ///
    /// # Parameters
    ///
    /// - `peri`: The I3C peripheral instance.
    /// - `scl`: The SCL pin.
    /// - `sda`: The SDA pin.
    /// - `tx_dma`: The DMA channel for transmitting data.
    /// - `rx_dma`: The DMA channel for receiving data (moved into the
    ///   per-instance BBQ state for the lifetime of this driver).
    /// - `rx_buffer`: A `'static` mutable byte slice used as the RX
    ///   `bbqueue::BBQueue` storage. Must be at least
    ///   `2 * max_rx_transaction` bytes long.
    /// - `max_rx_transaction`: Upper bound, in bytes, on the size of a
    ///   single I3C controller-write transaction the slave is willing to
    ///   buffer. Each RX DMA grant is opened at this size, guaranteeing
    ///   a single transaction is never split across a BBQ ring wrap.
    ///   Choose `>=` the largest controller write you expect (e.g. 64).
    /// - `_irq`: The interrupt binding for the I3C peripheral.
    /// - `config`: The configuration for the I3C target.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` on success.
    /// - `Err(SetupError)` if initialization fails.
    pub fn new_dma<T: Instance>(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        tx_dma: Peri<'d, impl Channel>,
        rx_dma: Peri<'static, impl Channel>,
        _irq: impl crate::interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
        rx_buffer: &'static mut [u8],
        max_rx_transaction: usize,
        config: Config,
    ) -> Result<Self, SetupError> {
        if max_rx_transaction == 0 || rx_buffer.len() < 2 * max_rx_transaction {
            return Err(SetupError::InvalidConfiguration);
        }

        T::Interrupt::unpend();

        // Safety: `_irq` ensures an Interrupt Handler exists.
        unsafe { T::Interrupt::enable() };

        let tx_dma = DmaChannel::new(tx_dma);
        let mut rx_dma = DmaChannel::new(rx_dma);

        tx_dma.enable_interrupt();
        rx_dma.enable_interrupt();

        // Wire the RX DMA completion IRQ to pend the I3C IRQ so we
        // rotate to the next grant promptly when the current one fills.
        // SAFETY: rx_dma is owned exclusively here; moved into the
        // BBQ state immediately below.
        unsafe { rx_dma.set_callback(T::dma_rx_complete_cb) };

        let inst = Self::new_inner(peri, scl, sda, tx_dma, config)?;

        // Move RX DMA + buffer into the per-instance BBQ state, open
        // the first grant, program DMA to fill it.
        //
        // SAFETY: We hold exclusive access to T's BbqState before any
        // interrupt that uses it can fire (we just enabled the NVIC line
        // but the slave has only just been configured; no bus traffic
        // has been ack'd yet at this point). `uninit_to_initing` enforces
        // single-init at runtime.
        unsafe {
            inst.bbq_state
                .init_rx::<T>(rx_dma, rx_buffer, max_rx_transaction, inst.info)
                .map_err(|_| SetupError::Other)?;
        }

        Ok(inst)
    }
}

// Public API: DMA-only async primitives.
impl<'d> I3c<'d> {
    /// Respond to a controller-initiated read with the contents of `buf`.
    ///
    /// The slave TX FIFO is fed by DMA. The first `buf.len() - 1` bytes are
    /// streamed via `SWDATAB`; the final byte is written to `SWDATABE` to
    /// emit the I3C T-bit / end-of-data marker. Mirrors the pattern used by
    /// the controller's `async_write` (DMA + `MWDATABE` last byte).
    ///
    /// This is the post-address-match read path: the controller has already
    /// matched our dynamic address with `R=1`. For the **IBI-initiated**
    /// path that needs the FIFO pre-loaded *before* the address-match
    /// window, use [`Self::dma_respond_to_read_with_ibi`] instead.
    ///
    /// # Cancellation safety
    ///
    /// Dropping the future disables the DMA request and clears
    /// `SDMACTRL.dmatb`, but bytes already shifted out cannot be recalled.
    pub async fn dma_respond_to_read(&mut self, buf: &[u8]) -> Result<ReadStatus, IOError> {
        if buf.is_empty() {
            return Ok(ReadStatus::Complete(0));
        }

        let dma_done = self.dma_tx_run(buf).await?;
        // If the controller terminated early, surface that as `EarlyStop` so
        // callers can react.
        match self.check_status() {
            Ok(()) => {}
            Err(IOError::Terminated) => return Ok(ReadStatus::EarlyStop(dma_done)),
            Err(e) => return Err(e),
        }
        Ok(ReadStatus::Complete(buf.len()))
    }

    /// Receive a controller-initiated write into `buf` via the
    /// always-on BBQ RX path.
    ///
    /// Awaits a committed bbqueue grant (committed by the BBQ ISR on
    /// either a bus Stop or a DMA major-loop completion), copies up to
    /// `buf.len()` bytes into `buf`, and releases the consumed portion
    /// of the grant. The DMA stays armed for the next transaction; no
    /// per-call DMA setup happens here.
    ///
    /// Returns the number of bytes copied (1..=`buf.len()`). Returns
    /// `Ok(0)` only if `buf` is empty.
    ///
    /// # Cancellation safety
    ///
    /// Dropping the future does not consume bytes from the bbqueue; a
    /// later call will see the still-committed bytes.
    pub async fn dma_respond_to_write(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        if buf.is_empty() {
            return Ok(0);
        }

        // Surface any errwarn that happened since the last call.
        self.check_status()?;

        // SAFETY: A live `I3c` implies the BbqState has been moved
        // through INITED + RXDMA_PRESENT in `new_dma`. The rx_queue
        // is therefore safe for shared access.
        let queue = unsafe { &*self.bbq_state.rx_queue.get() };
        let cons = queue.stream_consumer();
        let rgr = cons.wait_read().await;
        let avail = rgr.len();
        let n = buf.len().min(avail);
        buf[..n].copy_from_slice(&rgr[..n]);

        rgr.release(n);

        // If the IRQ couldn't open the next grant (ring transiently
        // full), pend a STOP to retry now that we've freed space. The
        // re-arm itself MUST happen in the IRQ — calling start_read_transfer
        // from user context races with the IRQ's own use of the bbqueue
        // producer (its `write_in_progress` lock is not re-entrant; an
        // IRQ preempting a user-side grant can permanently jam the
        // producer).
        let post_state = self.bbq_state.state.load(Ordering::Acquire);
        if (post_state & STATE_RXGR_ACTIVE) == 0 {
            self.info.regs().sintset().write(|w| w.set_stop(true));
        }

        // Surface any errwarn (e.g. ORUN from RX overflow) that happened
        // during this write phase so it gets pinned to the right call
        // rather than leaking into a later operation.
        self.check_status()?;

        Ok(n)
    }

    /// Wait until the bus state machine actually reports idle
    /// (`STNOTSTOP == Stopped`). Ignores the sticky `stop`/`start` flags so
    /// stale events from a previous transaction don't satisfy us.
    async fn wait_for_end_of_chain(&self) -> Result<(), IOError> {
        if self.info.regs().sstatus().read().stnotstop() == Stnotstop::Stopped {
            return Ok(());
        }

        self.info
            .wait_cell()
            .wait_for(|| {
                self.info.regs().sintset().write(|w| {
                    w.set_errwarn(true);
                    w.set_stop(true);
                });
                let status = self.info.regs().sstatus().read();
                status.errwarn() || status.stnotstop() == Stnotstop::Stopped
            })
            .await
            .map_err(|_| IOError::Other)?;
        // Clear the now-consumed Stop flag so the next wait isn't satisfied
        // by a stale latched event.
        self.info.regs().sstatus().write(|w| w.set_stop(true));
        Ok(())
    }

    /// Send an In-Band Interrupt (IBI) to the controller.
    ///
    /// Asserts an IBI request on the bus and waits for the controller to
    /// ACK it. The device must be configured with `Config::ibi_capable =
    /// true` (and optionally `Config::ibi_has_payload = true`) for this to
    /// be meaningful.
    ///
    /// If `payload` is non-empty, `payload[0]` is placed in `SCTRL.IBIDATA`
    /// as the mandatory data byte (requires `ibi_has_payload = true` /
    /// BCR\[2\]=1).
    ///
    /// If the future is dropped before completion, `SCTRL.EVENT` is
    /// restored to `NormalMode` to cancel the pending request.
    pub async fn dma_send_ibi(&mut self, payload: &[u8]) -> Result<(), IOError> {
        // Bail out early if the controller has disabled IBIs via DISEC CCC.
        if self.info.regs().sstatus().read().ibidis() == Ibidis::InterruptsDisabled {
            return Err(IOError::IbiDisabled);
        }

        self.info.regs().sctrl().modify(|w| {
            if let Some(&b) = payload.first() {
                w.set_ibidata(b);
            }
            w.set_event(SctrlEvent::Ibi);
        });

        // Ensure EVENT is restored to NormalMode if the future is cancelled.
        let info = self.info;
        let _on_drop = OnDrop::new(|| {
            info.regs().sctrl().modify(|w| w.set_event(SctrlEvent::NormalMode));
        });

        // Wait for the IBI cycle to complete: either the EVENT pulse fires
        // (HW-acked path) or the controller drives Sr after acking the IBI
        // header. Leave `start` latched so the following `listen()` sees it.
        self.info
            .wait_cell()
            .wait_for(|| {
                self.info.regs().sintset().write(|w| {
                    w.set_event(true);
                    w.set_errwarn(true);
                    w.set_start(true);
                });

                let status = self.info.regs().sstatus().read();
                status.errwarn() || status.event() || status.start() == SstatusStart::StartDetected
            })
            .await
            .map_err(|_| IOError::Other)?;

        let status = self.info.regs().sstatus().read();
        self.info.regs().sstatus().write(|w| {
            if status.event() {
                w.set_event(true);
            }
        });

        // Force EVENT back to NormalMode in case HW didn't auto-clear it
        // (some MCXA silicon never pulses the EVENT bit on payloadless
        // IBIs and leaves SCTRL.EVENT=Ibi latched, which blocks the next
        // address ACK).
        self.info.regs().sctrl().modify(|w| w.set_event(SctrlEvent::NormalMode));

        _on_drop.defuse();

        self.check_status()?;

        if status.event() && status.evdet() == Evdet::Nacked {
            return Err(IOError::IbiNacked);
        }

        Ok(())
    }

    /// Combined IBI + read response: pre-load TX DMA, raise IBI, wait for
    /// the controller to clock out the response.
    ///
    /// At I3C-SDR speeds the post-IBI Sr→addr window is too tight (~640 ns)
    /// for the slave software to load the TX FIFO after observing the IBI
    /// ACK. This method arms the DMA TX channel **before** asserting the
    /// IBI so HW already has bytes queued by the time the controller starts
    /// clocking the directed read that follows.
    ///
    /// Sequence:
    /// 1. Arm `tx_dma` against `SWDATAB` for `buf.len() - 1` bytes.
    /// 2. Set `SDMACTRL.dmatb = ENABLE_ONE_FRAME`, enable the DMA request.
    /// 3. Raise `SCTRL.EVENT = Ibi` (with optional MDB).
    /// 4. Wait for DMA completion.
    /// 5. Push the final byte to `SWDATABE` so HW emits the end-of-data
    ///    marker (T-bit) and the controller terminates the read cleanly.
    /// 6. On any exit path, disable the DMA request, clear
    ///    `SDMACTRL.dmatb`, and force `SCTRL.EVENT = NormalMode`.
    ///
    /// `buf` must be non-empty.
    pub async fn dma_respond_to_read_with_ibi(&mut self, buf: &[u8]) -> Result<(), IOError> {
        if buf.is_empty() {
            return Err(IOError::Other);
        }

        if self.info.regs().sstatus().read().ibidis() == Ibidis::InterruptsDisabled {
            return Err(IOError::IbiDisabled);
        }

        // Pre-arm the DMA TX path. Mirrors the controller's `async_write`:
        // DMA streams `len-1` bytes through SWDATAB1; SW writes the last
        // byte to SWDATABE to mark end-of-data.
        let (last, rest) = buf.split_last().unwrap();

        // Cleanup guard for any early exit (cancellation, error).
        let info = self.info;
        let regs_ptr = info.regs();
        let _ibi_drop = OnDrop::new(|| {
            regs_ptr.sctrl().modify(|w| w.set_event(SctrlEvent::NormalMode));
            regs_ptr.sdmactrl().modify(|w| {
                w.set_dmatb(SdmactrlDmatb::NotUsed);
                w.set_dmafb(SdmactrlDmafb::NotUsed);
            });
        });

        if !rest.is_empty() {
            self.dma_tx_arm(rest)?;
        }

        // Make sure the bus is actually idle before raising the IBI.
        // Phase A (`dma_respond_to_write`) is supposed to have waited for
        // end-of-chain, but if the caller skipped that step we'd race the
        // controller's Stop emission and corrupt this IBI.
        self.wait_for_end_of_chain().await?;

        // Clear any stale Stop flag so the post-IBI wait below observes
        // the *new* Stop the controller emits after this IBI, not the
        // pre-IBI bus-idle state.
        self.info.regs().sstatus().write(|w| w.set_stop(true));

        // Snapshot the BBQ IRQ's stop counter. The BBQ IRQ owns the
        // latched STOP flag (it W1Cs it as part of its rotation logic),
        // so neither `wait_for_end_of_chain` (level-triggered on
        // STNOTSTOP, which is already `Stopped` here) nor a direct poll
        // of `SSTATUS.STOP` would reliably catch the post-IBI Stop. The
        // counter is bumped from the IRQ after each W1C, giving us a
        // race-free way to wait for the next bus-end without coupling
        // to BBQ internals.
        let stop_seq_pre = self.bbq_state.stop_seq.load(Ordering::Acquire);

        // Raise the IBI. DMA is already armed and the request line enabled;
        // HW will push bytes into the TX FIFO as soon as TXNOTFULL is true.
        // Raising IBI without first awaiting DMA completion is required for
        // payloads larger than the TX FIFO depth (8 bytes): in that case the
        // first 8 bytes would fill the FIFO and DMA would stall waiting for
        // space — but the controller hasn't started clocking yet, so it
        // never drains. Raising IBI here lets the controller ack and start
        // pulling bytes, which unblocks DMA so it can stream the remainder.
        //
        // Set IBIDATA (MDB byte sent on the IBI) in the SAME write as
        // EVENT=Ibi, mirroring SDK's `I3C_SlaveRequestIBIWithData`. The
        // MDB is sourced from SCTRL.IBIDATA (out-of-band from the TX
        // FIFO); leaving it stale means a bogus MDB on the wire.
        let mdb = *buf.first().unwrap_or(&0);
        self.info.regs().sctrl().modify(|w| {
            w.set_ibidata(mdb);
            w.set_event(SctrlEvent::Ibi);
        });

        // Wait for DMA to drain `rest` into the FIFO. Safe to await here:
        // the controller is now draining the FIFO so DMA always makes
        // progress to completion.
        if !rest.is_empty() {
            poll_fn(|cx| {
                let _ = self.tx_dma.wait_cell().poll_wait(cx);
                if self.tx_dma.is_done() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            cortex_m::asm::dsb();

            self.info
                .regs()
                .sdmactrl()
                .modify(|w| w.set_dmatb(SdmactrlDmatb::NotUsed));
            unsafe {
                self.tx_dma.disable_request();
                self.tx_dma.clear_done();
            }
        }

        // Push the final byte to SWDATABE so HW emits the end-of-data
        // marker (T-bit) and the controller terminates the read cleanly.
        self.wait_tx_space().await?;
        self.info.regs().swdatabe().write(|w| w.set_data(*last));

        // Wait for the controller to complete the IBI handshake +
        // directed read and emit Stop. The BBQ IRQ bumps `stop_seq`
        // after W1C'ing each STOP latch; loop until it advances or an
        // error/warning surfaces.
        self.info
            .wait_cell()
            .wait_for(|| {
                self.info.regs().sintset().write(|w| {
                    w.set_errwarn(true);
                    w.set_stop(true);
                });
                self.info.regs().sstatus().read().errwarn()
                    || self.bbq_state.stop_seq.load(Ordering::Acquire) != stop_seq_pre
            })
            .await
            .map_err(|_| IOError::Other)?;

        // Force EVENT back to NormalMode in case HW didn't pulse.
        self.info.regs().sctrl().modify(|w| w.set_event(SctrlEvent::NormalMode));

        _ibi_drop.defuse();

        // Treat `Terminated` as success: the controller is free to chunk this
        // logical response into multiple wire-level reads (each bounded by
        // RDTERM and separated by Sr or Stop). Every chunk boundary that
        // falls mid-T=1-stream latches SERRWARN.TERM on the target, even
        // though the IP keeps draining the TX FIFO across the Sr. By the
        // time we reach this check, DMA has streamed `rest` into the FIFO
        // and SWDATABE has emitted the final T=0 byte — so the wire-level
        // payload is fully delivered regardless of how many TERM warnings
        // got latched along the way. Mirrors `dma_respond_to_read`, which
        // exposes the same condition as `ReadStatus::EarlyStop`.
        //
        // Genuine controller-side aborts (mid-payload Stop with bytes still
        // pending in DMA) surface earlier as Underrun/UnderrunNack on the
        // `wait_tx_space` / SWDATABE path, not here.
        match self.check_status() {
            Ok(()) | Err(IOError::Terminated) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Diagnostic: measure the TX FIFO depth by pushing bytes until full.
    ///
    /// Flushes the TX FIFO, then writes 0x00 bytes via `SWDATAB` one at a
    /// time, polling `SDATACTRL.TXCOUNT/TXFULL` after each write. Returns
    /// the maximum number of bytes the FIFO holds. Bus must be idle.
    pub fn measure_tx_fifo_depth(&mut self) -> u8 {
        self.flush_fifos();
        let mut count: u8 = 0;
        // Cap at 32 — the PAC's TXCOUNT field is 5 bits.
        while count < 32 {
            let dc = self.info.regs().sdatactrl().read();
            if dc.txfull() == crate::pac::i3c::SdatactrlTxfull::Txisfull {
                break;
            }
            self.info.regs().swdatab().write(|w| w.set_data(0));
            count += 1;
        }
        let final_count = self.info.regs().sdatactrl().read().txcount();
        let _ = final_count;
        self.flush_fifos();
        count
    }

    // ─── DMA helpers ────────────────────────────────────────────────────

    /// Arm the slave TX DMA channel against `SWDATAB1` for `buf.len()` bytes.
    /// Caller is responsible for waiting on completion / cleanup.
    ///
    /// `SWDATAB1` (offset 0x54) is the DMA-friendly alias of `SWDATAB`
    /// (offset 0x30); the NXP I3C IP requires DMA streaming through
    /// `SWDATAB1` rather than `SWDATAB`. Writing the streaming bytes to
    /// `SWDATAB` corrupts the SDR transmission for transfers ≥ 5 bytes
    /// and surfaces as `SPAR` at the target. Matches `fsl_i3c_edma.c`
    /// `I3C_SlavePrepareTxEDMA` which targets `SWDATAB1`.
    fn dma_tx_arm(&mut self, buf: &[u8]) -> Result<(), IOError> {
        let peri_addr = self.info.regs().swdatab1().as_ptr() as *mut u8;

        unsafe {
            self.tx_dma.disable_request();
            self.tx_dma.clear_done();
            self.tx_dma.clear_interrupt();
            self.tx_dma.set_request_source(self.tx_request);
            self.tx_dma
                .setup_write_to_peripheral(buf, peri_addr, false, TransferOptions::COMPLETE_INTERRUPT)?;

            self.info.regs().sdmactrl().modify(|w| {
                w.set_dmatb(SdmactrlDmatb::Enable);
                w.set_dmawidth(SdmactrlDmawidth::Byte0);
            });

            self.tx_dma.enable_request();
        }
        Ok(())
    }

    /// Run a full DMA TX of `buf` (DMA `len-1` bytes via `SWDATAB`, then
    /// SW-write the last byte to `SWDATABE`). Returns the number of bytes
    /// the DMA shipped before the SW end-byte (== `buf.len() - 1`).
    async fn dma_tx_run(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        let (last, rest) = buf.split_last().unwrap();
        let regs_ptr = self.info.regs();
        let _drop = OnDrop::new(|| {
            regs_ptr.sdmactrl().modify(|w| w.set_dmatb(SdmactrlDmatb::NotUsed));
        });

        if !rest.is_empty() {
            self.dma_tx_arm(rest)?;

            poll_fn(|cx| {
                let _ = self.tx_dma.wait_cell().poll_wait(cx);
                if self.tx_dma.is_done() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            cortex_m::asm::dsb();

            self.info
                .regs()
                .sdmactrl()
                .modify(|w| w.set_dmatb(SdmactrlDmatb::NotUsed));
            unsafe {
                self.tx_dma.disable_request();
                self.tx_dma.clear_done();
            }
        }

        self.wait_tx_space().await?;
        self.info.regs().swdatabe().write(|w| w.set_data(*last));

        _drop.defuse();
        Ok(rest.len())
    }

    async fn wait_tx_space(&self) -> Result<(), IOError> {
        if self.info.regs().sstatus().read().txnotfull() == SstatusTxnotfull::NotFull {
            return Ok(());
        }
        self.info
            .wait_cell()
            .wait_for(|| {
                self.info.regs().sintset().write(|w| {
                    w.set_errwarn(true);
                    w.set_txsend(true);
                });
                let status = self.info.regs().sstatus().read();
                status.errwarn() || status.txnotfull() == SstatusTxnotfull::NotFull
            })
            .await
            .map_err(|_| IOError::Other)?;
        // Disarm the level-triggered TX-ready interrupt; it would IRQ-storm
        // at idle.
        self.info.regs().sintclr().write(|w| w.set_txsend(true));
        Ok(())
    }

    #[allow(dead_code)]
    async fn wait_for_stop(&self) -> Result<(), IOError> {
        self.info
            .wait_cell()
            .wait_for(|| {
                self.info.regs().sintset().write(|w| {
                    w.set_errwarn(true);
                    w.set_stop(true);
                });
                let status = self.info.regs().sstatus().read();
                // Only the latched STOP flag — `stnotstop` is a level signal
                // and is true the moment the bus is idle, including *before*
                // an expected transaction has started. Callers that need
                // edge-triggered behavior must clear STOP themselves first.
                status.errwarn() || status.stop()
            })
            .await
            .map_err(|_| IOError::Other)?;
        self.info.regs().sstatus().write(|w| w.set_stop(true));
        Ok(())
    }

    /// Wait for any I3C target event.
    ///
    /// All event sources except TX-ready (STOP, START, MATCHED, HDR_MATCHED,
    /// CCC, CCC_HANDLED, DYNAMIC_ADDRESS_CHANGED, EVENT_STATUS_CHANGED,
    /// RX_PENDING) are armed unconditionally. Errors (`SERRWARN`) are also
    /// armed and surface as `IOError`.
    ///
    /// TX-ready (`TXNOTFULL`) is a level flag (true whenever the TX FIFO has
    /// space) and is gated on `SSTATUS.STREQRD == Busy` — the bit the
    /// hardware sets after the controller has matched our address with `R=1`.
    /// Armed unconditionally it would flood the NVIC at idle and on every
    /// write transaction. Gating it on STREQRD matches the NXP MCUXpresso SDK
    /// pattern (TX-ready dispatched only when the controller is actually
    /// reading us) without the side-bookkeeping the SDK does around
    /// `BUS_START` / buffer-empty / `BUS_STOP`. Once dispatched as
    /// [`Event::TxPending`] the interrupt is disabled until the caller's
    /// [`I3c::dma_respond_to_read`] re-arms it.
    pub async fn listen(&mut self) -> Result<Event, IOError> {
        loop {
            self.info
                .wait_cell()
                .wait_for(|| {
                    // Re-arm every interrupt source except TX-ready. SINTSET is
                    // W1S — bits not written are unchanged.
                    self.info.regs().sintset().write(|w| {
                        w.set_errwarn(true);
                        w.set_start(true);
                        w.set_matched(true);
                        w.set_stop(true);
                        w.set_rxpend(true);
                        w.set_dachg(true);
                        w.set_ccc(true);
                        w.set_ddrmatched(true);
                        w.set_chandled(true);
                        w.set_event(true);
                    });

                    let status = self.info.regs().sstatus().read();
                    if status.errwarn() {
                        return true;
                    }

                    // Arm TX-ready only when the controller is actively reading
                    // us; otherwise it would IRQ-storm during writes / at idle.
                    if status.streqrd() == Streqrd::Busy {
                        self.info.regs().sintset().write(|w| w.set_txsend(true));
                    }

                    status.stop()
                        || status.start() == SstatusStart::StartDetected
                        || status.matched()
                        || status.hdrmatch()
                        || status.ccc()
                        || status.chandled()
                        || status.dachg()
                        || status.event()
                        || status.rx_pend()
                        || self.bbq_state.has_pending()
                        || (status.txnotfull() == SstatusTxnotfull::NotFull && status.streqrd() == Streqrd::Busy)
                })
                .await
                .map_err(|_| IOError::Other)?;

            // Errors are surfaced first — they are W1C'd inside check_status().
            self.check_status().inspect_err(|_e| {
                #[cfg(feature = "defmt")]
                defmt::error!("[tgt listen] check_status err {:?}", _e);
            })?;

            let status = self.info.regs().sstatus().read();

            // Pick the highest-priority pending event and acknowledge it (W1C on
            // the matching SSTATUS bit). `txnotfull` and `rx_pend` are FIFO-state
            // flags (not W1C) — cleared implicitly by reading/writing the FIFO.
            //
            // Priority order mirrors the NXP SDK `I3C_SlaveTransferHandleIRQ`
            // dispatch sequence (fsl_i3c.c): BusStart → EventSent → ReceivedCCC
            // → Matched → TxReady → RxReady → BusStop. The key invariant is that
            // STOP is the *last* event reported — any RX bytes / CCC code / match
            // pending from this chain must be reported to the caller (and drained
            // by them) BEFORE we W1C the stop bit and flush the FIFOs. Otherwise a
            // full transaction that completes between two `listen()` calls would
            // be observed as Stop-only and the data lost.
            if status.start() == SstatusStart::StartDetected {
                self.info
                    .regs()
                    .sstatus()
                    .write(|w| w.set_start(SstatusStart::StartDetected));
                // NOTE: NXP SDK `I3C_SlaveTransferHandleBusStart` flushes the
                // TX FIFO here. We deliberately do *not* — for an IBI+read
                // sequence the TX FIFO was pre-loaded by
                // `dma_respond_to_read_with_ibi` and STREQRD does not transition
                // to Busy until *after* this Start fires. Flushing would
                // unconditionally destroy the pre-loaded response and cause an
                // UnderrunNack on the controller's directed read.
                return Ok(Event::Start);
            }
            if status.ccc() {
                self.info.regs().sstatus().write(|w| w.set_ccc(true));
                return Ok(Event::Ccc);
            }
            if status.matched() {
                self.info.regs().sstatus().write(|w| w.set_matched(true));
                // Match SDK behaviour: when the address match is for a read
                // (STREQRD=Busy), fire the transmit event directly from the match
                // dispatch instead of waiting another `listen()` iteration. The
                // controller starts clocking SCL almost immediately after the
                // header ACK, so any extra round-trip through user code risks a
                // TX-FIFO underrun before the first byte is queued.
                if status.streqrd() == Streqrd::Busy {
                    self.info.regs().sintclr().write(|w| w.set_txsend(true));
                    return Ok(Event::TxPending);
                }
                return Ok(Event::Matched);
            }
            if status.hdrmatch() {
                self.info.regs().sstatus().write(|w| w.set_hdrmatch(true));
                return Ok(Event::HdrMatched);
            }
            if status.txnotfull() == SstatusTxnotfull::NotFull && status.streqrd() == Streqrd::Busy {
                // Fallback path: STREQRD became Busy after the match was already
                // W1C'd (e.g. Sr→read on an already-matched DA). Disable TX-ready
                // until the caller re-arms it via `async_respond_to_read`;
                // TXNOTFULL is a level flag and would re-wake us otherwise.
                self.info.regs().sintclr().write(|w| w.set_txsend(true));
                return Ok(Event::TxPending);
            }
            if status.rx_pend() || self.bbq_state.has_pending() {
                return Ok(Event::RxPending);
            }
            if status.chandled() {
                self.info.regs().sstatus().write(|w| w.set_chandled(true));
                return Ok(Event::CccHandled);
            }
            if status.dachg() {
                self.info.regs().sstatus().write(|w| w.set_dachg(true));
                return Ok(Event::DynamicAddressChanged);
            }
            if status.event() {
                self.info.regs().sstatus().write(|w| w.set_event(true));
                return Ok(Event::EventStatusChanged);
            }
            if status.stop() {
                self.info.regs().sstatus().write(|w| w.set_stop(true));
                // Disable TX-ready on end-of-chain. Defensive: the
                // `async_respond_to_read` OnDrop normally handles this, but if
                // the read never started (e.g. controller aborted) leaving it
                // armed would IRQ-storm.
                self.info.regs().sintclr().write(|w| w.set_txsend(true));
                // Match NXP SDK `I3C_SlaveTransferHandleBusStop` (fsl_i3c.c): flush
                // BOTH FIFOs at end-of-chain. Safe here because any RX bytes that
                // arrived during this chain were already reported via prior
                // `RxPending` / `Ccc` events above.
                self.flush_fifos();
                return Ok(Event::Stop);
            }

            // Spurious wake: a wake condition was true when the closure ran
            // (e.g. STOP latched), but an IRQ consumed the responsible flag
            // (e.g. BBQ rotation W1Cs STOP) before this dispatch could read
            // it. Re-await rather than returning a spurious `Other`.
        }
    }
}

impl<'d> Drop for I3c<'d> {
    fn drop(&mut self) {
        self._scl.set_as_disabled();
        self._sda.set_as_disabled();
    }
}

/// I3C interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let info = T::info();
        let regs = info.regs();
        let bbq = T::bbq_state();
        let s_status = regs.sintmasked().read();

        // ===== BBQ RX path =====
        //
        // The BBQ owns the latched STOP flag. On either a bus Stop or
        // a DMA-major-loop completion (signaled by `dma_rx_complete_cb`
        // via the `STATE_RXDMA_COMPLETE` bit), we commit the bytes that
        // landed and immediately re-arm DMA into the next grant.
        //
        // We W1C SSTATUS.STOP but deliberately keep SINTSET.STOP armed
        // so the next transaction-end fires us again. `wait_for_end_of_chain`
        // and friends rely on the level-triggered `STNOTSTOP` instead of
        // the latched STOP and remain correct.
        let pre = bbq
            .state
            .fetch_and(!STATE_RXDMA_COMPLETE, Ordering::AcqRel);
        let rx_present = (pre & STATE_RXDMA_PRESENT) != 0;
        let dma_complete = (pre & STATE_RXDMA_COMPLETE) != 0;
        let stop_fired = s_status.stop();
        let mut handled_stop = false;
        if rx_present && (stop_fired || dma_complete) {
            let rxgr_active = (pre & STATE_RXGR_ACTIVE) != 0;

            // Only tear down + rebuild the RX DMA grant when there is
            // actual work to commit: either the major loop completed
            // (`dma_complete`), or a bus Stop fired AND we observed at
            // least one byte landing in the current grant. The slave
            // bus also emits a Stop after our outbound IBI directed
            // read (no RX bytes received during that phase); if we
            // finalized on every such Stop we'd needlessly disable DMA,
            // open a fresh grant, and reprogram TCD — leaving a small
            // window where the controller's next write can push bytes
            // into an unarmed RX FIFO and latch SERRWARN.ORUN.
            //
            // SAFETY: ISR context; rxdma + rxgr owned by BbqState while
            // RXDMA_PRESENT + RXGR_ACTIVE are set.
            let rx_has_data = if rxgr_active {
                unsafe {
                    let rxdma = &*bbq.rxdma.get();
                    let rxgr = &*bbq.rxgr.get();
                    rxdma.daddr() as usize != rxgr.as_ptr() as usize
                }
            } else {
                false
            };

            if rxgr_active && (dma_complete || (stop_fired && rx_has_data)) {
                // SAFETY: rxdma + rxgr present, ISR context.
                unsafe { bbq.finalize_read(info) };
            }
            let rx_idle = (bbq.state.load(Ordering::Acquire) & STATE_RXGR_ACTIVE) == 0;
            if rx_idle {
                // SAFETY: rxdma present, no active grant, ISR context.
                unsafe { bbq.start_read_transfer(info) };
            }
            if stop_fired {
                regs.sstatus().write(|w| w.set_stop(true));
                regs.sintset().write(|w| w.set_stop(true));
                bbq.stop_seq.fetch_add(1, Ordering::AcqRel);
                handled_stop = true;
            }
        }

        // Compute the remaining set of enables that fired (excluding the
        // STOP we just consumed above).
        let mut s_to_clear = s_status;
        if handled_stop {
            s_to_clear.set_stop(false);
        }

        if s_to_clear.0 != 0 {
            // The TX path is now DMA-driven, so the IRQ has no per-Start
            // refill work to do. Just W1C the source enables that fired
            // and wake the task. We deliberately leave `start` latched
            // so callers observing `SSTATUS.START` (e.g. `dma_send_ibi`)
            // can act on it; `listen()` W1Cs it explicitly.
            regs.sintclr().write(|w| w.0 = s_to_clear.0);
            info.wait_cell().wake();
            return;
        }

        if handled_stop {
            // BBQ handled this IRQ end-to-end. Still wake potential
            // waiters of `STNOTSTOP=Stopped` (e.g. `wait_for_end_of_chain`).
            info.wait_cell().wake();
            return;
        }

        // Spurious: NVIC asserted but no S-side masked event. Could be a
        // latched NVIC pending bit, an M-side enable, or some hardware
        // edge that already cleared by the time we read. Clear all enables
        // on both sides and unpend NVIC; `listen()` will re-arm what it
        // needs on its next poll. Do NOT touch SERRWARN here — if there's
        // a real error flag the listen closure must still observe it.
        let m_enables = regs.mintset().read();
        let s_enables = regs.sintset().read();
        if m_enables.0 != 0 {
            regs.mintclr().write(|w| w.0 = m_enables.0);
        }
        if s_enables.0 != 0 {
            // Keep STOP armed for the BBQ path.
            let mut e = s_enables;
            if rx_present {
                e.set_stop(false);
            }
            if e.0 != 0 {
                regs.sintclr().write(|w| w.0 = e.0);
            }
        }

        T::Interrupt::unpend();

        info.wait_cell().wake();
    }
}

// =====================================================================
// BBQ-backed RX state
// =====================================================================
//
// Persistent DMA from SRDATAB into a per-instance bqueue::BBQueue,
// with commit-and-rearm done in on_interrupt on every bus Stop. See
// mbassy-mcxa/src/lpuart/bbq.rs for the original pattern; this is a
// smaller, RX-only variant folded into 	arget.rs.

// A wrapper type representing a &'static mut [u8] buffer.
struct Container {
    ptr: NonNull<u8>,
    len: usize,
}

impl Storage for Container {
    /// SAFETY: The length and ptr destination of the Container are never changed.
    unsafe fn ptr_len(&self) -> (NonNull<u8>, usize) {
        (self.ptr, self.len)
    }
}

impl From<&'static mut [u8]> for Container {
    fn from(value: &'static mut [u8]) -> Self {
        Self {
            len: value.len(),
            // SAFETY: The input slice is guaranteed to contain a non-null value.
            ptr: unsafe { NonNull::new_unchecked(value.as_mut_ptr()) },
        }
    }
}

pub(crate) const STATE_UNINIT: u32 = 0b0000_0000;
pub(crate) const STATE_INITING: u32 = 0b0000_0001;
pub(crate) const STATE_INITED: u32 = 0b0000_0011;
pub(crate) const STATE_RXGR_ACTIVE: u32 = 0b0000_0100;
pub(crate) const STATE_RXDMA_PRESENT: u32 = 0b0000_1000;
pub(crate) const STATE_RXDMA_COMPLETE: u32 = 0b0001_0000;

/// Per-instance BBQ state for the I3C target RX path.
///
/// Constructed at compile time via BbqState::new() (one static per
/// I3C instance, allocated in the impl_i3c_instance! macro). The
/// instance owns the RX DMA channel and the bbqueue storage for the
/// lifetime of the I3c<'_> driver; both are moved in by
/// init_rx during I3c::new_dma.
pub struct BbqState {
    /// State flags. See STATE_* constants.
    pub(crate) state: AtomicU32,

    /// Monotonic counter incremented by `on_interrupt` whenever the
    /// hardware-latched `SSTATUS.STOP` bit is observed and W1C'd.
    /// `dma_respond_to_read_with_ibi` snapshots this before raising
    /// IBI and waits for it to advance, since the BBQ IRQ otherwise
    /// consumes the STOP latch before user-space can see it.
    pub(crate) stop_seq: core::sync::atomic::AtomicU8,

    /// The RX bbqueue. Only valid when STATE_RXDMA_PRESENT is set.
    rx_queue: GroundedCell<BBQueue<Container, AtomicCoord, MaiNotSpsc>>,
    /// The active RX grant (DMA write target). Only valid when
    /// STATE_RXDMA_PRESENT + STATE_RXGR_ACTIVE are both set.
    rxgr: GroundedCell<StreamGrantW<&'static BBQueue<Container, AtomicCoord, MaiNotSpsc>>>,
    /// The RX DMA channel. Only valid when STATE_RXDMA_PRESENT is set.
    rxdma: GroundedCell<DmaChannel<'static>>,
    /// The RX DMA request number. Only valid when STATE_RXDMA_PRESENT is set.
    rxdma_num: AtomicU8,

    /// Size in bytes of every RX grant opened by `start_read_transfer`.
    /// Set once in `init_rx` from the user-supplied `max_rx_transaction`
    /// parameter. Using `grant_exact(rx_grant_size)` (instead of
    /// `grant_max_remaining`) guarantees each grant is a single
    /// contiguous DMA region, so a single I3C controller transaction
    /// cannot be split across a BBQ ring wrap. The user must ensure
    /// `rx_buffer.len() >= 2 * rx_grant_size`.
    pub(crate) rx_grant_size: AtomicUsize,
}

// SAFETY: All shared mutable access is gated by atomic state bits and
// ISR-only access discipline.
unsafe impl Sync for BbqState {}

impl BbqState {
    pub(crate) const fn new() -> Self {
        Self {
            state: AtomicU32::new(0),
            stop_seq: core::sync::atomic::AtomicU8::new(0),
            rx_queue: GroundedCell::uninit(),
            rxgr: GroundedCell::uninit(),
            rxdma: GroundedCell::uninit(),
            rxdma_num: AtomicU8::new(0),
            rx_grant_size: AtomicUsize::new(0),
        }
    }

    /// Move from UNINIT to INITING, returning an error if the state
    /// was anything other than UNINIT (i.e. some other I3c is
    /// already using this peripheral).
    fn uninit_to_initing(&'static self) -> Result<(), ()> {
        self.state
            .compare_exchange(STATE_UNINIT, STATE_INITING, Ordering::AcqRel, Ordering::Acquire)
            .map(drop)
            .map_err(|_| ())
    }

    /// Install DMA + buffer, open the first grant, program DMA, and
    /// arm the bus STOP interrupt as the BBQ rotation trigger.
    ///
    /// ## SAFETY
    ///
    /// Caller must hold exclusive access to this BbqState (e.g. by
    /// being the sole constructor of I3c<T> for the corresponding
    /// peripheral). The I3C peripheral must already be configured.
    pub(crate) unsafe fn init_rx<T: Instance>(
        &'static self,
        rxdma: DmaChannel<'static>,
        rx_buffer: &'static mut [u8],
        max_rx_transaction: usize,
        info: &'static Info,
    ) -> Result<(), ()> {
        self.uninit_to_initing()?;

        // Each RX grant is exactly `max_rx_transaction` bytes (one whole
        // I3C controller-write transaction). Ring must hold at least two
        // so the IRQ can re-arm into a fresh contiguous slot while the
        // consumer is still draining the previous one.
        if max_rx_transaction == 0 || rx_buffer.len() < 2 * max_rx_transaction {
            return Err(());
        }
        self.rx_grant_size.store(max_rx_transaction, Ordering::Release);

        // Preload the request source once; subsequent re-arms only need
        // to clear flags + re-setup the destination.
        let req = T::RX_DMA_REQUEST;
        unsafe {
            rxdma.disable_request();
            rxdma.clear_done();
            rxdma.clear_interrupt();
            rxdma.set_request_source(req);
        }

        // SAFETY: We're in INITING; no IRQ touches these fields.
        let cont = Container::from(rx_buffer);
        unsafe {
            self.rx_queue.get().write(BBQueue::new_with_storage(cont));
            self.rxdma.get().write(rxdma);
            self.rxdma_num.store(req.number(), Ordering::Release);
        }

        // Publish INITED + RXDMA_PRESENT *before* opening the first
        // grant so that start_read_transfer sees the right state.
        self.state
            .store(STATE_INITED | STATE_RXDMA_PRESENT, Ordering::Release);

        // Open the first grant + program DMA + enable SDMACTRL.dmafb.
        // SAFETY: just published RXDMA_PRESENT and we hold exclusive
        // access; no grant is active.
        let started = unsafe { self.start_read_transfer(info) };
        if !started {
            return Err(());
        }

        // Arm bus STOP as the persistent BBQ rotation trigger.
        info.regs().sintset().write(|w| w.set_stop(true));

        Ok(())
    }

    /// Returns true if there are committed bytes the consumer side has
    /// not yet drained. Used by `listen()` to surface `Event::RxPending`
    /// since with BBQ the FIFO is continuously drained and the
    /// `SSTATUS.RX_PEND` flag never latches.
    fn has_pending(&'static self) -> bool {
        if (self.state.load(Ordering::Acquire) & STATE_RXDMA_PRESENT) == 0 {
            return false;
        }
        // SAFETY: RXDMA_PRESENT implies rx_queue was initialized.
        let queue = unsafe { &*self.rx_queue.get() };
        queue.stream_consumer().read().is_ok()
    }

    /// Close the active RX grant, committing the bytes DMA wrote.
    ///
    /// ## SAFETY
    ///
    /// * The driver must be initialized (STATE_RXDMA_PRESENT)
    /// * An RX grant must be active (STATE_RXGR_ACTIVE)
    /// * Must be called from ISR context (or with exclusive access)
    unsafe fn finalize_read(&'static self, info: &'static Info) {
        unsafe {
            let rxgr = self.rxgr.get().read();
            let rxdma = &mut *self.rxdma.get();

            // Disable the DMA request — DMA may not have finished moving
            // every FIFO byte (the bus Stop fires when SCL/SDA quiesce,
            // but a few bytes may still be queued in the RX FIFO).
            info.regs().sdmactrl().modify(|w| w.set_dmafb(SdmactrlDmafb::NotUsed));
            rxdma.disable_request();

            // Drain residual FIFO bytes. Bounded by FIFO depth at AHB
            // clock — microseconds. Mirrors the spin in the previous
            // per-call dma_rx_run path.
            let deadline = 2_000u32;
            let mut spins = 0u32;
            while !rxdma.is_done() && info.regs().sstatus().read().rx_pend() && spins < deadline {
                spins = spins.wrapping_add(1);
                cortex_m::asm::nop();
            }

            rxdma.clear_done();
            fence(Ordering::AcqRel);

            // Bytes-written = (current DMA destination addr) - (grant base addr),
            // clamped to grant capacity (in case the major loop completed
            // and CITER was reloaded from BITER).
            let daddr = rxdma.daddr() as usize;
            let sstrt = rxgr.as_ptr() as usize;
            let ttl = daddr.wrapping_sub(sstrt).min(rxgr.len());

            rxgr.commit(ttl);
        }
        self.state.fetch_and(!STATE_RXGR_ACTIVE, Ordering::AcqRel);
    }

    /// Open a new RX grant and program DMA into it. Returns alse
    /// if no grant could be obtained (ring full — consumer is behind).
    ///
    /// ## SAFETY
    ///
    /// * The driver must be initialized (STATE_RXDMA_PRESENT)
    /// * No RX grant must currently be active
    /// * Must be called from ISR context (or with exclusive access)
    unsafe fn start_read_transfer(&'static self, info: &'static Info) -> bool {
        let rx_queue = unsafe { &*self.rx_queue.get() };
        let prod = rx_queue.stream_producer();
        let grant_size = self.rx_grant_size.load(Ordering::Acquire);
        let mut wgr = match prod.grant_exact(grant_size) {
            Ok(g) => g,
            Err(_) => {
                // Ring is full / no contiguous space — consumer hasn't
                // caught up. The next dma_respond_to_write will pend us
                // once it releases bytes.
                return false;
            }
        };

        unsafe {
            let rxdma = &mut *self.rxdma.get();
            rxdma.disable_request();
            rxdma.clear_done();
            rxdma.clear_interrupt();

            let peri_addr = info.regs().srdatab().as_ptr() as *const u8;
            if rxdma
                .setup_read_from_peripheral(peri_addr, &mut wgr[..], false, TransferOptions::COMPLETE_INTERRUPT)
                .is_err()
            {
                return false;
            }

            info.regs().sdmactrl().modify(|w| {
                w.set_dmafb(SdmactrlDmafb::Enable);
                w.set_dmawidth(SdmactrlDmawidth::Byte0);
            });
            rxdma.enable_request();

            self.rxgr.get().write(wgr);
        }

        self.state.fetch_or(STATE_RXGR_ACTIVE, Ordering::AcqRel);

        true
    }
}
