use core::marker::PhantomData;
use core::ops::{BitAnd, BitOr, BitOrAssign};

use super::tsc_pin_roles;
use super::types::Group;

/// Pin defines
#[allow(missing_docs)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TscIOPin {
    Group1Io1,
    Group1Io2,
    Group1Io3,
    Group1Io4,
    Group2Io1,
    Group2Io2,
    Group2Io3,
    Group2Io4,
    Group3Io1,
    Group3Io2,
    Group3Io3,
    Group3Io4,
    Group4Io1,
    Group4Io2,
    Group4Io3,
    Group4Io4,
    Group5Io1,
    Group5Io2,
    Group5Io3,
    Group5Io4,
    Group6Io1,
    Group6Io2,
    Group6Io3,
    Group6Io4,
    #[cfg(any(tsc_v2, tsc_v3))]
    Group7Io1,
    #[cfg(any(tsc_v2, tsc_v3))]
    Group7Io2,
    #[cfg(any(tsc_v2, tsc_v3))]
    Group7Io3,
    #[cfg(any(tsc_v2, tsc_v3))]
    Group7Io4,
    #[cfg(tsc_v3)]
    Group8Io1,
    #[cfg(tsc_v3)]
    Group8Io2,
    #[cfg(tsc_v3)]
    Group8Io3,
    #[cfg(tsc_v3)]
    Group8Io4,
}

/// Represents a TSC I/O pin with associated group and role information.
///
/// This type combines a `TscIOPin` with phantom type parameters to statically
/// encode the pin's group and role. This allows for type-safe operations
/// on TSC pins within their specific contexts.
///
/// - `Group`: A type parameter representing the TSC group (e.g., `G1`, `G2`).
/// - `Role`: A type parameter representing the pin's role (e.g., `Channel`, `Sample`).
#[derive(Clone, Copy, Debug)]
pub struct TscIOPinWithRole<Group, Role: tsc_pin_roles::Role> {
    /// The underlying TSC I/O pin.
    pub pin: TscIOPin,
    pub(super) phantom: PhantomData<(Group, Role)>,
}

impl<G, R: tsc_pin_roles::Role> TscIOPinWithRole<G, R> {
    pub(super) fn get_pin(wrapped_pin: TscIOPinWithRole<G, R>) -> TscIOPin {
        wrapped_pin.pin
    }
}

impl TscIOPin {
    /// Maps this TscIOPin to the Group it belongs to.
    ///
    /// This method provides a convenient way to determine which Group
    /// a specific TSC I/O pin is associated with.
    pub const fn group(&self) -> Group {
        match self {
            TscIOPin::Group1Io1 | TscIOPin::Group1Io2 | TscIOPin::Group1Io3 | TscIOPin::Group1Io4 => Group::One,
            TscIOPin::Group2Io1 | TscIOPin::Group2Io2 | TscIOPin::Group2Io3 | TscIOPin::Group2Io4 => Group::Two,
            TscIOPin::Group3Io1 | TscIOPin::Group3Io2 | TscIOPin::Group3Io3 | TscIOPin::Group3Io4 => Group::Three,
            TscIOPin::Group4Io1 | TscIOPin::Group4Io2 | TscIOPin::Group4Io3 | TscIOPin::Group4Io4 => Group::Four,
            TscIOPin::Group5Io1 | TscIOPin::Group5Io2 | TscIOPin::Group5Io3 | TscIOPin::Group5Io4 => Group::Five,
            TscIOPin::Group6Io1 | TscIOPin::Group6Io2 | TscIOPin::Group6Io3 | TscIOPin::Group6Io4 => Group::Six,
            #[cfg(any(tsc_v2, tsc_v3))]
            TscIOPin::Group7Io1 | TscIOPin::Group7Io2 | TscIOPin::Group7Io3 | TscIOPin::Group7Io4 => Group::Seven,
            #[cfg(tsc_v3)]
            TscIOPin::Group8Io1 | TscIOPin::Group8Io2 | TscIOPin::Group8Io3 | TscIOPin::Group8Io4 => Group::Eight,
        }
    }

