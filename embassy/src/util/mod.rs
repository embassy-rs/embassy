//! Async utilities
mod drop_bomb;
mod forever;
mod mutex;
mod on_drop;
mod portal;
mod signal;

#[cfg_attr(feature = "executor-agnostic", path = "waker_agnostic.rs")]
mod waker;

pub use drop_bomb::*;
pub use forever::*;
pub use mutex::*;
pub use on_drop::*;
pub use portal::*;
pub use signal::*;
pub use waker::*;

pub unsafe trait Unborrow {
    type Target;
    unsafe fn unborrow(self) -> Self::Target;
}

unsafe impl<'a, T: Unborrow> Unborrow for &'a mut T {
    type Target = T::Target;
    unsafe fn unborrow(self) -> Self::Target {
        T::unborrow(core::ptr::read(self))
    }
}

pub trait Steal {
    unsafe fn steal() -> Self;
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
