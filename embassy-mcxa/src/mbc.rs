//! Main-flash access policy configuration with the Memory Block Checker.
//!
//! The application chooses shared policy slots and read/write/execute bits;
//! this driver never allocates slots or silently changes unrelated permissions.
//! [`Mbc::new`] preserves inherited configuration. [`Mbc::new_with_policy`] and
//! [`Mbc::apply_policy`] temporarily borrow an existing [`Glikey`], explicitly
//! open index 15, program the policy, verify readback, and close the grant.
//! Neither the policy description nor GLIKEY is retained.
//!
//! # Policy values
//!
//! ```
//! use embassy_mcxa::mbc::{Permission, PermissionSet, Policy, RegionConfig, SlotConfig, WritableSlot};
//!
//! const POLICY: Policy<'static> = Policy {
//!     slots: &[SlotConfig {
//!         slot: WritableSlot::Slot5,
//!         permissions: PermissionSet::from_array([Permission::Read, Permission::Write]),
//!     }],
//!     regions: &[RegionConfig {
//!         // Illustrative offsets only: reserve this region in your linker script.
//!         range: 0x20000..0x24000,
//!         slot: WritableSlot::Slot5,
//!     }],
//! };
//! // At runtime, `Permission::Read | Permission::Write` also constructs a set.
//! ```
//!
//! Region boundaries are byte offsets from main-flash start, aligned to MBC
//! blocks (8 KiB on A2, 16 KiB on A5), not erase sectors. A changed slot must
//! have no IFR consumers; all its current main-flash consumers must be explicitly
//! covered and retain that slot in this operation. There is no automatic migration.
//!
//! # Safety and failure
//!
//! Running system/bus clocks and permitted MMIO security/privilege access are
//! prerequisites. Unauthorized access can fault, not return an access error.
//! No clock/reset, flash-content, IFR-content, or persistent security change is
//! performed. Exclude competing register writers, including other bus masters.
//!
//! Mutation is unsafe because revoking flash reads can invalidate existing Rust
//! references. Keep required memory accessible through every intermediate and
//! failure state, including policy slices and the driver's own continuation.
//! Separately account for code, vectors, constants, interrupt/fault/panic paths,
//! ROM services, flash algorithms, debugger access, cache/prefetch and recovery.
//! The driver does not relocate execution or quiesce the whole system.
//!
//! Policy application is **not atomic**. Readback failure can follow earlier
//! successful writes; the failed write may also have changed hardware. Normal
//! GLIKEY close is attempted once after successful open, even on a mismatch;
//! closing is not rollback. Both target and close failures are preserved. Failed
//! enable causes no target writes or speculative close/reset. Faults/panics can
//! prevent cleanup; dropping the driver or an error does not restore anything.
//!
//! # Initial scope and limitations
//!
//! Only main-flash configuration and slots 0/4/5 are writable. No IFR mutation,
//! policy locking, locked-policy selection, automatic allocation, NSE/security
//! attribution, AHBSC setup, violation interrupt/capture/clear API, or MCXN support
//! is provided. Inherited locks are respected and affected NSE=1 blocks rejected.
//!
//! Chip-specific A2 RM 51.1.3 / A5 RM 64.1.3 say flash is SecureUser. Only
//! SUR/SUW/SUX are changed; other access-class bits are preserved. A5 RM 64.5.1's
//! generic NSE=0 execution wording conflicts with its chip-specific SUX/XOM
//! descriptions. Success verifies **programmed register values**, not effective
//! XOM, execution denial, pipeline/cache synchronization, or isolation. Do not
//! infer hardware enforcement from compilation or register readback alone.

use core::fmt;

use crate::glikey::{self, Glikey};
use crate::{Peri, pac, peripherals};

#[path = "mbc/policy.rs"]
mod policy;
pub use policy::{
    BlockConfig, BlockError, Geometry, MainFlashBlock, Permission, PermissionSet, Policy, PolicySlot, PolicyState,
    RegionConfig, SlotConfig, ValidationError, WritableSlot,
};
use policy::{Prepared, Snapshot};

struct Info {
    regs: pac::mbc::Mbc,
}

static INFO: Info = Info { regs: pac::MBC0 };

/// Geometry cannot safely be represented by this driver's implemented registers.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SetupError {
    /// Unknown device flash-size encoding or inconsistent/unsupported MBC geometry.
    UnsupportedGeometry,
}

/// A protected register attempted by policy application.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Register {
    /// Shared global policy register.
    Policy(WritableSlot),
    /// Packed main-flash selector word (eight blocks per word).
    MainWord(u8),
}

