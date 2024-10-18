use core::marker::PhantomData;
use core::ops::{BitAnd, BitOr, BitOrAssign};

use super::pin_roles;
use super::types::Group;

/// Pin defines
#[allow(missing_docs)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum IOPin {
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
/// This type combines an `tsc::IOPin` with phantom type parameters to statically
/// encode the pin's group and role. This allows for type-safe operations
/// on TSC pins within their specific contexts.
///
/// - `Group`: A type parameter representing the TSC group (e.g., `G1`, `G2`).
/// - `Role`: A type parameter representing the pin's role (e.g., `Channel`, `Sample`).
#[derive(Clone, Copy, Debug)]
pub struct IOPinWithRole<Group, Role: pin_roles::Role> {
    /// The underlying TSC I/O pin.
    pub pin: IOPin,
    pub(super) phantom: PhantomData<(Group, Role)>,
}

impl<G, R: pin_roles::Role> IOPinWithRole<G, R> {
    pub(super) fn get_pin(wrapped_pin: IOPinWithRole<G, R>) -> IOPin {
        wrapped_pin.pin
    }
}

impl IOPin {
    /// Maps this IOPin to the Group it belongs to.
    ///
    /// This method provides a convenient way to determine which Group
    /// a specific TSC I/O pin is associated with.
    pub const fn group(&self) -> Group {
        match self {
            IOPin::Group1Io1 | IOPin::Group1Io2 | IOPin::Group1Io3 | IOPin::Group1Io4 => Group::One,
            IOPin::Group2Io1 | IOPin::Group2Io2 | IOPin::Group2Io3 | IOPin::Group2Io4 => Group::Two,
            IOPin::Group3Io1 | IOPin::Group3Io2 | IOPin::Group3Io3 | IOPin::Group3Io4 => Group::Three,
            IOPin::Group4Io1 | IOPin::Group4Io2 | IOPin::Group4Io3 | IOPin::Group4Io4 => Group::Four,
            IOPin::Group5Io1 | IOPin::Group5Io2 | IOPin::Group5Io3 | IOPin::Group5Io4 => Group::Five,
            IOPin::Group6Io1 | IOPin::Group6Io2 | IOPin::Group6Io3 | IOPin::Group6Io4 => Group::Six,
            #[cfg(any(tsc_v2, tsc_v3))]
            IOPin::Group7Io1 | IOPin::Group7Io2 | IOPin::Group7Io3 | IOPin::Group7Io4 => Group::Seven,
            #[cfg(tsc_v3)]
            IOPin::Group8Io1 | IOPin::Group8Io2 | IOPin::Group8Io3 | IOPin::Group8Io4 => Group::Eight,
        }
    }

    /// Returns the `Group` associated with the given `IOPin`.
    pub fn get_group(pin: IOPin) -> Group {
        pin.group()
    }
}

impl BitOr<IOPin> for u32 {
    type Output = u32;
    fn bitor(self, rhs: IOPin) -> Self::Output {
        let rhs: u32 = rhs.into();
        self | rhs
    }
}

impl BitOr<u32> for IOPin {
    type Output = u32;
    fn bitor(self, rhs: u32) -> Self::Output {
        let val: u32 = self.into();
        val | rhs
    }
}

impl BitOr for IOPin {
    type Output = u32;
    fn bitor(self, rhs: Self) -> Self::Output {
        let val: u32 = self.into();
        let rhs: u32 = rhs.into();
        val | rhs
    }
}

impl BitOrAssign<IOPin> for u32 {
    fn bitor_assign(&mut self, rhs: IOPin) {
        let rhs: u32 = rhs.into();
        *self |= rhs;
    }
}

impl BitAnd<IOPin> for u32 {
    type Output = u32;
    fn bitand(self, rhs: IOPin) -> Self::Output {
        let rhs: u32 = rhs.into();
        self & rhs
    }
}

impl BitAnd<u32> for IOPin {
    type Output = u32;
    fn bitand(self, rhs: u32) -> Self::Output {
        let val: u32 = self.into();
        val & rhs
    }
}

impl IOPin {
    const fn to_u32(self) -> u32 {
        match self {
            IOPin::Group1Io1 => 0x00000001,
            IOPin::Group1Io2 => 0x00000002,
            IOPin::Group1Io3 => 0x00000004,
            IOPin::Group1Io4 => 0x00000008,
            IOPin::Group2Io1 => 0x00000010,
            IOPin::Group2Io2 => 0x00000020,
            IOPin::Group2Io3 => 0x00000040,
            IOPin::Group2Io4 => 0x00000080,
            IOPin::Group3Io1 => 0x00000100,
            IOPin::Group3Io2 => 0x00000200,
            IOPin::Group3Io3 => 0x00000400,
            IOPin::Group3Io4 => 0x00000800,
            IOPin::Group4Io1 => 0x00001000,
            IOPin::Group4Io2 => 0x00002000,
            IOPin::Group4Io3 => 0x00004000,
            IOPin::Group4Io4 => 0x00008000,
            IOPin::Group5Io1 => 0x00010000,
            IOPin::Group5Io2 => 0x00020000,
            IOPin::Group5Io3 => 0x00040000,
            IOPin::Group5Io4 => 0x00080000,
            IOPin::Group6Io1 => 0x00100000,
            IOPin::Group6Io2 => 0x00200000,
            IOPin::Group6Io3 => 0x00400000,
            IOPin::Group6Io4 => 0x00800000,
            #[cfg(any(tsc_v2, tsc_v3))]
            IOPin::Group7Io1 => 0x01000000,
            #[cfg(any(tsc_v2, tsc_v3))]
            IOPin::Group7Io2 => 0x02000000,
            #[cfg(any(tsc_v2, tsc_v3))]
            IOPin::Group7Io3 => 0x04000000,
            #[cfg(any(tsc_v2, tsc_v3))]
            IOPin::Group7Io4 => 0x08000000,
            #[cfg(tsc_v3)]
            IOPin::Group8Io1 => 0x10000000,
            #[cfg(tsc_v3)]
            IOPin::Group8Io2 => 0x20000000,
            #[cfg(tsc_v3)]
            IOPin::Group8Io3 => 0x40000000,
            #[cfg(tsc_v3)]
            IOPin::Group8Io4 => 0x80000000,
        }
    }
}

impl Into<u32> for IOPin {
    fn into(self) -> u32 {
        self.to_u32()
    }
}
