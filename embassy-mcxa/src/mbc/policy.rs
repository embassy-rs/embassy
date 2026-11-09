use core::ops::{BitOr, Range};

/// One permission in a SecureUser flash policy.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Permission {
    /// Permit data reads (SUR).
    Read,
    /// Permit flash write accesses (SUW); this does not program or erase flash.
    Write,
    /// Set SUX. The A5 manual's NSE and chip-specific execution descriptions
    /// conflict; this is not a guarantee of effective execution or XOM.
    Execute,
}

/// A set of independent read, write, and execute permissions.
///
/// Combine values with `Permission::Read | Permission::Execute`. In constants,
/// use [`Self::from_array`], since overloaded operators are not const.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PermissionSet {
    bits: u8,
}

impl PermissionSet {
    /// No permissions.
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Construct a set in const or runtime code. Repeated entries are harmless.
    pub const fn from_array<const N: usize>(permissions: [Permission; N]) -> Self {
        let mut bits = 0;
        let mut index = 0;
        while index < N {
            bits |= 1 << permissions[index] as u8;
            index += 1;
        }
        Self { bits }
    }

    /// Whether this set includes a permission.
    pub const fn contains(self, permission: Permission) -> bool {
        self.bits & (1 << permission as u8) != 0
    }
}

impl From<Permission> for PermissionSet {
    fn from(permission: Permission) -> Self {
        Self::from_array([permission])
    }
}

impl BitOr for Permission {
    type Output = PermissionSet;

    fn bitor(self, rhs: Self) -> Self::Output {
        PermissionSet::from_array([self, rhs])
    }
}

impl BitOr<Permission> for PermissionSet {
    type Output = Self;

    fn bitor(self, rhs: Permission) -> Self {
        self | Self::from(rhs)
    }
}

impl BitOr<PermissionSet> for Permission {
    type Output = PermissionSet;

    fn bitor(self, rhs: PermissionSet) -> Self::Output {
        PermissionSet::from(self) | rhs
    }
}

impl BitOr for PermissionSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

/// An application-configurable policy slot. No lock operation is provided.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum WritableSlot {
    /// GLBAC0; this slot cannot be locked.
    Slot0 = 0,
    /// GLBAC4; an inherited lock is respected.
    Slot4 = 4,
    /// GLBAC5; an inherited lock is respected.
    Slot5 = 5,
}

impl WritableSlot {
    pub(super) const ALL: [Self; 3] = [Self::Slot0, Self::Slot4, Self::Slot5];

    pub(super) const fn position(self) -> usize {
        match self {
            Self::Slot0 => 0,
            Self::Slot4 => 1,
            Self::Slot5 => 2,
        }
    }
}

/// Any hardware policy slot, for observation (including fixed/locked policies).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PolicySlot {
    /// GLBAC0.
    Slot0 = 0,
    /// Fixed GLBAC1.
    Slot1 = 1,
    /// Fixed GLBAC2.
    Slot2 = 2,
    /// Fixed GLBAC3.
    Slot3 = 3,
    /// GLBAC4.
    Slot4 = 4,
    /// GLBAC5.
    Slot5 = 5,
    /// Fixed GLBAC6.
    Slot6 = 6,
    /// Fixed GLBAC7.
    Slot7 = 7,
}

impl PolicySlot {
    pub(super) const ALL: [Self; 8] = [
        Self::Slot0,
        Self::Slot1,
        Self::Slot2,
        Self::Slot3,
        Self::Slot4,
        Self::Slot5,
        Self::Slot6,
        Self::Slot7,
    ];
}

impl From<WritableSlot> for PolicySlot {
    fn from(slot: WritableSlot) -> Self {
        match slot {
            WritableSlot::Slot0 => Self::Slot0,
            WritableSlot::Slot4 => Self::Slot4,
            WritableSlot::Slot5 => Self::Slot5,
        }
    }
}

/// Explicit permissions for an application-chosen shared slot.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SlotConfig {
    /// Slot to configure; changing it affects every block currently selecting it.
    pub slot: WritableSlot,
    /// SecureUser permission bits to program. Other access classes are preserved.
    pub permissions: PermissionSet,
}

/// A nonempty, block-aligned main-flash region selecting one policy slot.
#[derive(Debug, Clone)]
pub struct RegionConfig {
    /// Byte offsets from main-flash start, with an exclusive end. No rounding.
    pub range: Range<u32>,
    /// Slot selected for every MBC block in the range.
    pub slot: WritableSlot,
}

/// A borrowed application policy, validated completely before protected writes.
///
/// Slot definitions are optional for regions using existing permissions.
/// Unlisted blocks retain their selectors. A changed slot may not have IFR
/// consumers, and every current main-flash consumer must be explicitly covered
/// and retain that slot in this operation. There is no implicit slot allocation.
pub struct Policy<'a> {
    /// Unique slot definitions; identical existing permissions are a no-op.
    pub slots: &'a [SlotConfig],
    /// Nonoverlapping main-flash regions.
    pub regions: &'a [RegionConfig],
}