/// A complete-word mismatch, including fields which should have been preserved.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadbackFailure {
    /// Register whose readback did not match.
    pub register: Register,
    /// Expected complete register value.
    pub expected: u32,
    /// Sampled complete register value.
    pub observed: u32,
}

/// Target-application progress; this is not a resumable transaction token.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Progress {
    /// Number of changed policy registers successfully verified.
    pub verified_slots: u8,
    /// Number of changed selector words successfully verified.
    pub verified_words: u8,
    /// Last attempted write, which may have changed hardware even if not verified.
    pub last_attempt: Option<Register>,
}

/// What this operation establishes about the GLIKEY grant.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GrantState {
    /// No enable/close attempted. An inherited grant, if any, was left untouched.
    Untouched,
    /// Normal close succeeded and was verified by GLIKEY.
    Closed,
    /// Enable or close failed; this operation cannot establish closure.
    Unknown,
}

/// Primary application failure, independent of any subsequent close failure.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ApplyFailure {
    /// Complete preflight rejected the description before any writes.
    Validation(ValidationError),
    /// GLIKEY enable failed; no MBC writes or speculative close followed.
    Enable(glikey::Error),
    /// An MBC write did not produce the expected complete-word readback.
    Readback(ReadbackFailure),
}

/// Application diagnosis. Inspect both failures before choosing recovery.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct ApplyError {
    /// Primary failure, or `None` when all writes verified but close failed.
    pub failure: Option<ApplyFailure>,
    /// Normal close failure, independent of the primary failure.
    pub close_error: Option<glikey::Error>,
    /// Verified prefix and last attempted register; not all modified registers.
    pub progress: Progress,
    /// Grant outcome established by this operation.
    pub grant: GrantState,
    /// GLIKEY status sampled after stopping; no error evidence is cleared.
    pub glikey_status: glikey::Status,
}

/// Construction failure, retaining the MBC owner after attempted application.
#[derive(Debug)]
#[non_exhaustive]
pub enum NewWithPolicyError<'d> {
    /// Observational setup failed, before any protected writes.
    Setup(SetupError),
    /// Configuration may be partially applied. The returned owner permits
    /// inspection, not an assurance of restored configuration or grant closure.
    Apply {
        /// Owned MBC handle, without retained GLIKEY or policy borrows.
        mbc: Mbc<'d>,
        /// Full application and close diagnosis.
        error: ApplyError,
    },
}

/// Exclusive owner of MBC0. Other bus masters remain a platform responsibility.
pub struct Mbc<'d> {
    _peri: Peri<'d, peripherals::MBC0>,
    info: &'static Info,
    geometry: Geometry,
    ifr_blocks: [u16; 2],
}

impl fmt::Debug for Mbc<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Mbc")
            .field("geometry", &self.geometry)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "_mcxa2xx")]
fn nonsecure(value: pac::mbc::Nse) -> bool {
    value == pac::mbc::Nse::Notallowed
}

#[cfg(feature = "_mcxa5xx")]
fn nonsecure(value: bool) -> bool {
    value
}

macro_rules! decode_blocks {
    ($value:expr; $(($select:ident, $nse:ident)),+ $(,)?) => {{
        let value = $value;
        [$({
            BlockConfig {
                slot: PolicySlot::ALL[value.$select() as usize],
                nonsecure: nonsecure(value.$nse()),
            }
        }),+]
    }};
}

macro_rules! decode_word {
    ($value:expr) => {
        decode_blocks!($value;
            (mbacsel0, nse0), (mbacsel1, nse1), (mbacsel2, nse2), (mbacsel3, nse3),
            (mbacsel4, nse4), (mbacsel5, nse5), (mbacsel6, nse6), (mbacsel7, nse7))
    };
}

macro_rules! read_policy {
    ($reg:expr, $locked:expr) => {{
        let value = $reg.read();
        let mut permissions = PermissionSet::empty();
        if value.sur() {
            permissions = permissions | Permission::Read;
        }
        if value.suw() {
            permissions = permissions | Permission::Write;
        }
        if value.sux() {
            permissions = permissions | Permission::Execute;
        }
        PolicyState {
            permissions,
            locked: $locked(value),
        }
    }};
}

