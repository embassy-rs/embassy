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
///
/// Safety: this trait can be used to copy non-Copy types. Implementors must not cause
/// immediate UB when copied, and must not cause UB when copies are later used, provided they
/// are only used according the [`Self::unborrow`] safety contract.
///
pub unsafe trait Unborrow {
    /// Unborrow result type
    type Target;

    /// Unborrow a value.
    fn unborrow<'a>(self) -> Unborrowed<'a, Self::Target>
    where
        Self: 'a;
}

unsafe impl<'b, T: Unborrow> Unborrow for &'b mut T {
    type Target = T::Target;

    fn unborrow<'a>(self) -> Unborrowed<'a, Self::Target>
    where
        Self: 'a,
    {
        // Safety: This returns a copy of a singleton that's normally not
        // copiable. The returned copy must ONLY be used while the lifetime of `self` is
        // valid, as if it were accessed through `self` every time.
        T::unborrow(unsafe { core::ptr::read(self) })
    }
}

unsafe impl<'b, T> Unborrow for Unborrowed<'b, T> {
    type Target = T;

    fn unborrow<'a>(self) -> Unborrowed<'a, Self::Target>
    where
        Self: 'a,
    {
        self
    }
}

macro_rules! unsafe_impl_unborrow_tuples {
    ($($t:ident),+) => {
        unsafe impl<$($t),+> Unborrow for ($($t),+)
        where
            $(
                $t: Unborrow<Target = $t>
            ),+
        {
            type Target = ($($t),+);
            fn unborrow<'a>(self) -> Unborrowed<'a, Self::Target>
            where
                Self: 'a
            {
                Unborrowed::new(self)
            }
        }


    };
}

unsafe_impl_unborrow_tuples!(A, B);
unsafe_impl_unborrow_tuples!(A, B, C);
unsafe_impl_unborrow_tuples!(A, B, C, D);
unsafe_impl_unborrow_tuples!(A, B, C, D, E);
unsafe_impl_unborrow_tuples!(A, B, C, D, E, F);
unsafe_impl_unborrow_tuples!(A, B, C, D, E, F, G);
unsafe_impl_unborrow_tuples!(A, B, C, D, E, F, G, H);
unsafe_impl_unborrow_tuples!(A, B, C, D, E, F, G, H, I);
unsafe_impl_unborrow_tuples!(A, B, C, D, E, F, G, H, I, J);
unsafe_impl_unborrow_tuples!(A, B, C, D, E, F, G, H, I, J, K);
unsafe_impl_unborrow_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