/// Main-flash geometry, distinct from erase sectors or ROM flash-bank geometry.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Geometry {
    /// Implemented flash capacity in bytes, bounded by the device capacity.
    pub flash_size: u32,
    /// Bytes per MBC block.
    pub block_size: u32,
    /// Number of usable main-flash MBC blocks.
    pub block_count: u16,
}

/// A checked main-flash block identifier, obtained from `Mbc::block`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MainFlashBlock {
    pub(super) index: u16,
}

impl MainFlashBlock {
    /// Zero-based main-flash block index.
    pub const fn index(self) -> u16 {
        self.index
    }
}

/// An index is outside the discovered main-flash geometry.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum BlockError {
    /// The requested block does not exist on this device.
    OutOfBounds,
}

/// Sampled block configuration, not proof of effective access enforcement.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BlockConfig {
    /// Shared policy selected by this block.
    pub slot: PolicySlot,
    /// Inherited NSE setting; mutations reject affected blocks with this set.
    pub nonsecure: bool,
}

/// Sampled SecureUser permission bits and policy-register lock state.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PolicyState {
    /// Current SUR/SUW/SUX values; not a certification of effective permissions.
    pub permissions: PermissionSet,
    /// Policy and associated block selectors are locked until hardware reset.
    pub locked: bool,
}

/// Rejection before any GLIKEY or MBC write.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ValidationError {
    /// Empty, reversed, or outside the device's flash capacity.
    InvalidRange,
    /// A range boundary is not aligned to the MBC block size.
    UnalignedRange,
    /// Regions overlap, even if they select the same slot.
    OverlappingRegions,
    /// More than one definition for a slot.
    DuplicateSlot(WritableSlot),
    /// A changed slot is currently selected by an implemented IFR block.
    IfrConsumer(WritableSlot),
    /// A changed slot affects a main-flash block omitted from the policy.
    UncoveredConsumer(MainFlashBlock),
    /// A current consumer is moved away from a slot changed in this operation.
    MigratingConsumer(MainFlashBlock),
    /// A requested change would alter or select a locked policy.
    LockedPolicy(PolicySlot),
    /// A covered/affected block has unsupported inherited NSE set.
    NonsecureBlock(MainFlashBlock),
}

pub(super) struct Snapshot {
    pub geometry: Geometry,
    pub slots: [PolicyState; 8],
    pub main: [BlockConfig; 128],
    pub ifr: [BlockConfig; 40],
    pub ifr_count: usize,
}

#[derive(Debug)]
pub(super) struct Prepared {
    pub slots: [Option<PermissionSet>; 3],
    pub blocks: [Option<WritableSlot>; 128],
}

impl Prepared {
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(Option::is_none) && self.blocks.iter().all(Option::is_none)
    }
}

impl Snapshot {
    pub fn prepare(&self, policy: &Policy<'_>) -> Result<Prepared, ValidationError> {
        let mut result = Prepared {
            slots: [None; 3],
            blocks: [None; 128],
        };
        let mut definitions = [false; 3];
        for config in policy.slots {
            let position = config.slot.position();
            if definitions[position] {
                return Err(ValidationError::DuplicateSlot(config.slot));
            }
            definitions[position] = true;
            let current = self.slots[config.slot as usize];
            if current.permissions != config.permissions {
                if current.locked {
                    return Err(ValidationError::LockedPolicy(config.slot.into()));
                }
                if self.ifr[..self.ifr_count]
                    .iter()
                    .any(|block| block.slot == config.slot.into())
                {
                    return Err(ValidationError::IfrConsumer(config.slot));
                }
                result.slots[position] = Some(config.permissions);
            }
        }

        let mut assignments = [None; 128];
        for region in policy.regions {
            let range = &region.range;
            if range.start >= range.end || range.end > self.geometry.flash_size {
                return Err(ValidationError::InvalidRange);
            }
            let size = self.geometry.block_size;
            if !range.start.is_multiple_of(size) || !range.end.is_multiple_of(size) {
                return Err(ValidationError::UnalignedRange);
            }
            for index in range.start / size..range.end / size {
                let index = index as usize;
                if assignments[index].replace(region.slot).is_some() {
                    return Err(ValidationError::OverlappingRegions);
                }
                let block = MainFlashBlock { index: index as u16 };
                let current = self.main[index];
                if current.nonsecure {
                    return Err(ValidationError::NonsecureBlock(block));
                }
                if current.slot != region.slot.into() {
                    for slot in [current.slot, region.slot.into()] {
                        if self.slots[slot as usize].locked {
                            return Err(ValidationError::LockedPolicy(slot));
                        }
                    }
                    result.blocks[index] = Some(region.slot);
                }
            }
        }

        for slot in WritableSlot::ALL {
            if result.slots[slot.position()].is_none() {
                continue;
            }
            for index in 0..usize::from(self.geometry.block_count) {
                if self.main[index].slot != slot.into() {
                    continue;
                }
                let block = MainFlashBlock { index: index as u16 };
                match assignments[index] {
                    None => return Err(ValidationError::UncoveredConsumer(block)),
                    Some(destination) if destination != slot => return Err(ValidationError::MigratingConsumer(block)),
                    Some(_) => {}
                }
            }
        }
        Ok(result)
    }
}