impl<'d> Mbc<'d> {
    /// Takes ownership and reads geometry without changing configuration.
    ///
    /// Running clocks and authorized MBC/SYSCON access are prerequisites. No
    /// reset, gate operation, evidence clear, or GLIKEY construction occurs.
    pub fn new(peri: Peri<'d, peripherals::MBC0>) -> Result<Self, SetupError> {
        let regs = INFO.regs;
        let main = regs.mbc0_mem0_glbcfg().read();
        let ifr0 = regs.mbc0_mem1_glbcfg().read();
        let ifr1 = regs.mbc0_mem2_glbcfg().read();
        let flash_size = match pac::SYSCON.device_id0().read().flash_size() {
            pac::syscon::FlashSize::Size512kb => 512 * 1024,
            pac::syscon::FlashSize::Size1mb => 1024 * 1024,
            pac::syscon::FlashSize::Size2mb => 2 * 1024 * 1024,
            _ => return Err(SetupError::UnsupportedGeometry),
        };
        #[cfg(feature = "_mcxa2xx")]
        let (log2, ifr_max) = (13, [16, 4]);
        #[cfg(feature = "_mcxa5xx")]
        let (log2, ifr_max) = (14, [32, 8]);
        let block_size = 1u32 << log2;
        let block_count = flash_size / block_size;
        if main.size_log2() != log2 || main.nblks() > 128 || u32::from(main.nblks()) < block_count {
            return Err(SetupError::UnsupportedGeometry);
        }
        for (count, size, maximum) in [
            (ifr0.nblks(), ifr0.size_log2(), ifr_max[0]),
            (ifr1.nblks(), ifr1.size_log2(), ifr_max[1]),
        ] {
            if count > maximum || (count == 0 && size != 0) || (count != 0 && size != 11) {
                return Err(SetupError::UnsupportedGeometry);
            }
        }
        Ok(Self {
            _peri: peri,
            info: &INFO,
            geometry: Geometry {
                flash_size,
                block_size,
                block_count: block_count as u16,
            },
            ifr_blocks: [ifr0.nblks(), ifr1.nblks()],
        })
    }

