/// Unsafely unborrow an owned singleton out of a `&mut`.
///
/// It is intended to be implemented for owned peripheral singletons, such as `USART3` or `AnyPin`.
/// Unborrowing an owned `T` yields the same `T`. Unborrowing a `&mut T` yields a copy of the T.
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
    ///
    /// Safety: This returns a copy of a singleton that's normally not
    /// copiable. The returned copy must ONLY be used while the lifetime of `self` is
    /// valid, as if it were accessed through `self` every time.
    unsafe fn unborrow(self) -> Self::Target;
}

unsafe impl<'a, T: Unborrow> Unborrow for &'a mut T {
    type Target = T::Target;
    unsafe fn unborrow(self) -> Self::Target {
        T::unborrow(core::ptr::read(self))
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
            unsafe fn unborrow(self) -> Self::Target {
                self
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
