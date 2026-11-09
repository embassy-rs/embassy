//! Write-enable sequencing for documented security-sensitive register groups.
//!
//! The platform must provide running system/bus clocks and permitted security
//! and privilege access. An unauthorized MMIO access can fault; this driver does
//! not configure access policy. Like the NXP SDK examples, construction checks
//! the controller lock and resets the FSM before use. This closes inherited
//! grants, resets the sequence controls and WRITE_INDEX, and clears FSM error
//! evidence. Clocks, interrupt controls and persistent locks are not changed.
//!
//! Enabling a group only asserts its write-enable output: it neither owns nor
//! writes the protected registers, and does not authorize changing them. Their
//! drivers/platform remain responsible for access, ownership and safety. Opening
//! a group also permits other authorized bus masters to write it.
//!
//! Only the documented four-step FSM is supported. Invalid sequencing can enter
//! WR_DISABLED, which asserts **all protected reset outputs**, including the
//! debug-feature reset. Unexpected state is reported without automatic recovery.
//! There are no target-reset, persistent-lock, raw-index or raw-sequence APIs.
//! Dropping this handle does not close an open write-enable window. Explicitly
//! call [`Glikey::disable_write`]; cleanup must not depend on destructors.
//!
//! Targets follow A2 RM Rev. 2 section 53.1.2, A2 SRM Rev. 2 table 84, and
//! A5 RM Rev. 1 section 66.1.2. `VERSION.INDEX_CONFIG` is deliberately not used
//! as a bound: its count wording conflicts with the documented index-15 mapping.

use crate::pac::glikey::SfrLock;
use crate::{Peri, pac, peripherals};

struct Info {
    regs: pac::glikey::Glikey,
}

static INFO: Info = Info { regs: pac::GLIKEY0 };

/// A documented write-enable consumer on the selected device.
///
/// Reserved/unassigned indices cannot be constructed. A group can contain more
/// than one protected register; this is not ownership of those registers.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Target {
    /// SECCON test-access controls, including the one-time-write control.
    #[cfg(feature = "_mcxa5xx")]
    TestAccess = 0,
    /// SYSCON/SECCON DEBUG_FEATURES_DP.
    DebugFeatures = 1,
    /// SYSCON/SECCON SRAM_XEN_DP (SRAM execution permissions).
    SramExecute = 2,
    /// SYSCON/SECCON MSFCFG (flash-access configuration).
    #[cfg(any(
        feature = "_mcxa5xx",
        feature = "mcxa255",
        feature = "mcxa256",
        feature = "mcxa265",
        feature = "mcxa266"
    ))]
    FlashAccess = 3,
    /// SECCON BOOT_LOCKOUT.
    #[cfg(feature = "_mcxa5xx")]
    BootLockout = 4,
    /// SECCON BOOT_LOCKOUT_ADDR_DP.
    #[cfg(feature = "_mcxa5xx")]
    BootLockoutAddress = 5,
    /// SYSCON MISC Phantom, as named in the device control list.
    #[cfg(feature = "_mcxa5xx")]
    MiscPhantom = 6,
    /// SECCON SYSCON_UDF_CTRL_REG.
    #[cfg(feature = "_mcxa5xx")]
    UdfControl = 7,
    /// Secure AHB Control IDAU All Non-Secure register.
    #[cfg(feature = "_mcxa5xx")]
    NonSecureAccess = 14,
    /// All MBC registers.
    MemoryBlockChecker = 15,
}

/// Observed finite-state-machine state, not a proof of effective protection.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    /// Initial state; no write-enable output asserted.
    Init,
    /// First sequence step.
    Step1,
    /// Second sequence step.
    Step2,
    /// Third sequence step.
    Step3,
    /// Fourth sequence step.
    Step4,
    /// Selected group's write-enable output asserted.
    WriteEnabled,
    /// Selected index is locked until hardware reset.
    Locked,
    /// Selected group's reset output asserted.
    TargetReset,
    /// Error state; all protected reset outputs asserted.
    WriteDisabled,
    /// Unrecognized hardware encoding, retained for diagnosis.
    Unknown(u16),
}

impl State {
    fn decode(bits: u16) -> Self {
        match bits {
            0x16 => Self::Init,
            0x2c => Self::Step1,
            0x58 => Self::Step2,
            0xb0 => Self::Step3,
            0x160 => Self::Step4,
            0x1802 => Self::WriteEnabled,
            0xc01 => Self::Locked,
            0x1005 => Self::TargetReset,
            0x0b => Self::WriteDisabled,
            other => Self::Unknown(other),
        }
    }
}

