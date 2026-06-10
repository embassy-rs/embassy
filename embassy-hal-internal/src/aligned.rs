//! Extra traits for aligned

use core::mem;

use aligned::{Aligned, Alignment};
use as_slice::{AsMutSlice, AsSlice};

/// Create an aligned slice from an aligned array
pub trait AsAligned {
    /// Slice element
    type Element;
    /// Slice alignment
    type Alignment: Alignment;

    /// Create the slice
    fn as_aligned(&self) -> &Aligned<Self::Alignment, [Self::Element]>;
}

/// Create an aligned slice from an aligned array
pub trait AsMutAligned {
    /// Slice element
    type Element;
    /// Slice alignment
    type Alignment: Alignment;

    /// Create the slice
    fn as_mut_aligned(&mut self) -> &mut Aligned<Self::Alignment, [Self::Element]>;
}

impl<A, T> AsAligned for Aligned<A, T>
where
    A: Alignment,
    T: AsSlice,
{
    type Element = T::Element;
    type Alignment = A;

    #[inline]
    fn as_aligned(&self) -> &Aligned<A, [T::Element]> {
        unsafe { mem::transmute(T::as_slice(&**self)) }
    }
}

impl<A, T> AsMutAligned for Aligned<A, T>
where
    A: Alignment,
    T: AsMutSlice,
{
    type Element = T::Element;
    type Alignment = A;

    #[inline]
    fn as_mut_aligned(&mut self) -> &mut Aligned<A, [T::Element]> {
        unsafe { mem::transmute(T::as_mut_slice(&mut **self)) }
    }
}

/// Create an aligned value from a non-aligned value
pub trait ToAligned {
    /// Element
    type Element: ?Sized;

    /// Create a type-checked aligned value from a value that is aligned.
    fn to_aligned<A: Alignment>(&self) -> &Aligned<A, Self::Element>;
}

impl<T: ?Sized> ToAligned for &T {
    type Element = T;

    #[inline]
    fn to_aligned<A: Alignment>(&self) -> &Aligned<A, Self::Element> {
        assert!(self as *const _ as usize % A::ALIGN == 0);

        unsafe { mem::transmute(*self) }
    }
}

/// Create an aligned value from a non-aligned value
pub trait ToMutAligned {
    /// Element
    type Element: ?Sized;

    /// Create a type-checked aligned value from a value that is aligned.
    fn to_mut_aligned<A: Alignment>(&mut self) -> &mut Aligned<A, Self::Element>;
}

impl<T: ?Sized> ToMutAligned for T {
    type Element = T;

    #[inline]
    fn to_mut_aligned<A: Alignment>(&mut self) -> &mut Aligned<A, Self::Element> {
        assert!(self as *mut _ as *mut u8 as usize % A::ALIGN == 0);

        unsafe { mem::transmute(&mut *self) }
    }
}
