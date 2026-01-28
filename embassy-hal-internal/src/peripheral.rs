use core::marker::PhantomData;
use core::ops::Deref;

use crate::interrupt::{Binding, Handler, Interrupt};

/// An exclusive reference to a peripheral.
///
/// This is functionally the same as a `&'a mut T`. There's a few advantages in having
/// a dedicated struct instead:
///
/// - Memory efficiency: Peripheral singletons are typically either zero-sized (for concrete
///   peripherals like `PA9` or `SPI4`) or very small (for example `AnyPin`, which is 1 byte).
///   However `&mut T` is always 4 bytes for 32-bit targets, even if T is zero-sized.
///   Peripheral stores a copy of `T` instead, so it's the same size.
/// - Code size efficiency. If the user uses the same driver with both `SPI4` and `&mut SPI4`,
///   the driver code would be monomorphized two times. With Peri, the driver is generic
///   over a lifetime only. `SPI4` becomes `Peri<'static, SPI4>`, and `&mut SPI4` becomes
///   `Peri<'a, SPI4>`. Lifetimes don't cause monomorphization.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Peri<'a, T: PeripheralType> {
    inner: T,
    _lifetime: PhantomData<&'a mut T>,
}

impl<'a, T: PeripheralType> Peri<'a, T> {
    /// Create a new owned a peripheral.
    ///
    /// For use by HALs only.
    ///
    /// If you're an end user you shouldn't use this, you should use `steal()`
    /// on the actual peripheral types instead.
    #[inline]
    #[doc(hidden)]
    pub unsafe fn new_unchecked(inner: T) -> Self {
        Self {
            inner,
            _lifetime: PhantomData,
        }
    }

    /// Unsafely clone (duplicate) a peripheral singleton.
    ///
    /// # Safety
    ///
    /// This returns an owned clone of the peripheral. You must manually ensure
    /// only one copy of the peripheral is in use at a time. For example, don't
    /// create two SPI drivers on `SPI1`, because they will "fight" each other.
    ///
    /// You should strongly prefer using `reborrow()` instead. It returns a
    /// `Peri` that borrows `self`, which allows the borrow checker
    /// to enforce this at compile time.
    pub unsafe fn clone_unchecked(&self) -> Peri<'a, T> {
        Peri::new_unchecked(self.inner)
    }

    /// Reborrow into a "child" Peri.
    ///
    /// `self` will stay borrowed until the child Peripheral is dropped.
    pub fn reborrow(&mut self) -> Peri<'_, T> {
        // safety: we're returning the clone inside a new Peripheral that borrows
        // self, so user code can't use both at the same time.
        unsafe { self.clone_unchecked() }
    }

    /// Map the inner peripheral using `Into`.
    ///
    /// This converts from `Peri<'a, T>` to `Peri<'a, U>`, using an
    /// `Into` impl to convert from `T` to `U`.
    ///
    /// For example, this can be useful to.into() GPIO pins: converting from Peri<'a, PB11>` to `Peri<'a, AnyPin>`.
    #[inline]
    pub fn into<U>(self) -> Peri<'a, U>
    where
        T: Into<U>,
        U: PeripheralType,
    {
        unsafe { Peri::new_unchecked(self.inner.into()) }
    }

    /// Bind a Peri
    #[inline]
    pub fn bind<I>(self, _irq: &I) -> BoundPeri<'a, T>
    where
        T: BindablePeripheralType,
        I: Binding<T::Interrupt, T::Handler>,
    {
        unsafe { BoundPeri::new_unchecked(self.inner) }
    }
}

impl<'a, T: PeripheralType> Deref for Peri<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// An exclusive reference to a peripheral.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BoundPeri<'a, T: BindablePeripheralType> {
    inner: T,
    _lifetime: PhantomData<&'a mut T>,
}

impl<'a, T: BindablePeripheralType> BoundPeri<'a, T> {
    /// Create a new owned a peripheral.
    ///
    /// For use by HALs only.
    ///
    /// If you're an end user you shouldn't use this, you should use `steal()`
    /// on the actual peripheral types instead.
    #[inline]
    #[doc(hidden)]
    pub unsafe fn new_unchecked(inner: T) -> Self {
        Self {
            inner,
            _lifetime: PhantomData,
        }
    }

    /// Unsafely clone (duplicate) a peripheral singleton.
    ///
    /// # Safety
    ///
    /// This returns an owned clone of the peripheral. You must manually ensure
    /// only one copy of the peripheral is in use at a time. For example, don't
    /// create two SPI drivers on `SPI1`, because they will "fight" each other.
    ///
    /// You should strongly prefer using `reborrow()` instead. It returns a
    /// `Peri` that borrows `self`, which allows the borrow checker
    /// to enforce this at compile time.
    pub unsafe fn clone_unchecked(&self) -> BoundPeri<'a, T> {
        Self::new_unchecked(self.inner)
    }

    /// Reborrow into a "child" Peri.
    ///
    /// `self` will stay borrowed until the child Peripheral is dropped.
    pub fn reborrow(&mut self) -> BoundPeri<'_, T> {
        // safety: we're returning the clone inside a new Peripheral that borrows
        // self, so user code can't use both at the same time.
        unsafe { self.clone_unchecked() }
    }

    /// Map the inner peripheral using `Into`.
    ///
    /// This converts from `Peri<'a, T>` to `Peri<'a, U>`, using an
    /// `Into` impl to convert from `T` to `U`.
    ///
    /// For example, this can be useful to.into() GPIO pins: converting from Peri<'a, PB11>` to `Peri<'a, AnyPin>`.
    #[inline]
    pub fn into<U>(self) -> BoundPeri<'a, U>
    where
        T: Into<U>,
        U: BindablePeripheralType,
    {
        unsafe { BoundPeri::new_unchecked(self.inner.into()) }
    }
}

impl<'a, T: BindablePeripheralType> Deref for BoundPeri<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Marker trait for peripheral types.
pub trait PeripheralType: Copy + Sized {}

/// Marker trait for bindable types.
pub trait BindablePeripheralType: PeripheralType {
    /// Associated Interrupt
    type Interrupt: Interrupt;
    /// Associated Handler
    type Handler: Handler<Self::Interrupt>;
}