/// Independent hardware error flags. More than one can be asserted at once.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Errors {
    /// FSM error; only software/hardware reset clears it.
    pub fsm: bool,
    /// Write index out of bounds.
    pub write_index: bool,
    /// Read index out of bounds.
    pub read_index: bool,
}

impl Errors {
    /// Whether any error was sampled.
    pub const fn any(self) -> bool {
        self.fsm || self.write_index || self.read_index
    }
}

/// A nonconsuming, single-register status snapshot.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status {
    /// Observed FSM state.
    pub state: State,
    /// All hardware error flags.
    pub errors: Errors,
    /// Interrupt status, independent of interrupt enable.
    pub interrupt_pending: bool,
}

/// Controller initialization failure.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// The named access clock is gated or the controller is held in reset.
    ClockNotReady,
    /// Hardware reports the unsupported eight-step FSM.
    UnsupportedFsm,
    /// Controller registers are locked, or the lock encoding is not unlocked.
    /// No reset is issued in this case.
    ControllerLocked,
    /// Hardware error flags remain after the FSM reset.
    Hardware(Errors),
    /// The FSM reset did not produce INIT or the documented LOCKED state.
    UnexpectedState(State),
}

/// Controller operation failure. Hardware evidence is never automatically cleared.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Controller registers are locked, or the lock encoding is not unlocked.
    ControllerLocked,
    /// The selected target is already persistently locked.
    IndexLocked,
    /// Enabling requires INIT or LOCKED; rejected before any register writes.
    NotIdle,
    /// Closing requires INIT or WR_ENABLED; rejected without writes.
    NotWriteEnabled,
    /// Hardware error flags, including simultaneous errors.
    Hardware(Errors),
    /// A sequence write did not produce the expected state. No recovery attempted.
    UnexpectedState(State),
}

/// Exclusive owner of the GLIKEY sequence controller, not its protected targets.
pub struct Glikey<'d> {
    _peri: Peri<'d, peripherals::GLIKEY0>,
    info: &'static Info,
}

impl<'d> Glikey<'d> {
    /// Checks the controller lock and resets the FSM, as in the NXP SDK examples.
    ///
    /// Platform prerequisites from the module documentation apply. On A2 there
    /// is no named GLIKEY MRCC gate; live system/bus clocks are a prerequisite.
    /// On A5 the named gate and reset release are checked before GLIKEY access.
    /// The SDK's `GLIKEY_IsLocked` / `GLIKEY_SyncReset` sequence is used on both
    /// families: SFT_RST is written once, not polled (it always reads as zero).
    ///
    /// This deliberately discards inherited FSM state and FSM error evidence,
    /// including any open write-enable grant. It resets WRITE_INDEX to zero and
    /// the sequence controls to their defaults, but preserves READ_INDEX,
    /// interrupt controls, SFR_LOCK and all persistent index locks. It neither
    /// writes protected registers nor pulses the MRCC peripheral reset.
    ///
    /// The resulting state can be INIT or LOCKED because index locks survive
    /// software reset. [`Self::enable_write`] selects an unlocked target to leave
    /// LOCKED, as in the SDK's `GLIKEY_StartEnable`. Other post-reset states or
    /// errors are reported without another reset or recovery attempt.
    pub fn new(peri: Peri<'d, peripherals::GLIKEY0>) -> Result<Self, SetupError> {
        critical_section::with(|_| {
            if !crate::clocks::glikey_access_ready() {
                return Err(SetupError::ClockNotReady);
            }
            let this = Self {
                _peri: peri,
                info: &INFO,
            };
            if this.info.regs.version().read().fsm_config() {
                return Err(SetupError::UnsupportedFsm);
            }
            if this.info.regs.ctrl_1().read().sfr_lock() != SfrLock::Unlocked {
                return Err(SetupError::ControllerLocked);
            }
            this.info.regs.ctrl_0().modify(|w| w.set_sft_rst(true));
            cortex_m::asm::dsb();
            let status = this.status();
            if status.errors.any() {
                return Err(SetupError::Hardware(status.errors));
            }
            if !matches!(status.state, State::Init | State::Locked) {
                return Err(SetupError::UnexpectedState(status.state));
            }
            Ok(this)
        })
    }