    /// Returns the `Group` associated with the given `TscIOPin`.
    pub fn get_group(pin: TscIOPin) -> Group {
        pin.group()
    }
}

impl BitOr<TscIOPin> for u32 {
    type Output = u32;
    fn bitor(self, rhs: TscIOPin) -> Self::Output {
        let rhs: u32 = rhs.into();
        self | rhs
    }
}

impl BitOr<u32> for TscIOPin {
    type Output = u32;
    fn bitor(self, rhs: u32) -> Self::Output {
        let val: u32 = self.into();
        val | rhs
    }
}

impl BitOr for TscIOPin {
    type Output = u32;
    fn bitor(self, rhs: Self) -> Self::Output {
        let val: u32 = self.into();
        let rhs: u32 = rhs.into();
        val | rhs
    }
}

impl BitOrAssign<TscIOPin> for u32 {
    fn bitor_assign(&mut self, rhs: TscIOPin) {
        let rhs: u32 = rhs.into();
        *self |= rhs;
    }
}

impl BitAnd<TscIOPin> for u32 {
    type Output = u32;
    fn bitand(self, rhs: TscIOPin) -> Self::Output {
        let rhs: u32 = rhs.into();
        self & rhs
    }
}

impl BitAnd<u32> for TscIOPin {
    type Output = u32;
    fn bitand(self, rhs: u32) -> Self::Output {
        let val: u32 = self.into();
        val & rhs
    }
}

impl TscIOPin {
    const fn to_u32(self) -> u32 {
        match self {
            TscIOPin::Group1Io1 => 0x00000001,
            TscIOPin::Group1Io2 => 0x00000002,
            TscIOPin::Group1Io3 => 0x00000004,
            TscIOPin::Group1Io4 => 0x00000008,
            TscIOPin::Group2Io1 => 0x00000010,
            TscIOPin::Group2Io2 => 0x00000020,
            TscIOPin::Group2Io3 => 0x00000040,
            TscIOPin::Group2Io4 => 0x00000080,
            TscIOPin::Group3Io1 => 0x00000100,
            TscIOPin::Group3Io2 => 0x00000200,
            TscIOPin::Group3Io3 => 0x00000400,
            TscIOPin::Group3Io4 => 0x00000800,
            TscIOPin::Group4Io1 => 0x00001000,
            TscIOPin::Group4Io2 => 0x00002000,
            TscIOPin::Group4Io3 => 0x00004000,
            TscIOPin::Group4Io4 => 0x00008000,
            TscIOPin::Group5Io1 => 0x00010000,
            TscIOPin::Group5Io2 => 0x00020000,
            TscIOPin::Group5Io3 => 0x00040000,
            TscIOPin::Group5Io4 => 0x00080000,
            TscIOPin::Group6Io1 => 0x00100000,
            TscIOPin::Group6Io2 => 0x00200000,
            TscIOPin::Group6Io3 => 0x00400000,
            TscIOPin::Group6Io4 => 0x00800000,
            #[cfg(any(tsc_v2, tsc_v3))]
            TscIOPin::Group7Io1 => 0x01000000,
            #[cfg(any(tsc_v2, tsc_v3))]
            TscIOPin::Group7Io2 => 0x02000000,
            #[cfg(any(tsc_v2, tsc_v3))]
            TscIOPin::Group7Io3 => 0x04000000,
            #[cfg(any(tsc_v2, tsc_v3))]
            TscIOPin::Group7Io4 => 0x08000000,
            #[cfg(tsc_v3)]
            TscIOPin::Group8Io1 => 0x10000000,
            #[cfg(tsc_v3)]
            TscIOPin::Group8Io2 => 0x20000000,
            #[cfg(tsc_v3)]
            TscIOPin::Group8Io3 => 0x40000000,
            #[cfg(tsc_v3)]
            TscIOPin::Group8Io4 => 0x80000000,
        }
    }
}

impl Into<u32> for TscIOPin {
    fn into(self) -> u32 {
        self.to_u32()
    }
}
