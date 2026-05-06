//! Marker types for limiting access.

/// A trait for restricting one [`Access`] type to another [`Access`] type.
///
/// Restricting `Self` to `To` results in [`Self::Restricted`].
///
/// Restriction is a symmetric operation which is denoted by ∩, as it is the intersection of permissions.
/// The following table holds:
///
/// | `Self`        | `To`          | `Self` ∩ `To` |
/// | ------------- | ------------- | ------------- |
/// | `T`           | `T`           | `T`           |
/// | [`ReadWrite`] | `T`           | `T`           |
/// | [`NoAccess`]  | `T`           | [`NoAccess`]  |
/// | [`ReadOnly`]  | [`WriteOnly`] | [`NoAccess`]  |
pub trait RestrictAccess<To>: Access {
    /// The resulting [`Access`] type of `Self` restricted to `To`.
    type Restricted: Access;
}

impl<To: Access> RestrictAccess<To> for ReadWrite {
    type Restricted = To;
}

impl<To> RestrictAccess<To> for NoAccess {
    type Restricted = Self;
}

// Sadly, we cannot provide more generic implementations, since they would overlap.
macro_rules! restrict_impl {
    ($SelfT:ty, $To:ty, $Restricted:ty) => {
        impl RestrictAccess<$To> for $SelfT {
            type Restricted = $Restricted;
        }
    };
}

restrict_impl!(ReadOnly, ReadWrite, ReadOnly);
restrict_impl!(ReadOnly, ReadOnly, ReadOnly);
restrict_impl!(ReadOnly, WriteOnly, NoAccess);
restrict_impl!(ReadOnly, NoAccess, NoAccess);

restrict_impl!(WriteOnly, ReadWrite, WriteOnly);
restrict_impl!(WriteOnly, ReadOnly, NoAccess);
restrict_impl!(WriteOnly, WriteOnly, WriteOnly);
restrict_impl!(WriteOnly, NoAccess, NoAccess);

/// Sealed trait that is implemented for the types in this module.
pub trait Access: Copy + Default + private::Sealed {}

/// Helper trait that is implemented by [`ReadWrite`] and [`ReadOnly`].
pub trait Readable: Access {}
impl<A: RestrictAccess<ReadOnly, Restricted = ReadOnly>> Readable for A {}

/// Helper trait that is implemented by [`ReadWrite`] and [`WriteOnly`].
pub trait Writable: Access {}
impl<A: RestrictAccess<WriteOnly, Restricted = WriteOnly>> Writable for A {}

/// Zero-sized marker type for allowing both read and write access.
#[derive(Debug, Default, Copy, Clone)]
pub struct ReadWrite;
impl Access for ReadWrite {}

/// Zero-sized marker type for allowing only read access.
#[derive(Debug, Default, Copy, Clone)]
pub struct ReadOnly;
impl Access for ReadOnly {}

/// Zero-sized marker type for allowing only write access.
#[derive(Debug, Default, Copy, Clone)]
pub struct WriteOnly;
impl Access for WriteOnly {}

/// Zero-sized marker type that grants no access.
#[derive(Debug, Default, Copy, Clone)]
pub struct NoAccess;
impl Access for NoAccess {}

mod private {
    pub trait Sealed {}

    impl Sealed for super::ReadWrite {}
    impl Sealed for super::ReadOnly {}
    impl Sealed for super::WriteOnly {}
    impl Sealed for super::NoAccess {}
}