    /// Samples state and all error flags without clearing evidence.
    pub fn status(&self) -> Status {
        let status = self.info.regs.status().read();
        let errors = status.error_status();
        Status {
            state: State::decode(status.fsm_state()),
            errors: Errors {
                fsm: errors & 1 != 0,
                write_index: errors & 2 != 0,
                read_index: errors & 4 != 0,
            },
            interrupt_pending: status.int_status(),
        }
    }

    /// Selects READ_INDEX and reads its lock status, preserving other controls.
    ///
    /// Existing errors cause rejection before writes. This changes neither
    /// WRITE_INDEX nor the FSM sequence values. No locked index is unlocked.
    pub fn is_index_locked(&mut self, target: Target) -> Result<bool, Error> {
        critical_section::with(|_| self.read_index_lock(target))
    }

    /// Opens a documented group's write-enable window from INIT or LOCKED.
    ///
    /// Uses A2 figure 241 / A5 figure 293, checking each transition. Interrupts
    /// are masked only during this finite sequence, not throughout the open
    /// window. No protected register is written. On failure, stop and inspect
    /// status: there is no automatic reset, rollback or further sequence write.
    /// A LOCKED state left by initialization is moved to INIT by selecting the
    /// requested unlocked target before beginning its enable sequence. This does
    /// not clear or bypass any persistent index lock.
    pub fn enable_write(&mut self, target: Target) -> Result<(), Error> {
        critical_section::with(|_| {
            self.check_unlocked()?;
            let status = self.checked_status()?;
            if !matches!(status.state, State::Init | State::Locked) {
                return Err(Error::NotIdle);
            }
            if self.read_index_lock(target)? {
                return Err(Error::IndexLocked);
            }

            if status.state == State::Locked {
                self.info.regs.ctrl_0().modify(|w| {
                    w.set_write_index(target as u8);
                    w.set_wr_en_0(2);
                });
                self.expect_state(State::Init)?;
            }

            self.info.regs.ctrl_0().modify(|w| {
                w.set_write_index(target as u8);
                w.set_wr_en_0(1);
            });
            self.expect_state(State::Step1)?;
            self.info.regs.ctrl_1().modify(|w| w.set_wr_en_1(1));
            self.expect_state(State::Step2)?;
            self.info.regs.ctrl_0().modify(|w| w.set_wr_en_0(2));
            self.expect_state(State::Step3)?;
            self.info.regs.ctrl_1().modify(|w| w.set_wr_en_1(0));
            self.expect_state(State::Step4)?;
            self.info.regs.ctrl_0().modify(|w| w.set_wr_en_0(0));
            self.expect_state(State::WriteEnabled)
        })
    }

    /// Closes an open write-enable window without locking or resetting anything.
    ///
    /// INIT is an idempotent no-op. Other states are rejected without writes.
    /// This does not undo protected target writes or clear interrupt/error status.
    pub fn disable_write(&mut self) -> Result<(), Error> {
        critical_section::with(|_| {
            self.check_unlocked()?;
            match self.checked_status()?.state {
                State::Init => Ok(()),
                State::WriteEnabled => {
                    self.info.regs.ctrl_0().modify(|w| w.set_wr_en_0(2));
                    self.expect_state(State::Init)
                }
                _ => Err(Error::NotWriteEnabled),
            }
        })
    }

    fn check_unlocked(&self) -> Result<(), Error> {
        if self.info.regs.ctrl_1().read().sfr_lock() != SfrLock::Unlocked {
            return Err(Error::ControllerLocked);
        }
        Ok(())
    }

    fn checked_status(&self) -> Result<Status, Error> {
        let status = self.status();
        if status.errors.any() {
            return Err(Error::Hardware(status.errors));
        }
        Ok(status)
    }

    fn read_index_lock(&self, target: Target) -> Result<bool, Error> {
        self.check_unlocked()?;
        self.checked_status()?;
        self.info.regs.ctrl_1().modify(|w| w.set_read_index(target as u8));
        cortex_m::asm::dsb();
        self.checked_status()?;
        Ok(self.info.regs.status().read().lock_status())
    }

    fn expect_state(&self, expected: State) -> Result<(), Error> {
        cortex_m::asm::dsb();
        let status = self.checked_status()?;
        if status.state != expected {
            return Err(Error::UnexpectedState(status.state));
        }
        Ok(())
    }
}