    /// Constructs the owner and applies a policy through [`Self::apply_policy`].
    /// Returns the owner on application failure; dropping it performs no recovery.
    ///
    /// # Safety
    /// The complete intermediate/failure-state safety obligations of
    /// [`Self::apply_policy`] apply. Readback verifies bits, not effective A5
    /// execution enforcement; see the module's documentation conflict.
    pub unsafe fn new_with_policy(
        peri: Peri<'d, peripherals::MBC0>,
        glikey: &mut Glikey<'_>,
        policy: &Policy<'_>,
    ) -> Result<Self, NewWithPolicyError<'d>> {
        let mut mbc = Self::new(peri).map_err(NewWithPolicyError::Setup)?;
        // SAFETY: the caller assumes apply_policy's complete safety contract.
        match unsafe { mbc.apply_policy(glikey, policy) } {
            Ok(()) => Ok(mbc),
            Err(error) => Err(NewWithPolicyError::Apply { mbc, error }),
        }
    }

    /// Checked main-flash geometry, not erase-sector or ROM flash-bank geometry.
    pub fn geometry(&self) -> Geometry {
        self.geometry
    }

    /// Validates a main-flash block index against this device's density.
    pub fn block(&self, index: u16) -> Result<MainFlashBlock, BlockError> {
        if index >= self.geometry.block_count {
            return Err(BlockError::OutOfBounds);
        }
        Ok(MainFlashBlock { index })
    }

    /// Reads one checked block's selector and inherited NSE without clearing it.
    pub fn block_config(&self, block: MainFlashBlock) -> BlockConfig {
        let index = usize::from(block.index);
        decode_word!(self.info.regs.mbc0_dom0_mem0_blk_cfg_wn(index / 8).read())[index % 8]
    }

    /// Samples SecureUser permission bits and lock state without changing them.
    pub fn policy(&self, slot: PolicySlot) -> PolicyState {
        let regs = self.info.regs;
        match slot {
            PolicySlot::Slot0 => read_policy!(regs.mbc0_memn_glbac0(), |_| false),
            PolicySlot::Slot1 => read_policy!(regs.mbc0_memn_glbac1(), |v: pac::mbc::MemnGlbac1| v.lk()),
            PolicySlot::Slot2 => read_policy!(regs.mbc0_memn_glbac2(), |v: pac::mbc::MemnGlbac2| v.lk()),
            PolicySlot::Slot3 => read_policy!(regs.mbc0_memn_glbac3(), |v: pac::mbc::MemnGlbac3| v.lk()),
            PolicySlot::Slot4 => read_policy!(regs.mbc0_memn_glbac4(), |v: pac::mbc::MemnGlbac4| v.lk()),
            PolicySlot::Slot5 => read_policy!(regs.mbc0_memn_glbac5(), |v: pac::mbc::MemnGlbac5| v.lk()),
            PolicySlot::Slot6 => read_policy!(regs.mbc0_memn_glbac6(), |v: pac::mbc::MemnGlbac6| v.lk()),
            PolicySlot::Slot7 => read_policy!(regs.mbc0_memn_glbac7(), |v: pac::mbc::MemnGlbac7| v.lk()),
        }
    }

    /// Assigns one main-flash block to an existing, unlocked configurable policy.
    /// Uses the same preflight and explicit grant cleanup as policy application.
    ///
    /// # Safety
    /// The safety obligations of [`Self::apply_policy`] apply to the existing
    /// destination permissions as well as the current ones. Readback verifies
    /// bits, not effective A5 execution enforcement.
    pub unsafe fn set_block_policy(
        &mut self,
        glikey: &mut Glikey<'_>,
        block: MainFlashBlock,
        slot: WritableSlot,
    ) -> Result<(), ApplyError> {
        let start = u32::from(block.index) * self.geometry.block_size;
        let regions = [RegionConfig {
            range: start..start + self.geometry.block_size,
            slot,
        }];
        // SAFETY: the caller assumes the same obligations for the one-block policy.
        unsafe {
            self.apply_policy(
                glikey,
                &Policy {
                    slots: &[],
                    regions: &regions,
                },
            )
        }
    }

    /// Validates the complete policy, then programs slots and selector words in
    /// ascending order, with complete-word readback and one explicit grant close.
    /// No policy or GLIKEY borrow is retained. A no-op changes neither controller.
    ///
    /// Earlier writes and a mismatched write may have taken effect on failure.
    /// No rollback/reset/retry occurs. See [`ApplyError`] for independent target
    /// and close failures. Success verifies register values, not effective A5
    /// execution enforcement or cache/pipeline synchronization.
    ///
    /// # Safety
    /// Keep flash accessible as required by every Rust reference throughout all
    /// intermediate configurations, failures, and later use. In particular do
    /// not revoke reads from live referenced objects, policy slices, or the code
    /// and data needed to finish this operation and handle failure. Account for
    /// interrupt/fault handlers, vectors, constants, DMA, debugger, ROM/flash
    /// algorithms and recovery. Exclude all competing register writers through
    /// snapshot, programming and close; local interrupt masking cannot exclude
    /// other bus masters. The driver neither relocates code nor manages caches.
    pub unsafe fn apply_policy(&mut self, glikey: &mut Glikey<'_>, policy: &Policy<'_>) -> Result<(), ApplyError> {
        critical_section::with(|_| {
            let prepared = self.snapshot().prepare(policy).map_err(|error| ApplyError {
                failure: Some(ApplyFailure::Validation(error)),
                close_error: None,
                progress: Progress::default(),
                grant: GrantState::Untouched,
                glikey_status: glikey.status(),
            })?;
            if prepared.is_empty() {
                return Ok(());
            }
            if glikey.status().state == glikey::State::WriteEnabled {
                return Err(ApplyError {
                    failure: Some(ApplyFailure::Enable(glikey::Error::NotIdle)),
                    close_error: None,
                    progress: Progress::default(),
                    grant: GrantState::Untouched,
                    glikey_status: glikey.status(),
                });
            }
            glikey
                .enable_write(glikey::Target::MemoryBlockChecker)
                .map_err(|error| ApplyError {
                    failure: Some(ApplyFailure::Enable(error)),
                    close_error: None,
                    progress: Progress::default(),
                    grant: GrantState::Unknown,
                    glikey_status: glikey.status(),
                })?;

            let mut progress = Progress::default();
            let failure = self.program(&prepared, &mut progress).err().map(ApplyFailure::Readback);
            // Deliberately not an early `?`: close must also run on target failure.
            let close_error = glikey.disable_write().err();
            if failure.is_none() && close_error.is_none() {
                return Ok(());
            }
            Err(ApplyError {
                failure,
                close_error,
                progress,
                grant: if close_error.is_none() {
                    GrantState::Closed
                } else {
                    GrantState::Unknown
                },
                glikey_status: glikey.status(),
            })
        })
    }

    fn snapshot(&self) -> Snapshot {
        let empty = BlockConfig {
            slot: PolicySlot::Slot0,
            nonsecure: false,
        };
        let mut snapshot = Snapshot {
            geometry: self.geometry,
            slots: PolicySlot::ALL.map(|slot| self.policy(slot)),
            main: [empty; 128],
            ifr: [empty; 40],
            ifr_count: usize::from(self.ifr_blocks[0] + self.ifr_blocks[1]),
        };
        for index in 0..self.geometry.block_count {
            snapshot.main[usize::from(index)] = self.block_config(MainFlashBlock { index });
        }
        let regs = self.info.regs;
        for word in 0..usize::from(self.ifr_blocks[0]).div_ceil(8) {
            let blocks = match word {
                0 => decode_word!(regs.mbc0_dom0_mem1_blk_cfg_w0().read()),
                1 => decode_word!(regs.mbc0_dom0_mem1_blk_cfg_w1().read()),
                #[cfg(feature = "_mcxa5xx")]
                2 => decode_word!(regs.mbc0_dom0_mem1_blk_cfg_w2().read()),
                #[cfg(feature = "_mcxa5xx")]
                3 => decode_word!(regs.mbc0_dom0_mem1_blk_cfg_w3().read()),
                _ => unreachable!(), // Checked implemented geometry at construction.
            };
            let count = (usize::from(self.ifr_blocks[0]) - word * 8).min(8);
            snapshot.ifr[word * 8..word * 8 + count].copy_from_slice(&blocks[..count]);
        }
        let start = usize::from(self.ifr_blocks[0]);
        let count = usize::from(self.ifr_blocks[1]);
        if count != 0 {
            #[cfg(feature = "_mcxa2xx")]
            let blocks = decode_blocks!(regs.mbc0_dom0_mem2_blk_cfg_w0().read();
                (mbacsel0, nse0), (mbacsel1, nse1), (mbacsel2, nse2), (mbacsel3, nse3));
            #[cfg(feature = "_mcxa5xx")]
            let blocks = decode_word!(regs.mbc0_dom0_mem2_blk_cfg_w0().read());
            snapshot.ifr[start..start + count].copy_from_slice(&blocks[..count]);
        }
        snapshot
    }

    fn program(&self, prepared: &Prepared, progress: &mut Progress) -> Result<(), ReadbackFailure> {
        for slot in WritableSlot::ALL {
            if let Some(permissions) = prepared.slots[slot.position()] {
                let register = Register::Policy(slot);
                progress.last_attempt = Some(register);
                macro_rules! write_policy {
                    ($reg:expr) => {{
                        let reg = $reg;
                        let mut expected = reg.read();
                        expected.set_sur(permissions.contains(Permission::Read));
                        expected.set_suw(permissions.contains(Permission::Write));
                        expected.set_sux(permissions.contains(Permission::Execute));
                        reg.write_value(expected);
                        cortex_m::asm::dsb();
                        let observed = reg.read().0;
                        if observed != expected.0 {
                            return Err(ReadbackFailure {
                                register,
                                expected: expected.0,
                                observed,
                            });
                        }
                    }};
                }
                match slot {
                    WritableSlot::Slot0 => write_policy!(self.info.regs.mbc0_memn_glbac0()),
                    WritableSlot::Slot4 => write_policy!(self.info.regs.mbc0_memn_glbac4()),
                    WritableSlot::Slot5 => write_policy!(self.info.regs.mbc0_memn_glbac5()),
                }
                progress.verified_slots += 1;
            }
        }
        for word in 0..usize::from(self.geometry.block_count).div_ceil(8) {
            let assignments = &prepared.blocks[word * 8..word * 8 + 8];
            if assignments.iter().all(Option::is_none) {
                continue;
            }
            let register = Register::MainWord(word as u8);
            let reg = self.info.regs.mbc0_dom0_mem0_blk_cfg_wn(word);
            let mut expected = reg.read();
            for (field, slot) in assignments.iter().enumerate() {
                let Some(slot) = slot else { continue };
                let select = match slot {
                    WritableSlot::Slot0 => pac::mbc::Mbacsel::Glbac0,
                    WritableSlot::Slot4 => pac::mbc::Mbacsel::Glbac4,
                    WritableSlot::Slot5 => pac::mbc::Mbacsel::Glbac5,
                };
                match field {
                    0 => expected.set_mbacsel0(select),
                    1 => expected.set_mbacsel1(select),
                    2 => expected.set_mbacsel2(select),
                    3 => expected.set_mbacsel3(select),
                    4 => expected.set_mbacsel4(select),
                    5 => expected.set_mbacsel5(select),
                    6 => expected.set_mbacsel6(select),
                    7 => expected.set_mbacsel7(select),
                    _ => unreachable!(),
                }
            }
            progress.last_attempt = Some(register);
            reg.write_value(expected);
            cortex_m::asm::dsb();
            let observed = reg.read().0;
            if observed != expected.0 {
                return Err(ReadbackFailure {
                    register,
                    expected: expected.0,
                    observed,
                });
            }
            progress.verified_words += 1;
        }
        Ok(())
    }
}
