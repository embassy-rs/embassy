//! Voltatile pointer type.

use core::cmp::Ordering;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::{fmt, hash};

use crate::volatile_ptr::access::ReadWrite;

mod access;
mod macros;
mod operations;
mod slice;

/// Wraps a pointer to make accesses to the referenced value volatile.
///
/// Allows volatile reads and writes on the referenced value. The referenced value needs to
/// be `Copy` for reading and writing, as volatile reads and writes take and return copies
/// of the value.
///
/// Since not all volatile resources (e.g. memory mapped device registers) are both readable
/// and writable, this type supports limiting the allowed access types through an optional second
/// generic parameter `A` that can be one of `ReadWrite`, `ReadOnly`, or `WriteOnly`. It defaults
/// to `ReadWrite`, which allows all operations.
///
/// The size of this struct is the same as the size of the contained reference.
#[must_use]
#[repr(transparent)]
pub struct VolatilePtr<'a, T, A = ReadWrite>
where
    T: ?Sized,
{
    pointer: NonNull<T>,
    reference: PhantomData<&'a T>,
    access: PhantomData<A>,
}

impl<'a, T, A> Copy for VolatilePtr<'a, T, A> where T: ?Sized {}

impl<T, A> Clone for VolatilePtr<'_, T, A>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, A> fmt::Debug for VolatilePtr<'_, T, A>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.pointer.as_ptr(), f)
    }
}

impl<T, A> fmt::Pointer for VolatilePtr<'_, T, A>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.pointer.as_ptr(), f)
    }
}

impl<T, A> PartialEq for VolatilePtr<'_, T, A>
where
    T: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.pointer.as_ptr(), other.pointer.as_ptr())
    }
}

impl<T, A> Eq for VolatilePtr<'_, T, A> where T: ?Sized {}

impl<T, A> PartialOrd for VolatilePtr<'_, T, A>
where
    T: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, A> Ord for VolatilePtr<'_, T, A>
where
    T: ?Sized,
{
    fn cmp(&self, other: &Self) -> Ordering {
        #[allow(ambiguous_wide_pointer_comparisons)]
        Ord::cmp(&self.pointer.as_ptr(), &other.pointer.as_ptr())
    }
}

impl<T, A> hash::Hash for VolatilePtr<'_, T, A>
where
    T: ?Sized,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.pointer.as_ptr().hash(state);
    }
}
