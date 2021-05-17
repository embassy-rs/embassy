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

pub trait Unborrow {
    type Target;
    unsafe fn unborrow(self) -> Self::Target;
}

pub trait Steal {
    unsafe fn steal() -> Self;
}

macro_rules! impl_unborrow_tuples {
    ($($t:ident),+) => {
        impl<$($t),+> Unborrow for ($($t),+)
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

        impl<'a, $($t),+> Unborrow for &'a mut($($t),+)
        where
            $(
                $t: Unborrow<Target = $t>
            ),+
        {
            type Target = ($($t),+);
            unsafe fn unborrow(self) -> Self::Target {
                ::core::ptr::read(self)
            }
        }

    };
}

impl_unborrow_tuples!(A, B);
impl_unborrow_tuples!(A, B, C);
impl_unborrow_tuples!(A, B, C, D);
impl_unborrow_tuples!(A, B, C, D, E);
impl_unborrow_tuples!(A, B, C, D, E, F);
impl_unborrow_tuples!(A, B, C, D, E, F, G);
impl_unborrow_tuples!(A, B, C, D, E, F, G, H);
impl_unborrow_tuples!(A, B, C, D, E, F, G, H, I);
impl_unborrow_tuples!(A, B, C, D, E, F, G, H, I, J);
impl_unborrow_tuples!(A, B, C, D, E, F, G, H, I, J, K);
impl_unborrow_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
