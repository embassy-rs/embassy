//! Extra traits for aligned

use core::mem;

use aligned::{Aligned, Alignment};

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
