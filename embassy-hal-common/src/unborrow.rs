use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

/// This is essentially a `&mut T`, but it is the size of `T` not the size
/// of a pointer. This is useful if T is a zero sized type.
pub struct Unborrowed<'a, T> {
    inner: T,
    _lifetime: PhantomData<&'a mut T>,
}

impl<'a, T> Unborrowed<'a, T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _lifetime: PhantomData,
        }
    }

    pub unsafe fn into_inner(self) -> T {
        self.inner
    }
}

impl<'a, T> Deref for Unborrowed<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> DerefMut for Unborrowed<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Unsafely unborrow an owned singleton out of a `&mut`.
///
/// It is intended to be implemented for owned peripheral singletons, such as `USART3` or `AnyPin`.
/// Unborrowing an owned `T` yields an `Unborrowed<'static, T>`.
/// Unborrowing a `&'a mut T` yields an `Unborrowed<'a, T>`.
///
/// This allows writing HAL drivers that either own or borrow their peripherals, but that don't have
/// to store pointers in the borrowed case.
pub trait Unborrow: Sized {
    /// Unborrow result type
    type Target;

    unsafe fn unborrow_unchecked(&mut self) -> Self::Target;

    /// Unborrow a value.
    #[inline]
    fn unborrow<'a>(mut self) -> Unborrowed<'a, Self::Target>
    where
        Self: 'a,
    {
        Unborrowed::new(unsafe { self.unborrow_unchecked() })
    }
}

impl<'b, T: DerefMut> Unborrow for T
where
    T::Target: Unborrow,
{
    type Target = <T::Target as Unborrow>::Target;

    #[inline]
    unsafe fn unborrow_unchecked(&mut self) -> Self::Target {
        self.deref_mut().unborrow_unchecked()
    }
}
